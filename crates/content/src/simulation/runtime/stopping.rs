//! Relative completion, independent absolute ceiling and source exhaustion.
use super::execution::PreparedExecution;
use crate::simulation::timer_queue::{EventHead, HeadKind};
use crate::simulation::{GameActionAdmissionOrder, GameScenarioRequest, GameStopPolicy};

pub(super) fn victory_time_limit(relative_duration_us: u64) -> String {
    format!(
        "VICTORY_NOT_REACHED: combat-time limit of {} seconds reached before all targets were defeated.",
        combat_seconds(relative_duration_us)
    )
}

fn combat_seconds(time_us: u64) -> String {
    let seconds = time_us / 1_000_000;
    let micros = time_us % 1_000_000;
    if micros == 0 {
        seconds.to_string()
    } else {
        format!("{seconds}.{micros:06}")
            .trim_end_matches('0')
            .into()
    }
}

impl PreparedExecution {
    pub(super) fn stop_deadline(&self, request: &GameScenarioRequest) -> Result<u64, String> {
        if request.stop == GameStopPolicy::Victory {
            return Ok(request.absolute_limit_us);
        }
        let now = self.state.mode_clock.absolute_us();
        let remaining = request
            .relative_duration_us
            .saturating_sub(self.state.mode_clock.relative_us());
        if self.state.mode_clock.is_paused()
            || remaining > request.absolute_limit_us.saturating_sub(now)
        {
            Ok(request.absolute_limit_us)
        } else {
            now.checked_add(remaining)
                .ok_or_else(|| "relative endpoint overflow".into())
        }
    }
    pub(super) fn apply_source_halt(&mut self, halt: &crate::simulation::GameSourceHalt) {
        self.success = matches!(
            halt.kind,
            crate::simulation::GameSourceHaltKind::Pending
                | crate::simulation::GameSourceHaltKind::AbsoluteLimit { .. }
        );
        if let crate::simulation::GameSourceHaltKind::AbsoluteLimit { limit_us } = halt.kind {
            self.limit_reached =
                Some(crate::simulation::GameExecutionLimit::AbsoluteTime { limit_us });
        }
        self.execution_completed = false;
        self.halt_diagnostic = Some(halt.diagnostic.message.clone());
        self.pending_events = self.queue.len()
            + usize::from(matches!(
                halt.kind,
                crate::simulation::GameSourceHaltKind::Pending
                    | crate::simulation::GameSourceHaltKind::AbsoluteLimit { .. }
            ));
    }

    pub(super) fn victory_settled(&self, request: &GameScenarioRequest, head: EventHead) -> bool {
        (request.stop == GameStopPolicy::Victory || self.live_source)
            && self.all_targets_defeated()
            && (head.time_us > self.state.current_time_us || head.kind != HeadKind::Other)
    }

    pub(super) fn stop_at_horizon(&mut self, request: &GameScenarioRequest, event: EventHead) {
        let sequential =
            request.action_admission_order == GameActionAdmissionOrder::AfterSettledEvents;
        if request.stop == GameStopPolicy::Time
            && self.pending_energy_wait.is_none()
            && self.parked_energy_actions.is_empty()
            && self.unqueued_energy_waits == 0
            && !matches!(event.kind, HeadKind::EnergyWait { .. })
        {
            // Time completion clips any admitted action or authored wait. The
            // unprocessed event remains visible without requiring its tail.
            self.pending_events = self.queue.len();
            return;
        }
        if self.halt_diagnostic.is_none() && self.pending_energy_wait.is_none() {
            self.halt_diagnostic = Some(if request.stop == GameStopPolicy::Victory {
                victory_time_limit(request.absolute_limit_us)
            } else {
                match event.kind {
                    HeadKind::EnergyWait { index: wait_index } => format!(
                        "LIVE_ENERGY_WAIT_PENDING: gate {wait_index} starts at {} after the horizon",
                        event.time_us
                    ),
                    _ => format!(
                        "SCENARIO_HORIZON_PENDING: next event at {} exceeds horizon {}",
                        event.time_us, self.state.current_time_us
                    ),
                }
            });
        }
        self.pending_events = self.pending_event_count();
        self.execution_completed = false;
        if sequential && !request.source_exhausted {
            self.pending_events += 1;
        }
    }

    pub(super) fn finish_stop_policy(
        &mut self,
        request: &GameScenarioRequest,
    ) -> Result<(), String> {
        if !self.success {
            return Ok(());
        }
        if self.all_targets_defeated() {
            self.execution_completed = true;
            self.halt_diagnostic = None;
            self.pending_events = self.queue.len();
            return Ok(());
        }
        if request.stop == GameStopPolicy::Victory {
            self.execution_completed = false;
            self.halt_diagnostic.get_or_insert_with(|| {
                if self.state.current_time_us >= request.absolute_limit_us {
                    victory_time_limit(request.absolute_limit_us)
                } else {
                    format!(
                        "VICTORY_NOT_REACHED: rotation and pending effects ended before all targets were defeated (combat-time limit: {} seconds).",
                        combat_seconds(request.absolute_limit_us)
                    )
                }
            });
            self.pending_events = self.pending_events.max(self.queue.len()) + 1;
            return Ok(());
        }
        // Every in-horizon event has settled, including unresolved energy gates.
        // Empty gaps consume real combat time without scheduler polling.
        let end = self.stop_deadline(request)?;
        self.advance_clock(request, end)?;
        if self.state.mode_clock.relative_us() < request.relative_duration_us {
            self.execution_completed = false;
            self.limit_reached = Some(crate::simulation::GameExecutionLimit::AbsoluteTime {
                limit_us: request.absolute_limit_us,
            });
            self.halt_diagnostic = Some(format!(
                "ABSOLUTE_TIME_LIMIT: absolute ceiling {} seconds reached at {} relative seconds of {} requested seconds.",
                combat_seconds(request.absolute_limit_us),
                combat_seconds(self.state.mode_clock.relative_us()),
                combat_seconds(request.relative_duration_us)
            ));
        }
        self.state.expire_buffs(&mut self.trace);
        Ok(())
    }
}
