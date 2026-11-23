use crate::acceptance_spec::AcceptanceScenarioKind;
use crate::execution::ExecutionLimits;
use crate::formula::Ratio;
use crate::game_batch::transform_timing;
use crate::published_teams::{ActorOverride, get_character_profile};
use crate::simulation::{
    self, ExactFraction, GameActionCommand, GameActorConfig, GameActorStaticStats,
    GameScenarioReport, GameScenarioRequest, ResearchCritPolicy, ResearchCritRollPolicy,
};
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

mod model;
pub use model::{
    DEFAULT_PILOT_HYPOTHESIS, PilotBudget, PilotCheckpoint, PilotExecutionUnit,
    PilotProfilingSummary, PilotReport, PilotRequest, PilotUnitFailure, PilotUnitResult,
    PrunedCandidateReceipt, StageGWorkloadProjection, VERSION,
};
mod defaults;
pub use defaults::default_pilot_request;
mod scenarios;
pub use scenarios::{build_pilot_scenario, preview_entry_offset_us};
mod measurement;
pub(crate) use measurement::execute_pilot_unit;
mod runner;
pub use runner::run_pilot;

#[cfg(test)]
mod tests;
