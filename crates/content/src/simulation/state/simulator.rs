use super::actor::*;
use super::status::*;
use super::target::*;
use crate::formula::Ratio;
use crate::simulation::{
    ExactFraction, GameActorStateSummary, GameEventPayload, GameEventReceipt, GameScenarioRequest,
    GameTargetStateSummary,
};
use std::collections::BTreeMap;
use std::hash::BuildHasher;

#[derive(Clone, Copy, Default)]
pub(crate) struct AbilityTimingOverrides {
    pub(crate) recovery_lock: Option<u64>,
    pub(crate) ready_at: Option<u64>,
}

impl AbilityTimingOverrides {
    pub(crate) fn recovery(lock: Option<u64>) -> Self {
        Self {
            recovery_lock: lock,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct GameSimulatorState {
    pub(crate) record_detail: bool,
    pub(crate) healing_receipts: Vec<crate::simulation::GameHealingReceipt>,
    pub(crate) seed: u64,
    pub(crate) damage_bonuses:
        Option<std::sync::Arc<[crate::simulation::hit::PreparedDamageBonuses]>>,
    pub(crate) origins: crate::simulation::origin::Origins,
    pub(crate) combat_stamina: Option<crate::simulation::CombatStaminaReport>,
    pub(crate) input_receipts: Vec<crate::simulation::PlayerInputReceipt>,
    pub(crate) evidence_catalog: Vec<ntesim_data::native_action_metadata::NativeEvidence>,
    pub(crate) metadata_catalog: Vec<crate::simulation::metadata::NormalizedActionPacketMetadata>,
    // Per-run normalized IDs are assigned on first use. The lookup key keeps
    // the complete source metadata except for the separately returned group.
    packet_metadata_ids: ntesim_core::FastHashMap<
        u64,
        Vec<(crate::simulation::metadata::ActionPacketMetadata, usize)>,
    >,
    pub(crate) receipt_model_assumptions: BTreeMap<String, String>,
    pub(crate) execution_model_assumptions: BTreeMap<String, String>,
    pub(crate) current_time_us: u64,
    pub(crate) mode_clock: ntesim_core::kernel::timing::RuntimeModeClock,
    pub(crate) cinematic_casts: BTreeMap<usize, crate::simulation::pause::CinematicCast>,
    pub(crate) replay_clocks: BTreeMap<
        ntesim_core::kernel::timing::PauseOwner,
        crate::characters::replay_clock::ReplayClock,
    >,
    pub(crate) timer_bindings: BTreeMap<usize, String>,
    pub(crate) active_actor_index: usize,
    /// Authored composition does not depend on which members are still alive.
    team_has_three_elements: Result<bool, String>,
    pub(crate) actors: Vec<ActorRuntime>,
    pub(crate) targets: Vec<TargetRuntime>,
    pub(crate) hits_settled: usize,
    pub(crate) rounded_damage_total: u128,
    pub(crate) structural_hp_changes: Vec<crate::simulation::GameStructuralHpChange>,
    pub(crate) atk_transfers: Vec<crate::simulation::model::GameAtkTransferReceipt>,
    pub(crate) max_proc_depth_reached: usize,
    pub(crate) buffs: ntesim_core::buffs::dispatch::BuffManager,
    pub(crate) reactions: crate::simulation::reactions::ReactionRuntime,
}

impl GameSimulatorState {
    pub(crate) fn team_has_three_elements(&self) -> Result<bool, String> {
        self.team_has_three_elements
            .as_ref()
            .copied()
            .map_err(Clone::clone)
    }

    pub(crate) fn add_rounded_damage(&mut self, amount: u64) -> Result<(), String> {
        self.rounded_damage_total = self
            .rounded_damage_total
            .checked_add(u128::from(amount))
            .ok_or("rounded damage total overflow")?;
        Ok(())
    }
    pub(crate) fn apply_buff(
        &mut self,
        owner_actor: usize,
        declaration: TimedBuffDeclaration,
    ) -> Result<(), String> {
        apply_timed_buff(
            &mut self.buffs,
            declaration,
            owner_actor,
            self.mode_clock.relative_us(),
        )
    }
    pub(in crate::simulation) fn prepare_ability_admission(
        &self,
        actor_index: usize,
        ability_id: &str,
        energy_cost: u64,
        recovery_us: u64,
        cooldown_us: u64,
    ) -> Result<
        ntesim_core::kernel::action::TimingAdmission,
        crate::simulation::rejection::AdmissionFailure,
    > {
        self.prepare_ability_with_timing(
            actor_index,
            ability_id,
            energy_cost,
            recovery_us,
            cooldown_us,
            AbilityTimingOverrides::default(),
        )
    }

    pub(in crate::simulation) fn prepare_ability_with_timing(
        &self,
        actor_index: usize,
        ability_id: &str,
        energy_cost: u64,
        recovery_us: u64,
        cooldown_us: u64,
        overrides: AbilityTimingOverrides,
    ) -> Result<
        ntesim_core::kernel::action::TimingAdmission,
        crate::simulation::rejection::AdmissionFailure,
    > {
        if actor_index != self.active_actor_index {
            return Err(crate::simulation::rejection::AdmissionFailure {
                reason: crate::simulation::GameAdmissionReason::ActorNotActive {
                    active_actor_index: self.active_actor_index,
                },
                message: format!(
                    "ACTOR_NOT_ACTIVE: actor {actor_index} cannot invoke ability while actor {} is on field",
                    self.active_actor_index
                ),
            });
        }
        let actor = &self.actors[actor_index];
        if actor.hp == 0 {
            return Err(crate::simulation::rejection::AdmissionFailure {
                reason: crate::simulation::GameAdmissionReason::Other {
                    message: "actor is defeated".into(),
                },
                message: format!("DEFEATED_ACTOR: actor {actor_index} cannot act at zero HP"),
            });
        }
        let ready_at_us = overrides.ready_at.unwrap_or_else(|| {
            actor
                .ability_cooldowns
                .get(ability_id)
                .copied()
                .unwrap_or(0)
        });
        let timing = ntesim_core::kernel::action::TimingState {
            last_start_us: actor.last_action_start_us,
            // Owner gates describe this input family. A live continuation may
            // gate a hold/Jump after the general next-action recovery is ready.
            locked_until_us: overrides.recovery_lock.unwrap_or(actor.locked_until_us),
            ready_at_us,
        }
        .prepare(ability_id, self.current_time_us, recovery_us, cooldown_us)
        .map_err(|error| {
            let reason = crate::simulation::rejection::timing_reason(&error);
            let message = match error {
            ntesim_core::kernel::action::ActionError::RecoveryLock { until } => {
                format!("ACTOR_LOCKED: actor {actor_index} is locked until {until}")
            }
            ntesim_core::kernel::action::ActionError::Cooldown { action, until } => {
                format!("ABILITY_ON_COOLDOWN: {action} on cooldown until {until}")
            }
            ntesim_core::kernel::action::ActionError::TimeReversal { last, now } => {
                format!(
                    "TIME_REVERSAL: actor {actor_index} last acted at {last}, current time is {now}"
                )
            }
            ntesim_core::kernel::action::ActionError::Overflow => {
                "actor action timing deadline overflow".into()
            }
            other => format!("ability timing admission failed: {other:?}"),
            };
            crate::simulation::rejection::AdmissionFailure { reason, message }
        })?;
        if actor.energy.current() < energy_cost {
            return Err(crate::simulation::rejection::AdmissionFailure {
                reason: crate::simulation::GameAdmissionReason::Energy {
                    required: energy_cost,
                    available: actor.energy.current(),
                },
                message: format!(
                    "RESOURCE_SHORTAGE: actor {actor_index} energy {} is insufficient for cost {energy_cost}",
                    actor.energy.current()
                ),
            });
        }
        Ok(timing)
    }

    pub(in crate::simulation) fn prepare_switch_admission(
        &self,
        cooldown_us: u64,
        recovery_override: Option<u64>,
    ) -> Result<
        ntesim_core::kernel::action::TimingAdmission,
        crate::simulation::rejection::AdmissionFailure,
    > {
        let actor = &self.actors[self.active_actor_index];
        ntesim_core::kernel::action::TimingState {
            last_start_us: actor.last_action_start_us,
            locked_until_us: recovery_override.unwrap_or(actor.locked_until_us),
            ready_at_us: actor
                .switch_cooldown_until_us
                .max(actor.switch_action_locked_until_us),
        }
        .prepare("switch_actor", self.current_time_us, 0, cooldown_us)
        .map_err(|error| {
            let reason = crate::simulation::rejection::timing_reason(&error);
            let message = match error {
                ntesim_core::kernel::action::ActionError::RecoveryLock { until } => {
                    format!(
                        "ACTOR_LOCKED: cannot switch while active actor is locked until {until}"
                    )
                }
                ntesim_core::kernel::action::ActionError::Cooldown { until, .. } => {
                    format!("SWITCH_ON_COOLDOWN: switch on cooldown until {until}")
                }
                ntesim_core::kernel::action::ActionError::TimeReversal { last, now } => {
                    format!("TIME_REVERSAL: actor last acted at {last}, current time is {now}")
                }
                other => format!("switch timing admission failed: {other:?}"),
            };
            crate::simulation::rejection::AdmissionFailure { reason, message }
        })
    }

    pub(crate) fn verify_invariants(&self) -> Result<(), String> {
        if let Some(stamina) = &self.combat_stamina {
            stamina.verify()?;
        }
        if self.active_actor_index >= self.actors.len() {
            return Err(format!(
                "active actor index {} exceeds team size {}",
                self.active_actor_index,
                self.actors.len()
            ));
        }

        for (i, a) in self.actors.iter().enumerate() {
            let max_hp = effective_hp_max(
                &self.buffs,
                a.hp_max_base,
                &a.hp_max_static_bonus,
                i,
                a.hp_max,
            )?;
            if a.hp > max_hp {
                return Err(format!("actor {i} HP {} exceeds max {max_hp}", a.hp));
            }
            if !a.energy_conserved() {
                return Err(format!(
                    "actor {i} energy conservation violated: initial ({}) + granted ({}) != current ({}) + spent ({}) + overflow ({})",
                    a.initial_energy,
                    a.granted_energy,
                    a.energy.current(),
                    a.spent_energy,
                    a.overflow_energy
                ));
            }
        }

        for (j, t) in self.targets.iter().enumerate() {
            if t.hp_current > t.hp_max {
                return Err(format!(
                    "target {j} HP {} exceeds max {}",
                    t.hp_current, t.hp_max
                ));
            }
        }

        Ok(())
    }

    pub(crate) fn intern_evidence(
        &mut self,
        evidence: &ntesim_data::native_action_metadata::NativeEvidence,
    ) -> usize {
        if let Some(index) = self
            .evidence_catalog
            .iter()
            .position(|item| item == evidence)
        {
            return index;
        }
        let index = self.evidence_catalog.len();
        self.evidence_catalog.push(evidence.clone());
        index
    }

    pub(crate) fn record_receipt_assumptions(
        &mut self,
        assumptions: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Vec<String>, String> {
        let mut ids = std::collections::BTreeSet::new();
        for (id, description) in assumptions {
            if self
                .receipt_model_assumptions
                .insert(id.clone(), description.clone())
                .is_some_and(|previous| previous != description)
            {
                return Err(format!(
                    "resolved model assumption has conflicting descriptions: {id}"
                ));
            }
            ids.insert(id);
        }
        Ok(ids.into_iter().collect())
    }

    pub(crate) fn intern_provenance(
        &mut self,
        provenance: &crate::simulation::metadata::MetadataProvenance,
    ) -> Result<crate::simulation::metadata::NormalizedMetadataProvenance, String> {
        match provenance {
            crate::simulation::metadata::MetadataProvenance::Native { evidence } => Ok(
                crate::simulation::metadata::NormalizedMetadataProvenance::Native {
                    evidence_id: self.intern_evidence(evidence),
                },
            ),
            crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
                id,
                description,
            } => {
                self.record_receipt_assumptions([(id.clone(), description.clone())])?;
                Ok(
                    crate::simulation::metadata::NormalizedMetadataProvenance::AcceptedModelHypothesis {
                        hypothesis_id: id.clone(),
                    },
                )
            }
        }
    }

    pub(crate) fn intern_action_packet_metadata(
        &mut self,
        metadata: &crate::simulation::metadata::ActionPacketMetadata,
    ) -> Result<(usize, Option<String>), String> {
        use crate::simulation::metadata::{
            NormalizedActionMetadata, NormalizedActionPacketBinding,
            NormalizedActionPacketMetadata, NormalizedPacketMetadata,
        };
        let hash = ntesim_core::fast_hash::FxBuildHasher.hash_one((
            &metadata.action.ability_id,
            &metadata.packet.damage_effect_id,
        ));
        if let Some((_, index)) = self.packet_metadata_ids.get(&hash).and_then(|entries| {
            entries.iter().find(|(key, _)| {
                key.action == metadata.action
                    && key.packet == metadata.packet
                    && key.binding.kind == metadata.binding.kind
                    && key.binding.initiating_ability_id == metadata.binding.initiating_ability_id
                    && key.binding.packet_ga_name == metadata.binding.packet_ga_name
                    && key.binding.provenance == metadata.binding.provenance
            })
        }) {
            return Ok((*index, metadata.binding.energy_group_id.clone()));
        }
        let action = NormalizedActionMetadata {
            categories: metadata.action.categories.clone(),
            gameplay_tags: metadata.action.gameplay_tags.clone(),
            ability_id: metadata.action.ability_id.clone(),
            native_ability_index: metadata.action.native_ability_index,
            input_role: metadata.action.input_role,
            input_role_provenance: metadata
                .action
                .input_role_provenance
                .as_ref()
                .map(|p| self.intern_provenance(p))
                .transpose()?,
            primary_slot: metadata.action.primary_slot,
            source_evidence_id: metadata
                .action
                .source_evidence
                .as_ref()
                .map(|e| self.intern_evidence(e)),
            identity_evidence_id: metadata
                .action
                .identity_evidence
                .as_ref()
                .map(|e| self.intern_evidence(e)),
            provenance: self.intern_provenance(&metadata.action.provenance)?,
        };
        let packet = NormalizedPacketMetadata {
            damage_effect_id: metadata.packet.damage_effect_id.clone(),
            native_ga_name: metadata.packet.native_ga_name.clone(),
            native_damage_type_ex: metadata.packet.native_damage_type_ex.clone(),
            native_damage_source_category: metadata.packet.native_damage_source_category.clone(),
            damage_kind: metadata.packet.damage_kind,
            element: metadata.packet.element.clone(),
            fixed_crit_rate: metadata.packet.fixed_crit_rate,
            fixed_crit_rate_provenance: metadata
                .packet
                .fixed_crit_rate_provenance
                .as_ref()
                .map(|p| self.intern_provenance(p))
                .transpose()?,
            unbal_value: metadata.packet.unbal_value,
            unbal_value_provenance: metadata
                .packet
                .unbal_value_provenance
                .as_ref()
                .map(|p| self.intern_provenance(p))
                .transpose()?,
            semantic_tags: metadata.packet.semantic_tags.clone(),
            semantic_provenance: metadata
                .packet
                .semantic_provenance
                .as_ref()
                .map(|p| self.intern_provenance(p))
                .transpose()?,
            source_evidence_id: metadata
                .packet
                .source_evidence
                .as_ref()
                .map(|e| self.intern_evidence(e)),
            provenance: self.intern_provenance(&metadata.packet.provenance)?,
            packet_kind: metadata.packet.packet_kind,
        };
        let binding = NormalizedActionPacketBinding {
            kind: metadata.binding.kind.clone(),
            initiating_ability_id: metadata.binding.initiating_ability_id.clone(),
            packet_ga_name: metadata.binding.packet_ga_name.clone(),
            provenance: metadata
                .binding
                .provenance
                .as_ref()
                .map(|p| self.intern_provenance(p))
                .transpose()?,
        };
        let normalized = NormalizedActionPacketMetadata {
            action,
            packet,
            binding,
        };
        let metadata_id = if let Some(index) = self
            .metadata_catalog
            .iter()
            .position(|existing| existing == &normalized)
        {
            index
        } else {
            let index = self.metadata_catalog.len();
            self.metadata_catalog.push(normalized);
            index
        };
        const MAX_PACKET_LOOKUPS: usize = 512;
        if self.metadata_catalog.len() <= MAX_PACKET_LOOKUPS {
            let mut key = metadata.clone();
            key.binding.energy_group_id = None;
            // Every key field except the execution-local group is represented
            // in the normalized catalog, so unique keys fit the same bound.
            self.packet_metadata_ids
                .entry(hash)
                .or_default()
                .push((key, metadata_id));
        }
        Ok((metadata_id, metadata.binding.energy_group_id.clone()))
    }
}

impl GameSimulatorState {
    #[cfg(test)]
    pub(crate) fn new(request: &GameScenarioRequest) -> Self {
        Self::new_with_seed(request, request.seed)
    }

    pub(crate) fn new_with_seed(request: &GameScenarioRequest, seed: u64) -> Self {
        let mut actors = Vec::with_capacity(request.actors.len());
        for a in &request.actors {
            let (hp_max_base, hp_max_static_bonus) = super::actor::initial_hp_projection(a)
                .expect("validated actor HP projection must remain representable");
            actors.push(ActorRuntime {
                combat_hp_losses: crate::registry::observes_vitals(request).then(Vec::new),
                aiming: false,
                hp: a.initial_hp,
                hp_max: a.initial_hp_max,
                hp_max_base,
                hp_max_static_bonus,
                shields: Vec::new(),
                energy: ntesim_core::kernel::state::Pool::new(a.initial_energy, a.energy_capacity)
                    .expect("scenario validation requires a legal initial energy pool"),
                cycle: ntesim_core::kernel::state::Pool::new(
                    a.initial_cycle,
                    crate::simulation::reactions::CYCLE_CAPACITY,
                )
                .expect("scenario validation requires a legal initial Cycle pool"),
                locked_until_us: 0,
                action_clock: None,
                world_time_us: 0,
                cycle_ledger: crate::simulation::cycle::Ledger::new(a.initial_cycle),
                switch_cooldown_until_us: 0,
                switch_action_locked_until_us: 0,
                ability_cooldowns: Default::default(),
                last_action_start_us: 0,
                field_time_absolute_us: 0,
                field_time_relative_us: 0,
                character: crate::characters::RuntimeState::new(&a.character_id),
                equipment: crate::gear::initialize_runtime(a, actors.len(), seed)
                    .expect("validated equipment runtime initialization must succeed"),
                initial_energy: a.initial_energy,
                packet_energy_groups: Default::default(),
                single_support_contact: None,
                single_support_start_us: None,
                packet_group_ordinal: 0,
                granted_energy: 0,
                spent_energy: 0,
                overflow_energy: 0,
                charge_multiplier: a
                    .gear_procs
                    .iter()
                    .find_map(|p| match p {
                        crate::simulation::GearProcConfig::ChargeEfficiency { multiplier } => {
                            Some(*multiplier)
                        }
                        _ => None,
                    })
                    .unwrap_or(Ratio {
                        numerator: 1,
                        denominator: 1,
                    }),
                charge_remainder: ExactFraction {
                    numerator: 0,
                    denominator: 1,
                },
            });
        }

        let mut targets = Vec::with_capacity(request.targets.len());
        for t in &request.targets {
            targets.push(TargetRuntime {
                original_attack: t.original_atk,
                attack: t.original_atk.map(|value| ExactFraction {
                    numerator: value.into(),
                    denominator: 1,
                }),
                control_immune: t.control_immune,
                base_defense: t.base_defense,
                is_boss: t.is_boss,
                hp_current: t.hp_current,
                hp_max: t.hp_max,
                defense: t.defense,
                resistance: t.resistance,
                airborne_until_us: t.airborne_until_us,
                break_state: t.break_profile.as_ref().map(|profile| TargetBreakRuntime {
                    original_capacity: ExactFraction::from_ratio(profile.capacity)
                        .expect("validated target original break capacity"),
                    incoming_efficiency_multiplier: ExactFraction {
                        numerator: 1,
                        denominator: 1,
                    },
                    capacity: ExactFraction::from_ratio(profile.capacity)
                        .expect("validated target break capacity"),
                    gauge: ExactFraction::from_ratio(profile.current_gauge)
                        .expect("validated target break gauge"),
                    broken_duration_us: profile.broken_duration_us,
                    broken_until_us: None,
                    generation: 0,
                    broken_by_actor: None,
                    rose_extension_used: false,
                }),
            });
        }

        Self {
            record_detail: true,
            healing_receipts: Vec::new(),
            seed,
            damage_bonuses: None,
            origins: crate::simulation::origin::Origins::default(),
            input_receipts: Vec::new(),
            evidence_catalog: Vec::new(),
            metadata_catalog: Vec::new(),
            packet_metadata_ids: Default::default(),
            receipt_model_assumptions: BTreeMap::new(),
            combat_stamina: request
                .combat_stamina
                .as_ref()
                .map(crate::simulation::CombatStaminaReport::new),
            execution_model_assumptions: BTreeMap::new(),
            current_time_us: 0,
            mode_clock: Default::default(),
            cinematic_casts: Default::default(),
            replay_clocks: Default::default(),
            timer_bindings: Default::default(),
            active_actor_index: request.active_actor_index,
            team_has_three_elements: crate::gear::source_stats::team_has_three_elements(request),
            actors,
            targets,
            hits_settled: 0,
            rounded_damage_total: 0,
            structural_hp_changes: Vec::new(),
            atk_transfers: Vec::new(),
            max_proc_depth_reached: 0,
            buffs: ntesim_core::buffs::dispatch::BuffManager::default(),
            reactions: crate::simulation::reactions::ReactionRuntime::new(seed),
        }
    }

    pub(crate) fn expire_buffs(&mut self, trace: &mut Vec<GameEventReceipt>) {
        let mut expired_shields = Vec::new();
        for (actor_index, actor) in self.actors.iter_mut().enumerate() {
            let expired = actor.expire_shields(self.current_time_us);
            for (shield_id, expires_at_us) in expired {
                expired_shields.push((expires_at_us, actor_index, shield_id));
            }
        }
        expired_shields.sort();
        for (time_us, actor_index, shield_id) in expired_shields {
            trace.push(GameEventReceipt {
                time_us,
                phase: "BuffTickOrExpiry".into(),
                event_kind: "actor_shield_expired".into(),
                active_actor: self.active_actor_index,
                hit_index: None,
                from_actor: None,
                to_actor: Some(actor_index),
                energy_change: None,
                energy_fanout: None,
                data: Some(GameEventPayload {
                    shield_id: Some(shield_id),
                    ..Default::default()
                }),
            });
        }
        let gear_tracker = crate::gear::pre_expire_gear_buffs(&self.buffs, self.actors.len());
        let has_hp_suit_buff = self
            .buffs
            .instances()
            .iter()
            .any(|b| b.id == crate::sets::hp_suit::BUFF_ID);
        let mut max_hp_before = [0u64; 4];
        if has_hp_suit_buff {
            let count = self.actors.len().min(4);
            for (actor, hp_before) in max_hp_before.iter_mut().enumerate().take(count) {
                *hp_before = effective_hp_max(
                    &self.buffs,
                    self.actors[actor].hp_max_base,
                    &self.actors[actor].hp_max_static_bonus,
                    actor,
                    self.actors[actor].hp_max,
                )
                .unwrap_or(0);
            }
        }
        self.buffs.advance_time(self.mode_clock.relative_us());
        crate::gear::post_expire_gear_buffs(
            gear_tracker,
            &self.buffs,
            self.current_time_us,
            self.active_actor_index,
            trace,
        );
        if has_hp_suit_buff {
            let count = self.actors.len().min(4);
            for (actor, &before) in max_hp_before.iter().enumerate().take(count) {
                let effective = match effective_hp_max(
                    &self.buffs,
                    self.actors[actor].hp_max_base,
                    &self.actors[actor].hp_max_static_bonus,
                    actor,
                    self.actors[actor].hp_max,
                ) {
                    Ok(effective) => effective,
                    Err(_) => continue,
                };
                if effective < before && self.actors[actor].hp > effective {
                    self.actors[actor].hp = effective;
                    trace.push(GameEventReceipt {
                        time_us: self.current_time_us,
                        phase: "BuffTickOrExpiry".into(),
                        event_kind: "max_hp_stack_expired".into(),
                        active_actor: self.active_actor_index,
                        hit_index: None,
                        from_actor: None,
                        to_actor: Some(actor),
                        energy_change: None,
                        energy_fanout: None,
                        data: Some(GameEventPayload {
                            hp_max_before: Some(before),
                            hp_max: Some(effective),
                            reason: Some("hp_clamped".into()),
                            ..Default::default()
                        }),
                    });
                }
            }
        }
    }

    pub(crate) fn actor_summaries(
        &self,
        request: &GameScenarioRequest,
    ) -> Result<Vec<GameActorStateSummary>, String> {
        let mut actor_summaries = Vec::with_capacity(self.actors.len());
        for (i, a) in self.actors.iter().enumerate() {
            actor_summaries.push(GameActorStateSummary {
                slot: i,
                character_id: request.actors[i].character_id.clone(),
                attack_mode: crate::characters::player_inputs::attack_mode(a).into(),
                hp: a.hp,
                hp_max: effective_hp_max(
                    &self.buffs,
                    a.hp_max_base,
                    &a.hp_max_static_bonus,
                    i,
                    a.hp_max,
                )
                .unwrap_or(a.hp_max),
                energy: a.energy.current(),
                energy_capacity: a.energy.capacity(),
                active_shields: a
                    .shields
                    .iter()
                    .filter(|shield| shield.expires_at_us > self.current_time_us)
                    .map(|shield| crate::simulation::GameActorShieldSummary {
                        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
                        id: shield.id.clone(),
                        source: shield.source.clone(),
                        remaining_hp: shield.remaining_hp,
                        max_hp: shield.max_hp,
                        expires_at_us: shield.expires_at_us,
                    })
                    .collect(),
                fireflies_stacks: crate::gear::active_stacks(&self.buffs, i),
                active_fireflies_stacks: crate::gear::stacks(&self.buffs, i)?,
                energy_conserved: a.energy_conserved(),
                initial_energy: a.initial_energy,
                granted_energy: a.granted_energy,
                spent_energy: a.spent_energy,
                overflow_energy: a.overflow_energy,
                charge_remainder: (!a.charge_remainder.is_zero()).then_some(a.charge_remainder),
                cycle: a.cycle_ledger.summary(&a.cycle),
                resources: crate::registry::resource_summaries(a),
                fractional_resources: crate::registry::fractional_resource_summaries(
                    a,
                    self.current_time_us,
                )?,
            });
        }
        Ok(actor_summaries)
    }

    pub(crate) fn target_summaries(&self) -> Result<Vec<GameTargetStateSummary>, String> {
        use ntesim_core::buffs::dispatch::BuffDuration;
        let mut target_summaries = Vec::with_capacity(self.targets.len());
        for (j, t) in self.targets.iter().enumerate() {
            let statuses = self
                .buffs
                .instances()
                .iter()
                .filter(|buff| buff.recipient.matches_target_index(j))
                .map(|buff| {
                    let expires_at_us = match &buff.duration {
                        BuffDuration::Timed { expires_at_us, .. } => *expires_at_us,
                        BuffDuration::Permanent => {
                            return Err("target status cannot have permanent duration".into());
                        }
                    };
                    let resistance_reductions = buff
                        .stat_modifiers
                        .iter()
                        .filter_map(|(key, value)| {
                            key.strip_prefix("resistance_reduction:").map(|element| {
                                (
                                    element.to_owned(),
                                    ExactFraction {
                                        numerator: value.numerator.into(),
                                        denominator: value.denominator.into(),
                                    },
                                )
                            })
                        })
                        .collect();
                    Ok(crate::simulation::model::GameTargetStatusSummary {
                        clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
                        source_actor_index: buff.owner_actor,
                        id: buff.id.clone(),
                        expires_at_us,
                        semantic_tags: buff.semantic_tags.clone(),
                        stacks: buff.stacks,
                        resistance_reductions,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let attack = t
                .attack
                .zip(t.original_attack)
                .map(|(attack, original)| {
                    let reduction = crate::runtime_api::target_stat_modifier(
                        &self.buffs,
                        j,
                        self.mode_clock.relative_us(),
                        "atk_percent_reduction",
                    )?;
                    attack.sub(ExactFraction::new(original.into(), 1)?.mul(reduction)?)
                })
                .transpose()?;
            target_summaries.push(GameTargetStateSummary {
                attack,
                control_immune: t.control_immune,
                base_defense: t.base_defense,
                target_index: j,
                is_boss: t.is_boss,
                hp_current: t.hp_current,
                hp_max: t.hp_max,
                defense: t.defense,
                airborne_until_us: t.airborne_until_us,
                statuses,
                break_state: t.break_state.as_ref().map(|break_state| {
                    crate::simulation::model::GameTargetBreakStateSummary {
                        capacity: break_state.capacity,
                        gauge: break_state.gauge,
                        broken_duration_us: break_state.broken_duration_us,
                        broken_until_us: break_state.broken_until_us,
                        generation: break_state.generation,
                    }
                }),
            });
        }
        Ok(target_summaries)
    }
}
