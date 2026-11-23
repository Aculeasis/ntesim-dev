//! Budgeted seed/timing experiments over explicit incomplete game hypotheses.
use crate::execution::ExecutionLimits;
use crate::formula::Ratio;
use crate::simulation::{
    ExactFraction, GameActionCommand, GameScenarioRequest, validate_game_request,
};
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: &str = "research-game-seed-batch-v1";
const MAX_SEEDS: usize = 1000;
const MAX_TIMINGS: usize = 3;
const MAX_TOTAL_EVENTS: u64 = 1_000_000;
const MAX_RECEIPT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimingPolicy {
    ScaleRecoveryAndHitDelayV1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub seeds: Vec<u64>,
    pub timing_policy: TimingPolicy,
    pub timing_hypothesis: String,
    /// Exact multipliers >= 1. Command timestamps and cooldowns remain fixed.
    pub timing_multipliers: Vec<Ratio>,
    pub max_events_per_run: u64,
    pub max_total_events: u64,
    /// Budget for the sum of compact retained run-receipt JSON bytes.
    pub max_run_receipt_bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct Run {
    pub quality: Quality,
    pub seed: u64,
    pub timing_index: usize,
    pub input_sha256: String,
    pub report_sha256: Option<String>,
    pub success: bool,
    pub complete: bool,
    pub events_processed: Option<usize>,
    pub pending_events: Option<usize>,
    pub total_damage_exact: Option<ExactFraction>,
    pub halt_diagnostic: Option<String>,
    /// Preserve partial traces for failed/pending trials; successful trials replay by hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_report: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Statistics {
    pub count: usize,
    pub minimum: ExactFraction,
    pub maximum: ExactFraction,
    pub sum: ExactFraction,
    pub mean: ExactFraction,
    pub approximate_sample_variance: Option<f64>,
    pub approximate_standard_error: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimingStatistics {
    pub quality: Quality,
    pub timing_index: usize,
    pub multiplier: Ratio,
    pub statistics: Statistics,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub batch_version: String,
    pub content_id: String,
    pub status: String,
    pub quality: Quality,
    pub game_dps_computed: bool,
    pub acceptance_ready: bool,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub model_assumptions: BTreeMap<String, String>,
    pub complete: bool,
    pub requested_runs: usize,
    pub eligible_runs: usize,
    pub estimated_max_events: u64,
    pub events_processed: usize,
    /// False when an engine error returned no event receipt; work remains bounded.
    pub event_counts_complete: bool,
    pub retained_run_receipt_bytes: usize,
    pub input_sha256: String,
    pub scenario_sha256: String,
    pub batch_request_sha256: String,
    pub request: Request,
    pub runs: Vec<Run>,
    /// All-or-none across the entire requested experiment, never success-only means.
    pub timing_statistics: Option<Vec<TimingStatistics>>,
    pub halt_diagnostic: Option<String>,
    pub uncertainty_policy: String,
    pub engine_version: String,
    pub engine_source_sha256: String,
}

fn scale_time(value: u64, multiplier: Ratio) -> Result<u64, String> {
    let product = u128::from(value) * u128::from(multiplier.numerator);
    // Ceil explicitly; do not turn a positive duration into zero.
    let rounded = product.div_ceil(u128::from(multiplier.denominator));
    u64::try_from(rounded).map_err(|_| "scaled duration exceeds u64".into())
}

pub fn transform_timing(
    scenario: &GameScenarioRequest,
    multiplier: Ratio,
) -> Result<GameScenarioRequest, String> {
    if multiplier.denominator == 0 || multiplier.numerator < multiplier.denominator {
        return Err("timing multiplier must have nonzero denominator and be >= 1".into());
    }
    let mut request = scenario.clone();
    for action in &mut request.actions {
        if let GameActionCommand::InvokeAbility {
            recovery_us,
            hit_delay_us,
            ..
        } = action
        {
            *recovery_us = scale_time(*recovery_us, multiplier)?;
            *hit_delay_us = scale_time(*hit_delay_us, multiplier)?;
        }
    }
    Ok(request)
}

fn compare(a: ExactFraction, b: ExactFraction) -> Result<std::cmp::Ordering, String> {
    let left = a
        .numerator
        .checked_mul(b.denominator)
        .ok_or("statistic comparison overflow")?;
    let right = b
        .numerator
        .checked_mul(a.denominator)
        .ok_or("statistic comparison overflow")?;
    Ok(left.cmp(&right))
}

fn statistics(values: &[ExactFraction]) -> Result<Statistics, String> {
    let anchor = *values.first().ok_or("empty batch sample")?;
    let mut sum = ExactFraction::new(0, 1)?;
    let mut minimum = anchor;
    let mut maximum = anchor;
    let mut centered_mean = 0.0;
    let mut m2 = 0.0;
    for (i, value) in values.iter().copied().enumerate() {
        sum = sum.add(value)?;
        if compare(value, minimum)?.is_lt() {
            minimum = value;
        }
        if compare(value, maximum)?.is_gt() {
            maximum = value;
        }
        let left = value
            .numerator
            .checked_mul(anchor.denominator)
            .ok_or("statistic centering overflow")?;
        let right = anchor
            .numerator
            .checked_mul(value.denominator)
            .ok_or("statistic centering overflow")?;
        let denominator = value
            .denominator
            .checked_mul(anchor.denominator)
            .ok_or("statistic centering overflow")?;
        let delta_exact = if left >= right {
            (left - right) as f64
        } else {
            -((right - left) as f64)
        } / denominator as f64;
        let delta = delta_exact - centered_mean;
        centered_mean += delta / (i + 1) as f64;
        m2 += delta * (delta_exact - centered_mean);
    }
    let count = values.len();
    let variance = (count > 1).then(|| m2.max(0.0) / (count - 1) as f64);
    Ok(Statistics {
        count,
        minimum,
        maximum,
        sum,
        mean: sum.div(ExactFraction::new(count as u128, 1)?)?,
        approximate_sample_variance: variance,
        approximate_standard_error: variance.map(|v| (v / count as f64).sqrt()),
    })
}

/// In-memory only; no filesystem, workers, ambient clocks or implicit seed domains.
pub fn run(scenario_bytes: &[u8], request_bytes: &[u8]) -> Result<Report, String> {
    run_with_limits(scenario_bytes, request_bytes, &ExecutionLimits::default())
}

pub(crate) fn run_with_limits(
    scenario_bytes: &[u8],
    request_bytes: &[u8],
    caller_limits: &ExecutionLimits,
) -> Result<Report, String> {
    if scenario_bytes.len() > 4 * 1024 * 1024 || request_bytes.len() > 1024 * 1024 {
        return Err("research batch input size limit exceeded".into());
    }
    let scenario: GameScenarioRequest =
        serde_json::from_slice(scenario_bytes).map_err(|e| e.to_string())?;
    let request: Request = serde_json::from_slice(request_bytes).map_err(|e| e.to_string())?;
    if scenario.stochastic_crit.is_none() {
        return Err("game batch requires an explicit stochastic_crit hypothesis".into());
    }
    if request.schema_version != 1 || request.seeds.is_empty() || request.seeds.len() > MAX_SEEDS {
        return Err("research batch requires schema 1 and 1..=1000 explicit seeds".into());
    }
    if request.seeds.iter().collect::<BTreeSet<_>>().len() != request.seeds.len() {
        return Err("research batch seeds must be distinct".into());
    }
    if request.timing_multipliers.is_empty()
        || request.timing_multipliers.len() > MAX_TIMINGS
        || request.timing_hypothesis.trim().is_empty()
    {
        return Err(
            "research batch requires 1..=3 explicit timing multipliers and a hypothesis".into(),
        );
    }
    let limits = ExecutionLimits {
        max_events: request.max_events_per_run.min(caller_limits.max_events),
        ..*caller_limits
    };
    if request.max_events_per_run == 0
        || request.max_events_per_run > ExecutionLimits::default().max_events
        || request.max_total_events == 0
        || request.max_total_events > MAX_TOTAL_EVENTS
        || request.max_run_receipt_bytes == 0
        || request.max_run_receipt_bytes > MAX_RECEIPT_BYTES
    {
        return Err("research batch event/receipt budgets are outside supported bounds".into());
    }
    let requested_runs = request.seeds.len() * request.timing_multipliers.len();
    let estimated_max_events = (requested_runs as u64)
        .checked_mul(request.max_events_per_run)
        .ok_or("batch work overflow")?;
    if estimated_max_events > request.max_total_events {
        return Err("research batch worst-case work exceeds max_total_events".into());
    }
    let mut prepared = Vec::new();
    let mut timing_keys = BTreeSet::new();
    for multiplier in &request.timing_multipliers {
        let reduced = ExactFraction::from_ratio(*multiplier)?;
        if !timing_keys.insert((reduced.numerator, reduced.denominator)) {
            return Err("duplicate equivalent timing multiplier".into());
        }
        let transformed = transform_timing(&scenario, *multiplier)?;
        validate_game_request(&transformed, &limits)?;
        prepared.push(transformed);
    }
    let mut identity = VERSION.as_bytes().to_vec();
    for bytes in [scenario_bytes, request_bytes] {
        identity.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        identity.extend_from_slice(bytes);
    }
    let limit_bytes = serde_json::to_vec(&limits).map_err(|error| error.to_string())?;
    identity.extend_from_slice(&(limit_bytes.len() as u64).to_be_bytes());
    identity.extend_from_slice(&limit_bytes);
    let mut runs = Vec::with_capacity(requested_runs);
    let mut samples = vec![Vec::new(); prepared.len()];
    let mut eligible_runs = 0;
    let mut retained_bytes = 0usize;
    let mut events_processed = 0;
    let mut event_counts_complete = true;
    let mut halt_diagnostic = None;
    'timings: for (timing_index, transformed) in prepared.iter().enumerate() {
        // Keep preparation errors as per-run failures, as in independent execution.
        let rotation = ntesim_content::simulation::PreparedGameScenario::new(transformed, &limits);
        for seed in &request.seeds {
            let mut seeded = transformed.clone();
            seeded.seed = *seed;
            let input_sha256 = sha256(&serde_json::to_vec(&seeded).map_err(|e| e.to_string())?);
            let result = match &rotation {
                Ok(rotation) => rotation.run(*seed),
                Err(error) => Err(error.clone()),
            };
            let row = match result {
                Ok(report) => {
                    let value = serde_json::to_value(&report).map_err(|e| e.to_string())?;
                    events_processed += report.total_events_processed;
                    let complete = super::eligibility::validate_completed(&report).is_ok();
                    Run {
                        quality: Quality::Incomplete,
                        seed: *seed,
                        timing_index,
                        input_sha256,
                        report_sha256: Some(sha256(
                            &serde_json::to_vec(&value).map_err(|e| e.to_string())?,
                        )),
                        success: report.success,
                        complete,
                        events_processed: Some(report.total_events_processed),
                        pending_events: Some(report.pending_events),
                        total_damage_exact: Some(report.total_damage_exact),
                        halt_diagnostic: report.halt_diagnostic,
                        partial_report: (!complete).then_some(value),
                    }
                }
                Err(error) => {
                    event_counts_complete = false;
                    Run {
                        quality: Quality::Incomplete,
                        seed: *seed,
                        timing_index,
                        input_sha256,
                        report_sha256: None,
                        success: false,
                        complete: false,
                        events_processed: None,
                        pending_events: None,
                        total_damage_exact: None,
                        halt_diagnostic: Some(error),
                        partial_report: None,
                    }
                }
            };
            let bytes = serde_json::to_vec(&row).map_err(|e| e.to_string())?.len();
            if retained_bytes + bytes > request.max_run_receipt_bytes {
                halt_diagnostic = Some(format!(
                    "run receipt budget exhausted at timing {timing_index}, seed {seed}"
                ));
                break 'timings;
            }
            retained_bytes += bytes;
            if row.complete {
                eligible_runs += 1;
                samples[timing_index]
                    .push(row.total_damage_exact.ok_or("complete run has no damage")?);
            }
            runs.push(row);
        }
    }
    let complete = runs.len() == requested_runs && eligible_runs == requested_runs;
    let timing_statistics = if complete {
        Some(
            samples
                .iter()
                .enumerate()
                .map(|(i, values)| {
                    Ok(TimingStatistics {
                        quality: Quality::Incomplete,
                        timing_index: i,
                        multiplier: request.timing_multipliers[i],
                        statistics: statistics(values)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        )
    } else {
        None
    };
    Ok(Report {
        schema_version: 1, batch_version: VERSION.into(), content_id: scenario.content_id,
        status: "INCOMPLETE".into(), quality: Quality::Incomplete,
        game_dps_computed: false, acceptance_ready: false,
        model_assumptions: scenario.model_assumptions,
        complete, requested_runs, eligible_runs, estimated_max_events, events_processed, event_counts_complete,
        retained_run_receipt_bytes: retained_bytes, input_sha256: sha256(&identity),
        scenario_sha256: sha256(scenario_bytes), batch_request_sha256: sha256(request_bytes),
        request, runs, timing_statistics, halt_diagnostic,
        uncertainty_policy: "Sample variance/standard error describe only declared RNG hypotheses; unbounded game-model/timing uncertainty remains separate. No validated DPS, confidence interval or sustainability claim.".into(),
        engine_version: ENGINE_VERSION.into(), engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
    })
}

#[cfg(test)]
mod tests;
