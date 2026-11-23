use std::cell::RefCell;

use ntesim_content::simulation::{
    self, GameScenarioProgramRequest, GameScenarioReport, GameScenarioRequest,
};
use ntesim_core::execution::ExecutionLimits;
use ntesim_research::{
    composition::{self, CompositionReport},
    game_batch,
    rotation::{self, Report as RotationReport},
    tabular_verification::{self, TabularVerificationReport},
};

use super::model::*;
use super::source_api;

thread_local! {
    // One rotation per thread/instance. Compare complete canonical inputs (except
    // seed), rather than an incomplete build key or a collision-prone hash.
    pub(crate) static PREPARED_GAME: RefCell<Option<(Vec<u8>, simulation::PreparedGameScenario)>> =
        const { RefCell::new(None) };
    pub(crate) static PREPARED_GAME_PROGRAM: RefCell<Option<(Vec<u8>, simulation::PreparedGameProgram)>> =
        const { RefCell::new(None) };
}

pub fn dispatch_simulate_rotation(input: &[u8]) -> Result<RotationReport, String> {
    let req: WasmRotationRequest =
        serde_json::from_slice(input).map_err(|e| format!("invalid rotation request: {e}"))?;
    let fixture_bytes = serde_json::to_vec(&req.fixture)
        .map_err(|e| format!("failed to serialize rotation fixture: {e}"))?;
    if let Some(seed) = req.seed {
        rotation::run_seeded(&fixture_bytes, &req.rotation, seed)
    } else {
        rotation::run(&fixture_bytes, &req.rotation)
    }
}

pub fn dispatch_simulate_game(input: &[u8]) -> Result<GameScenarioReport, String> {
    with_prepared_game(input, |prepared, seed| prepared.run(seed))
}

pub fn dispatch_simulate_game_program(input: &[u8]) -> Result<GameScenarioReport, String> {
    let mut program: GameScenarioProgramRequest =
        serde_json::from_slice(input).map_err(|e| format!("invalid game program request: {e}"))?;
    let seed = program.request.seed;
    program.request.seed = 0;
    let key = serde_json::to_vec(&program).map_err(|e| e.to_string())?;
    let limits = ExecutionLimits::default();
    PREPARED_GAME_PROGRAM.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.as_ref().is_none_or(|(previous, _)| previous != &key) {
            let prepared = simulation::PreparedGameProgram::new(&program, &limits)?;
            *cache = Some((key, prepared));
        }
        cache
            .as_ref()
            .ok_or("internal preparation cache missing live program")?
            .1
            .run(seed)
    })
}

pub fn dispatch_simulate_game_batch(input: &[u8]) -> Result<Vec<GameScenarioReport>, String> {
    let mut req: WasmGameBatchRequest =
        serde_json::from_slice(input).map_err(|e| format!("invalid game batch request: {e}"))?;
    validate_seed_range(req.start_seed, req.count)?;
    req.scenario.seed = 0;
    let key = serde_json::to_vec(&req.scenario).map_err(|e| e.to_string())?;
    let limits = ExecutionLimits::default();
    PREPARED_GAME.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.as_ref().is_none_or(|(previous, _)| previous != &key) {
            let prepared = simulation::PreparedGameScenario::new(&req.scenario, &limits)?;
            *cache = Some((key, prepared));
        }
        let prepared = &cache
            .as_ref()
            .ok_or("internal preparation cache missing scenario")?
            .1;
        let mut reports = Vec::with_capacity(req.count as usize);
        for i in 0..req.count {
            reports.push(prepared.run(req.start_seed + u64::from(i))?);
        }
        Ok(reports)
    })
}

pub fn dispatch_simulate_game_program_batch(
    input: &[u8],
) -> Result<Vec<GameScenarioReport>, String> {
    let mut req: WasmGameProgramBatchRequest = serde_json::from_slice(input)
        .map_err(|e| format!("invalid game program batch request: {e}"))?;
    validate_seed_range(req.start_seed, req.count)?;
    req.program.request.seed = 0;
    let key = serde_json::to_vec(&req.program).map_err(|e| e.to_string())?;
    let limits = ExecutionLimits::default();
    PREPARED_GAME_PROGRAM.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.as_ref().is_none_or(|(previous, _)| previous != &key) {
            let prepared = simulation::PreparedGameProgram::new(&req.program, &limits)?;
            *cache = Some((key, prepared));
        }
        let prepared = &cache
            .as_ref()
            .ok_or("internal preparation cache missing live program")?
            .1;
        let mut reports = Vec::with_capacity(req.count as usize);
        for i in 0..req.count {
            reports.push(prepared.run(req.start_seed + u64::from(i))?);
        }
        Ok(reports)
    })
}

pub(crate) fn validate_seed_range(start: u64, count: u32) -> Result<(), String> {
    start
        .checked_add(u64::from(count.saturating_sub(1)))
        .ok_or_else(|| "batch seed range overflows u64".to_string())?;
    Ok(())
}

pub(crate) fn with_prepared_game<T>(
    input: &[u8],
    execute: impl FnOnce(&simulation::PreparedGameScenario, u64) -> Result<T, String>,
) -> Result<T, String> {
    let mut request: GameScenarioRequest =
        serde_json::from_slice(input).map_err(|e| format!("invalid game scenario request: {e}"))?;
    let seed = request.seed;
    request.seed = 0;
    let key = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
    let limits = ExecutionLimits::default();
    PREPARED_GAME.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.as_ref().is_none_or(|(previous, _)| previous != &key) {
            let prepared = simulation::PreparedGameScenario::new(&request, &limits)?;
            *cache = Some((key, prepared));
        }
        execute(
            &cache
                .as_ref()
                .ok_or("internal preparation cache missing scenario")?
                .1,
            seed,
        )
    })
}

pub fn dispatch_run_batch(input: &[u8]) -> Result<game_batch::Report, String> {
    let req: WasmBatchRequest =
        serde_json::from_slice(input).map_err(|e| format!("invalid batch request: {e}"))?;
    let scenario_bytes = serde_json::to_vec(&req.scenario)
        .map_err(|e| format!("failed to serialize batch scenario: {e}"))?;
    let request_bytes = serde_json::to_vec(&req.request)
        .map_err(|e| format!("failed to serialize batch request: {e}"))?;
    game_batch::run(&scenario_bytes, &request_bytes)
}

pub fn dispatch_verify_tabular(input: &[u8]) -> Result<TabularVerificationReport, String> {
    let req: WasmTabularRequest = if input.is_empty() {
        WasmTabularRequest { cases: None }
    } else {
        serde_json::from_slice(input)
            .map_err(|e| format!("invalid tabular verification request: {e}"))?
    };
    tabular_verification::run_tabular_verification(req.cases.as_deref())
}

pub fn dispatch_enumerate_compositions(input: &[u8]) -> Result<CompositionReport, String> {
    let req: WasmCompositionRequest = if input.is_empty() {
        WasmCompositionRequest { variants_csv: None }
    } else {
        serde_json::from_slice(input).map_err(|e| format!("invalid composition request: {e}"))?
    };
    let coverage_val = ntesim_data::character::released_inventory();
    let variants_csv = req.variants_csv.as_deref().unwrap_or(DEFAULT_VARIANTS_CSV);
    composition::enumerate_compositions(&coverage_val, variants_csv)
}

pub fn dispatch_invoke(input: &[u8]) -> Result<WasmInvokeSuccessResponse, String> {
    let envelope: WasmInvokeEnvelope =
        serde_json::from_slice(input).map_err(|e| format!("invalid invoke envelope: {e}"))?;
    let payload_bytes = serde_json::to_vec(&envelope.payload)
        .map_err(|e| format!("failed to serialize payload: {e}"))?;

    let data_val = match envelope.action.as_str() {
        "export_source_report"
        | "replay_source_report"
        | "compare_source_reports"
        | "search_source_reports" => {
            super::source_reports::dispatch(&envelope.action, &payload_bytes)?
        }
        "start_source_trials"
        | "resume_source_trials"
        | "step_source_trials"
        | "aggregate_trials"
        | "run_source_sample"
        | "run_source_chunk"
        | "add_source_observations"
        | "snapshot_source_trials" => {
            super::source_trials::dispatch(&envelope.action, &payload_bytes)?
        }
        "inspect_script"
        | "compile_script"
        | "validate_script"
        | "run_script"
        | "compile_source"
        | "validate_source"
        | "run_compiled_source"
        | "edit_script_options"
        | "parse_option_input"
        | "model_assumption" => source_api::dispatch(&envelope.action, &payload_bytes)?,
        "actor_projection" => {
            let request: ntesim_content::build::studio_projection::Request =
                serde_json::from_slice(&payload_bytes)
                    .map_err(|e| format!("invalid actor projection request: {e}"))?;
            let tables = ntesim_data::stat_projection::frozen_tables();
            let result = ntesim_content::build::studio_projection::project(tables, &request)?;
            serde_json::to_value(result)
                .map_err(|e| format!("failed to serialize actor projection: {e}"))?
        }
        "version" => serde_json::to_value(version_info())
            .map_err(|e| format!("failed to serialize version: {e}"))?,
        "build_started_at" => serde_json::to_value(WASM_BUILD_STARTED_AT)
            .map_err(|e| format!("failed to serialize build_started_at: {e}"))?,
        "simulate_rotation" => {
            let res = dispatch_simulate_rotation(&payload_bytes)?;
            serde_json::to_value(res).map_err(|e| format!("failed to serialize rotation: {e}"))?
        }
        "simulate_game" => {
            let res = dispatch_simulate_game(&payload_bytes)?;
            serde_json::to_value(res).map_err(|e| format!("failed to serialize game: {e}"))?
        }
        "simulate_game_batch" => {
            let res = dispatch_simulate_game_batch(&payload_bytes)?;
            serde_json::to_value(res).map_err(|e| format!("failed to serialize game batch: {e}"))?
        }
        "simulate_game_program" => {
            let res = dispatch_simulate_game_program(&payload_bytes)?;
            serde_json::to_value(res)
                .map_err(|e| format!("failed to serialize game program: {e}"))?
        }
        "simulate_game_program_batch" => {
            let res = dispatch_simulate_game_program_batch(&payload_bytes)?;
            serde_json::to_value(res)
                .map_err(|e| format!("failed to serialize game program batch: {e}"))?
        }
        "run_batch" => {
            let res = dispatch_run_batch(&payload_bytes)?;
            serde_json::to_value(res).map_err(|e| format!("failed to serialize batch: {e}"))?
        }
        "verify_tabular" => {
            let res = dispatch_verify_tabular(&payload_bytes)?;
            serde_json::to_value(res).map_err(|e| format!("failed to serialize tabular: {e}"))?
        }
        "enumerate_compositions" => {
            let res = dispatch_enumerate_compositions(&payload_bytes)?;
            serde_json::to_value(res)
                .map_err(|e| format!("failed to serialize compositions: {e}"))?
        }
        other => return Err(format!("unsupported action: {other}")),
    };

    Ok(WasmInvokeSuccessResponse {
        status: "ok".to_string(),
        action: envelope.action,
        environment: default_environment_metadata(),
        data: data_val,
    })
}
