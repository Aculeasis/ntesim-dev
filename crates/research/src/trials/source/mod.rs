//! Bounded experiments over authored native programs, never synthetic fixtures.
use ntesim_content::{
    dsl::compiler::CompileLimits,
    simulation::{ExactFraction, PreparedSourceScenario, SourceScenarioReport},
};
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use ntesim_core::execution::ExecutionLimits;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

mod compare;
pub use compare::{Comparison, SignedFraction, compare};
pub mod search;

const VERSION: &str = "native-source-experiment-v1";
const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub seeds: Vec<u64>,
    pub max_events_per_run: u64,
    pub max_total_events: u64,
    pub max_receipt_bytes: usize,
}

impl Request {
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.seeds.is_empty()
            || self.seeds.len() > 1000
            || self.seeds.iter().collect::<BTreeSet<_>>().len() != self.seeds.len()
        {
            return Err("source experiment requires schema 1 and 1..=1000 distinct seeds".into());
        }
        if self.max_events_per_run == 0
            || self.max_events_per_run > ExecutionLimits::default().max_events
            || self.max_total_events == 0
            || self.max_total_events > 1_000_000
            || self.max_receipt_bytes == 0
            || self.max_receipt_bytes > 64 * 1024 * 1024
        {
            return Err("source experiment budgets exceed supported nonzero bounds".into());
        }
        Ok(())
    }

    pub(super) fn prepare(&self, source: &str) -> Result<PreparedSourceScenario, String> {
        self.validate()?;
        if source.len() > MAX_SOURCE_BYTES {
            return Err("source exceeds 4 MiB".into());
        }
        PreparedSourceScenario::new(
            source,
            CompileLimits::default(),
            &ExecutionLimits {
                max_events: self.max_events_per_run,
                ..Default::default()
            },
        )
        .map_err(|diagnostics| {
            serde_json::to_string(&diagnostics).unwrap_or_else(|e| e.to_string())
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Run {
    pub seed: u64,
    pub eligible: bool,
    pub execution: Option<SourceScenarioReport>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Batch {
    pub schema_version: u32,
    pub experiment_version: String,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub input_sha256: String,
    pub source_sha256: String,
    pub request: Request,
    pub compiled: ntesim_content::dsl::source::CompiledSourceProgram,
    pub complete: bool,
    pub eligible_runs: usize,
    pub events_charged: u64,
    pub retained_receipt_bytes: usize,
    pub halt_diagnostic: Option<String>,
    pub runs: Vec<Run>,
    /// All requested seeds must qualify; failures never form a success-only mean.
    pub mean_damage: Option<ExactFraction>,
    pub mean_dps: Option<ExactFraction>,
    pub uncertainty_policy: String,
}

pub(super) fn identity(parts: &[&[u8]]) -> String {
    let mut bytes = VERSION.as_bytes().to_vec();
    bytes.extend_from_slice(
        ntesim_contracts::ClockContract::RelativeCombatV1
            .as_str()
            .as_bytes(),
    );
    bytes.extend_from_slice(
        ntesim_contracts::TimerPolicy::PausedWorldV1
            .as_str()
            .as_bytes(),
    );
    for part in parts {
        bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
        bytes.extend_from_slice(part);
    }
    sha256(&bytes)
}

pub(super) fn encode(value: &impl Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|e| e.to_string())
}

pub fn eligible(execution: &SourceScenarioReport) -> bool {
    completed(execution) && execution.report.game_dps_computed
}

/// Raw bounded combat can complete with an undefined relative damage rate.
/// Studio retains it; comparison and search still require `eligible` rates.
pub fn completed(execution: &SourceScenarioReport) -> bool {
    let report = &execution.report;
    report.quality == Quality::BoundedHypothesis
        && super::eligibility::validate_completed(report).is_ok()
        && report.validate_clocks().is_ok()
        && execution.diagnostic.is_none()
}

pub fn dps(execution: &SourceScenarioReport) -> Result<ExactFraction, String> {
    if execution.report.relative_elapsed_us == 0 {
        return Err("Relative DPS is unavailable: zero relative combat duration.".into());
    }
    if !eligible(execution) {
        return Err("Relative DPS requires a complete eligible source execution.".into());
    }
    execution.report.total_damage_exact.mul(ExactFraction::new(
        1_000_000,
        u128::from(execution.report.relative_elapsed_us),
    )?)
}

fn run_trial(program: &PreparedSourceScenario, seed: u64, request: &Request) -> (Run, u64) {
    match program.run(seed) {
        Ok(execution) => {
            let events = execution.report.total_events_processed as u64;
            (
                Run {
                    seed,
                    eligible: eligible(&execution),
                    execution: Some(execution),
                    error: None,
                },
                events,
            )
        }
        // No trustworthy processed count survives an internal error.
        Err(error) => (
            Run {
                seed,
                eligible: false,
                execution: None,
                error: Some(error),
            },
            request.max_events_per_run,
        ),
    }
}

pub fn batch(source: &str, request: &Request) -> Result<Batch, String> {
    let program = request.prepare(source)?;
    let mut result = Batch {
        schema_version: 1, experiment_version: VERSION.into(),
        engine_version: ENGINE_VERSION.into(), engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
        input_sha256: identity(&[source.as_bytes(), &encode(request)?]),
        source_sha256: sha256(source.as_bytes()), request: request.clone(), compiled: program.export(),
        complete: false, eligible_runs: 0, events_charged: 0, retained_receipt_bytes: 0,
        halt_diagnostic: None, runs: Vec::new(), mean_damage: None, mean_dps: None,
        uncertainty_policy: "Explicit seed sampling of the declared model; seed variation does not measure model uncertainty or establish game evidence.".into(),
    };
    for seed in &request.seeds {
        if result.events_charged + request.max_events_per_run > request.max_total_events {
            result.halt_diagnostic =
                Some("total event budget cannot admit another complete trial".into());
            break;
        }
        let (row, events) = run_trial(&program, *seed, request);
        result.events_charged += events;
        let size = encode(&row)?.len();
        if result.retained_receipt_bytes + size > request.max_receipt_bytes {
            result.halt_diagnostic = Some("retained receipt budget exhausted".into());
            break;
        }
        result.retained_receipt_bytes += size;
        result.eligible_runs += usize::from(row.eligible);
        result.runs.push(row);
    }
    result.complete =
        result.runs.len() == request.seeds.len() && result.eligible_runs == request.seeds.len();
    if result.complete {
        let mut damage = ExactFraction::new(0, 1)?;
        let mut rate = damage;
        for row in &result.runs {
            let execution = row
                .execution
                .as_ref()
                .ok_or("eligible run missing execution")?;
            damage = damage.add(execution.report.total_damage_exact)?;
            rate = rate.add(dps(execution)?)?;
        }
        let count = ExactFraction::new(result.runs.len() as u128, 1)?;
        result.mean_damage = Some(damage.div(count)?);
        result.mean_dps = Some(rate.div(count)?);
    }
    Ok(result)
}

/// Compare only common dummy/clock/incoming assumptions. Owner-specific build
/// models remain attached to each execution and may differ with legal gear.
pub(super) fn context(program: &PreparedSourceScenario) -> Result<Value, String> {
    let mut value = serde_json::to_value(program.export().request).map_err(|e| e.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or("scenario context must be an object")?;
    for key in [
        "actors",
        "active_actor_index",
        "model_assumptions",
        "seed",
        "incoming_damage_events",
        "incoming_dummy_sources",
        "incoming_damage_basis",
    ] {
        object.remove(key);
    }
    let actors = program.export().request.actors;
    let mut initial = Vec::new();
    for actor in actors {
        initial.push(serde_json::json!({
            "character_id": actor.character_id,
            "initial_energy": actor.initial_energy,
            "initial_cycle": actor.initial_cycle,
            "initial_resources": actor.initial_resources,
            "hp_ratio": ExactFraction::new(u128::from(actor.initial_hp), u128::from(actor.initial_hp_max))?,
        }));
    }
    object.insert("party_initial_conditions".into(), Value::Array(initial));
    object.insert(
        "authored_scenario_assumptions".into(),
        serde_json::to_value(program.scenario_assumptions()).map_err(|e| e.to_string())?,
    );
    Ok(value)
}

#[cfg(test)]
mod tests;
