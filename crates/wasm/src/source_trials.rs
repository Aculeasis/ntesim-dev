//! A Worker owns one prepared live source run; Rust owns trial admission/summary.
use ntesim_content::{
    dsl::{compiler::SourceRequest, source::CompiledSourceProgram},
    simulation::{
        GameScenarioReport, GameScenarioRequest, PreparedSourceScenario, SourceScenarioReport,
        SourceTrialOutcome,
    },
};
use ntesim_research::trials::studio::{
    Aggregate, Collector, Observation, Sample, observe, observe_batch,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;

struct Session {
    program: std::rc::Rc<PreparedSourceScenario>,
    collector: Collector,
    total: u64,
    failed: bool,
}

// Compact observations move directly into the collector or a chunk response.
#[allow(clippy::large_enum_variant)]
enum ObservedTrial {
    Sample(Box<SourceScenarioReport>),
    Batch(Observation),
    Failed(Box<SourceScenarioReport>),
}

fn execute_trial(program: &PreparedSourceScenario, offset: u64) -> Result<ObservedTrial, String> {
    let seed = program.seed().checked_add(offset).ok_or("seed overflow")?;
    if offset == 0 {
        let execution = Box::new(program.run_sample(seed)?);
        return Ok(if ntesim_research::trials::source::completed(&execution) {
            ObservedTrial::Sample(execution)
        } else {
            ObservedTrial::Failed(execution)
        });
    }
    match program.run_batch(seed)? {
        SourceTrialOutcome::Completed(trial) => Ok(ObservedTrial::Batch(observe_batch(trial))),
        SourceTrialOutcome::Failed(execution) => Ok(ObservedTrial::Failed(execution)),
    }
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
    static CACHED_SOURCE_SCENARIO: RefCell<Option<PreparedSourceCache>> = const { RefCell::new(None) };
}

struct PreparedSourceCache {
    identity: Vec<u8>,
    program: std::rc::Rc<PreparedSourceScenario>,
}

pub(super) fn compile_source(request: &SourceRequest) -> Result<Value, String> {
    let warm = CACHED_SOURCE_SCENARIO.with(|cell| {
        cell.borrow()
            .as_ref()
            .filter(|entry| entry.program.matches_source(request))
            .map(|entry| std::rc::Rc::clone(&entry.program))
    });
    if let Some(program) = warm {
        return serde_json::to_value(program.compile_response()).map_err(|error| error.to_string());
    }
    let (response, program) = ntesim_content::dsl::source::prepare_response(request);
    if let (Some(artifact), Some(program)) = (&response.compiled, program) {
        let identity = serde_json::to_vec(artifact).map_err(|error| error.to_string())?;
        CACHED_SOURCE_SCENARIO.with(|cell| {
            *cell.borrow_mut() = Some(PreparedSourceCache {
                identity,
                program: std::rc::Rc::new(program),
            });
        });
    }
    serde_json::to_value(response).map_err(|error| error.to_string())
}

fn get_or_prepare_source_scenario(
    artifact: &CompiledSourceProgram,
) -> Result<std::rc::Rc<PreparedSourceScenario>, String> {
    if artifact.schema_version != 1
        || artifact.engine_version != ntesim_contracts::ENGINE_VERSION
        || artifact.engine_source_sha256 != ntesim_contracts::ENGINE_SOURCE_SHA256
    {
        return Err("Compiled source engine/schema identity mismatch.".into());
    }
    // Canonical typed encoding includes source limits, request, engine and
    // trial range. An edited/tampered artifact must pass the import path.
    let identity = serde_json::to_vec(artifact).map_err(|error| error.to_string())?;
    CACHED_SOURCE_SCENARIO.with(|cell| {
        let mut cache = cell.borrow_mut();
        if let Some(entry) = cache.as_ref()
            && entry.identity == identity
        {
            return Ok(std::rc::Rc::clone(&entry.program));
        }
        let program = std::rc::Rc::new(PreparedSourceScenario::from_export(
            artifact,
            &Default::default(),
        )?);
        *cache = Some(PreparedSourceCache {
            identity,
            program: std::rc::Rc::clone(&program),
        });
        Ok(program)
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChunkRequest {
    compiled: CompiledSourceProgram,
    start_offset: u64,
    count: u32,
}

#[derive(Serialize)]
struct ChunkResponse {
    first_execution: Option<Box<SourceScenarioReport>>,
    failure: Option<Box<SourceScenarioReport>>,
    samples: Vec<Sample>,
    observations: Vec<Observation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AddObservationsRequest {
    observations: Vec<Observation>,
    elapsed_ms: f64,
    #[serde(default)]
    snapshot: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    offset: u64,
    count: u32,
    elapsed_ms: f64,
}

#[derive(Serialize)]
struct Update {
    first_execution: Option<Box<SourceScenarioReport>>,
    samples_delta: Vec<Sample>,
    aggregate: Option<Aggregate>,
    failure: Option<Box<SourceScenarioReport>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reports {
    request: GameScenarioRequest,
    total: u64,
    reports: Vec<GameScenarioReport>,
    elapsed_ms: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleRequest {
    compiled: CompiledSourceProgram,
    seed: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resume {
    compiled: CompiledSourceProgram,
    samples: Vec<Sample>,
    elapsed_ms: f64,
}

pub(super) fn dispatch(action: &str, payload: &[u8]) -> Result<Value, String> {
    match action {
        "resume_source_trials" => {
            SESSION.with(|session| *session.borrow_mut() = None);
            let input: Resume = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
            if input.samples.is_empty()
                || input.samples.len() as u64 >= input.compiled.trials
                || !input.elapsed_ms.is_finite()
                || input.elapsed_ms < 0.
            {
                return Err("resume requires an incomplete prefix and finite elapsed time".into());
            }
            dispatch(
                "start_source_trials",
                &serde_json::to_vec(&input.compiled).map_err(|e| e.to_string())?,
            )?;
            SESSION.with(|cell| {
                // Publish only after every retained sample has been independently reproduced.
                let mut session = cell
                    .borrow_mut()
                    .take()
                    .ok_or("no prepared source trials")?;
                let mut first_execution = None;
                for offset in 0..input.samples.len() {
                    match execute_trial(&session.program, offset as u64)? {
                        ObservedTrial::Sample(execution) => {
                            session.collector.add(&execution.report)?;
                            first_execution = Some(execution);
                        }
                        ObservedTrial::Batch(row) => session.collector.add_observation(row)?,
                        ObservedTrial::Failed(_) => {
                            return Err("retained trial is no longer eligible".into());
                        }
                    }
                }
                // Aggregate data always comes from reproduced reports, never the caller.
                if session.collector.samples() != input.samples {
                    return Err("retained sample mismatch".into());
                }
                let update = serde_json::to_value(Update {
                    first_execution,
                    samples_delta: session.collector.samples(),
                    aggregate: Some(session.collector.snapshot(input.elapsed_ms)?),
                    failure: None,
                })
                .map_err(|e| e.to_string())?;
                *cell.borrow_mut() = Some(session);
                Ok(update)
            })
        }
        "snapshot_source_trials" => {
            let elapsed_ms: f64 = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
            SESSION.with(|session| {
                let session = session.borrow();
                let session = session.as_ref().ok_or("no prepared source trials")?;
                serde_json::to_value(session.collector.snapshot(elapsed_ms)?)
                    .map_err(|e| e.to_string())
            })
        }
        "run_source_sample" => {
            let input: SampleRequest =
                serde_json::from_slice(payload).map_err(|e| e.to_string())?;
            if input.seed > 9_007_199_254_740_991 {
                return Err("sample seed must be a JavaScript safe integer".into());
            }
            let program = get_or_prepare_source_scenario(&input.compiled)?;
            serde_json::to_value(program.run(input.seed)?).map_err(|e| e.to_string())
        }
        "start_source_trials" => {
            // A rejected replacement cannot leave the old run available.
            SESSION.with(|session| *session.borrow_mut() = None);
            let artifact: CompiledSourceProgram =
                serde_json::from_slice(payload).map_err(|e| e.to_string())?;
            const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
            let last = artifact
                .request
                .seed
                .checked_add(artifact.trials.saturating_sub(1))
                .ok_or("seed overflow")?;
            if artifact.trials == 0 || artifact.trials > MAX_SAFE_INTEGER || last > MAX_SAFE_INTEGER
            {
                return Err("trial count and seed range must be JavaScript safe integers".into());
            }
            let program = get_or_prepare_source_scenario(&artifact)?;
            let collector = Collector::new(artifact.request.clone(), artifact.trials)?;
            SESSION.with(|session| {
                *session.borrow_mut() = Some(Session {
                    program,
                    collector,
                    total: artifact.trials,
                    failed: false,
                })
            });
            Ok(serde_json::json!({"trials": artifact.trials}))
        }
        "step_source_trials" => step_source_trials(payload),
        "aggregate_trials" => {
            let input: Reports = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
            let mut collector = Collector::new(input.request, input.total)?;
            for report in &input.reports {
                collector.add(report)?;
            }
            serde_json::to_value(serde_json::json!({
                "aggregate": collector.snapshot(input.elapsed_ms)?,
                "samples": collector.samples(),
            }))
            .map_err(|e| e.to_string())
        }
        "run_source_chunk" => run_source_chunk(payload),
        "add_source_observations" => add_source_observations(payload),
        _ => Err(format!("unsupported source trial action: {action}")),
    }
}

fn step_source_trials(payload: &[u8]) -> Result<Value, String> {
    let step: Step = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    SESSION.with(|session| {
        let mut session = session.borrow_mut();
        let session = session.as_mut().ok_or("no prepared source trials")?;
        let offset = session.collector.completed() as u64;
        if session.failed
            || step.offset != offset
            || step.count == 0
            || step.count > 25
            || offset
                .checked_add(u64::from(step.count))
                .is_none_or(|end| end > session.total)
            || !step.elapsed_ms.is_finite()
            || step.elapsed_ms < 0.
        {
            return Err("invalid source trial offset/count/state/time".into());
        }
        let mut first_execution = None;
        let mut failure = None;
        for i in 0..step.count {
            let result = execute_trial(&session.program, offset + u64::from(i)).and_then(|trial| {
                match trial {
                    ObservedTrial::Sample(execution) => {
                        session.collector.add(&execution.report)?;
                        first_execution = Some(execution);
                    }
                    ObservedTrial::Batch(row) => session.collector.add_observation(row)?,
                    ObservedTrial::Failed(execution) => failure = Some(execution),
                }
                Ok(())
            });
            if let Err(error) = result {
                session.failed = true;
                return Err(error);
            }
            if failure.is_some() {
                session.failed = true;
                break;
            }
        }
        serde_json::to_value(Update {
            first_execution,
            failure,
            samples_delta: session
                .collector
                .samples()
                .into_iter()
                .skip(offset as usize)
                .collect(),
            aggregate: if session.collector.completed() > 0 {
                Some(session.collector.snapshot(step.elapsed_ms)?)
            } else {
                None
            },
        })
        .map_err(|e| e.to_string())
    })
}

fn run_source_chunk(payload: &[u8]) -> Result<Value, String> {
    let input: ChunkRequest = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let program = get_or_prepare_source_scenario(&input.compiled)?;
    let mut first_execution = None;
    let mut failure = None;
    let mut samples = Vec::with_capacity(input.count as usize);
    let mut observations = Vec::with_capacity(input.count as usize);

    for i in 0..input.count {
        let offset = input
            .start_offset
            .checked_add(u64::from(i))
            .ok_or("trial offset overflow")?;
        let row = match execute_trial(&program, offset)? {
            ObservedTrial::Sample(execution) => {
                let row = observe(program.request(), &execution.report, true)?;
                first_execution = Some(execution);
                row
            }
            ObservedTrial::Batch(row) => row,
            ObservedTrial::Failed(execution) => {
                failure = Some(execution);
                break;
            }
        };
        samples.push(row.sample.clone());
        observations.push(row);
    }
    serde_json::to_value(ChunkResponse {
        first_execution,
        failure,
        samples,
        observations,
    })
    .map_err(|e| e.to_string())
}

fn add_source_observations(payload: &[u8]) -> Result<Value, String> {
    let input: AddObservationsRequest =
        serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    SESSION.with(|session| {
        let mut session = session.borrow_mut();
        let session = session.as_mut().ok_or("no prepared source trials")?;
        for row in input.observations {
            session.collector.add_observation(row)?;
        }
        if input.snapshot {
            serde_json::to_value(Some(session.collector.snapshot(input.elapsed_ms)?))
        } else {
            serde_json::to_value(None::<ntesim_research::trials::studio::Aggregate>)
        }
        .map_err(|e| e.to_string())
    })
}
