use super::model::GameTargetStatusSummary;
use super::state::GameSimulatorState;
use super::{
    ActiveBuffRef, BuffTimelineDiff, ExactFraction, GameActorStateDelta, GameActorStateSummary,
    GameBuffCatalogEntry, GameFirefliesStack, GameScenarioRequest, GameStaminaState,
    GameStateBoundary, GameStateFrame, GameTargetStateSummary,
};
use ntesim_core::buffs::dispatch::{ActiveBuffInstance, BuffDuration, BuffRecipient, DecayPolicy};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(crate) struct Recorder {
    pub(crate) buff_catalog: Vec<GameBuffCatalogEntry>,
    pub(crate) timer_catalog: Vec<super::GameTimerDefinition>,
    previous_timers: Vec<super::GameTimerRef>,
    previous_actors: Vec<GameActorStateSummary>,
    previous_targets: Vec<GameTargetStateSummary>,
    previous_buffs: Vec<ActiveBuffRef>,
    previous_stamina: Option<GameStaminaState>,
    previous_meta: Option<(usize, usize, usize, usize)>,
    pub(crate) frames: Vec<GameStateFrame>,
}

impl Recorder {
    pub(crate) fn encode_buffs(&mut self, instances: &[ActiveBuffInstance]) -> Vec<ActiveBuffRef> {
        instances
            .iter()
            .map(|buff| {
                let entry = GameBuffCatalogEntry::from_instance(buff);
                let buff_def_id = match self.buff_catalog.iter().position(|e| e == &entry) {
                    Some(index) => index,
                    None => {
                        let index = self.buff_catalog.len();
                        self.buff_catalog.push(entry);
                        index
                    }
                };
                ActiveBuffRef {
                    instance_id: buff.instance_id,
                    buff_def_id,
                    stacks: buff.stacks,
                    duration: buff.duration.clone(),
                }
            })
            .collect()
    }

    pub(crate) fn encode_timers(
        &mut self,
        state: &GameSimulatorState,
    ) -> Result<Vec<super::GameTimerRef>, String> {
        use ntesim_core::kernel::timing::ClockDomain;
        let now = state.current_time_us;
        let world = state.mode_clock.relative_us();
        let object = state
            .mode_clock
            .elapsed_us(ClockDomain::ObjectLifetime)
            .ok_or("missing object clock")?;
        let mut timers = Vec::new();
        if let Some(stamina) = &state.combat_stamina
            && stamina.recovery_ready_at_us > now
        {
            timers.push((
                super::GameTimerDefinition {
                    owner: None,
                    key: "stamina_recovery".into(),
                    clock_basis: ClockDomain::World,
                },
                world
                    .checked_add(stamina.recovery_ready_at_us - now)
                    .ok_or("stamina observation overflow")?,
            ));
        }
        for (owner, actor) in state.actors.iter().enumerate() {
            for (ability, deadline) in actor.ability_cooldowns.semantic_deadlines() {
                if deadline > world {
                    timers.push((
                        super::GameTimerDefinition {
                            owner: Some(owner),
                            key: format!("cooldown:{ability}"),
                            clock_basis: ClockDomain::World,
                        },
                        deadline,
                    ));
                }
            }
            let gear = &actor.equipment;
            for (key, ready) in [
                ("be_happy", gear.be_happy.ready_at_us),
                ("bopu", gear.bopu.ready_at_us),
                ("demon_blade", gear.demon_blade.ready_at_us),
                ("far_side_moon", gear.far_side_moon.ready_at_us),
                ("mind_royale", gear.mind_royale.ready_at_us),
                ("moon", gear.moon.ready_at_us),
                ("noros", gear.noros.ready_at_us),
                ("spider", gear.spider.ready_at_us),
                ("whale", gear.whale.ready_at_us),
                ("bit_game.off_field", gear.bit_game.off_field_ready_at_us),
                ("bit_game.on_field", gear.bit_game.on_field_ready_at_us),
            ] {
                if ready > now {
                    timers.push((
                        super::GameTimerDefinition {
                            owner: Some(owner),
                            key: format!("proc:{key}"),
                            clock_basis: ClockDomain::World,
                        },
                        world
                            .checked_add(ready - now)
                            .ok_or("proc observation overflow")?,
                    ));
                }
            }
            use crate::characters::RuntimeState;
            let deployed = match &actor.character {
                RuntimeState::Edgar(state) => {
                    state.field.as_ref().map(|field| ("field", field.until))
                }
                RuntimeState::Haniel(state) => state.hootie_until_us.map(|end| ("hootie", end)),
                RuntimeState::Baicang(state) if state.domain_start_us <= now => {
                    Some(("domain", state.domain_until_us))
                }
                RuntimeState::Iroi(state)
                    if state.dream_active && state.dream_release_us <= now =>
                {
                    Some(("dream", state.dream_end_us))
                }
                RuntimeState::Nanally(state) => Some(("underboss", state.underboss_until_us)),
                _ => None,
            };
            if let Some((key, end)) = deployed
                && end > now
            {
                let deadline = object
                    .checked_add(end - now)
                    .ok_or("object observation deadline overflow")?;
                timers.push((
                    super::GameTimerDefinition {
                        owner: Some(owner),
                        key: key.into(),
                        clock_basis: ClockDomain::ObjectLifetime,
                    },
                    deadline,
                ));
            }
        }
        let mut refs = Vec::with_capacity(timers.len());
        for (definition, expires_at_us) in timers {
            let timer_def_id = self
                .timer_catalog
                .iter()
                .position(|entry| *entry == definition)
                .unwrap_or_else(|| {
                    let index = self.timer_catalog.len();
                    self.timer_catalog.push(definition);
                    index
                });
            refs.push(super::GameTimerRef {
                timer_def_id,
                expires_at_us,
            });
        }
        refs.sort_by_key(|timer| timer.timer_def_id);
        Ok(refs)
    }

    pub(crate) fn observe(
        &mut self,
        state: &GameSimulatorState,
        request: &GameScenarioRequest,
        boundary: GameStateBoundary,
        hits_end: usize,
        trace_end: usize,
    ) -> Result<(), String> {
        let healing_end = state.healing_receipts.len();
        let actors = state.actor_summaries(request)?;
        let targets = state.target_summaries()?;
        let buffs = self.encode_buffs(state.buffs.instances());
        let stamina = state.combat_stamina.as_ref().map(|s| GameStaminaState {
            current: s.current,
            spent: s.total_spent,
            recovered: s.total_recovered,
            recovery_ready_at_us: s.recovery_ready_at_us,
        });
        let meta = (state.active_actor_index, hits_end, healing_end, trace_end);
        let timers = self.encode_timers(state)?;

        if matches!(
            boundary,
            GameStateBoundary::Initial | GameStateBoundary::Final
        ) || self.frames.is_empty()
        {
            let frame = GameStateFrame {
                time_us: state.current_time_us,
                boundary,
                active_actor_index: state.active_actor_index,
                actors: actors
                    .iter()
                    .map(GameActorStateDelta::from_summary)
                    .collect(),
                targets: targets.clone(),
                buffs: Some(buffs.clone()),
                buff_diff: None,
                stamina: stamina.clone(),
                timers: Some(timers.clone()),
                hits_end,
                healing_end,
                trace_end,
            };
            self.previous_actors = actors;
            self.previous_targets = targets;
            self.previous_buffs = buffs;
            self.previous_stamina = stamina;
            self.previous_timers = timers;
            self.previous_meta = Some(meta);
            self.frames.push(frame);
            return Ok(());
        }

        let mut actor_deltas = Vec::new();
        for curr in &actors {
            let prev = self
                .previous_actors
                .get(curr.slot)
                .ok_or("missing previous actor state in timeline recorder")?;
            if let Some(delta) = GameActorStateDelta::diff(prev, curr)? {
                actor_deltas.push(delta);
            }
        }
        let target_deltas = targets
            .iter()
            .filter(|t| self.previous_targets.get(t.target_index) != Some(*t))
            .cloned()
            .collect::<Vec<_>>();
        let buff_diff = compute_buff_diff(&self.previous_buffs, &buffs);
        let stamina_delta = (stamina != self.previous_stamina)
            .then(|| stamina.clone())
            .flatten();

        let timer_delta = (self.previous_timers != timers).then(|| timers.clone());
        if actor_deltas.is_empty()
            && target_deltas.is_empty()
            && buff_diff.is_none()
            && stamina_delta.is_none()
            && timer_delta.is_none()
            && self.previous_meta == Some(meta)
        {
            self.previous_actors = actors;
            self.previous_targets = targets;
            self.previous_buffs = buffs;
            self.previous_stamina = stamina;
            self.previous_meta = Some(meta);
            return Ok(());
        }

        self.previous_actors = actors;
        self.previous_targets = targets;
        self.previous_buffs = buffs;
        self.previous_stamina = stamina;
        self.previous_timers = timers;
        self.previous_meta = Some(meta);
        self.frames.push(GameStateFrame {
            time_us: state.current_time_us,
            boundary,
            active_actor_index: state.active_actor_index,
            actors: actor_deltas,
            targets: target_deltas,
            buffs: None,
            buff_diff,
            stamina: stamina_delta,
            timers: timer_delta,
            hits_end,
            healing_end,
            trace_end,
        });
        Ok(())
    }
}

fn compute_buff_diff(
    previous: &[ActiveBuffRef],
    current: &[ActiveBuffRef],
) -> Option<BuffTimelineDiff> {
    let mut remove = Vec::new();
    for prev in previous {
        if !current.iter().any(|c| c.instance_id == prev.instance_id) {
            remove.push(prev.instance_id);
        }
    }
    let mut upsert = Vec::new();
    for curr in current {
        if previous.iter().find(|p| p.instance_id == curr.instance_id) != Some(curr) {
            upsert.push(curr.clone());
        }
    }
    if remove.is_empty() && upsert.is_empty() {
        None
    } else {
        Some(BuffTimelineDiff { upsert, remove })
    }
}

fn validate_buff_ref(
    r: &ActiveBuffRef,
    catalog: &[GameBuffCatalogEntry],
    actor_count: usize,
) -> Result<(), String> {
    if r.instance_id == 0 {
        return Err("buff ref instance_id must be positive".into());
    }
    let def = catalog
        .get(r.buff_def_id)
        .ok_or("buff ref references unknown buff_catalog entry")?;
    if def.clock_basis != ntesim_core::kernel::timing::ClockDomain::World
        || def.id.is_empty()
        || def.source.is_empty()
        || def.owner_actor >= actor_count
        || def.max_stacks == 0
        || r.stacks == 0
        || r.stacks > def.max_stacks
    {
        return Err("buff ref violates buff_catalog definition bounds".into());
    }
    Ok(())
}

fn validate_unique_buff_ids(buffs: &[ActiveBuffRef]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for buff in buffs {
        if !seen.insert(buff.instance_id) {
            return Err("duplicate buff instance_id in boundary snapshot".into());
        }
    }
    Ok(())
}

fn advance_buffs_to(
    buffs: &mut Vec<ActiveBuffRef>,
    catalog: &[GameBuffCatalogEntry],
    now_us: u64,
) -> BTreeSet<usize> {
    let mut hp_max_changed_actors = BTreeSet::new();
    let mut i = 0;
    while i < buffs.len() {
        let def = &catalog[buffs[i].buff_def_id];
        let mut expired = false;
        let mut one_stack_decay = false;
        let mut dur_step = 0;

        let is_expired = match buffs[i].duration {
            BuffDuration::Timed {
                expires_at_us,
                duration_us,
            } => {
                dur_step = duration_us;
                now_us >= expires_at_us
            }
            BuffDuration::Permanent => false,
        };

        if is_expired {
            if def.decay_policy == DecayPolicy::OneByOne && buffs[i].stacks > 1 {
                one_stack_decay = true;
            } else {
                expired = true;
            }
        }

        if (expired || one_stack_decay)
            && def.id == crate::sets::hp_suit::BUFF_ID
            && def
                .stat_modifiers
                .contains_key(crate::sets::hp_suit::MAX_HP_BONUS_KEY)
        {
            hp_max_changed_actors.insert(def.owner_actor);
        }

        if expired {
            buffs.remove(i);
        } else if one_stack_decay {
            let old_expiry = match buffs[i].duration {
                BuffDuration::Timed { expires_at_us, .. } => expires_at_us,
                _ => unreachable!(),
            };
            buffs[i].stacks -= 1;
            buffs[i].duration = BuffDuration::Timed {
                duration_us: dur_step,
                expires_at_us: old_expiry
                    .checked_add(dur_step)
                    .expect("validated decay horizon"),
            };
        } else {
            i += 1;
        }
    }
    hp_max_changed_actors
}

fn reconstruct_target_statuses(
    buffs: &[ActiveBuffRef],
    catalog: &[GameBuffCatalogEntry],
    target_index: usize,
) -> Result<Vec<GameTargetStatusSummary>, String> {
    buffs
        .iter()
        .filter(|r| {
            catalog[r.buff_def_id]
                .recipient
                .matches_target_index(target_index)
        })
        .map(|r| {
            let def = &catalog[r.buff_def_id];
            let expires_at_us = match r.duration {
                BuffDuration::Timed { expires_at_us, .. } => expires_at_us,
                BuffDuration::Permanent => {
                    return Err("target status cannot have permanent duration".into());
                }
            };
            let resistance_reductions = def
                .stat_modifiers
                .iter()
                .filter_map(|(key, value)| {
                    key.strip_prefix("resistance_reduction:")
                        .map(|element| Ok((element.to_owned(), ExactFraction::from_ratio(*value)?)))
                })
                .collect::<Result<BTreeMap<_, _>, String>>()?;
            Ok(GameTargetStatusSummary {
                clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
                source_actor_index: def.owner_actor,
                id: def.id.clone(),
                expires_at_us,
                semantic_tags: def.semantic_tags.clone(),
                stacks: r.stacks,
                resistance_reductions,
            })
        })
        .collect()
}

fn fireflies_stacks_for_actor(
    buffs: &[ActiveBuffRef],
    catalog: &[GameBuffCatalogEntry],
    actor_slot: usize,
) -> Result<(u32, Vec<GameFirefliesStack>), String> {
    let mut total = 0u32;
    let mut stacks = Vec::new();
    for r in buffs {
        let def = &catalog[r.buff_def_id];
        if def.id == "fireflies.crit_damage"
            && def.source == "set:fireflies"
            && def.owner_actor == actor_slot
            && matches!(def.recipient, BuffRecipient::SpecificActor(idx) if idx == actor_slot)
        {
            total += r.stacks;
            let expires_at_us = match r.duration {
                BuffDuration::Timed { expires_at_us, .. } => expires_at_us,
                BuffDuration::Permanent => {
                    return Err("Fireflies stacks must be timed".into());
                }
            };
            let ratio = def
                .stat_modifiers
                .get("crit_damage_bonus")
                .copied()
                .ok_or("Fireflies stack lacks critical damage value")?;
            stacks.push(GameFirefliesStack {
                expires_at_us,
                crit_damage_bonus: ExactFraction::from_ratio(ratio)?,
            });
        }
    }
    Ok((total, stacks))
}

struct TimelineFold {
    actors: Vec<GameActorStateSummary>,
    targets: Vec<GameTargetStateSummary>,
    buffs: Vec<ActiveBuffRef>,
    stamina: Option<GameStaminaState>,
    last_non_final_time_us: u64,
}

fn validate_buff_diff(
    diff: &BuffTimelineDiff,
    catalog: &[GameBuffCatalogEntry],
    n_actors: usize,
    folded_buffs: &mut Vec<ActiveBuffRef>,
) -> Result<(), String> {
    if diff.remove.is_empty() && diff.upsert.is_empty() {
        return Err("empty buff_diff emitted in timeline frame".into());
    }
    let mut remove_ids = BTreeSet::new();
    for &id in &diff.remove {
        if !remove_ids.insert(id) {
            return Err("duplicate id in buff_diff.remove".into());
        }
        if !folded_buffs.iter().any(|b| b.instance_id == id) {
            return Err("buff_diff.remove references inactive instance_id".into());
        }
    }
    let mut upsert_ids = BTreeSet::new();
    for item in &diff.upsert {
        if !upsert_ids.insert(item.instance_id) {
            return Err("duplicate instance_id in buff_diff.upsert".into());
        }
        if remove_ids.contains(&item.instance_id) {
            return Err("buff_diff cannot both remove and upsert the same instance_id".into());
        }
        validate_buff_ref(item, catalog, n_actors)?;
    }
    folded_buffs.retain(|b| !remove_ids.contains(&b.instance_id));
    for item in &diff.upsert {
        if let Some(existing) = folded_buffs
            .iter_mut()
            .find(|b| b.instance_id == item.instance_id)
        {
            if existing == item {
                return Err("no-op buff_diff.upsert emitted".into());
            }
            *existing = item.clone();
        } else {
            folded_buffs.push(item.clone());
        }
    }
    Ok(())
}

fn apply_frame_deltas(
    frame: &GameStateFrame,
    catalog: &[GameBuffCatalogEntry],
    n_actors: usize,
    fold: &mut TimelineFold,
) -> Result<(), String> {
    let mut seen_slots = BTreeSet::new();
    for delta in &frame.actors {
        if !seen_slots.insert(delta.slot) {
            return Err("duplicate actor delta slot in timeline frame".into());
        }
        if !delta.has_changes() {
            return Err("empty actor delta emitted in timeline frame".into());
        }
        delta.apply_to(&mut fold.actors[delta.slot])?;
    }
    let mut seen_targets = BTreeSet::new();
    for target in &frame.targets {
        if !seen_targets.insert(target.target_index) {
            return Err("duplicate target slot in timeline frame".into());
        }
        fold.targets[target.target_index] = target.clone();
    }
    if let Some(diff) = &frame.buff_diff {
        validate_buff_diff(diff, catalog, n_actors, &mut fold.buffs)?;
    }
    if let Some(stamina) = &frame.stamina {
        fold.stamina = Some(stamina.clone());
    }
    fold.last_non_final_time_us = frame.time_us;
    Ok(())
}

impl super::GameScenarioReport {
    pub fn validate_state_timeline(&self) -> Result<(), String> {
        self.validate_clocks()?;
        if self.timer_pauses.is_none() {
            return Err("detailed sample is missing native timer-pause observations".into());
        }
        self.validate_timeline_impl()
    }

    fn validate_timeline_boundaries(&self) -> Result<TimelineFold, String> {
        if self.state_timeline.len() < 2 {
            return Err("state timeline requires initial and final boundary frames".into());
        }
        let first = self
            .state_timeline
            .first()
            .ok_or("missing initial state frame")?;
        let last = self
            .state_timeline
            .last()
            .ok_or("missing final state frame")?;
        let n_actors = self.final_actor_states.len();
        let n_targets = self.final_target_states.len();
        for def in &self.buff_catalog {
            if def.clock_basis != ntesim_core::kernel::timing::ClockDomain::World
                || def.id.is_empty()
                || def.source.is_empty()
                || def.owner_actor >= n_actors
                || def.max_stacks == 0
            {
                return Err("buff_catalog contains invalid definition".into());
            }
        }
        let initial_buffs = first
            .buffs
            .as_ref()
            .ok_or("initial state frame is missing buffs snapshot")?;
        let final_buffs = last
            .buffs
            .as_ref()
            .ok_or("final state frame is missing buffs snapshot")?;
        let initial_actors = first
            .actors
            .iter()
            .map(GameActorStateDelta::into_summary)
            .collect::<Result<Vec<_>, _>>()?;
        let final_actors = last
            .actors
            .iter()
            .map(GameActorStateDelta::into_summary)
            .collect::<Result<Vec<_>, _>>()?;
        if first.boundary != GameStateBoundary::Initial
            || first.time_us != 0
            || first.buff_diff.is_some()
            || last.boundary != GameStateBoundary::Final
            || last.time_us != self.absolute_elapsed_us
            || last.time_us != self.final_buff_state.time_us
            || last.buff_diff.is_some()
            || initial_actors.len() != n_actors
            || initial_actors.iter().enumerate().any(|(i, a)| a.slot != i)
            || first.targets.len() != n_targets
            || first
                .targets
                .iter()
                .enumerate()
                .any(|(j, t)| t.target_index != j)
            || final_actors != self.final_actor_states
            || last.targets != self.final_target_states
            || final_buffs != &self.final_buff_state.instances
            || last.active_actor_index != self.final_buff_state.active_actor_index
            || last.hits_end != self.hit_receipts.len()
            || last.healing_end != self.healing_receipts.len()
            || last.trace_end != self.trace.len()
        {
            return Err("state timeline does not reconcile native boundary states/receipts".into());
        }
        validate_unique_buff_ids(initial_buffs)?;
        for buff in initial_buffs {
            validate_buff_ref(buff, &self.buff_catalog, n_actors)?;
        }
        validate_unique_buff_ids(final_buffs)?;
        for buff in final_buffs {
            validate_buff_ref(buff, &self.buff_catalog, n_actors)?;
        }
        Ok(TimelineFold {
            actors: initial_actors,
            targets: first.targets.clone(),
            buffs: initial_buffs.clone(),
            stamina: first.stamina.clone(),
            last_non_final_time_us: first.time_us,
        })
    }

    fn validate_frame_receipts(
        &self,
        frame: &GameStateFrame,
        ends: (usize, usize, usize),
    ) -> Result<(), String> {
        let n_actors = self.final_actor_states.len();
        for hit in &self.hit_receipts[ends.0..frame.hits_end] {
            if hit.time_us != frame.time_us {
                return Err("state timeline differs from hit time".into());
            }
        }
        for heal in &self.healing_receipts[ends.1..frame.healing_end] {
            if heal.time_us != frame.time_us
                || heal.source_actor_index >= n_actors
                || heal.recipient_actor_index >= n_actors
                || heal.applied.checked_add(heal.excess) != Some(heal.gross)
                || heal.hp_before.checked_add(heal.applied) != Some(heal.hp_after)
            {
                return Err("state timeline healing settlement does not reconcile".into());
            }
        }
        Ok(())
    }

    fn reconcile_folded_timeline(
        &self,
        mut fold: TimelineFold,
        last: &GameStateFrame,
    ) -> Result<(), String> {
        let idle_gap = last.time_us > fold.last_non_final_time_us;
        let hp_max_changed_actors = if idle_gap {
            advance_buffs_to(
                &mut fold.buffs,
                &self.buff_catalog,
                self.relative_elapsed_us,
            )
        } else {
            BTreeSet::new()
        };
        if fold.buffs != self.final_buff_state.instances {
            return Err("folded timeline buff_diff does not match final_buff_state".into());
        }
        if !idle_gap {
            if fold.actors != self.final_actor_states {
                return Err("folded timeline actor deltas do not match final_actor_states".into());
            }
            if fold.targets != self.final_target_states {
                return Err(
                    "folded timeline target states do not match final_target_states".into(),
                );
            }
            if fold.stamina != last.stamina {
                return Err("folded timeline stamina does not match final frame".into());
            }
            return Ok(());
        }
        self.reconcile_idle_tail(&fold, last.time_us, &hp_max_changed_actors)
    }

    fn reconcile_idle_tail(
        &self,
        fold: &TimelineFold,
        last_time_us: u64,
        hp_max_changed_actors: &BTreeSet<usize>,
    ) -> Result<(), String> {
        for (folded_actor, final_actor) in fold.actors.iter().zip(&self.final_actor_states) {
            if folded_actor.slot != final_actor.slot
                || folded_actor.character_id != final_actor.character_id
                || folded_actor.attack_mode != final_actor.attack_mode
                || folded_actor.energy != final_actor.energy
                || folded_actor.energy_capacity != final_actor.energy_capacity
                || folded_actor.energy_conserved != final_actor.energy_conserved
                || folded_actor.initial_energy != final_actor.initial_energy
                || folded_actor.granted_energy != final_actor.granted_energy
                || folded_actor.spent_energy != final_actor.spent_energy
                || folded_actor.overflow_energy != final_actor.overflow_energy
                || folded_actor.charge_remainder != final_actor.charge_remainder
                || folded_actor.cycle != final_actor.cycle
                || folded_actor.resources != final_actor.resources
            {
                return Err(
                    "folded timeline actor non-expiry state does not match final_actor_states"
                        .into(),
                );
            }
            let mut folded_frac = folded_actor.fractional_resources.clone();
            let mut final_frac = final_actor.fractional_resources.clone();
            folded_frac.remove("hotori.timepiece");
            final_frac.remove("hotori.timepiece");
            if folded_frac != final_frac {
                return Err(
                    "folded timeline actor fractional resources do not match final_actor_states"
                        .into(),
                );
            }
            let surviving_shields = folded_actor
                .active_shields
                .iter()
                .filter(|s| s.expires_at_us > last_time_us)
                .cloned()
                .collect::<Vec<_>>();
            if surviving_shields != final_actor.active_shields {
                return Err("folded timeline shields do not match final_actor_states".into());
            }
            let (fireflies_stacks, active_fireflies) =
                fireflies_stacks_for_actor(&fold.buffs, &self.buff_catalog, folded_actor.slot)?;
            if fireflies_stacks != final_actor.fireflies_stacks
                || active_fireflies != final_actor.active_fireflies_stacks
            {
                return Err(
                    "folded timeline fireflies stacks do not match final_actor_states".into(),
                );
            }
            if hp_max_changed_actors.contains(&folded_actor.slot) {
                if final_actor.hp_max > folded_actor.hp_max
                    || final_actor.hp > folded_actor.hp
                    || final_actor.hp > final_actor.hp_max
                {
                    return Err(
                        "folded timeline HP clamp after buff expiry does not reconcile".into(),
                    );
                }
            } else if folded_actor.hp != final_actor.hp || folded_actor.hp_max != final_actor.hp_max
            {
                return Err("folded timeline actor HP does not match final_actor_states".into());
            }
        }
        for (j, (mut folded_target, final_target)) in fold
            .targets
            .iter()
            .cloned()
            .zip(&self.final_target_states)
            .enumerate()
        {
            folded_target.statuses =
                reconstruct_target_statuses(&fold.buffs, &self.buff_catalog, j)?;
            if &folded_target != final_target {
                return Err(
                    "folded timeline target state does not match final_target_states".into(),
                );
            }
        }
        Ok(())
    }

    fn validate_timeline_impl(&self) -> Result<(), String> {
        let mut fold = self.validate_timeline_boundaries()?;
        let n_actors = self.final_actor_states.len();
        let n_targets = self.final_target_states.len();
        let last = self
            .state_timeline
            .last()
            .ok_or("missing final state frame")?;

        let mut ends = (0, 0, 0);
        let mut time = 0;
        let mut ordinal = 0;
        let last_idx = self.state_timeline.len() - 1;
        for (idx, frame) in self.state_timeline.iter().enumerate() {
            if frame.time_us < time
                || frame.time_us > self.absolute_elapsed_us
                || frame.hits_end < ends.0
                || frame.hits_end > self.hit_receipts.len()
                || frame.healing_end < ends.1
                || frame.healing_end > self.healing_receipts.len()
                || frame.trace_end < ends.2
                || frame.trace_end > self.trace.len()
                || frame.active_actor_index >= n_actors
                || frame.actors.iter().any(|a| a.slot >= n_actors)
                || frame.targets.iter().any(|t| t.target_index >= n_targets)
            {
                return Err("state timeline order/identity/receipt prefix is invalid".into());
            }
            if idx == 0 {
                if frame.boundary != GameStateBoundary::Initial {
                    return Err("state timeline must start with Initial boundary".into());
                }
            } else if idx == last_idx {
                if frame.boundary != GameStateBoundary::Final {
                    return Err("state timeline must end with Final boundary".into());
                }
            } else {
                match frame.boundary {
                    GameStateBoundary::Event { ordinal: current } => {
                        if current <= ordinal || current > self.total_events_processed {
                            return Err("state timeline event ordinal is invalid".into());
                        }
                        ordinal = current;
                    }
                    _ => {
                        return Err(
                            "intermediate state timeline frame must have Event boundary".into()
                        );
                    }
                }
                if frame.buffs.is_some() {
                    return Err(
                        "intermediate state timeline frame must use buff_diff instead of buffs"
                            .into(),
                    );
                }
                apply_frame_deltas(frame, &self.buff_catalog, n_actors, &mut fold)?;
            }
            self.validate_frame_receipts(frame, ends)?;
            ends = (frame.hits_end, frame.healing_end, frame.trace_end);
            time = frame.time_us;
        }

        self.reconcile_folded_timeline(fold, last)
    }
}
