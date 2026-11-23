use super::{GameActionCommand, SourceLocation, Trial, Value};
use crate::execution::CausalPriority;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};

impl Trial<'_> {
    pub(super) fn target_gauge(
        &mut self,
        target: &crate::dsl::Expression,
        gauge: Option<&Value>,
        loc: &SourceLocation,
    ) -> Result<(), String> {
        self.command_budget()?;
        let target_index =
            crate::dsl::action_parameters::target(target, self.request.targets.len())?;
        let (numerator, denominator) = gauge
            .map(|value| value.number()?.to_safe_ratio())
            .transpose()?
            .unwrap_or((0, 1));
        let now = self.execution.state.current_time_us;
        let index = self.request.actions.len();
        let command = GameActionCommand::TargetGauge {
            timestamp_us: now,
            target_index,
            gauge: crate::formula::Ratio {
                numerator,
                denominator,
            },
        };
        if self.execution.state.record_detail {
            self.replay_actions.push(command.clone());
            self.locations.push(loc.clone());
        }
        self.request.actions.push(command);
        self.packets.push(None);
        self.execution.seq = self
            .execution
            .seq
            .checked_add(1)
            .ok_or("event sequence overflow")?;
        self.execution.queue.push(ScheduledGameEvent {
            time_us: now,
            priority: CausalPriority::Observation,
            seq: self.execution.seq,
            data: GameEventData::ActionAdmission {
                command_index: index,
            },
        });
        self.advance(now)
    }
}
