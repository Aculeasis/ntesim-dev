use super::program::*;
use super::report::*;
use crate::execution::ExecutionLimits;
use crate::rng::Rng;
use crate::simulation::events::{GameEventData, HitPacket};
use crate::simulation::state::GameSimulatorState;
use crate::simulation::timer_queue::{EventQueue as BinaryHeap, HeadKind};
use crate::simulation::{
    ExactFraction, GameEventPayload, GameEventReceipt, GameScenarioReport, GameScenarioRequest,
};
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality};
use std::collections::BTreeMap;

pub(crate) struct PreparedExecution {
    pub(super) live_source: bool,
    pub(super) timeline: crate::simulation::timeline::Recorder,
    pub(super) batch: Option<super::batch::BatchReceipts>,
    pub(crate) limit_reached: Option<crate::simulation::GameExecutionLimit>,
    pub(super) input_time: crate::simulation::GameTimeLedger,
    pub(super) active_input: Option<usize>,
    pub(crate) observe_vitals: bool,
    pub(crate) state: GameSimulatorState,
    pub(crate) queue: BinaryHeap,
    pub(crate) seq: u64,
    pub(crate) parked_energy_actions: Vec<crate::simulation::events::ScheduledGameEvent>,
    pub(crate) pending_energy_wait: Option<PendingLiveEnergyWait>,
    pub(crate) unqueued_energy_waits: usize,
    pub(crate) rng: Option<Rng>,
    pub(crate) hit_receipts: Vec<crate::simulation::GameHitReceipt>,
    pub(crate) incoming_damage_receipts: Vec<crate::simulation::GameActorDamageReceipt>,
    pub(crate) hp_cost_receipts: Vec<crate::simulation::GameActorHpCostReceipt>,
    pub(crate) interrupt_receipts: Vec<crate::simulation::GameActorInterruptReceipt>,
    pub(crate) parry_receipts: Vec<crate::simulation::GameActorParryReceipt>,
    pub(super) action_parries: BTreeMap<usize, usize>,
    pub(crate) combat_transition_receipts: Vec<crate::simulation::GameActorCombatTransitionReceipt>,
    pub(crate) live_branch_receipts: Vec<crate::simulation::GameLiveBranchReceipt>,
    pub(crate) live_energy_wait_receipts: Vec<crate::simulation::GameLiveEnergyWaitReceipt>,
    pub(crate) action_metadata_ledger: Vec<crate::simulation::ActionMetadataOccurrence>,
    pub(crate) trace: Vec<GameEventReceipt>,
    pub(crate) pending_events: usize,
    pub(crate) halt_diagnostic: Option<String>,
    pub(crate) rejected_action: Option<crate::simulation::GameRejectedAction>,
    pub(crate) success: bool,
    pub(crate) execution_completed: bool,
    pub(crate) events_processed: usize,
    pub(crate) accumulated_shield_hp_us: u128,
    pub(crate) accumulated_relative_shield_hp_us: u128,
}

impl PreparedExecution {
    pub(crate) fn new(
        request: &GameScenarioRequest,
        seed: u64,
        branches: &[crate::simulation::GameLiveBranch],
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
    ) -> Result<(Self, BTreeMap<String, String>), String> {
        Self::new_with_timeline_option(request, seed, branches, energy_waits, true)
    }

    pub(crate) fn new_with_timeline_option(
        request: &GameScenarioRequest,
        seed: u64,
        branches: &[crate::simulation::GameLiveBranch],
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
        record_timeline: bool,
    ) -> Result<(Self, BTreeMap<String, String>), String> {
        let rng = request
            .stochastic_crit
            .as_ref()
            .map(|policy| Rng::new(seed, policy.actor_sequences.clone()))
            .transpose()?;
        let mut state = GameSimulatorState::new_with_seed(request, seed);
        for (config, actor) in request.actors.iter().zip(&mut state.actors) {
            crate::registry::initialize_actor(config, actor, request)?;
        }
        for (actor_index, config) in request.actors.iter().enumerate() {
            crate::registry::initialize_team_effect(
                config,
                actor_index,
                &mut state.buffs,
                request,
                &mut state.execution_model_assumptions,
            )?;
        }
        state.record_detail = record_timeline;
        state.mode_clock = ntesim_core::kernel::timing::RuntimeModeClock::new(record_timeline);
        let policy = ntesim_contracts::HypothesisId::CombatRelativeClockPolicyV1;
        state
            .execution_model_assumptions
            .insert(policy.as_str().into(), policy.description().into());
        state.verify_invariants()?;

        let (mut queue, mut seq, parked_energy_actions) =
            crate::simulation::queue::initial_queue(request, branches, energy_waits);
        let mut trace = Vec::new();
        let lifecycle_assumptions =
            crate::gear::initialize(&mut crate::runtime_api::CallbackContext {
                state: &mut state,
                request,
                queue: &mut queue,
                sequence: &mut seq,
                trace: &mut trace,
            })?;
        {
            let active_index = state.active_actor_index;
            let mut context = crate::runtime_api::CallbackContext {
                state: &mut state,
                request,
                queue: &mut queue,
                sequence: &mut seq,
                trace: &mut trace,
            };
            crate::registry::initialize_combat(&mut context)?;
            crate::registry::on_field_entry(
                &mut context,
                active_index,
                crate::simulation::GameCausalTrigger::InitialState,
            )?;
        }
        let mut timeline = crate::simulation::timeline::Recorder::default();
        if record_timeline {
            timeline.observe(
                &state,
                request,
                crate::simulation::GameStateBoundary::Initial,
                0,
                trace.len(),
            )?;
        }
        let execution = Self {
            live_source: request.source_exhausted
                && request.action_admission_order
                    == crate::simulation::GameActionAdmissionOrder::AfterSettledEvents,
            timeline,
            batch: (!record_timeline).then(|| super::batch::BatchReceipts::new(request)),
            limit_reached: None,
            input_time: Default::default(),
            active_input: None,
            observe_vitals: crate::registry::observes_vitals(request),
            state,
            queue,
            seq,
            parked_energy_actions,
            pending_energy_wait: None,
            unqueued_energy_waits: energy_waits.len().saturating_sub(1),
            rng,
            hit_receipts: Vec::new(),
            incoming_damage_receipts: Vec::new(),
            hp_cost_receipts: Vec::new(),
            interrupt_receipts: Vec::new(),
            parry_receipts: Vec::new(),
            action_parries: super::parry::index(request),
            combat_transition_receipts: Vec::new(),
            live_branch_receipts: Vec::new(),
            live_energy_wait_receipts: Vec::new(),
            action_metadata_ledger: Vec::new(),
            trace,
            pending_events: 0,
            halt_diagnostic: None,
            rejected_action: None,
            success: true,
            execution_completed: true,
            events_processed: 0,
            accumulated_shield_hp_us: 0,
            accumulated_relative_shield_hp_us: 0,
        };
        Ok((execution, lifecycle_assumptions))
    }

    pub(crate) fn all_targets_defeated(&self) -> bool {
        !self.state.targets.is_empty() && self.state.targets.iter().all(|t| t.hp_current == 0)
    }

    pub(crate) fn process_events(
        &mut self,
        request: &GameScenarioRequest,
        limits: &ExecutionLimits,
        prepared_packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
        branches: &[crate::simulation::GameLiveBranch],
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
    ) -> Result<(), String> {
        self.process_events_to(
            request,
            limits,
            prepared_packets,
            branches,
            energy_waits,
            None,
        )
    }

    /// Settle a source continuation's deadline, including all equal-time causal
    /// work. An empty gap advances once; it does not create scheduler events.
    pub(super) fn advance_to(
        &mut self,
        request: &GameScenarioRequest,
        limits: &ExecutionLimits,
        packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
        deadline_us: u64,
    ) -> Result<(), String> {
        if deadline_us < self.state.current_time_us || deadline_us > request.absolute_limit_us {
            return Err("source deadline outside the current combat interval".into());
        }
        self.process_events_to(request, limits, packets, &[], &[], Some(deadline_us))?;
        if self.success && !self.all_targets_defeated() {
            let deadline_us = deadline_us.min(self.stop_deadline(request)?);
            self.advance_clock(request, deadline_us)?;
            if let Some(stamina) = &mut self.state.combat_stamina {
                stamina.advance(deadline_us)?;
            }
            self.state.expire_buffs(&mut self.trace);
        }
        Ok(())
    }

    fn process_events_to(
        &mut self,
        request: &GameScenarioRequest,
        limits: &ExecutionLimits,
        prepared_packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
        branches: &[crate::simulation::GameLiveBranch],
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
        deadline_us: Option<u64>,
    ) -> Result<(), String> {
        while let Some(head) = self
            .queue
            .peek()
            .filter(|event| deadline_us.is_none_or(|deadline| event.time_us <= deadline))
        {
            if self.victory_settled(request, head) {
                break;
            }
            let bound = self.stop_deadline(request)?;
            if head.time_us > bound {
                self.advance_clock(request, bound)?;
                self.stop_at_horizon(request, head);
                break;
            }
            if request.stop == crate::simulation::GameStopPolicy::Time
                && head.time_us == bound
                && matches!(head.kind, HeadKind::Admission | HeadKind::EnergyWait { .. })
            {
                self.advance_clock(request, bound)?;
                self.stop_at_horizon(request, head);
                break;
            }
            let event = self.queue.pop().expect("in-horizon event");
            if (self.events_processed as u64) >= limits.max_events {
                self.limit_reached = Some(crate::simulation::GameExecutionLimit::EventCount {
                    limit: limits.max_events,
                });
                self.record_pending_events();
                self.halt_diagnostic = Some(format!(
                    "Simulation event budget exhausted: limit {} events processed. The run stopped before settling the next event.",
                    limits.max_events
                ));
                self.success = false;
                self.execution_completed = false;
                break;
            }
            self.events_processed += 1;
            let vitals = self
                .observe_vitals
                .then(|| crate::simulation::vitals::Snapshot::capture(&self.state))
                .transpose()?;
            let trigger = crate::simulation::vitals::trigger(&event.data, event.seq);
            self.advance_clock(request, event.time_us)?;
            if let Some(stamina) = &mut self.state.combat_stamina {
                stamina.advance(event.time_us)?;
            }
            self.state.expire_buffs(&mut self.trace);
            let replay = self.queue.event_replay().map(|context| {
                (
                    context,
                    crate::characters::hotori::replay::begin_callback(&mut self.state, context),
                )
            });
            let continuing = self.process_event(
                event.data,
                request,
                prepared_packets,
                branches,
                energy_waits,
            );
            if let Some((context, control)) = replay {
                crate::characters::hotori::replay::finish_callback(
                    &mut self.state,
                    &self.queue,
                    context,
                    control,
                );
            }
            self.queue.end_event();
            let continuing = continuing?;
            if continuing && let Some(vitals) = vitals {
                self.settle_vitals(request, vitals, trigger)?;
            }
            while let Some(actor) = self
                .state
                .cinematic_casts
                .keys()
                .copied()
                .find(|&actor| self.state.actors[actor].hp == 0)
            {
                crate::simulation::pause::cancel_cinematic(
                    &mut self.state,
                    &mut self.queue,
                    actor,
                    None,
                )?;
            }
            self.queue.sync(&self.state.mode_clock)?;
            self.queue.set_bud_stop_exception(self.state.actors.iter().any(|actor|
                actor.hp > 0 && matches!(&actor.character, crate::characters::RuntimeState::Hotori(owner)
                    if owner.timepiece.breakthrough >= 2 && owner.timepiece.mode_active(self.state.current_time_us))));
            if self.batch.is_some() {
                self.observe_batch()?;
            } else {
                self.timeline.observe(
                    &self.state,
                    request,
                    crate::simulation::GameStateBoundary::Event {
                        ordinal: self.events_processed,
                    },
                    self.hit_receipts.len(),
                    self.trace.len(),
                )?;
            }
            if !continuing {
                break;
            }
        }
        if deadline_us.is_none() {
            self.finish_event_loop(request)?;
        }
        Ok(())
    }

    fn settle_vitals(
        &mut self,
        request: &GameScenarioRequest,
        snapshot: crate::simulation::vitals::Snapshot,
        trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        snapshot.settle(
            &mut crate::runtime_api::CallbackContext {
                state: &mut self.state,
                request,
                queue: &mut self.queue,
                sequence: &mut self.seq,
                trace: &mut self.trace,
            },
            trigger,
        )
    }

    pub(super) fn record_pending_events(&mut self) {
        self.pending_events = self.pending_event_count() + 1;
        self.execution_completed = false;
    }

    pub(super) fn pending_event_count(&self) -> usize {
        self.queue.len()
            + parked_energy_action_count(
                &self.parked_energy_actions,
                self.pending_energy_wait.as_ref(),
            )
            + self.unqueued_energy_waits
    }

    fn settle_dummy_combat(
        &mut self,
        request: &GameScenarioRequest,
        event_index: usize,
    ) -> Result<(), String> {
        crate::simulation::dummy::settle(
            &mut crate::runtime_api::CallbackContext {
                state: &mut self.state,
                request,
                queue: &mut self.queue,
                sequence: &mut self.seq,
                trace: &mut self.trace,
            },
            event_index,
        )
    }

    pub(crate) fn process_event(
        &mut self,
        event: GameEventData,
        request: &GameScenarioRequest,
        prepared_packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
        branches: &[crate::simulation::GameLiveBranch],
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
    ) -> Result<bool, String> {
        match event {
            GameEventData::CinematicRelease { context } => {
                crate::simulation::pause::release_cinematic(
                    &mut self.state,
                    &mut self.queue,
                    &mut self.seq,
                    context,
                )?;
            }
            GameEventData::DummyCombat { event_index } => {
                self.settle_dummy_combat(request, event_index)?;
            }
            GameEventData::ActionAdmission { command_index } => {
                if !self.admit_action(request, prepared_packets, command_index)? {
                    return Ok(false);
                }
                self.record_active_input(request, command_index);
                self.queue_action_parry(command_index)?;
            }
            GameEventData::HitDelivery {
                command_index,
                actor_index,
                target_index,
                ability_id,
                damage_effect_id,
                atk_rate,
                scaling_stat,
                metadata,
            } => self.settle_hit(
                request,
                prepared_packets,
                HitPacket {
                    command_index,
                    actor_index,
                    target_index,
                    ability_id,
                    damage_effect_id,
                    atk_rate,
                    scaling_stat,
                    metadata,
                },
                None,
            )?,
            GameEventData::FollowupHit {
                parent_hit_index,
                packet,
            } => {
                self.settle_hit(request, prepared_packets, *packet, Some(parent_hit_index))?;
            }
            GameEventData::TargetUnbalance {
                command_index,
                actor_index,
                target_index,
                unbal_value,
                owner_efficiency_bonus,
            } => {
                if crate::simulation::break_state::apply_unbalance(
                    request,
                    &mut self.state,
                    &mut self.queue,
                    &mut self.seq,
                    &mut self.trace,
                    actor_index,
                    target_index,
                    crate::simulation::break_state::GaugeChange {
                        unbal_value,
                        owner_efficiency_bonus,
                    },
                )? {
                    crate::simulation::break_damage::schedule(
                        request,
                        &mut self.state,
                        &mut self.queue,
                        &mut self.seq,
                        command_index,
                        target_index,
                    )?;
                }
            }
            GameEventData::TargetBreakDamage(packet) => crate::simulation::break_damage::settle(
                &mut self.event_context(request, prepared_packets),
                *packet,
            )?,
            GameEventData::TargetBreakRecovery {
                target_index,
                generation,
            } => crate::simulation::break_state::recover(
                &mut self.state,
                &mut self.trace,
                target_index,
                generation,
            )?,
            GameEventData::Content(event) => crate::callbacks::apply(
                event,
                &mut crate::runtime_api::CallbackContext {
                    state: &mut self.state,
                    request,
                    queue: &mut self.queue,
                    sequence: &mut self.seq,
                    trace: &mut self.trace,
                },
            )?,
            GameEventData::DaffodilStepAfterSupport { owner, command } => {
                if let Some(step) = crate::characters::daffodil::clock::step_after_support(
                    request,
                    &mut self.state,
                    &mut self.queue,
                    owner,
                    command,
                )? {
                    crate::simulation::support::dispatch_owner_entry(
                        &mut self.event_context(request, prepared_packets),
                        command,
                        owner,
                        crate::runtime_api::OwnerEntryAction::Attack(step),
                    )?;
                }
            }
            GameEventData::IncomingDamage {
                event_index,
                actor_index,
                delivered_damage,
            } => {
                self.settle_incoming_damage(request, event_index, actor_index, delivered_damage)?
            }
            GameEventData::ActorInterrupt { event_index } => {
                self.settle_interrupt(request, event_index)?;
            }
            GameEventData::ActorParry { event_index } => {
                self.settle_owner_event(request, event_index, true)?
            }
            GameEventData::CombatTransition { event_index } => {
                self.settle_owner_event(request, event_index, false)?
            }
            GameEventData::HpCost { event_index } => {
                self.hp_cost_receipts
                    .push(crate::simulation::hp_cost::settle(
                        &mut crate::runtime_api::CallbackContext {
                            state: &mut self.state,
                            request,
                            queue: &mut self.queue,
                            sequence: &mut self.seq,
                            trace: &mut self.trace,
                        },
                        event_index,
                    )?);
            }
            GameEventData::LiveBranch { branch_index } => process_live_branch(
                &self.state,
                &mut self.queue,
                branches,
                energy_waits,
                &mut self.parked_energy_actions,
                self.pending_energy_wait.as_mut(),
                branch_index,
                &mut self.live_branch_receipts,
                &mut self.trace,
            )?,
            GameEventData::LiveEnergyWait { wait_index } => {
                self.begin_live_energy_wait(
                    &energy_waits[wait_index],
                    wait_index,
                    energy_waits,
                    branches,
                )?;
            }
            GameEventData::ResumeLiveEnergyWait => {
                self.resume_live_energy_wait(energy_waits, branches)?;
            }
            GameEventData::SwitchActor {
                command_index,
                target_actor_index,
                cooldown_us,
            } => {
                if !self.switch_input(
                    request,
                    prepared_packets,
                    command_index,
                    target_actor_index,
                    cooldown_us,
                )? {
                    return Ok(false);
                }
                self.queue_action_parry(command_index)?;
            }
            event @ (GameEventData::ReactionPistil(_)
            | GameEventData::ReactionScorchTick(_)
            | GameEventData::ReactionNovaDetonate(_)
            | GameEventData::ReactionNovaEnd(_)
            | GameEventData::ReactionRemoraEnd { .. }
            | GameEventData::ReactionScorchRefresh { .. }) => {
                self.settle_reaction_event(request, prepared_packets, event)?;
            }
        }
        if let Err(error) = self.state.verify_invariants() {
            self.halt_diagnostic = Some(format!("INVARIANT_VIOLATION: {error}"));
            self.success = false;
            self.execution_completed = false;
            return Ok(false);
        }
        self.queue_energy_resume();
        Ok(true)
    }

    fn settle_owner_event(
        &mut self,
        request: &GameScenarioRequest,
        event_index: usize,
        parry: bool,
    ) -> Result<(), String> {
        let mut ctx = crate::runtime_api::CallbackContext {
            state: &mut self.state,
            request,
            queue: &mut self.queue,
            sequence: &mut self.seq,
            trace: &mut self.trace,
        };
        if parry {
            self.parry_receipts
                .push(crate::simulation::parry::settle(&mut ctx, event_index)?);
        } else {
            self.combat_transition_receipts
                .push(crate::simulation::combat::settle(&mut ctx, event_index)?);
        }
        Ok(())
    }

    fn settle_interrupt(
        &mut self,
        request: &GameScenarioRequest,
        event_index: usize,
    ) -> Result<(), String> {
        let receipt = crate::simulation::interrupt::settle(
            &mut crate::runtime_api::CallbackContext {
                state: &mut self.state,
                request,
                queue: &mut self.queue,
                sequence: &mut self.seq,
                trace: &mut self.trace,
            },
            event_index,
        )?;
        self.interrupt_receipts.push(receipt);
        Ok(())
    }

    fn settle_reaction_event(
        &mut self,
        request: &GameScenarioRequest,
        prepared_packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
        event: GameEventData,
    ) -> Result<(), String> {
        let mut ctx = self.event_context(request, prepared_packets);
        match event {
            GameEventData::ReactionPistil(event) => {
                crate::simulation::reactions::settle_pistil(&mut ctx, *event)
            }
            GameEventData::ReactionScorchTick(event) => {
                crate::simulation::reactions::settle_scorch_tick(&mut ctx, *event)
            }
            GameEventData::ReactionNovaDetonate(event) => {
                crate::simulation::reactions::settle_nova(&mut ctx, *event)
            }
            GameEventData::ReactionNovaEnd(event) => {
                crate::simulation::reactions::end_nova(&mut ctx, *event)
            }
            GameEventData::ReactionRemoraEnd {
                target_index,
                generation,
            } => crate::simulation::reactions::end_remora(&mut ctx, target_index, generation),
            GameEventData::ReactionScorchRefresh {
                source_actor_index,
                target_index,
                word_generation,
            } => crate::simulation::reactions::refresh_scorch_from_word(
                &mut ctx,
                source_actor_index,
                target_index,
                word_generation,
            ),
            _ => unreachable!("reaction dispatcher requires a reaction event"),
        }
    }

    pub(crate) fn event_context<'a>(
        &'a mut self,
        request: &'a GameScenarioRequest,
        prepared_packets: &'a [Option<crate::simulation::metadata::ActionPacketMetadata>],
    ) -> crate::simulation::context::EventContext<'a> {
        crate::simulation::context::EventContext {
            request,
            prepared_packets,
            state: &mut self.state,
            queue: &mut self.queue,
            sequence: &mut self.seq,
            trace: &mut self.trace,
            rng: &mut self.rng,
            hits: &mut self.hit_receipts,
            action_metadata_ledger: &mut self.action_metadata_ledger,
        }
    }

    fn admit_action(
        &mut self,
        request: &GameScenarioRequest,
        prepared_packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
        command_index: usize,
    ) -> Result<bool, String> {
        let mut ctx = crate::simulation::context::EventContext {
            request,
            prepared_packets,
            state: &mut self.state,
            queue: &mut self.queue,
            sequence: &mut self.seq,
            trace: &mut self.trace,
            rng: &mut self.rng,
            hits: &mut self.hit_receipts,
            action_metadata_ledger: &mut self.action_metadata_ledger,
        };
        if let Some(error) = crate::simulation::admission::admit(&mut ctx, command_index)? {
            let actor_index = match request.actions[command_index] {
                crate::simulation::GameActionCommand::TargetGauge { .. } => {
                    self.state.active_actor_index
                }
                crate::simulation::GameActionCommand::InvokeAbility { actor_index, .. }
                | crate::simulation::GameActionCommand::PlayerInput { actor_index, .. } => {
                    actor_index
                }
                _ => unreachable!(),
            };
            self.rejected_action = Some(crate::simulation::GameRejectedAction {
                command_index,
                time_us: self.state.current_time_us,
                actor_index,
                reason: error.reason,
            });
            self.halt_diagnostic = Some(error.message);
            self.success = false;
            self.execution_completed = false;
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn settle_hit(
        &mut self,
        request: &GameScenarioRequest,
        prepared_packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
        hit: HitPacket,
        parent_hit_index: Option<usize>,
    ) -> Result<(), String> {
        crate::simulation::hit::settle(
            &mut self.event_context(request, prepared_packets),
            crate::simulation::hit::HitIdentity {
                parent_hit_index,
                command_index: hit.command_index,
                actor_index: hit.actor_index,
                target_index: hit.target_index,
            },
            hit.ability_id,
            hit.damage_effect_id,
            hit.atk_rate,
            hit.scaling_stat,
            *hit.metadata,
        )
    }

    fn settle_incoming_damage(
        &mut self,
        request: &GameScenarioRequest,
        event_index: usize,
        actor_index: usize,
        delivered_damage: u64,
    ) -> Result<(), String> {
        let recipient_actor = if actor_index == usize::MAX {
            self.state.active_actor_index
        } else {
            actor_index
        };
        self.incoming_damage_receipts
            .push(crate::simulation::incoming::settle(
                &mut crate::runtime_api::CallbackContext {
                    state: &mut self.state,
                    request,
                    queue: &mut self.queue,
                    sequence: &mut self.seq,
                    trace: &mut self.trace,
                },
                event_index,
                recipient_actor,
                delivered_damage,
            )?);
        Ok(())
    }

    pub(crate) fn switch_actor(
        &mut self,
        request: &GameScenarioRequest,
        prepared_packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
        command_index: usize,
        target_actor_index: usize,
        cooldown_us: u64,
    ) -> Result<bool, String> {
        let old_actor = self.state.active_actor_index;
        if request.actors[old_actor].character_id == "character:1075"
            && self.state.actors[old_actor]
                .character
                .iroi()
                .is_ok_and(|s| s.regressed_allies.contains_key(&target_actor_index))
        {
            let in_ivory = self.state.actors[old_actor]
                .character
                .iroi()
                .is_ok_and(|s| s.dream_active && s.dream_is_ivory && !s.revive_used_in_dream);
            if in_ivory {
                crate::characters::iroi::regression::revive_regression(
                    request,
                    &mut self.state,
                    old_actor,
                    target_actor_index,
                )?;
            } else {
                return self.execute_sheep_support(
                    request,
                    command_index,
                    old_actor,
                    target_actor_index,
                );
            }
        }
        if self.state.actors[target_actor_index].hp == 0 {
            self.rejected_action = Some(crate::simulation::GameRejectedAction {
                command_index,
                time_us: self.state.current_time_us,
                actor_index: old_actor,
                reason: crate::simulation::GameAdmissionReason::Other {
                    message: "target actor is defeated".into(),
                },
            });
            self.halt_diagnostic = Some(format!(
                "DEFEATED_ACTOR: cannot switch to actor {target_actor_index} at zero HP"
            ));
            self.success = false;
            self.execution_completed = false;
            return Ok(false);
        }
        if old_actor == target_actor_index {
            if !self.parry_switch_admission(request, command_index, target_actor_index, None) {
                return Ok(false);
            }
            self.trace.push(GameEventReceipt {
                time_us: self.state.current_time_us,
                phase: "ActionAdmission".into(),
                event_kind: "switch_actor_noop".into(),
                active_actor: old_actor,
                hit_index: None,
                from_actor: Some(old_actor),
                to_actor: Some(target_actor_index),
                energy_change: None,
                energy_fanout: None,
                data: None,
            });
            return Ok(true);
        }
        let recovery_override = crate::registry::recovery_lock_override(
            request,
            &request.actors[old_actor],
            &self.state.actors[old_actor],
            "switch_actor",
            self.state.current_time_us,
        )?;
        let timing = match self
            .state
            .prepare_switch_admission(cooldown_us, recovery_override.map(|(time, _)| time))
        {
            Ok(timing) => timing,
            Err(error) => {
                self.rejected_action = Some(crate::simulation::GameRejectedAction {
                    command_index,
                    time_us: self.state.current_time_us,
                    actor_index: old_actor,
                    reason: error.reason,
                });
                self.halt_diagnostic = Some(error.message);
                self.success = false;
                self.execution_completed = false;
                return Ok(false);
            }
        };
        let reaction = crate::simulation::reactions::validate_on_switch(
            request,
            &self.state,
            old_actor,
            target_actor_index,
            request.aim_target(command_index),
        )?;
        let owner_entry = crate::registry::entry_action(
            request,
            &self.state,
            old_actor,
            target_actor_index,
            reaction.is_some(),
        )?;
        if !self.parry_switch_admission(
            request,
            command_index,
            target_actor_index,
            owner_entry.as_ref(),
        ) {
            return Ok(false);
        }
        if let Some((_, model)) = recovery_override {
            crate::runtime_api::record_hypothesis_id(
                request,
                &mut self.state.execution_model_assumptions,
                model,
            )?;
        }
        self.state.actors[old_actor].switch_cooldown_until_us = timing.ready_at_us;
        self.state.actors[old_actor].last_action_start_us = self.state.current_time_us;
        crate::simulation::pause::cancel_cinematic(
            &mut self.state,
            &mut self.queue,
            old_actor,
            None,
        )?;
        self.queue.sync(&self.state.mode_clock)?;
        self.state.active_actor_index = target_actor_index;
        self.trace.push(GameEventReceipt {
            time_us: self.state.current_time_us,
            phase: "ActionAdmission".into(),
            event_kind: "switch_actor".into(),
            active_actor: self.state.active_actor_index,
            hit_index: None,
            from_actor: Some(old_actor),
            to_actor: Some(target_actor_index),
            energy_change: None,
            energy_fanout: None,
            data: None,
        });
        {
            let mut context = crate::runtime_api::CallbackContext {
                state: &mut self.state,
                request,
                queue: &mut self.queue,
                sequence: &mut self.seq,
                trace: &mut self.trace,
            };
            crate::registry::on_field_exit(&mut context, old_actor)?;
            crate::registry::on_field_entry(
                &mut context,
                target_actor_index,
                crate::simulation::GameCausalTrigger::FieldEntry { command_index },
            )?;
            crate::gear::on_switch(&mut context, old_actor)?;
        }
        let mut ctx = crate::simulation::context::EventContext {
            request,
            prepared_packets,
            state: &mut self.state,
            queue: &mut self.queue,
            sequence: &mut self.seq,
            trace: &mut self.trace,
            rng: &mut self.rng,
            hits: &mut self.hit_receipts,
            action_metadata_ledger: &mut self.action_metadata_ledger,
        };
        if let Some(entry) = owner_entry {
            crate::simulation::support::dispatch_owner_entry(
                &mut ctx,
                command_index,
                target_actor_index,
                entry,
            )?;
        }
        crate::simulation::reactions::admit_on_switch(
            &mut ctx,
            command_index,
            old_actor,
            target_actor_index,
            reaction,
        )?;
        Ok(true)
    }

    fn check_sheep_support_admission(
        &mut self,
        request: &GameScenarioRequest,
        command_index: usize,
        old_actor: usize,
        target_actor_index: usize,
        now: u64,
    ) -> Result<Option<(crate::characters::iroi::regression::LambType, bool)>, String> {
        let until_us = self.state.actors[old_actor]
            .switch_cooldown_until_us
            .max(self.state.actors[old_actor].switch_action_locked_until_us);
        if now < until_us {
            self.rejected_action = Some(crate::simulation::GameRejectedAction {
                command_index,
                time_us: now,
                actor_index: old_actor,
                reason: crate::simulation::GameAdmissionReason::Cooldown { until_us },
            });
            self.halt_diagnostic = Some(format!(
                "SWITCH_ON_COOLDOWN: switch on cooldown until {until_us}"
            ));
            self.success = false;
            self.execution_completed = false;
            return Ok(None);
        }
        if self.state.actors[old_actor].cycle.current() < 100 {
            self.rejected_action = Some(crate::simulation::GameRejectedAction {
                command_index,
                time_us: now,
                actor_index: old_actor,
                reason: crate::simulation::GameAdmissionReason::Other {
                    message: format!(
                        "INSUFFICIENT_CYCLE: Iroi requires 100 Cycle for sheep Support (available: {})",
                        self.state.actors[old_actor].cycle.current()
                    ),
                },
            });
            self.halt_diagnostic =
                Some("INSUFFICIENT_CYCLE: Iroi requires 100 Cycle for sheep Support".into());
            self.success = false;
            self.execution_completed = false;
            return Ok(None);
        }

        let iroi_state = self.state.actors[old_actor].character.iroi()?;
        let lamb_type = iroi_state.regressed_allies[&target_actor_index];
        let qte_cd_until = iroi_state
            .lamb_cooldowns
            .get(&(target_actor_index, 0))
            .copied()
            .unwrap_or(0);
        if now < qte_cd_until {
            self.rejected_action = Some(crate::simulation::GameRejectedAction {
                command_index,
                time_us: now,
                actor_index: old_actor,
                reason: crate::simulation::GameAdmissionReason::Cooldown {
                    until_us: qte_cd_until,
                },
            });
            self.halt_diagnostic = Some("COOLDOWN: lamb QTE is on cooldown".into());
            self.success = false;
            self.execution_completed = false;
            return Ok(None);
        }

        let ally_element = crate::registry::get_character_descriptor_by_id(
            &request.actors[target_actor_index].character_id,
        )?
        .element;
        let is_cosmos = matches!(ally_element.to_uppercase().as_str(), "COSMOS" | "ETHER");
        let is_incant = matches!(
            ally_element.to_uppercase().as_str(),
            "INCANTATION" | "CHAOS"
        );
        if !is_cosmos && !is_incant {
            self.rejected_action = Some(crate::simulation::GameRejectedAction {
                command_index,
                time_us: now,
                actor_index: old_actor,
                reason: crate::simulation::GameAdmissionReason::Other {
                    message:
                        "represented ally element must be Cosmos or Incantation for sheep Support"
                            .into(),
                },
            });
            self.halt_diagnostic = Some(
                "NONADJACENT_ELEMENT: represented ally element cannot produce QTE Guang or Zhou"
                    .into(),
            );
            self.success = false;
            self.execution_completed = false;
            return Ok(None);
        }

        Ok(Some((lamb_type, is_cosmos)))
    }

    fn execute_sheep_support(
        &mut self,
        request: &GameScenarioRequest,
        command_index: usize,
        old_actor: usize,
        target_actor_index: usize,
    ) -> Result<bool, String> {
        let now = self.state.current_time_us;
        let Some((lamb_type, is_cosmos)) = self.check_sheep_support_admission(
            request,
            command_index,
            old_actor,
            target_actor_index,
            now,
        )?
        else {
            return Ok(false);
        };

        self.state.actors[old_actor]
            .cycle
            .spend(100)
            .map_err(|e| format!("Cycle spend failed: {e:?}"))?;
        self.state.actors[old_actor].cycle_ledger.spend(100)?;

        let cd_us = match lamb_type {
            crate::characters::iroi::regression::LambType::Gun => 1_000_000,
            _ => 6_000_000,
        };
        self.state.actors[old_actor]
            .character
            .iroi_mut()?
            .lamb_cooldowns
            .insert((target_actor_index, 0), now.saturating_add(cd_us));

        let effect = sheep_qte_effect(lamb_type, is_cosmos);

        let metadata = crate::simulation::metadata::resolve_requested_action_packet(
            &request.content_id,
            ntesim_data::skill_levels::frozen::tables(),
            &request.actors[old_actor].character_id,
            "GA_Oneiroi_Skill",
            effect,
            None,
            &self.state.execution_model_assumptions,
        )?;

        for target_index in 0..request.targets.len() {
            if self.state.targets[target_index].hp_current == 0 {
                continue;
            }
            self.seq += 1;
            self.queue
                .push(crate::simulation::events::ScheduledGameEvent {
                    time_us: now,
                    priority: crate::execution::CausalPriority::HitDelivery,
                    seq: self.seq,
                    data: crate::simulation::events::GameEventData::HitDelivery {
                        command_index,
                        actor_index: old_actor,
                        target_index,
                        ability_id: "GA_Oneiroi_Skill".into(),
                        damage_effect_id: effect.into(),
                        atk_rate: ntesim_core::formula::Ratio::one(),
                        scaling_stat: crate::simulation::NativeScalingStat::BaseAtk,
                        metadata: Box::new(metadata.clone()),
                    },
                });
        }

        let reaction_result = if is_cosmos {
            ntesim_data::reaction_values::ReactionResult::Blossom
        } else {
            ntesim_data::reaction_values::ReactionResult::Hexed
        };
        let mut ctx = crate::simulation::context::EventContext {
            request,
            prepared_packets: &[],
            state: &mut self.state,
            queue: &mut self.queue,
            sequence: &mut self.seq,
            trace: &mut self.trace,
            rng: &mut self.rng,
            hits: &mut self.hit_receipts,
            action_metadata_ledger: &mut self.action_metadata_ledger,
        };
        crate::simulation::reactions::execute_sheep_switch_reaction(
            &mut ctx,
            command_index,
            old_actor,
            target_actor_index,
            reaction_result,
        )?;

        crate::runtime_api::record_hypothesis_id(
            request,
            &mut self.state.execution_model_assumptions,
            crate::characters::iroi::regression::HYPOTHESIS_ID,
        )?;

        self.trace.push(GameEventReceipt {
            time_us: now,
            phase: "ActionAdmission".into(),
            event_kind: "iroi_sheep_support".into(),
            active_actor: old_actor,
            hit_index: None,
            from_actor: Some(old_actor),
            to_actor: Some(target_actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                lamb_type: Some(format!("{lamb_type:?}")),
                ..Default::default()
            }),
        });

        Ok(true)
    }

    fn queue_energy_resume(&mut self) {
        if let Some(wait) = self.pending_energy_wait.as_mut()
            && !wait.resume_queued
            && self.state.actors[wait.instruction.actor_index]
                .energy
                .current()
                >= wait.instruction.energy_at_least
        {
            self.seq += 1;
            self.queue
                .push(crate::simulation::events::ScheduledGameEvent {
                    time_us: self.state.current_time_us,
                    priority: crate::execution::CausalPriority::Observation,
                    seq: self.seq,
                    data: GameEventData::ResumeLiveEnergyWait,
                });
            wait.resume_queued = true;
        }
    }

    fn begin_live_energy_wait(
        &mut self,
        instruction: &crate::simulation::GameLiveEnergyWait,
        wait_index: usize,
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
        branches: &[crate::simulation::GameLiveBranch],
    ) -> Result<(), String> {
        if self.pending_energy_wait.is_some() {
            return Err("a live energy wait started before the previous wait completed".into());
        }
        let energy_at_start = self.state.actors[instruction.actor_index].energy.current();
        let receipt_index = self.live_energy_wait_receipts.len();
        self.live_energy_wait_receipts
            .push(crate::simulation::GameLiveEnergyWaitReceipt {
                started_time_us: self.state.current_time_us,
                actor_index: instruction.actor_index,
                energy_at_start,
                energy_at_least: instruction.energy_at_least,
                resumed_time_us: None,
                energy_at_resume: None,
                after_action_index: instruction.after_action_index,
                source_line: instruction.source_line,
                source_column: instruction.source_column,
            });
        self.trace.push(GameEventReceipt {
            time_us: self.state.current_time_us,
            phase: "Observation".into(),
            event_kind: "live_energy_wait_started".into(),
            active_actor: self.state.active_actor_index,
            hit_index: None,
            from_actor: Some(instruction.actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                energy: Some(energy_at_start),
                threshold: Some(instruction.energy_at_least),
                ..Default::default()
            }),
        });
        self.pending_energy_wait = Some(PendingLiveEnergyWait {
            instruction: instruction.clone(),
            wait_index,
            started_time_us: self.state.current_time_us,
            receipt_index,
            parked_actions: std::mem::take(&mut self.parked_energy_actions),
            resume_queued: false,
        });
        if energy_at_start >= instruction.energy_at_least {
            self.complete_live_energy_wait(energy_waits, branches)?;
        }
        Ok(())
    }

    fn resume_live_energy_wait(
        &mut self,
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
        branches: &[crate::simulation::GameLiveBranch],
    ) -> Result<(), String> {
        let should_resume = {
            let wait = self
                .pending_energy_wait
                .as_mut()
                .ok_or("live energy wait resumed without a pending wait")?;
            wait.resume_queued = false;
            self.state.actors[wait.instruction.actor_index]
                .energy
                .current()
                >= wait.instruction.energy_at_least
        };
        if should_resume {
            self.complete_live_energy_wait(energy_waits, branches)?;
        }
        Ok(())
    }

    fn complete_live_energy_wait(
        &mut self,
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
        branches: &[crate::simulation::GameLiveBranch],
    ) -> Result<(), String> {
        let mut wait = self
            .pending_energy_wait
            .take()
            .ok_or("live energy wait completed without a pending wait")?;
        let energy_at_resume = self.state.actors[wait.instruction.actor_index]
            .energy
            .current();
        if energy_at_resume < wait.instruction.energy_at_least {
            self.pending_energy_wait = Some(wait);
            return Err("live energy wait completed below its threshold".into());
        }
        let elapsed = self
            .state
            .current_time_us
            .checked_sub(wait.started_time_us)
            .ok_or("live energy wait resumed before it started")?;
        for event in &mut wait.parked_actions {
            event.time_us = event
                .time_us
                .checked_add(elapsed)
                .ok_or("live energy wait shifted an action outside the supported clock range")?;
        }
        self.release_energy_wait_segment(&mut wait, energy_waits, branches)?;
        let receipt = self
            .live_energy_wait_receipts
            .get_mut(wait.receipt_index)
            .ok_or("live energy wait receipt was lost")?;
        receipt.resumed_time_us = Some(self.state.current_time_us);
        receipt.energy_at_resume = Some(energy_at_resume);
        self.trace.push(GameEventReceipt {
            time_us: self.state.current_time_us,
            phase: "Observation".into(),
            event_kind: "live_energy_wait_resumed".into(),
            active_actor: self.state.active_actor_index,
            hit_index: None,
            from_actor: Some(wait.instruction.actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                energy: Some(energy_at_resume),
                threshold: Some(wait.instruction.energy_at_least),
                delay_us: Some(elapsed),
                ..Default::default()
            }),
        });
        Ok(())
    }

    fn release_energy_wait_segment(
        &mut self,
        wait: &mut PendingLiveEnergyWait,
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
        branches: &[crate::simulation::GameLiveBranch],
    ) -> Result<(), String> {
        let next_index = wait.wait_index + 1;
        let Some(next) = energy_waits.get(next_index) else {
            self.queue.extend(std::mem::take(&mut wait.parked_actions));
            self.parked_energy_actions.clear();
            return Ok(());
        };
        let mut ready = Vec::new();
        for event in std::mem::take(&mut wait.parked_actions) {
            if scheduled_action_index(&event, branches)
                .is_some_and(|index| index < next.after_action_index)
            {
                ready.push(event);
            } else {
                self.parked_energy_actions.push(event);
            }
        }
        self.queue.extend(ready);
        let accumulated_delay = self
            .state
            .current_time_us
            .checked_sub(wait.instruction.timestamp_us)
            .ok_or("live energy wait resumed before its authored timestamp")?;
        let next_time = next
            .timestamp_us
            .checked_add(accumulated_delay)
            .ok_or("next live energy wait exceeds the supported clock range")?;
        self.seq += 1;
        self.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: next_time,
                priority: crate::execution::CausalPriority::Observation,
                seq: self.seq,
                data: GameEventData::LiveEnergyWait {
                    wait_index: next_index,
                },
            });
        self.unqueued_energy_waits = self.unqueued_energy_waits.saturating_sub(1);
        Ok(())
    }

    fn finish_event_loop(&mut self, request: &GameScenarioRequest) -> Result<(), String> {
        if let Some(wait) = self.pending_energy_wait.as_ref() {
            self.execution_completed = false;
            if self.halt_diagnostic.is_none() {
                self.halt_diagnostic = Some(format!(
                    "LIVE_ENERGY_WAIT_PENDING: actor {} did not reach energy {} before the horizon",
                    wait.instruction.actor_index, wait.instruction.energy_at_least
                ));
            }
        }
        let parked_count = parked_energy_action_count(
            &self.parked_energy_actions,
            self.pending_energy_wait.as_ref(),
        );
        if parked_count > 0 || self.pending_energy_wait.is_some() || self.unqueued_energy_waits > 0
        {
            self.pending_events = self.pending_events.max(
                self.queue.len()
                    + parked_count
                    + usize::from(self.pending_energy_wait.is_some())
                    + self.unqueued_energy_waits,
            );
        }
        self.finish_stop_policy(request)
    }

    pub(super) fn observe_batch(&mut self) -> Result<(), String> {
        self.batch
            .as_mut()
            .expect("batch execution")
            .observe_boundary(
                &mut self.state,
                &self.hit_receipts,
                &mut self.trace,
                &mut self.action_metadata_ledger,
                &mut self.incoming_damage_receipts,
            )
    }

    pub(super) fn field_times(&self) -> Vec<ntesim_contracts::TimeBasisPair<u64>> {
        self.state
            .actors
            .iter()
            .map(|actor| ntesim_contracts::TimeBasisPair {
                relative: Some(actor.field_time_relative_us),
                absolute: Some(actor.field_time_absolute_us),
            })
            .collect()
    }

    pub(super) fn available_shield_hp(&self) -> ntesim_contracts::TimeBasisPair<f64> {
        let relative = self.state.mode_clock.relative_us();
        let absolute = self.state.current_time_us;
        ntesim_contracts::TimeBasisPair {
            relative: (relative > 0)
                .then(|| self.accumulated_relative_shield_hp_us as f64 / relative as f64),
            absolute: (absolute > 0)
                .then(|| self.accumulated_shield_hp_us as f64 / absolute as f64),
        }
    }

    pub(super) fn reaction_counts(&self) -> BTreeMap<String, u32> {
        crate::simulation::ReactionKind::ALL
            .into_iter()
            .filter_map(|kind| {
                let count = self.state.reactions.reaction_counts[kind as usize];
                (count > 0).then(|| (kind.name().to_string(), count))
            })
            .collect()
    }

    pub(super) fn into_batch(
        mut self,
        request: &GameScenarioRequest,
        seed: u64,
        lifecycle_assumptions: BTreeMap<String, String>,
    ) -> Result<Option<super::batch::CompletedSourceTrial>, String> {
        if let Some(stamina) = &mut self.state.combat_stamina {
            stamina.advance(self.state.current_time_us)?;
        }
        let mut assumptions = BTreeMap::new();
        merge_model_assumptions(
            &mut assumptions,
            lifecycle_assumptions
                .into_iter()
                .chain(std::mem::take(&mut self.state.receipt_model_assumptions))
                .chain(std::mem::take(&mut self.state.execution_model_assumptions)),
        )?;
        let quality = source_quality(request, &assumptions);
        if !self.success
            || !self.execution_completed
            || self.halt_diagnostic.is_some()
            || self.state.verify_invariants().is_err()
            || quality != Quality::BoundedHypothesis
        {
            return Ok(None);
        }
        self.observe_batch()?;
        self.batch
            .take()
            .expect("batch execution")
            .finish(&self, request, seed, quality)
            .map(Some)
    }

    pub(crate) fn into_report(
        mut self,
        request: &GameScenarioRequest,
        seed: u64,
        lifecycle_assumptions: BTreeMap<String, String>,
    ) -> Result<GameScenarioReport, String> {
        if let Some(stamina) = &mut self.state.combat_stamina {
            stamina.advance(self.state.current_time_us)?;
        }
        let mut resolved_model_assumptions = BTreeMap::new();
        let receipt_model_assumptions = std::mem::take(&mut self.state.receipt_model_assumptions);
        let callback_model_assumptions =
            std::mem::take(&mut self.state.execution_model_assumptions);
        merge_model_assumptions(
            &mut resolved_model_assumptions,
            lifecycle_assumptions
                .into_iter()
                .chain(receipt_model_assumptions)
                .chain(callback_model_assumptions.clone()),
        )?;
        let mut action_metadata_ledger: Vec<crate::simulation::ActionMetadataOccurrence> =
            Vec::new();
        for entry in self.action_metadata_ledger.drain(..) {
            if let Some(existing) = action_metadata_ledger.iter_mut().find(|existing| {
                existing.switch_origin == entry.switch_origin
                    && existing.execution_model_assumption_ids
                        == entry.execution_model_assumption_ids
                    && existing.actor_index == entry.actor_index
                    && existing.ability_id == entry.ability_id
                    && existing.damage_effect_id == entry.damage_effect_id
                    && existing.metadata_id == entry.metadata_id
                    && existing.energy_group_id == entry.energy_group_id
            }) {
                existing.command_indices.extend(entry.command_indices);
            } else {
                action_metadata_ledger.push(entry);
            }
        }
        let proc_rng = crate::gear::proc_rng_receipts(request, &self.state.actors)?;
        self.timeline.observe(
            &self.state,
            request,
            crate::simulation::GameStateBoundary::Final,
            self.hit_receipts.len(),
            self.trace.len(),
        )?;
        let proc_selection_receipts = crate::gear::proc_selection_receipts(&self.state.actors);
        let invariants_passed = self.state.verify_invariants().is_ok();
        self.input_time.finish();
        let quality = source_quality(request, &resolved_model_assumptions);
        let game_dps_computed = quality == Quality::BoundedHypothesis
            && self.success
            && self.execution_completed
            && invariants_passed
            && self.state.mode_clock.relative_us() > 0;
        let final_buffs = self.timeline.encode_buffs(self.state.buffs.instances());
        let reaction_counts = self.reaction_counts();
        let available_shield_hp = self.available_shield_hp();
        let field_times = self.field_times();
        let report = GameScenarioReport {
            dps_unavailable_reason: if self.state.mode_clock.relative_us() == 0 {
                Some(crate::simulation::model::GameDpsUnavailableReason::ZeroRelativeDuration)
            } else if !self.success || !self.execution_completed || !invariants_passed {
                Some(crate::simulation::model::GameDpsUnavailableReason::IncompleteExecution)
            } else if !game_dps_computed {
                Some(crate::simulation::model::GameDpsUnavailableReason::UnassessedModel)
            } else {
                None
            },
            clock_contract: request.clock_contract,
            timer_policy: request.timer_policy,
            timer_bindings: std::mem::take(&mut self.state.timer_bindings),
            timer_catalog: Some(self.timeline.timer_catalog),
            absolute_limit_us: request.absolute_limit_us,
            relative_elapsed_us: self.state.mode_clock.relative_us(),
            timer_pauses: Some(
                self.state
                    .mode_clock
                    .observed_pauses(self.state.current_time_us)
                    .map_err(|e| e.to_string())?,
            ),
            state_timeline: self.timeline.frames,
            healing_receipts: std::mem::take(&mut self.state.healing_receipts),
            limit_reached: self.limit_reached,
            input_time: self.input_time,
            autonomous_events: self.state.origins.receipts(),
            combat_stamina: self.state.combat_stamina.take(),
            player_input_receipts: std::mem::take(&mut self.state.input_receipts),
            schema_version: 1,
            content_id: request.content_id.clone(),
            status: if self.success { "ok" } else { "failed" }.into(),
            quality,
            game_dps_computed,
            model_assumptions: request.model_assumptions.clone(),
            resolved_model_assumptions,
            callback_model_assumptions,
            evidence_catalog: std::mem::take(&mut self.state.evidence_catalog),
            metadata_catalog: std::mem::take(&mut self.state.metadata_catalog),
            buff_catalog: self.timeline.buff_catalog,
            success: self.success,
            execution_completed: self.execution_completed,
            pending_events: self.pending_events,
            relative_duration_us: request.relative_duration_us,
            absolute_elapsed_us: self.state.current_time_us,
            seed,
            stochastic_crit: request.stochastic_crit.clone(),
            rng: self
                .rng
                .map(serde_json::to_value)
                .transpose()
                .map_err(|e| e.to_string())?,
            proc_rng: (!proc_rng.is_empty()).then_some(serde_json::Value::Array(proc_rng)),
            reaction_rng: self.state.reactions.rng_audit()?,
            reaction_counts,
            invariants_passed,
            total_events_processed: self.events_processed,
            hits_settled: self.state.hits_settled,
            max_proc_depth_reached: self.state.max_proc_depth_reached,
            total_damage_dealt: self.state.rounded_damage_total as f64,
            total_damage_exact: ExactFraction::new(self.state.rounded_damage_total, 1)?,
            hit_receipts: self.hit_receipts,
            live_branch_receipts: self.live_branch_receipts,
            live_energy_wait_receipts: self.live_energy_wait_receipts,
            incoming_damage_receipts: self.incoming_damage_receipts,
            hp_cost_receipts: self.hp_cost_receipts,
            interrupt_receipts: self.interrupt_receipts,
            parry_receipts: self.parry_receipts,
            combat_transition_receipts: self.combat_transition_receipts,
            proc_selection_receipts,
            action_metadata_ledger,
            talent_projection_receipts: Vec::new(),
            final_actor_states: self.state.actor_summaries(request)?,
            structural_hp_changes: self.state.structural_hp_changes.clone(),
            atk_transfers: self.state.atk_transfers.clone(),
            final_target_states: self.state.target_summaries()?,
            final_buff_state: crate::simulation::model::GameBuffStateSummary {
                time_us: self.state.current_time_us,
                active_actor_index: self.state.active_actor_index,
                instances: final_buffs,
            },
            trace: self.trace,
            halt_diagnostic: self.halt_diagnostic,
            rejected_action: self.rejected_action,
            available_shield_hp,
            field_times,
            engine_version: ENGINE_VERSION.into(),
            engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
        };
        report.validate_damage_parentage()?;
        report.validate_state_timeline()?;
        report.input_time.validate(report.absolute_elapsed_us)?;
        Ok(report)
    }
}

fn sheep_qte_effect(
    lamb_type: crate::characters::iroi::regression::LambType,
    is_cosmos: bool,
) -> &'static str {
    match (lamb_type, is_cosmos) {
        (crate::characters::iroi::regression::LambType::Gun, true) => {
            "GE_Player_Oneiroi_GunAI_QTE_Guang_Damage"
        }
        (crate::characters::iroi::regression::LambType::Gun, false) => {
            "GE_Player_Oneiroi_GunAI_QTE_Zhou_Damage"
        }
        (crate::characters::iroi::regression::LambType::Syringe, true) => {
            "GE_Player_Oneiroi_SyringeAI_QTE_Guang_Damage"
        }
        (crate::characters::iroi::regression::LambType::Syringe, false) => {
            "GE_Player_Oneiroi_SyringeAI_QTE_Zhou_Damage"
        }
        (crate::characters::iroi::regression::LambType::Trumpet, true) => {
            "GE_Player_Oneiroi_TrumpetAI_QTE_Guang_Damage"
        }
        (crate::characters::iroi::regression::LambType::Trumpet, false) => {
            "GE_Player_Oneiroi_TrumpetAI_QTE_Zhou_Damage"
        }
    }
}

#[cfg(test)]
#[path = "execution_discord_tests.rs"]
mod discord_tests;
