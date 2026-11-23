use super::*;

pub const DEFAULT_MAX_DISPATCH_DEPTH: usize = 4;

#[derive(Clone, Debug, Serialize)]
pub struct DispatchEngine {
    pub(super) team_size: usize,
    pub(super) active_actor: usize,
    pub(super) buffs: BuffManager,
    pub(super) icd: IcdTracker,
    pub(super) callbacks: Vec<CallbackRegistration>,
    pub(super) resources: BTreeMap<String, Pool>,
    pub(super) last_time_us: u64,
    pub(super) max_depth: usize,
}

impl DispatchEngine {
    pub fn active_actor(&self) -> usize {
        self.active_actor
    }
    pub fn buffs(&self) -> &BuffManager {
        &self.buffs
    }
    pub fn callbacks(&self) -> &[CallbackRegistration] {
        &self.callbacks
    }
    pub fn resources(&self) -> &BTreeMap<String, Pool> {
        &self.resources
    }

    pub fn new(
        team_size: usize,
        initial_active: usize,
        resources: BTreeMap<String, Pool>,
    ) -> Result<Self, DispatchError> {
        if team_size == 0 || team_size > 4 {
            return Err(DispatchError::InvalidTeamSize(team_size));
        }
        if initial_active >= team_size {
            return Err(DispatchError::UnknownActor(initial_active));
        }
        Ok(Self {
            team_size,
            active_actor: initial_active,
            buffs: BuffManager::default(),
            icd: IcdTracker::default(),
            callbacks: Vec::new(),
            resources,
            last_time_us: 0,
            max_depth: DEFAULT_MAX_DISPATCH_DEPTH,
        })
    }

    pub fn apply_buff(
        &mut self,
        buff: BuffDefinition,
        owner_actor: usize,
        now_us: u64,
    ) -> Result<(), DispatchError> {
        if owner_actor >= self.team_size {
            return Err(DispatchError::UnknownActor(owner_actor));
        }
        if let BuffRecipient::SpecificActor(index) = buff.recipient
            && index >= self.team_size
        {
            return Err(DispatchError::UnknownActor(index));
        }
        let mut next = self.clone();
        next.advance_time(now_us)?;
        next.buffs.apply_buff(buff, owner_actor, now_us)?;
        *self = next;
        Ok(())
    }

    pub fn register_callback(
        &mut self,
        callback: CallbackRegistration,
    ) -> Result<(), DispatchError> {
        if callback.owner_actor >= self.team_size {
            return Err(DispatchError::UnknownActor(callback.owner_actor));
        }
        if self
            .callbacks
            .iter()
            .any(|existing| existing.id == callback.id)
        {
            return Err(DispatchError::DuplicateCallback(callback.id));
        }
        if matches!(callback.icd_rule, IcdRule::HitCountOnly { hit_interval: 0 }) {
            return Err(DispatchError::InvalidIcd(callback.id.clone()));
        }
        if let IcdScope::SharedGroup(group) = &callback.icd_scope
            && self.callbacks.iter().any(|existing| {
                matches!(&existing.icd_scope,
                IcdScope::SharedGroup(other) if other == group)
                    && existing.icd_rule != callback.icd_rule
            })
        {
            return Err(DispatchError::InvalidIcd(callback.id.clone()));
        }
        match &callback.payload {
            ProcPayload::GrantPoolResource { pool, .. } if !self.resources.contains_key(pool) => {
                return Err(DispatchError::UnknownPool(pool.clone()));
            }
            ProcPayload::TriggerHeal { target_actor, .. } if *target_actor >= self.team_size => {
                return Err(DispatchError::UnknownActor(*target_actor));
            }
            ProcPayload::ApplyBuff { buff } => {
                buff.validate()?;
                if let BuffRecipient::SpecificActor(index) = buff.recipient
                    && index >= self.team_size
                {
                    return Err(DispatchError::UnknownActor(index));
                }
            }
            _ => {}
        }
        self.callbacks.push(callback);
        Ok(())
    }

    pub fn advance_time(&mut self, now_us: u64) -> Result<(), DispatchError> {
        if now_us < self.last_time_us {
            return Err(DispatchError::TimeReversal {
                last_us: self.last_time_us,
                now_us,
            });
        }
        self.last_time_us = now_us;
        self.buffs.advance_time(now_us);
        Ok(())
    }

    pub fn dispatch(
        &mut self,
        event: CombatEvent,
        now_us: u64,
    ) -> Result<DispatchReceipt, DispatchError> {
        let validate_actor = |index| {
            if index < self.team_size {
                Ok(())
            } else {
                Err(DispatchError::UnknownActor(index))
            }
        };
        match &event {
            CombatEvent::ActionStarted { actor_index, .. } => validate_actor(*actor_index)?,
            CombatEvent::HitDealt { source_actor, .. }
            | CombatEvent::DamageReceived { source_actor, .. } => validate_actor(*source_actor)?,
            CombatEvent::HealDealt {
                source_actor,
                target_actor,
                ..
            }
            | CombatEvent::ShieldApplied {
                source_actor,
                target_actor,
                ..
            } => {
                validate_actor(*source_actor)?;
                validate_actor(*target_actor)?;
            }
            CombatEvent::ActorSwitched {
                from_index,
                to_index,
            } => {
                validate_actor(*from_index)?;
                validate_actor(*to_index)?;
                if *from_index != self.active_actor {
                    return Err(DispatchError::InvalidSwitchFrom {
                        expected: self.active_actor,
                        actual: *from_index,
                    });
                }
            }
            CombatEvent::TargetDied { .. } => {}
        }
        let before = self.clone();
        self.advance_time(now_us)?;
        if let CombatEvent::ActorSwitched { to_index, .. } = &event {
            self.active_actor = *to_index;
        }
        let mut procs_triggered = Vec::new();
        if let Err(error) = self.dispatch_internal(&event, now_us, 0, &mut procs_triggered) {
            *self = before;
            return Err(error);
        }
        Ok(DispatchReceipt {
            event,
            timestamp_us: now_us,
            depth: 0,
            procs_triggered,
        })
    }

    pub(crate) fn dispatch_internal(
        &mut self,
        event: &CombatEvent,
        now_us: u64,
        depth: usize,
        receipts: &mut Vec<ProcReceipt>,
    ) -> Result<(), DispatchError> {
        if depth > self.max_depth {
            return Err(DispatchError::RecursionDepthExceeded {
                max_depth: self.max_depth,
            });
        }

        let active_actor = self.active_actor;
        let mut eligible_indices = Vec::new();

        for (i, cb) in self.callbacks.iter().enumerate() {
            if cb
                .max_triggers
                .is_some_and(|max_t| cb.triggers_so_far >= max_t)
            {
                continue;
            }

            let matches_all = cb
                .conditions
                .iter()
                .all(|c| c.matches(event, cb.owner_actor, active_actor));

            if !matches_all {
                continue;
            }

            let scope = match (&cb.icd_scope, event) {
                (IcdScope::PerTarget(_), CombatEvent::HitDealt { target_id, .. }) => {
                    IcdScope::PerTarget(target_id.clone())
                }
                (
                    IcdScope::PerTarget(_),
                    CombatEvent::DamageReceived { target_id, .. }
                    | CombatEvent::TargetDied { target_id },
                ) => IcdScope::PerTarget(target_id.clone()),
                (other_scope, _) => other_scope.clone(),
            };

            // Every matching event must advance hit-count ICD, including suppressed hits.
            eligible_indices.push((i, scope));
        }

        for (idx, scope) in eligible_indices {
            let (cb_id, owner, payload) = {
                let cb = &mut self.callbacks[idx];
                if !self
                    .icd
                    .record_event(&cb.id, &scope, &cb.icd_rule, now_us)?
                {
                    continue;
                }
                cb.triggers_so_far = cb
                    .triggers_so_far
                    .checked_add(1)
                    .ok_or(DispatchError::ArithmeticOverflow)?;
                (cb.id.clone(), cb.owner_actor, cb.payload.clone())
            };

            match payload {
                ProcPayload::GrantPoolResource { pool, amount } => {
                    let p = self
                        .resources
                        .get_mut(&pool)
                        .ok_or_else(|| DispatchError::UnknownPool(pool.clone()))?;
                    let grant = p.grant(amount);
                    receipts.push(ProcReceipt {
                        callback_id: cb_id,
                        owner_actor: owner,
                        triggered_at_us: now_us,
                        payload_kind: "grant_pool_resource".to_string(),
                        details: serde_json::json!({
                            "pool": pool,
                            "amount": amount,
                            "applied": grant.applied,
                            "overflow": grant.excess,
                        }),
                    });
                }
                ProcPayload::ApplyBuff { buff } => {
                    let buff_id = buff.id.clone();
                    self.buffs.apply_buff(buff, owner, now_us)?;
                    receipts.push(ProcReceipt {
                        callback_id: cb_id,
                        owner_actor: owner,
                        triggered_at_us: now_us,
                        payload_kind: "apply_buff".to_string(),
                        details: serde_json::json!({
                            "buff_id": buff_id,
                        }),
                    });
                }
                ProcPayload::TriggerBonusDamage {
                    element,
                    multiplier_percent,
                    target_id,
                } => {
                    let base_damage = match event {
                        CombatEvent::HitDealt { damage, .. }
                        | CombatEvent::DamageReceived { damage, .. } => *damage,
                        _ => return Err(DispatchError::InvalidBonusDamageBasis),
                    };
                    let scaled_damage = u64::try_from(
                        u128::from(base_damage) * u128::from(multiplier_percent) / 100,
                    )
                    .map_err(|_| DispatchError::ArithmeticOverflow)?;
                    receipts.push(ProcReceipt {
                        callback_id: cb_id.clone(),
                        owner_actor: owner,
                        triggered_at_us: now_us,
                        payload_kind: "trigger_bonus_damage".to_string(),
                        details: serde_json::json!({
                            "element": element,
                            "multiplier_percent": multiplier_percent,
                            "target_id": target_id,
                            "base_damage": base_damage,
                            "damage": scaled_damage,
                        }),
                    });

                    let child_event = CombatEvent::HitDealt {
                        source_actor: owner,
                        target_id,
                        action_id: format!("{cb_id}:proc"),
                        element,
                        is_crit: false,
                        damage: scaled_damage,
                    };
                    self.dispatch_internal(&child_event, now_us, depth + 1, receipts)?;
                }
                ProcPayload::TriggerHeal {
                    amount,
                    target_actor,
                } => {
                    receipts.push(ProcReceipt {
                        callback_id: cb_id.clone(),
                        owner_actor: owner,
                        triggered_at_us: now_us,
                        payload_kind: "trigger_heal".to_string(),
                        details: serde_json::json!({
                            "amount": amount,
                            "target_actor": target_actor,
                        }),
                    });

                    let child_event = CombatEvent::HealDealt {
                        source_actor: owner,
                        target_actor,
                        amount,
                    };
                    self.dispatch_internal(&child_event, now_us, depth + 1, receipts)?;
                }
            }
        }

        Ok(())
    }
}
