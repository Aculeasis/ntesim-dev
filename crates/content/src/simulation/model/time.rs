//! Exclusive player-input time on the real combat axis, independent of damage.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameTimeCause {
    ActionRecovery,
    SwitchRecovery,
    AuthoredWait,
    WaitUntil,
    EnergyGate,
    ScriptExhaustion,
    Idle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTimeSource {
    pub occurrence_index: usize,
    pub loc: crate::dsl::SourceLocation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameTimeContextKind {
    Action { command_index: usize },
    Wait { command_index: usize },
    WaitUntil { command_index: usize },
    Energy { required: u64 },
}

/// Resolved source continuation boundaries retained by native replay. These
/// annotate input occupation only; they cannot grant readiness or schedule work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameTimeContext {
    pub start_us: u64,
    pub end_us: u64,
    pub actor_index: usize,
    pub source: GameTimeSource,
    pub context: GameTimeContextKind,
}

impl GameTimeContext {
    pub(crate) fn validate_all(request: &super::GameScenarioRequest) -> Result<(), String> {
        let mut end = 0;
        for context in &request.input_time_contexts {
            if context.start_us < end
                || context.end_us < context.start_us
                || context.end_us > request.absolute_limit_us
                || context.actor_index >= request.actors.len()
                || context.source.loc.line == 0
                || context.source.loc.column == 0
                || context.source.loc.length == 0
                || context
                    .source
                    .loc
                    .offset
                    .checked_add(context.source.loc.length)
                    .is_none()
            {
                return Err("Invalid or overlapping source input-time context.".into());
            }
            match context.context {
                GameTimeContextKind::Energy { required } => {
                    if required > request.actors[context.actor_index].energy_capacity {
                        return Err("Input-time energy threshold exceeds actor capacity.".into());
                    }
                }
                GameTimeContextKind::Action { command_index } => {
                    if !matches!(
                        request.actions.get(command_index),
                        Some(
                            super::GameActionCommand::InvokeAbility { .. }
                                | super::GameActionCommand::PlayerInput { .. }
                                | super::GameActionCommand::SwitchActor { .. }
                        )
                    ) {
                        return Err("Input-time action context requires a combat command.".into());
                    }
                }
                GameTimeContextKind::Wait { command_index }
                | GameTimeContextKind::WaitUntil { command_index } => {
                    if !matches!(request.actions.get(command_index), Some(super::GameActionCommand::Wait { timestamp_us, .. }) if *timestamp_us == context.start_us)
                    {
                        return Err(
                            "Input-time wait context requires its authored wait command.".into(),
                        );
                    }
                }
            }
            end = context.end_us;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTimeEnergy {
    pub required: u64,
    pub available: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTimeInterval {
    pub start_us: u64,
    pub end_us: u64,
    pub duration_us: u64,
    pub actor_index: usize,
    pub cause: GameTimeCause,
    pub command_index: Option<usize>,
    pub source: Option<GameTimeSource>,
    pub energy: Option<GameTimeEnergy>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTimeTotal {
    pub actor_index: usize,
    pub cause: GameTimeCause,
    pub duration_us: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTimeLedger {
    pub intervals: Vec<GameTimeInterval>,
    pub totals: Vec<GameTimeTotal>,
    pub warnings: Vec<GameTimeWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTimeWarning {
    pub actor_index: usize,
    pub cause: GameTimeCause,
    pub episodes: usize,
    pub total_duration_us: u64,
    pub longest_duration_us: u64,
    pub first_start_us: u64,
}

impl GameTimeLedger {
    /// Classification policy, not a combat mechanic or a judgment of a rotation.
    pub const PROLONGED_US: u64 = 5_000_000;

    pub(crate) fn finish(&mut self) {
        self.warnings = self.prolonged_warnings();
    }

    fn prolonged_warnings(&self) -> Vec<GameTimeWarning> {
        let mut warnings = std::collections::BTreeMap::new();
        let mut intervals = self.intervals.iter().peekable();
        while let Some(first) = intervals.next() {
            let mut end = first.end_us;
            while let Some(next) = intervals.peek()
                && next.actor_index == first.actor_index
                && next.cause == first.cause
                && next.start_us == end
            {
                end = intervals.next().expect("peeked interval").end_us;
            }
            let duration = end - first.start_us;
            if duration < Self::PROLONGED_US
                || matches!(
                    first.cause,
                    GameTimeCause::ActionRecovery | GameTimeCause::SwitchRecovery
                )
            {
                continue;
            }
            let warning =
                warnings
                    .entry((first.actor_index, first.cause))
                    .or_insert(GameTimeWarning {
                        actor_index: first.actor_index,
                        cause: first.cause,
                        episodes: 0,
                        total_duration_us: 0,
                        longest_duration_us: 0,
                        first_start_us: first.start_us,
                    });
            warning.episodes += 1;
            warning.total_duration_us += duration;
            warning.longest_duration_us = warning.longest_duration_us.max(duration);
        }
        warnings.into_values().collect()
    }

    pub(crate) fn push(&mut self, interval: GameTimeInterval) {
        if interval.duration_us == 0 {
            return;
        }
        if let Some(last) = self.intervals.last_mut()
            && last.end_us == interval.start_us
            && last.actor_index == interval.actor_index
            && last.cause == interval.cause
            && last.command_index == interval.command_index
            && last.source == interval.source
            && last.energy == interval.energy
        {
            last.end_us = interval.end_us;
            last.duration_us += interval.duration_us;
        } else {
            self.intervals.push(interval.clone());
        }
        if let Some(total) = self.totals.iter_mut().find(|total| {
            total.actor_index == interval.actor_index && total.cause == interval.cause
        }) {
            total.duration_us += interval.duration_us;
        } else {
            self.totals.push(GameTimeTotal {
                actor_index: interval.actor_index,
                cause: interval.cause,
                duration_us: interval.duration_us,
            });
            self.totals
                .sort_by_key(|total| (total.actor_index, total.cause));
        }
    }

    pub fn validate(&self, absolute_elapsed_us: u64) -> Result<(), String> {
        let mut end = 0;
        let mut rebuilt = Self::default();
        for interval in &self.intervals {
            if interval.start_us != end
                || interval.end_us <= interval.start_us
                || interval.duration_us != interval.end_us - interval.start_us
            {
                return Err(
                    "Input-time intervals do not form an exclusive combat-time partition.".into(),
                );
            }
            rebuilt.push(interval.clone());
            end = interval.end_us;
        }
        if end != absolute_elapsed_us || rebuilt.totals != self.totals {
            return Err("Input-time totals differ from elapsed combat time or intervals.".into());
        }
        if self.prolonged_warnings() != self.warnings {
            return Err("Input-time warnings differ from their exact intervals.".into());
        }
        Ok(())
    }
}
