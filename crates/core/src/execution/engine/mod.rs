use super::limits::{CausalPriority, ExecutionLimits};
use super::model::{
    ActorState, AuditEvent, ExecutionError, ExecutionScenarioReport, ExecutionScenarioRequest,
    HaltDiagnostic, ResourcePoolReport, ResourcePoolState, ShieldState, StepAction, TargetState,
};
use crate::combat::DamageKind;
use ntesim_contracts::sha256;
use std::collections::{BTreeMap, BinaryHeap};

mod queue;
pub(crate) use queue::QueuedEvent;
mod state;
pub(crate) use state::SimState;
mod damage;
pub(crate) use damage::{compute_damage_mitigation, settle_target_damage};
mod invariants;
pub(crate) use invariants::verify_state_invariants;
mod settlement;
pub(crate) use settlement::execute_step_action;
mod runtime;
pub use runtime::{evaluate_execution_scenario, run_execution_scenario};
