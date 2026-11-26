use crate::execution::ExecutionLimits;
#[cfg(test)]
use crate::simulation::state::GameSimulatorState;
use crate::simulation::{GameScenarioReport, GameScenarioRequest};
use serde_json::Value;
use serde_json::value::RawValue;
#[cfg(test)]
use std::collections::BTreeMap;

mod batch;
mod execution;
mod parry;
mod program;
mod report;
mod source;
mod stopping;
mod time;
pub use batch::{CompletedSourceTrial, SourceTrialOutcome};
pub use source::{
    PreparedSourceScenario, SourceBoundary, SourceCheckpoint, SourceControlReport,
    run_compiled_source, run_source,
};

#[cfg(test)]
pub(crate) use execution::PreparedExecution;
#[cfg(not(test))]
use execution::PreparedExecution;
use program::validate_live_program;
pub use report::BorrowedGameScenarioReport;
#[cfg(test)]
pub(crate) use report::merge_model_assumptions;
pub fn run_game_scenario(
    request: &GameScenarioRequest,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    let report = run_game_scenario_report(request, limits)?;
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

/// Typed result for adapters; avoid materializing and decoding a second JSON tree.
pub fn run_game_scenario_report(
    request: &GameScenarioRequest,
    limits: &ExecutionLimits,
) -> Result<GameScenarioReport, String> {
    PreparedGameScenario::new(request, limits)?.run(request.seed)
}

/// Validated immutable rotation shared by independent seeded executions.
/// No combat state, event queue or RNG survives a call to `run`.
pub struct PreparedGameScenario {
    request: GameScenarioRequest,
    talent_receipts: Vec<Value>,
    serialized_talents: Box<RawValue>,
    packets: Vec<Option<crate::simulation::metadata::ActionPacketMetadata>>,
    limits: ExecutionLimits,
}

impl PreparedGameScenario {
    pub fn new(request: &GameScenarioRequest, limits: &ExecutionLimits) -> Result<Self, String> {
        #[cfg(feature = "performance-diagnostics")]
        let _span = crate::performance::Span::new("scenario.prepare");
        let packets = crate::simulation::validation::validate_and_resolve(request, limits)?;
        let projected = crate::simulation::talents::project(request)?;
        let (request, talent_receipts, packets) = match projected {
            Some((request, receipts)) => {
                let packets =
                    crate::simulation::validation::validate_and_resolve(&request, limits)?;
                (request, receipts, packets)
            }
            None => (request.clone(), Vec::new(), packets),
        };
        let serialized_talents =
            serde_json::value::to_raw_value(&talent_receipts).map_err(|error| error.to_string())?;
        Ok(Self {
            request,
            talent_receipts,
            serialized_talents,
            packets,
            limits: *limits,
        })
    }

    pub fn run(&self, seed: u64) -> Result<GameScenarioReport, String> {
        Ok(self.run_borrowed(seed)?.into_owned())
    }

    pub fn run_borrowed(&self, seed: u64) -> Result<BorrowedGameScenarioReport<'_>, String> {
        self.run_borrowed_with_program(seed, &[], &[])
    }

    fn run_borrowed_with_program(
        &self,
        seed: u64,
        branches: &[crate::simulation::GameLiveBranch],
        energy_waits: &[crate::simulation::GameLiveEnergyWait],
    ) -> Result<BorrowedGameScenarioReport<'_>, String> {
        // Validation/projection depend on the rotation and policy, never on seed.
        // The runtime receives the actual seed, with fresh RNG and mutable state.
        let report = run_prepared(
            &self.request,
            seed,
            &self.limits,
            &self.packets,
            branches,
            energy_waits,
        )?;
        Ok(BorrowedGameScenarioReport {
            report,
            talents: &self.talent_receipts,
            serialized_talents: &self.serialized_talents,
        })
    }
}

/// Prepared live NTE program. The immutable scenario and packet metadata are
/// validated once; combat state, branch outcomes and RNG are fresh per run.
pub struct PreparedGameProgram {
    scenario: PreparedGameScenario,
    branches: Vec<crate::simulation::GameLiveBranch>,
    energy_waits: Vec<crate::simulation::GameLiveEnergyWait>,
}

impl PreparedGameProgram {
    pub fn new(
        program: &crate::simulation::GameScenarioProgramRequest,
        limits: &ExecutionLimits,
    ) -> Result<Self, String> {
        validate_live_program(program)?;
        Ok(Self {
            scenario: PreparedGameScenario::new(&program.request, limits)?,
            branches: program.branches.clone(),
            energy_waits: program.energy_waits.clone(),
        })
    }

    pub fn run(&self, seed: u64) -> Result<GameScenarioReport, String> {
        Ok(self
            .scenario
            .run_borrowed_with_program(seed, &self.branches, &self.energy_waits)?
            .into_owned())
    }
}

pub fn run_game_program_report(
    program: &crate::simulation::GameScenarioProgramRequest,
    limits: &ExecutionLimits,
) -> Result<GameScenarioReport, String> {
    PreparedGameProgram::new(program, limits)?.run(program.request.seed)
}

fn run_prepared(
    request: &GameScenarioRequest,
    seed: u64,
    limits: &ExecutionLimits,
    prepared_packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
    branches: &[crate::simulation::GameLiveBranch],
    energy_waits: &[crate::simulation::GameLiveEnergyWait],
) -> Result<GameScenarioReport, String> {
    let (mut execution, lifecycle_assumptions) =
        PreparedExecution::new(request, seed, branches, energy_waits)?;
    if let Some(halt) = &request.source_halt {
        if seed != request.seed {
            return Err("Source halt replay requires its executed seed.".into());
        }
        if !branches.is_empty() || !energy_waits.is_empty() {
            return Err("Source halt cannot be combined with a live program wrapper.".into());
        }
        if let crate::simulation::GameSourceHaltKind::EventLimit { limit } = halt.kind {
            let replay_limits = ExecutionLimits {
                max_events: limit.min(limits.max_events),
                ..*limits
            };
            execution.process_events(request, &replay_limits, prepared_packets, &[], &[])?;
            if execution.limit_reached
                != Some(crate::simulation::GameExecutionLimit::EventCount { limit })
                || execution.state.current_time_us != halt.time_us
                || execution.halt_diagnostic.as_deref() != Some(halt.diagnostic.message.as_str())
            {
                return Err("Source event-budget halt does not match native execution.".into());
            }
        } else {
            execution.advance_to(request, limits, prepared_packets, halt.time_us)?;
            if !execution.success || execution.state.current_time_us != halt.time_us {
                return Err("Source halt does not match the settled native prefix.".into());
            }
            execution.apply_source_halt(halt);
        }
    } else {
        execution.process_events(request, limits, prepared_packets, branches, energy_waits)?;
    }
    execution.into_report(request, seed, lifecycle_assumptions)
}

#[cfg(test)]
#[path = "../tests/devils_blood.rs"]
mod devils_blood_tests;

#[cfg(test)]
#[path = "../tests/diabolos.rs"]
mod diabolos_tests;

#[cfg(test)]
#[path = "../tests/stain_admission.rs"]
mod stain_admission_tests;

#[cfg(test)]
#[path = "../tests/chaos_remora.rs"]
mod chaos_remora_tests;
#[cfg(test)]
#[path = "../tests/remora.rs"]
mod remora_tests;

#[cfg(test)]
#[path = "../tests/chiz_passives.rs"]
mod chiz_passives_tests;

#[cfg(test)]
#[path = "../tests/zankou_passives.rs"]
mod zankou_passive_tests;

#[cfg(test)]
#[path = "../tests/street_boxer.rs"]
mod street_boxer_tests;

#[cfg(test)]
mod assumption_merge_tests {
    use super::merge_model_assumptions;

    #[test]
    fn callback_healing_hypothesis_reaches_resolved_report_assumptions() {
        let mut resolved = std::collections::BTreeMap::new();
        merge_model_assumptions(
            &mut resolved,
            [(
                crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.into(),
                crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID
                    .description()
                    .into(),
            )],
        )
        .unwrap();
        assert_eq!(
            resolved[crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.as_str()],
            crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description()
        );
        assert!(
            merge_model_assumptions(
                &mut resolved,
                [(
                    crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.into(),
                    "conflicting".into()
                )]
            )
            .is_err()
        );
    }
}
