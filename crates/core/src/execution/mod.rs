mod engine;
mod limits;
mod model;

pub use engine::{evaluate_execution_scenario, run_execution_scenario};
pub use limits::{CausalPriority, ExecutionLimits};
pub use model::{
    ActorSpec, ActorState, AuditEvent, ExecutionError, ExecutionScenarioReport,
    ExecutionScenarioRequest, ExecutionStepSpec, HaltDiagnostic, ResourcePoolReport,
    ResourcePoolSpec, ResourcePoolState, ShieldSpec, ShieldState, StepAction, TargetSpec,
    TargetState,
};

#[cfg(test)]
mod tests;
