use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReactionEngine {
    targets: BTreeMap<String, TargetProfile>,
    actors: Vec<ActorCombatProfile>,
    break_enabled: bool,
    active_scorch: BTreeMap<String, ScorchInstance>,
    last_time_us: u64,
}

type AttackDelivery = (
    Vec<HitDeliveryReceipt>,
    Vec<BreakEventReceipt>,
    Vec<ReactionReceipt>,
);

impl ReactionEngine {
    pub fn targets(&self) -> &BTreeMap<String, TargetProfile> {
        &self.targets
    }

    pub fn actors(&self) -> &[ActorCombatProfile] {
        &self.actors
    }

    pub fn break_enabled(&self) -> bool {
        self.break_enabled
    }

    pub fn active_scorch(&self) -> &BTreeMap<String, ScorchInstance> {
        &self.active_scorch
    }

    pub fn last_time_us(&self) -> u64 {
        self.last_time_us
    }

    pub fn new(
        targets: Vec<TargetProfile>,
        actors: Vec<ActorCombatProfile>,
        break_enabled: bool,
    ) -> Result<Self, String> {
        if actors.is_empty() || actors.len() > 4 || targets.is_empty() {
            return Err("reaction engine requires one to four actors and nonempty targets".into());
        }
        for (slot, actor) in actors.iter().enumerate() {
            if actor.actor_index != slot
                || actor.level == 0
                || actor.break_intensity.denominator == 0
            {
                return Err(format!("invalid actor profile at {slot}"));
            }
        }
        let mut target_map = BTreeMap::new();
        for t in targets {
            if t.id.is_empty()
                || target_map.contains_key(&t.id)
                || t.current_break_gauge > t.break_cap
                || t.resistances.values().any(|r| r.denominator == 0)
            {
                return Err(format!("invalid or duplicate target {}", t.id));
            }
            if break_enabled && !t.resistances.contains_key("Physical") {
                return Err(format!(
                    "target {} missing required Physical resistance while break is enabled",
                    t.id
                ));
            }
            if let BreakState::Broken {
                broken_at_us,
                duration_us,
            } = t.break_state
            {
                broken_at_us
                    .checked_add(duration_us)
                    .ok_or("break recovery overflow")?;
            }
            target_map.insert(t.id.clone(), t);
        }
        Ok(Self {
            targets: target_map,
            actors,
            break_enabled,
            active_scorch: BTreeMap::new(),
            last_time_us: 0,
        })
    }

    pub fn advance_time(&mut self, now_us: u64) -> Result<Vec<ReactionReceipt>, String> {
        self.transition(|next| next.advance_time_inner(now_us))
    }
    fn transition<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut next = self.clone();
        let receipt = operation(&mut next)?;
        *self = next;
        Ok(receipt)
    }
    fn advance_time_inner(&mut self, now_us: u64) -> Result<Vec<ReactionReceipt>, String> {
        if now_us < self.last_time_us {
            return Err(format!("time reversal: {now_us} < {}", self.last_time_us));
        }
        self.last_time_us = now_us;
        let mut tick_receipts = Vec::new();

        for target in self.targets.values_mut() {
            match target.break_state {
                BreakState::Broken {
                    broken_at_us,
                    duration_us,
                } if now_us >= broken_at_us.saturating_add(duration_us) => {
                    target.break_state = BreakState::Unbroken;
                    target.current_break_gauge = target.break_cap;
                }
                _ => {}
            }
        }

        let mut expired_scorch = Vec::new();
        for (target_id, scorch) in self.active_scorch.iter_mut() {
            while now_us >= scorch.next_tick_us && scorch.next_tick_us < scorch.expires_at_us {
                let dmg = SCORCH_BASE_DAMAGE * scorch.stacks as u64;
                tick_receipts.push(ReactionReceipt {
                    target_id: target_id.clone(),
                    source_actor: scorch.source_actor,
                    kind: ReactionKind::Scorch,
                    damage: dmg,
                    timestamp_us: scorch.next_tick_us,
                    details: serde_json::json!({
                        "stacks": scorch.stacks,
                        "periodic_tick": true,
                    }),
                });
                scorch.next_tick_us = scorch
                    .next_tick_us
                    .checked_add(SCORCH_TICK_INTERVAL_US)
                    .ok_or("scorch tick deadline overflow")?;
            }
            if now_us >= scorch.expires_at_us {
                expired_scorch.push(target_id.clone());
            }
        }

        for id in expired_scorch {
            self.active_scorch.remove(&id);
        }

        tick_receipts
            .sort_by(|a, b| (a.timestamp_us, &a.target_id).cmp(&(b.timestamp_us, &b.target_id)));
        Ok(tick_receipts)
    }

    pub fn resolve_targets(&self, rule: &TargetingRule) -> Vec<String> {
        match rule {
            TargetingRule::SingleTarget { target_id } => {
                if self.targets.contains_key(target_id) {
                    vec![target_id.clone()]
                } else {
                    Vec::new()
                }
            }
            TargetingRule::AllTargets => self.targets.keys().cloned().collect(),
            TargetingRule::SplashRadius {
                center_target,
                radius_mm,
            } => {
                let center_pos = match self.targets.get(center_target) {
                    Some(t) => t.position,
                    None => return Vec::new(),
                };
                let mut matched = Vec::new();
                for (id, t) in &self.targets {
                    if t.position.is_within_radius(&center_pos, *radius_mm) {
                        matched.push(id.clone());
                    }
                }
                matched
            }
        }
    }

    pub fn process_attack(
        &mut self,
        source_actor: usize,
        targeting: &TargetingRule,
        damage: u64,
        break_shred: u64,
        energy_gain: u64,
        now_us: u64,
    ) -> Result<AttackDelivery, String> {
        self.transition(|next| {
            next.process_attack_inner(
                source_actor,
                targeting,
                damage,
                break_shred,
                energy_gain,
                now_us,
            )
        })
    }
    fn process_attack_inner(
        &mut self,
        source_actor: usize,
        targeting: &TargetingRule,
        damage: u64,
        break_shred: u64,
        energy_gain: u64,
        now_us: u64,
    ) -> Result<AttackDelivery, String> {
        if source_actor >= self.actors.len() {
            return Err(format!("actor not found: {source_actor}"));
        }
        match targeting {
            TargetingRule::SingleTarget { target_id } if !self.targets.contains_key(target_id) => {
                return Err(format!("target not found: {target_id}"));
            }
            TargetingRule::SplashRadius { center_target, .. }
                if !self.targets.contains_key(center_target) =>
            {
                return Err(format!("center target not found: {center_target}"));
            }
            _ => {}
        }
        let ticks = self.advance_time_inner(now_us)?;

        let target_ids = self.resolve_targets(targeting);
        if target_ids.is_empty() {
            return Ok((Vec::new(), Vec::new(), ticks));
        }

        // Grant energy per event (strictly not multiplied by target count)
        if energy_gain > 0 && source_actor < self.actors.len() {
            let _ = self.actors[source_actor].energy_pool.grant(energy_gain);
        }

        let mut hit_receipts = Vec::new();
        let mut break_events = Vec::new();

        for tid in target_ids {
            let target = match self.targets.get_mut(&tid) {
                Some(t) => t,
                None => continue,
            };

            hit_receipts.push(HitDeliveryReceipt {
                source_actor,
                target_id: tid.clone(),
                damage,
                break_shred,
                timestamp_us: now_us,
            });

            if self.break_enabled && target.break_state == BreakState::Unbroken && break_shred > 0 {
                target.current_break_gauge = target.current_break_gauge.saturating_sub(break_shred);
                if target.current_break_gauge == 0 {
                    now_us
                        .checked_add(DEFAULT_BREAK_DURATION_US)
                        .ok_or("break recovery overflow")?;
                    target.break_state = BreakState::Broken {
                        broken_at_us: now_us,
                        duration_us: DEFAULT_BREAK_DURATION_US,
                    };

                    let original_cap = target.break_cap;
                    let def = target.defense;
                    let res =
                        target.resistances.get("Physical").copied().ok_or_else(|| {
                            format!("break Physical resistance missing for {tid}")
                        })?;

                    let mut contributions = Vec::new();
                    let mut total_break = 0u64;

                    for act in &self.actors {
                        let intensity_f = act.break_intensity.numerator as f64
                            / act.break_intensity.denominator as f64;
                        let res_f = res.numerator as f64 / res.denominator as f64;

                        let req = BreakDamageRequest {
                            base_break: act.base_break as f64,
                            intensity: intensity_f,
                            original_cap: original_cap as f64,
                            attacker_level: act.level,
                            enemy_defense: def as f64,
                            resistance: res_f,
                            extra_bonus: 0.0,
                            shred: 0.0,
                            ignore: 0.0,
                            special: 1.0,
                            negative_res_model: NegativeResistanceModel::StandardRational,
                        };
                        let dmg_f = break_damage(&req).map_err(|e| format!("{e:?}"))?;
                        if !dmg_f.is_finite() || dmg_f < 0.0 || dmg_f >= 18446744073709551616.0 {
                            return Err("break damage outside u64 range".into());
                        }
                        let actor_dmg = dmg_f.floor() as u64;

                        contributions.push(BreakHitContribution {
                            actor_index: act.actor_index,
                            character_id: act.character_id.clone(),
                            damage: actor_dmg,
                        });
                        total_break = total_break
                            .checked_add(actor_dmg)
                            .ok_or("team break damage overflow")?;
                    }

                    break_events.push(BreakEventReceipt {
                        target_id: tid.clone(),
                        triggered_at_us: now_us,
                        duration_us: DEFAULT_BREAK_DURATION_US,
                        contributions,
                        total_break_damage: total_break,
                    });
                }
            }
        }

        Ok((hit_receipts, break_events, ticks))
    }

    pub fn trigger_reaction(
        &mut self,
        source_actor: usize,
        target_id: &str,
        kind: ReactionKind,
        now_us: u64,
    ) -> Result<Vec<ReactionReceipt>, String> {
        self.transition(|next| next.trigger_reaction_inner(source_actor, target_id, kind, now_us))
    }
    fn trigger_reaction_inner(
        &mut self,
        source_actor: usize,
        target_id: &str,
        kind: ReactionKind,
        now_us: u64,
    ) -> Result<Vec<ReactionReceipt>, String> {
        if source_actor >= self.actors.len() {
            return Err(format!("actor not found: {source_actor}"));
        }

        if !self.targets.contains_key(target_id) {
            return Err(format!("target not found: {target_id}"));
        }
        if kind == ReactionKind::Scorch && now_us.checked_add(SCORCH_DURATION_US).is_none() {
            return Err("scorch expiry overflow".into());
        }
        let mut receipts = self.advance_time_inner(now_us)?;

        let receipt = match kind {
            ReactionKind::Blossom => {
                let dmg = BLOSSOM_BASE_DAMAGE;
                ReactionReceipt {
                    target_id: target_id.to_string(),
                    source_actor,
                    kind: ReactionKind::Blossom,
                    damage: dmg,
                    timestamp_us: now_us,
                    details: serde_json::json!({ "instant": true }),
                }
            }
            ReactionKind::Nova => {
                let dmg = NOVA_BASE_DAMAGE;
                ReactionReceipt {
                    target_id: target_id.to_string(),
                    source_actor,
                    kind: ReactionKind::Nova,
                    damage: dmg,
                    timestamp_us: now_us,
                    details: serde_json::json!({ "detonation": true }),
                }
            }
            ReactionKind::Scorch => {
                let next_tick_us = now_us
                    .checked_add(SCORCH_TICK_INTERVAL_US)
                    .ok_or("scorch tick deadline overflow")?;
                let expires_at_us = now_us
                    .checked_add(SCORCH_DURATION_US)
                    .ok_or("scorch expiry deadline overflow")?;
                let entry =
                    self.active_scorch
                        .entry(target_id.to_string())
                        .or_insert(ScorchInstance {
                            source_actor,
                            stacks: 0,
                            next_tick_us,
                            expires_at_us,
                        });
                entry.stacks = (entry.stacks + 1).min(SCORCH_MAX_STACKS);
                // This harness attributes refreshed stacks to the latest applier.
                entry.source_actor = source_actor;
                entry.expires_at_us = expires_at_us;

                ReactionReceipt {
                    target_id: target_id.to_string(),
                    source_actor,
                    kind: ReactionKind::Scorch,
                    damage: 0, // initial application has no instant damage; ticks periodically
                    timestamp_us: now_us,
                    details: serde_json::json!({
                        "stacks": entry.stacks,
                        "duration_us": SCORCH_DURATION_US,
                    }),
                }
            }
            ReactionKind::Remora => ReactionReceipt {
                target_id: target_id.to_string(),
                source_actor,
                kind: ReactionKind::Remora,
                damage: 10_000,
                timestamp_us: now_us,
                details: serde_json::json!({ "bomb": true }),
            },
        };

        receipts.push(receipt);
        Ok(receipts)
    }
}
