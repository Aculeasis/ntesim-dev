use crate::report_context::constant_report;
use ntesim_contracts::sha256;
use serde_json::{Value, json};

pub(crate) fn combat_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_core::combat::Request =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::team::TABLES, |_tables, _coverage| {
        ntesim_core::combat::evaluate_request(&request)
    })
}

pub(crate) fn lifecycle_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_core::lifecycle::LifecycleScenarioRequest =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::team::TABLES, |_tables, _coverage| {
        ntesim_core::lifecycle::evaluate_lifecycle_scenario(&request)
    })
}

pub(crate) fn dispatch_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_core::dispatch::DispatchScenarioRequest =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::team::TABLES, |_tables, _coverage| {
        ntesim_core::dispatch::evaluate_dispatch_scenario(&request)
    })
}

pub(crate) fn timing_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_research::timing_verification::TimingScenarioRequest =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::team::TABLES, |_tables, _coverage| {
        ntesim_research::timing_verification::evaluate_timing_scenario(&request)
    })
}

pub(crate) fn reaction_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_core::reaction::ReactionScenarioRequest =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::team::TABLES, |_tables, _coverage| {
        ntesim_core::reaction::evaluate_reaction_scenario(&request)
    })
}

pub(crate) fn execution_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_core::execution::ExecutionScenarioRequest =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::team::TABLES, |_tables, _coverage| {
        ntesim_core::execution::evaluate_execution_scenario(&request)
    })
}

pub(crate) fn game_runner_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_content::simulation::GameScenarioRequest =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::team::TABLES, |_tables, _coverage| {
        let limits = ntesim_core::execution::ExecutionLimits::default();
        ntesim_content::simulation::run_game_scenario(&request, &limits)
    })
}

pub(crate) fn game_program_report(input: &[u8]) -> Result<Value, String> {
    let program: ntesim_content::simulation::GameScenarioProgramRequest =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::team::TABLES, |_tables, _coverage| {
        let limits = ntesim_core::execution::ExecutionLimits::default();
        let report = ntesim_content::simulation::run_game_program_report(&program, &limits)?;
        serde_json::to_value(report).map_err(|e| e.to_string())
    })
}

fn parse_game_build_request(input: &[u8]) -> Result<ntesim_content::build::Request, String> {
    if input.len() > 4 * 1024 * 1024 {
        return Err("built scenario input exceeds 4 MiB".into());
    }
    serde_json::from_slice(input).map_err(|e| e.to_string())
}

fn game_build_tables(request: &ntesim_content::build::Request) -> Vec<String> {
    let mut names: Vec<String> = ntesim_data::equipment::TABLES
        .iter()
        .map(|name| name.to_string())
        .collect();
    if request.action_projection.is_some() {
        for name in ntesim_data::damage_values::TABLES {
            if !names.iter().any(|existing| existing == name) {
                names.push(name.to_owned());
            }
        }
    }
    if request.gear_projection.is_some() {
        for name in ntesim_data::weapon_passive_values::TABLES {
            if !names.iter().any(|existing| existing == name) {
                names.push(name.to_owned());
            }
        }
    }
    names
}

pub(crate) fn game_build_report(input: &[u8]) -> Result<Value, String> {
    let request = parse_game_build_request(input)?;
    let names = game_build_tables(&request);
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    constant_report(input, &names, |tables, coverage| {
        ntesim_content::build::run(
            tables,
            coverage,
            &request,
            &ntesim_core::execution::ExecutionLimits::default(),
        )
    })
}

pub(crate) fn game_built_batch_report(input: &[u8], batch: &[u8]) -> Result<Value, String> {
    let request = parse_game_build_request(input)?;
    if batch.len() > 1024 * 1024 {
        return Err("built batch request exceeds 1 MiB".into());
    }
    let names = game_build_tables(&request);
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    constant_report(input, &names, |tables, coverage| {
        let compiled = ntesim_content::build::compile(
            tables,
            coverage,
            &request,
            &ntesim_core::execution::ExecutionLimits::default(),
        )?;
        let scenario = serde_json::to_vec(&compiled.receipt["compiled_scenario"])
            .map_err(|e| e.to_string())?;
        let batch_report = ntesim_research::game_batch::run(&scenario, batch)?;
        let mut identity = b"ntesim-built-game-batch-v1".to_vec();
        for bytes in [input, batch] {
            identity.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
            identity.extend_from_slice(bytes);
        }
        Ok(json!({
            "kind":"built-game-batch-research-v1", "schema_version":1,
            "content_id":request.scenario.content_id, "status":"INCOMPLETE", "quality":"incomplete",
            "game_dps_computed":false, "acceptance_ready":false,
            "complete":batch_report.complete, "input_sha256":sha256(&identity),
            "build_request_sha256":sha256(input), "batch_request_sha256":sha256(batch),
            "compiled_scenario_sha256":sha256(&scenario),
            "build_binding":compiled.receipt, "batch":batch_report,
        }))
    })
}

pub(crate) fn game_batch_report(scenario: &[u8], request: &[u8]) -> Result<Value, String> {
    constant_report(
        scenario,
        &ntesim_data::team::TABLES,
        |_tables, _coverage| {
            serde_json::to_value(ntesim_research::game_batch::run(scenario, request)?)
                .map_err(|e| e.to_string())
        },
    )
}
