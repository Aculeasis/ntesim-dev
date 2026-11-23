//! Successful batch trials retain numeric observations. Detailed runs retain
//! authored inputs, replay boundaries and the complete public receipt log.
use super::execution::PreparedExecution;
use crate::simulation::{
    GameActorDamageReceipt, GameEnergyReceipt, GameEnergySource, GameEventReceipt,
    GameScenarioRequest, PlayerInput, SourceScenarioReport, metadata::PrimaryActionSlot,
};
use ntesim_contracts::{Quality, TimeBasisPair};
use std::collections::{BTreeMap, BTreeSet};

// Keep the regular batch result inline to avoid an allocation for every seed.
#[allow(clippy::large_enum_variant)]
pub enum SourceTrialOutcome {
    Completed(CompletedSourceTrial),
    Failed(Box<SourceScenarioReport>),
}

#[derive(Debug)]
pub struct CompletedSourceTrial {
    pub seed: u64,
    pub quality: Quality,
    pub relative_elapsed_us: u64,
    pub absolute_elapsed_us: u64,
    pub total_damage: u128,
    pub hits: usize,
    pub events: usize,
    pub damage_by_actor: Vec<(u128, u64)>,
    pub damage_by_source: BTreeMap<(usize, String, String), (u128, u64)>,
    pub damage_by_target: Vec<u128>,
    pub bucket_us: u64,
    pub bins: Vec<u128>,
    pub actions: Vec<BTreeMap<&'static str, u64>>,
    pub energy: BTreeMap<(usize, GameEnergySource, String), (f64, f64, f64)>,
    pub final_energy: Vec<u64>,
    pub healing: [f64; 3],
    pub absorbed: f64,
    pub available_shield_hp: TimeBasisPair<f64>,
    pub reaction_counts: BTreeMap<String, u32>,
    pub field_times: Vec<TimeBasisPair<u64>>,
}

/// These maps own each source or trace kind only once per trial. Integer
/// damage bins can be merged without changing rounding or summation order.
pub(super) struct BatchReceipts {
    hits_observed: usize,
    input_time_end_us: u64,
    by_actor: Vec<(u128, u64)>,
    by_target: Vec<u128>,
    by_source: Vec<BTreeMap<String, BTreeMap<String, (u128, u64)>>>,
    bucket_us: u64,
    bins: Vec<u128>,
    actions: Vec<BTreeMap<&'static str, u64>>,
    admission_roles: BTreeMap<usize, Option<PrimaryActionSlot>>,
    invocations: Vec<(usize, usize)>,
    player_controls: BTreeSet<usize>,
    energy: BTreeMap<(usize, GameEnergySource), BTreeMap<String, (f64, f64, f64)>>,
    healing: [f64; 3],
    absorbed: f64,
}

fn add_damage(total: &mut u128, amount: u128) -> Result<(), String> {
    *total = total
        .checked_add(amount)
        .ok_or("rounded observation overflow")?;
    Ok(())
}

fn count_hit(totals: &mut (u128, u64), amount: u128) -> Result<(), String> {
    add_damage(&mut totals.0, amount)?;
    totals.1 = totals.1.checked_add(1).ok_or("hit count overflow")?;
    Ok(())
}

fn shield_absorption(deliveries: &[GameActorDamageReceipt]) -> f64 {
    deliveries.iter().fold(0., |total, delivery| {
        total + absorption(delivery) + shield_absorption(&delivery.redirected)
    })
}

fn absorption(delivery: &GameActorDamageReceipt) -> f64 {
    delivery
        .absorbed_by_shields
        .iter()
        .map(|shield| shield.amount as f64)
        .sum()
}

impl BatchReceipts {
    pub(super) fn new(request: &GameScenarioRequest) -> Self {
        let actors = request.actors.len();
        Self {
            hits_observed: 0,
            input_time_end_us: 0,
            by_actor: vec![(0, 0); actors],
            by_target: vec![0; request.targets.len()],
            by_source: vec![BTreeMap::new(); actors],
            bucket_us: 1_000_000,
            bins: Vec::new(),
            actions: vec![BTreeMap::new(); actors],
            admission_roles: BTreeMap::new(),
            invocations: Vec::new(),
            player_controls: BTreeSet::new(),
            energy: BTreeMap::new(),
            healing: [0.; 3],
            absorbed: 0.,
        }
    }

    pub(super) fn observe_input_time(
        &mut self,
        interval: &crate::simulation::GameTimeInterval,
    ) -> Result<(), String> {
        if interval.start_us != self.input_time_end_us
            || interval.end_us <= interval.start_us
            || interval.duration_us != interval.end_us - interval.start_us
        {
            return Err(
                "Input-time intervals do not form an exclusive combat-time partition.".into(),
            );
        }
        self.input_time_end_us = interval.end_us;
        Ok(())
    }

    fn fit_bins(&mut self, time_us: u64) -> Result<(), String> {
        while time_us.div_ceil(self.bucket_us) > 240 {
            self.bucket_us = self
                .bucket_us
                .checked_mul(2)
                .ok_or("absolute timeline bucket overflow")?;
            for index in 0..self.bins.len().div_ceil(2) {
                let mut damage = self.bins[index * 2];
                if let Some(next) = self.bins.get(index * 2 + 1) {
                    add_damage(&mut damage, *next)?;
                }
                self.bins[index] = damage;
            }
            self.bins.truncate(self.bins.len().div_ceil(2));
        }
        Ok(())
    }

    fn count_action(&mut self, slot: usize, action: &'static str) -> Result<(), String> {
        let count = self
            .actions
            .get_mut(slot)
            .ok_or("invalid admitted actor")?
            .entry(action)
            .or_default();
        *count = count.checked_add(1).ok_or("action count overflow")?;
        Ok(())
    }

    fn energy_grant(&mut self, kind: &str, grant: &crate::simulation::GameEnergyGrantReceipt) {
        let by_kind = self
            .energy
            .entry((grant.recipient_actor_index, grant.source))
            .or_default();
        let (gross, retained, overflow) = match by_kind.get_mut(kind) {
            Some(totals) => totals,
            None => by_kind.entry(kind.into()).or_default(),
        };
        *gross += grant.gross as f64;
        *retained += grant.retained as f64;
        *overflow += grant.overflow as f64;
    }

    fn record_trace(&mut self, event: &GameEventReceipt) {
        if let Some(GameEnergyReceipt::Grant(grant)) = &event.energy_change {
            self.energy_grant(&event.event_kind, grant);
        }
        if let Some(fanout) = &event.energy_fanout {
            for grant in fanout.grants() {
                self.energy_grant(&event.event_kind, &grant);
            }
        }
        if let Some(spend) = event.energy_spend()
            && event.event_kind == "invoke_ability"
        {
            self.invocations
                .push((spend.command_index, spend.recipient_actor_index));
        }
    }

    pub(super) fn observe_boundary(
        &mut self,
        state: &mut crate::simulation::state::GameSimulatorState,
        hits: &[crate::simulation::GameHitReceipt],
        trace: &mut Vec<GameEventReceipt>,
        admissions: &mut Vec<crate::simulation::ActionMetadataOccurrence>,
        incoming: &mut Vec<GameActorDamageReceipt>,
    ) -> Result<(), String> {
        let time_us = state.current_time_us;
        for index in self.hits_observed..hits.len() {
            let hit = &hits[index];
            if hit.time_us != time_us {
                return Err("state timeline differs from hit time".into());
            }
            crate::simulation::model::validate_hit_parent(hits, index)?;
            let amount = u128::from(hit.rounded_damage);
            count_hit(
                self.by_actor
                    .get_mut(hit.actor_index)
                    .ok_or("invalid damage actor")?,
                amount,
            )?;
            add_damage(
                self.by_target
                    .get_mut(hit.target_index)
                    .ok_or("invalid damage target")?,
                amount,
            )?;
            let abilities = &mut self.by_source[hit.actor_index];
            let effects = match abilities.get_mut(hit.ability_id.as_str()) {
                Some(effects) => effects,
                None => abilities.entry(hit.ability_id.clone()).or_default(),
            };
            let damage = match effects.get_mut(hit.damage_effect_id.as_str()) {
                Some(damage) => damage,
                None => effects.entry(hit.damage_effect_id.clone()).or_default(),
            };
            count_hit(damage, amount)?;
            self.fit_bins(time_us)?;
            let bin = (time_us / self.bucket_us) as usize;
            if bin >= self.bins.len() {
                self.bins.resize(bin + 1, 0);
            }
            add_damage(&mut self.bins[bin], amount)?;
        }
        self.hits_observed = hits.len();
        for event in trace.drain(..) {
            self.record_trace(&event);
        }
        for occurrence in admissions.drain(..) {
            let role = state
                .metadata_catalog
                .get(occurrence.metadata_id)
                .and_then(|metadata| metadata.action.primary_slot);
            for command in occurrence.command_indices {
                self.admission_roles.entry(command).or_insert(role);
            }
        }
        for input in state.input_receipts.drain(..) {
            let kind = match input.input {
                PlayerInput::Special => "special",
                PlayerInput::Dash => "dash",
                PlayerInput::Arc => "arc",
                PlayerInput::Jump => "jump",
            };
            self.player_controls.insert(input.command_index);
            self.count_action(input.actor_index, kind)?;
        }
        let actors = state.actors.len();
        for heal in state.healing_receipts.drain(..) {
            if heal.time_us != time_us
                || heal.source_actor_index >= actors
                || heal.recipient_actor_index >= actors
                || heal.applied.checked_add(heal.excess) != Some(heal.gross)
                || heal.hp_before.checked_add(heal.applied) != Some(heal.hp_after)
            {
                return Err("state timeline healing settlement does not reconcile".into());
            }
            self.healing[0] += heal.gross as f64;
            self.healing[1] += heal.applied as f64;
            self.healing[2] += heal.excess as f64;
        }
        for delivery in incoming.drain(..) {
            self.absorbed =
                self.absorbed + absorption(&delivery) + shield_absorption(&delivery.redirected);
        }
        Ok(())
    }

    pub(super) fn finish(
        mut self,
        execution: &PreparedExecution,
        request: &GameScenarioRequest,
        seed: u64,
        quality: Quality,
    ) -> Result<CompletedSourceTrial, String> {
        let absolute_elapsed_us = execution.state.current_time_us;
        let relative_elapsed_us = execution.state.mode_clock.relative_us();
        if self.input_time_end_us != absolute_elapsed_us {
            return Err("Input-time partition differs from elapsed combat time.".into());
        }
        let field_times = execution.field_times();
        crate::simulation::model::validate_elapsed(
            relative_elapsed_us,
            absolute_elapsed_us,
            request.relative_duration_us,
            request.absolute_limit_us,
            execution.state.actors.len(),
            &field_times,
        )?;
        if absolute_elapsed_us == 0
            || execution.state.mode_clock.absolute_us() != absolute_elapsed_us
            || execution.state.actors.len() != request.actors.len()
            || execution.state.targets.len() != request.targets.len()
        {
            return Err("missing actor states or positive execution duration".into());
        }
        let final_energy = execution.state.actors.iter().enumerate().map(|(slot, actor)| {
            let sums = self.energy.iter().filter(|((owner, _), _)| *owner == slot)
                .flat_map(|(_, kinds)| kinds.values())
                .fold((0., 0., 0.), |totals, energy| (totals.0 + energy.0, totals.1 + energy.1, totals.2 + energy.2));
            let retained = actor.granted_energy.checked_sub(actor.overflow_energy)
                .ok_or("energy overflow exceeds granted energy")?;
            if actor.initial_energy != request.actors[slot].initial_energy
                || actor.energy.current() > actor.energy.capacity()
                || sums != (actor.granted_energy as f64, retained as f64, actor.overflow_energy as f64)
            {
                return Err("Energy source trace does not reconcile with gross/retained/overflow ledger.".to_string());
            }
            Ok(actor.energy.current())
        }).collect::<Result<Vec<_>, _>>()?;
        self.fit_bins(absolute_elapsed_us)?;
        let count = absolute_elapsed_us.div_ceil(self.bucket_us) as usize;
        if self.bins.len() > count {
            let last = self.bins.pop().expect("elapsed boundary bin");
            add_damage(&mut self.bins[count - 1], last)?;
        }
        self.bins.resize(count, 0);
        let total_damage = execution.state.rounded_damage_total;
        if self
            .bins
            .iter()
            .try_fold(0_u128, |total, &value| total.checked_add(value))
            != Some(total_damage)
            || self.hits_observed != execution.state.hits_settled
        {
            return Err("damage bins and hit count differ from settlement ledger".into());
        }
        for (command, slot) in std::mem::take(&mut self.invocations) {
            if self.player_controls.contains(&command) {
                continue;
            }
            let kind = match self.admission_roles.get(&command).copied().flatten() {
                Some(PrimaryActionSlot::Normal) => "attack",
                Some(PrimaryActionSlot::Skill) => "skill",
                Some(PrimaryActionSlot::Ultimate) => "burst",
                None => "invoke",
            };
            self.count_action(slot, kind)?;
        }
        Ok(CompletedSourceTrial {
            seed,
            quality,
            relative_elapsed_us,
            absolute_elapsed_us,
            total_damage,
            hits: execution.state.hits_settled,
            events: execution.events_processed,
            damage_by_actor: self.by_actor,
            damage_by_source: self
                .by_source
                .into_iter()
                .enumerate()
                .flat_map(|(actor, abilities)| {
                    abilities.into_iter().flat_map(move |(ability, effects)| {
                        effects
                            .into_iter()
                            .map(move |(effect, sums)| ((actor, ability.clone(), effect), sums))
                    })
                })
                .collect(),
            damage_by_target: self.by_target,
            bucket_us: self.bucket_us,
            bins: self.bins,
            actions: self.actions,
            energy: self
                .energy
                .into_iter()
                .flat_map(|((slot, source), kinds)| {
                    kinds
                        .into_iter()
                        .map(move |(kind, sums)| ((slot, source, kind), sums))
                })
                .collect(),
            final_energy,
            healing: self.healing,
            absorbed: self.absorbed,
            available_shield_hp: execution.available_shield_hp(),
            reaction_counts: execution.reaction_counts(),
            field_times,
        })
    }
}
