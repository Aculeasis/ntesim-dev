use crate::acceptance_spec::AcceptanceScenarioKind;
use crate::execution::ExecutionLimits;
use crate::formula::Ratio;
use crate::game_batch::{self, Statistics, TimingPolicy};
use crate::published_teams::{ActorOverride, get_character_profile, parse_published_variants_csv};
#[cfg(test)]
pub use crate::simulation::ResearchCritRollPolicy;
use crate::simulation::{self, GameScenarioReport, ResearchCritPolicy};
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::{BTreeMap, BTreeSet};

mod model;
pub use model::{
    DEFAULT_FINALIST_HYPOTHESIS, FinalistClassification, FinalistEvaluationReport,
    FinalistTimingSummary, FinalistValidationRequest,
};
mod defaults;
pub(crate) use defaults::{default_scenario_kind, default_timing_multipliers};
mod evaluation;
pub use evaluation::evaluate_finalist;

#[cfg(test)]
mod tests;
