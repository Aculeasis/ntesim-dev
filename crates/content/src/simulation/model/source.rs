//! Native source execution transport and occurrence identities.
use super::{GameScenarioReport, GameScenarioRequest};
use crate::dsl::{SourceLocation, validation::Diagnostic};
use serde::{Deserialize, Serialize};

/// Authored scenario laws, independent of a particular seed's sampled events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceScenarioAssumptions {
    pub duration_specified: bool,
    pub periodic_damage: Vec<SourcePeriodicDamageAssumption>,
    pub fixed_damage: Vec<SourceFixedDamageAssumption>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourcePeriodicDamageAssumption {
    pub interval_min_us: u64,
    pub interval_max_us: u64,
    pub amount_min: u64,
    pub amount_max: u64,
    /// None selects the actor active at each delivery.
    pub actor_index: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceFixedDamageAssumption {
    pub event: super::GameActorDamageEvent,
    pub source_target: Option<usize>,
    pub basis: Option<super::GameIncomingDamageBasis>,
}

/// A source-only terminal boundary, after the settled native input prefix.
/// It cannot turn a failed or unfinished execution into a successful one.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameSourceHalt {
    pub time_us: u64,
    pub kind: GameSourceHaltKind,
    pub diagnostic: Diagnostic,
    pub context: SourceFailureContext,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameSourceHaltKind {
    Failed,
    Pending,
    EventLimit { limit: u64 },
    AbsoluteLimit { limit_us: u64 },
}

impl GameSourceHalt {
    pub(crate) fn validate(&self, request: &GameScenarioRequest) -> Result<(), String> {
        use crate::dsl::validation::DiagnosticCode;
        let loc = &self.diagnostic.loc;
        if request.action_admission_order != super::GameActionAdmissionOrder::AfterSettledEvents
            || (request.source_exhausted
                && !matches!(
                    self.kind,
                    GameSourceHaltKind::EventLimit { .. }
                        | GameSourceHaltKind::AbsoluteLimit { .. }
                ))
            || self.time_us > request.absolute_limit_us
            || self.context.command_index.is_some()
            || loc.line == 0
            || loc.column == 0
            || loc.length == 0
            || loc.offset.checked_add(loc.length).is_none()
            || self.diagnostic.message.is_empty()
            || request
                .actions
                .iter()
                .any(|action| action.timestamp_us() > self.time_us)
            || request
                .input_time_contexts
                .iter()
                .any(|context| context.end_us > self.time_us)
        {
            return Err("Invalid source halt boundary.".into());
        }
        let valid_cause = match self.kind {
            GameSourceHaltKind::Failed => matches!(
                self.diagnostic.code,
                DiagnosticCode::Evaluation
                    | DiagnosticCode::Arithmetic
                    | DiagnosticCode::WorkLimit
                    | DiagnosticCode::ActionLimit
            ),
            GameSourceHaltKind::Pending => {
                self.time_us <= request.absolute_limit_us
                    && matches!(
                        self.diagnostic.code,
                        DiagnosticCode::Horizon | DiagnosticCode::EnergyGate
                    )
            }
            GameSourceHaltKind::EventLimit { limit } => {
                limit > 0 && self.diagnostic.code == DiagnosticCode::EventLimit
            }
            GameSourceHaltKind::AbsoluteLimit { limit_us } => {
                limit_us == request.absolute_limit_us
                    && self.time_us == limit_us
                    && self.diagnostic.code == DiagnosticCode::Horizon
            }
        };
        if !valid_cause {
            return Err("Invalid source halt cause.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceActionWindow {
    Attack,
    Hold,
    Jump,
    Dash,
    Skill,
    Swap,
}

impl SourceActionWindow {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Attack => "attack",
            Self::Hold => "hold",
            Self::Jump => "jump",
            Self::Dash => "dash",
            Self::Skill => "skill",
            Self::Swap => "swap",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceOccurrenceKind {
    TargetGauge,
    Action,
    Wait,
    WaitUntil,
    WaitForEnergy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceOccurrence {
    pub index: usize,
    pub loc: SourceLocation,
    pub kind: SourceOccurrenceKind,
    pub started_us: u64,
    pub completed_us: Option<u64>,
    /// Requested point at which the source yields; the montage can still run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_window: Option<SourceActionWindow>,
    /// Includes the implicit switch when the requested actor was off field.
    pub command_indices: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceScenarioReport {
    pub source: String,
    /// Executed input before talent projection, with dynamic timestamps.
    pub request: GameScenarioRequest,
    pub report: GameScenarioReport,
    pub action_locations: Vec<SourceLocation>,
    pub occurrences: Vec<SourceOccurrence>,
    pub diagnostic: Option<Diagnostic>,
    pub failure_context: Option<SourceFailureContext>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceFailureContext {
    /// Failed yielded input, or the attempted next occurrence when expression
    /// evaluation/work limits fail before yielding another input.
    pub occurrence_index: usize,
    /// Only actual rejected admission supplies a failed combat command.
    pub command_index: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceRunResponse {
    pub execution: Option<SourceScenarioReport>,
    pub diagnostics: Vec<Diagnostic>,
}
