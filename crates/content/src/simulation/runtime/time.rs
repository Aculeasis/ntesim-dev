//! Record time before settling a timestamp. Equal-time work adds no interval.
use super::execution::PreparedExecution;
use crate::simulation::{
    GameActionCommand, GameScenarioRequest, GameTimeCause, GameTimeContextKind, GameTimeEnergy,
    GameTimeInterval,
};

fn action_cause(request: &GameScenarioRequest, command: Option<usize>) -> GameTimeCause {
    if command.is_some_and(|index| {
        matches!(
            request.actions[index],
            GameActionCommand::SwitchActor { .. }
        )
    }) {
        GameTimeCause::SwitchRecovery
    } else {
        GameTimeCause::ActionRecovery
    }
}

impl PreparedExecution {
    pub(super) fn switch_input(
        &mut self,
        request: &GameScenarioRequest,
        packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
        command_index: usize,
        target_actor_index: usize,
        cooldown_us: u64,
    ) -> Result<bool, String> {
        let previous = self.state.active_actor_index;
        let admitted = self.switch_actor(
            request,
            packets,
            command_index,
            target_actor_index,
            cooldown_us,
        )?;
        if admitted && previous != self.state.active_actor_index {
            self.active_input = Some(command_index);
        }
        Ok(admitted)
    }

    pub(super) fn record_active_input(&mut self, request: &GameScenarioRequest, index: usize) {
        if !matches!(
            request.actions[index],
            GameActionCommand::TargetGauge { .. }
        ) {
            self.active_input = Some(index);
        }
    }

    pub(super) fn advance_clock(
        &mut self,
        request: &GameScenarioRequest,
        end_us: u64,
    ) -> Result<(), String> {
        let from_us = self.state.current_time_us;
        let delta = end_us
            .checked_sub(from_us)
            .ok_or("runtime clock reversal")?;
        let world_paused = !self
            .state
            .mode_clock
            .progresses(ntesim_core::kernel::timing::ClockDomain::World);
        let object_paused = !self
            .state
            .mode_clock
            .progresses(ntesim_core::kernel::timing::ClockDomain::ObjectLifetime);
        for (context, clock) in &mut self.state.replay_clocks {
            if !self.state.mode_clock.progresses(
                ntesim_core::kernel::timing::ClockDomain::ReplayAction { context: *context },
            ) {
                clock.freeze(context.owner, from_us, delta)?;
            }
        }
        for (owner_index, actor) in self.state.actors.iter_mut().enumerate() {
            crate::simulation::timer_state::owner_timers(
                &mut actor.character,
                from_us,
                delta,
                world_paused,
                object_paused,
            )?;
            let shift = |deadline: &mut u64| -> Result<(), String> {
                if *deadline > from_us {
                    *deadline = deadline
                        .checked_add(delta)
                        .ok_or("paused timer projection overflow")?;
                }
                Ok(())
            };
            if world_paused {
                crate::simulation::timer_state::gear_timers(&mut actor.equipment, from_us, delta)?;
                shift(&mut actor.switch_cooldown_until_us)?;
                for shield in &mut actor.shields {
                    shift(&mut shield.expires_at_us)?;
                }
                if let crate::characters::RuntimeState::Hotori(owner) = &mut actor.character {
                    let timepiece = &mut owner.timepiece;
                    timepiece.settle_to(from_us)?;
                    timepiece.settled_until_us = end_us;
                    if let Some(deadline) = &mut timepiece.rewind_until_us {
                        shift(deadline)?;
                    }
                    if let Some(deadline) = &mut timepiece.a3_until_us {
                        shift(deadline)?;
                    }
                    shift(&mut timepiece.a3_icd_until_us)?;
                    if let Some(mode) = &mut timepiece.mode {
                        let domain = ntesim_core::kernel::timing::ClockDomain::Action {
                            context: ntesim_core::kernel::timing::PauseOwner {
                                owner: owner_index,
                                generation: mode.root_command_index as u64,
                            },
                            advance_during_active_stop: true,
                        };
                        if !self.state.mode_clock.progresses(domain) {
                            shift(&mut mode.entry_us)?;
                            shift(&mut mode.end_us)?;
                            shift(&mut timepiece.switch_locked_until_us)?;
                            if let Some(deadline) = &mut timepiece.katana_next_input_us {
                                shift(deadline)?;
                            }
                        }
                    }
                }
            }
            if actor
                .action_clock
                .is_some_and(|domain| !self.state.mode_clock.progresses(domain))
            {
                shift(&mut actor.locked_until_us)?;
                shift(&mut actor.switch_action_locked_until_us)?;
                if let crate::characters::RuntimeState::Iroi(state) = &mut actor.character
                    && state.dream_active
                    && state.dream_release_us > from_us
                {
                    shift(&mut state.dream_release_us)?;
                }
                crate::simulation::timer_state::owner_action(
                    &mut actor.character,
                    owner_index,
                    from_us,
                    delta,
                )?;
            }
        }
        if world_paused && let Some(stamina) = &mut self.state.combat_stamina {
            stamina.advance(from_us)?;
            if stamina.recovery_ready_at_us > from_us {
                stamina.recovery_ready_at_us = stamina
                    .recovery_ready_at_us
                    .checked_add(delta)
                    .ok_or("stamina timer overflow")?;
            }
            stamina.updated_at_us = end_us;
        }
        if world_paused {
            self.state.reactions.pause_world(from_us, delta)?;
            for target in &mut self.state.targets {
                crate::simulation::timer_state::future(
                    &mut target.airborne_until_us,
                    from_us,
                    delta,
                )?;
                if let Some(state) = &mut target.break_state
                    && let Some(deadline) = &mut state.broken_until_us
                {
                    crate::simulation::timer_state::future(deadline, from_us, delta)?;
                }
            }
        }
        let relative_from_us = self.state.mode_clock.relative_us();
        self.state
            .mode_clock
            .advance_to(end_us)
            .map_err(|e| e.to_string())?;
        let relative_end_us = self.state.mode_clock.relative_us();
        let relative_delta = relative_end_us
            .checked_sub(relative_from_us)
            .ok_or("relative clock reversal")?;
        assert!(
            relative_delta == 0 || relative_delta == delta,
            "segment invariant violated: relative delta must be 0 or equal to absolute delta"
        );
        for actor in &mut self.state.actors {
            actor.world_time_us = self.state.mode_clock.relative_us();
            actor
                .ability_cooldowns
                .advance(end_us, self.state.mode_clock.relative_us())
                .map_err(|e| e.to_string())?;
        }
        if end_us > from_us
            && let Some(actor) = self.state.actors.get_mut(self.state.active_actor_index)
        {
            actor.field_time_absolute_us = actor
                .field_time_absolute_us
                .checked_add(delta)
                .ok_or("actor absolute field time overflow")?;
            actor.field_time_relative_us = actor
                .field_time_relative_us
                .checked_add(relative_delta)
                .ok_or("actor relative field time overflow")?;
        }
        if end_us > from_us
            && let Some(actor) = self.state.actors.get(self.state.active_actor_index)
            && actor.hp > 0
        {
            for shield in &actor.shields {
                let active_end = end_us.min(shield.expires_at_us);
                if active_end > from_us {
                    let dt = u128::from(active_end - from_us);
                    let hp = u128::from(shield.remaining_hp);
                    self.accumulated_shield_hp_us = self
                        .accumulated_shield_hp_us
                        .checked_add(hp * dt)
                        .ok_or("shield HP integral overflow")?;
                    if relative_delta > 0 {
                        self.accumulated_relative_shield_hp_us = self
                            .accumulated_relative_shield_hp_us
                            .checked_add(hp * dt)
                            .ok_or("relative shield HP integral overflow")?;
                    }
                }
            }
        }
        let mut now = self.state.current_time_us;
        while now < end_us {
            let mut interval = self.time_interval(request, now, end_us);
            interval.duration_us = interval.end_us - now;
            now = interval.end_us;
            if let Some(batch) = &mut self.batch {
                batch.observe_input_time(&interval)?;
            } else {
                self.input_time.push(interval);
            }
        }
        self.state.current_time_us = end_us;
        self.queue.sync(&self.state.mode_clock)?;
        Ok(())
    }

    fn time_interval(&self, request: &GameScenarioRequest, now: u64, end: u64) -> GameTimeInterval {
        let actor_index = self.state.active_actor_index;
        let mut interval = GameTimeInterval {
            start_us: now,
            end_us: end,
            duration_us: 0,
            actor_index,
            cause: GameTimeCause::ScriptExhaustion,
            command_index: None,
            source: None,
            energy: None,
        };
        let index = request
            .input_time_contexts
            .partition_point(|context| context.end_us <= now);
        let context = request.input_time_contexts.get(index);
        if let Some(context) = context {
            interval.end_us = interval.end_us.min(if context.start_us > now {
                context.start_us
            } else {
                context.end_us
            });
        }
        let context = context.filter(|context| context.start_us <= now);
        let runtime = &self.state.actors[actor_index];
        let recovery = crate::characters::source_choices::completion_deadline(runtime, now);
        if recovery > now {
            interval.cause = action_cause(request, self.active_input);
            interval.command_index = self.active_input;
            interval.end_us = interval.end_us.min(recovery);
            if let Some(context) = context
                && matches!(context.context, GameTimeContextKind::Action { command_index } if Some(command_index) == self.active_input)
            {
                interval.source = Some(context.source.clone());
            }
        } else if let Some(context) = context {
            interval.actor_index = context.actor_index;
            interval.source = Some(context.source.clone());
            match context.context {
                GameTimeContextKind::Action { command_index } => {
                    interval.cause = action_cause(request, Some(command_index));
                    interval.command_index = Some(command_index);
                }
                GameTimeContextKind::Wait { command_index }
                | GameTimeContextKind::WaitUntil { command_index } => {
                    interval.cause =
                        if matches!(context.context, GameTimeContextKind::WaitUntil { .. }) {
                            GameTimeCause::WaitUntil
                        } else {
                            GameTimeCause::AuthoredWait
                        };
                    interval.command_index = Some(command_index);
                }
                GameTimeContextKind::Energy { required } => {
                    interval.cause = GameTimeCause::EnergyGate;
                    interval.energy = Some(GameTimeEnergy {
                        required,
                        available: self.state.actors[context.actor_index].energy.current(),
                    });
                }
            }
        } else if let Some(wait) = &self.pending_energy_wait {
            interval.cause = GameTimeCause::EnergyGate;
            interval.actor_index = wait.instruction.actor_index;
            interval.energy = Some(GameTimeEnergy {
                required: wait.instruction.energy_at_least,
                available: self.state.actors[wait.instruction.actor_index]
                    .energy
                    .current(),
            });
        } else {
            self.scheduled_gap(request, now, &mut interval);
        }
        interval
    }

    fn scheduled_gap(
        &self,
        request: &GameScenarioRequest,
        now: u64,
        interval: &mut GameTimeInterval,
    ) {
        if let Some(branch) =
            self.live_branch_receipts.iter().rev().find(|branch| {
                branch.time_us <= now && now < branch.time_us + branch.selected_wait_us
            })
        {
            interval.cause = GameTimeCause::AuthoredWait;
            interval.end_us = interval
                .end_us
                .min(branch.time_us + branch.selected_wait_us);
            return;
        }
        for (index, command, time) in self.action_times(request) {
            if time > i128::from(now) {
                interval.cause = GameTimeCause::Idle;
                interval.end_us = i128::from(interval.end_us).min(time) as u64;
            }
            if let GameActionCommand::Wait { duration_us, .. } = command {
                let end = self.wait_end(index, time, *duration_us);
                if time <= i128::from(now) && i128::from(now) < end {
                    interval.cause = GameTimeCause::AuthoredWait;
                    interval.command_index = Some(index);
                    interval.end_us = i128::from(interval.end_us).min(end) as u64;
                    return;
                }
            }
        }
    }
}
