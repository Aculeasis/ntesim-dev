use crate::acceptance_spec::{
    AcceptanceDeliverables, AcceptanceScenarioKind, compile_acceptance_deliverables,
};
use crate::composition::{self, CompositionReport};
use crate::execution::ExecutionLimits;
use crate::finalists::{
    FinalistClassification, FinalistEvaluationReport, FinalistValidationRequest, evaluate_finalist,
};
use crate::formula::Ratio;
use crate::pilot::build_pilot_scenario;
use crate::simulation::ExactFraction;
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use std::thread;

mod model;
pub use model::{
    AcceptanceCheckpoint, AcceptancePhase, AcceptanceRunReport, AcceptanceRunRequest,
    AcceptanceWorkerConfig, CompositionScreeningFailure, CompositionScreeningSummary,
    DEFAULT_ACCEPTANCE_HYPOTHESIS, TopCandidateEntry, VERSION,
};
mod defaults;
pub use defaults::default_acceptance_request;
mod screening;
pub(crate) use screening::screen_single_composition;
mod pipeline;
pub use pipeline::run_acceptance_pipeline;

#[cfg(test)]
mod tests;
