//! Synthetic seed ensembles; failure/pending events never enter aggregate samples.
use crate::{rotation, synthetic};
use ntesim_contracts::{Diagnostic, ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: &str = "synthetic-seed-batch-v1";
pub const MAX_SEEDS: usize = 1000;
pub const MAX_EVENTS: usize = 1_000_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema_version: u32,
    seeds: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub struct Run {
    pub seed: u64,
    pub input_sha256: String,
    /// SHA256 of compact serde JSON Value (sorted object keys) of the full report.
    pub report_sha256: String,
    pub success: bool,
    pub complete: bool,
    pub pending_events: usize,
    pub real_time_us: u64,
    pub mode_time_us: u64,
    pub diagnostics: Vec<Diagnostic>,
    /// Exact decimal integer totals from settled hits, including partial runs.
    pub target_metrics: BTreeMap<String, BTreeMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct Statistics {
    pub count: usize,
    pub minimum: String,
    pub maximum: String,
    pub sum: String,
    pub mean_numerator: String,
    pub mean_denominator: usize,
    pub approximate_mean: f64,
    pub approximate_sample_variance: Option<f64>,
    pub approximate_standard_error: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub batch_version: String,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub quality: Quality,
    pub input_sha256: String,
    pub fixture_sha256: String,
    pub rotation_sha256: String,
    pub request_sha256: String,
    pub success: bool,
    pub complete: bool,
    pub requested_runs: usize,
    pub eligible_runs: usize,
    pub estimated_max_events: usize,
    pub runs: Vec<Run>,
    /// All-or-none: no success-only average if any requested run fails/is pending.
    pub target_statistics: Option<BTreeMap<String, BTreeMap<String, Statistics>>>,
    pub uncertainty_policy: String,
}

fn metrics(report: &synthetic::Result) -> Result<BTreeMap<String, BTreeMap<String, u128>>, String> {
    let mut totals = BTreeMap::new();
    for (name, target) in &report.targets {
        totals.insert(
            name.clone(),
            BTreeMap::from([
                ("final_hp".into(), u128::from(target.hp())),
                ("hp_lost".into(), 0),
                ("shield_absorbed".into(), 0),
                ("overkill".into(), 0),
                ("offered_damage".into(), 0),
            ]),
        );
    }
    for event in report.trace.iter().filter(|event| event.phase == "hit") {
        let Some(damage) = event.changes.get("damage") else {
            continue;
        };
        let name = event.changes["target"]
            .as_str()
            .ok_or("hit target missing")?;
        let hp = u128::from(damage["hp_lost"].as_u64().ok_or("hit HP loss missing")?);
        let overkill = u128::from(damage["overkill"].as_u64().ok_or("hit overkill missing")?);
        let absorbed = damage["absorbed"]
            .as_array()
            .ok_or("hit shield receipt missing")?
            .iter()
            .try_fold(0_u128, |sum, part| {
                sum.checked_add(u128::from(
                    part["amount"].as_u64().ok_or("shield amount missing")?,
                ))
                .ok_or("metric overflow")
            })?;
        let offered = hp
            .checked_add(overkill)
            .and_then(|n| n.checked_add(absorbed))
            .ok_or("metric overflow")?;
        let target = totals
            .get_mut(name)
            .ok_or("hit target absent from final state")?;
        for (metric, value) in [
            ("hp_lost", hp),
            ("overkill", overkill),
            ("shield_absorbed", absorbed),
            ("offered_damage", offered),
        ] {
            let total = target.get_mut(metric).unwrap();
            *total = total.checked_add(value).ok_or("metric overflow")?;
        }
    }
    Ok(totals)
}

fn statistics(values: &[u128]) -> Result<Statistics, String> {
    let count = values.len();
    let anchor = *values.first().ok_or("empty statistic sample")?;
    let sum = values.iter().try_fold(0_u128, |sum, value| {
        sum.checked_add(*value).ok_or("statistic sum overflow")
    })?;
    // Center in exact integers before f64 conversion, preserving small differences
    // between large adjacent totals. Floating outputs remain explicitly approximate.
    let mut centered_mean = 0.0;
    let mut m2 = 0.0;
    for (index, value) in values.iter().enumerate() {
        let centered = if *value >= anchor {
            (*value - anchor) as f64
        } else {
            -((anchor - *value) as f64)
        };
        let delta = centered - centered_mean;
        centered_mean += delta / (index + 1) as f64;
        m2 += delta * (centered - centered_mean);
    }
    let variance = (count > 1).then(|| m2.max(0.0) / (count - 1) as f64);
    Ok(Statistics {
        count,
        minimum: values.iter().min().unwrap().to_string(),
        maximum: values.iter().max().unwrap().to_string(),
        sum: sum.to_string(),
        mean_numerator: sum.to_string(),
        mean_denominator: count,
        approximate_mean: sum as f64 / count as f64,
        approximate_sample_variance: variance,
        approximate_standard_error: variance.map(|value| (value / count as f64).sqrt()),
    })
}

fn prepare_inputs(
    fixture: &[u8],
    rotation_source: &str,
    request_bytes: &[u8],
) -> Result<(rotation::Prepared, Request, usize), String> {
    if request_bytes.len() > 1_048_576 {
        return Err("batch request exceeds 1 MiB".into());
    }
    let request: Request = serde_json::from_slice(request_bytes).map_err(|e| e.to_string())?;
    if request.schema_version != 1 || request.seeds.is_empty() || request.seeds.len() > MAX_SEEDS {
        return Err("batch requires schema 1 and 1..=1000 explicit seeds".into());
    }
    if request.seeds.iter().collect::<BTreeSet<_>>().len() != request.seeds.len() {
        return Err("batch seed list contains duplicates".into());
    }
    let prepared = rotation::prepare(fixture, rotation_source)?;
    let estimated_max_events = prepared
        .max_events()?
        .checked_mul(request.seeds.len())
        .ok_or("batch event estimate overflow")?;
    if estimated_max_events > MAX_EVENTS {
        return Err("batch estimated events exceed 1000000".into());
    }
    Ok((prepared, request, estimated_max_events))
}

pub(crate) fn context_and_work(
    fixture: &[u8],
    source: &str,
    request: &[u8],
) -> Result<(serde_json::Value, usize), String> {
    let (prepared, _, work) = prepare_inputs(fixture, source, request)?;
    Ok((prepared.comparison_context(), work))
}

pub fn run(fixture: &[u8], rotation_source: &str, request_bytes: &[u8]) -> Result<Report, String> {
    let (prepared, request, estimated_max_events) =
        prepare_inputs(fixture, rotation_source, request_bytes)?;
    let mut identity = VERSION.as_bytes().to_vec();
    for bytes in [fixture, rotation_source.as_bytes(), request_bytes] {
        identity.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        identity.extend_from_slice(bytes);
    }
    let mut runs = Vec::new();
    let mut samples: BTreeMap<String, BTreeMap<String, Vec<u128>>> = BTreeMap::new();
    let mut eligible_runs = 0;
    for seed in &request.seeds {
        let report = prepared
            .run(Some(*seed))
            .map_err(|error| format!("seed {seed}: {error}"))?;
        let canonical = serde_json::to_value(&report).map_err(|e| e.to_string())?;
        let hash = sha256(&serde_json::to_vec(&canonical).map_err(|e| e.to_string())?);
        let simulation = report.simulation;
        let complete = simulation.success && simulation.pending_events == 0;
        let totals = metrics(&simulation)?;
        if complete {
            eligible_runs += 1;
            for (target, values) in &totals {
                for (metric, value) in values {
                    samples
                        .entry(target.clone())
                        .or_default()
                        .entry(metric.clone())
                        .or_default()
                        .push(*value);
                }
            }
        }
        runs.push(Run {
            seed: *seed,
            input_sha256: simulation.input_sha256,
            report_sha256: hash,
            success: simulation.success,
            complete,
            pending_events: simulation.pending_events,
            real_time_us: simulation.real_time_us,
            mode_time_us: simulation.mode_time_us,
            diagnostics: simulation.diagnostics,
            target_metrics: totals
                .into_iter()
                .map(|(target, values)| {
                    (
                        target,
                        values
                            .into_iter()
                            .map(|(metric, value)| (metric, value.to_string()))
                            .collect(),
                    )
                })
                .collect(),
        });
    }
    let complete = eligible_runs == request.seeds.len();
    let summary = if complete {
        Some(
            samples
                .into_iter()
                .map(|(target, values)| {
                    let stats = values
                        .into_iter()
                        .map(|(metric, values)| Ok((metric, statistics(&values)?)))
                        .collect::<Result<_, String>>()?;
                    Ok((target, stats))
                })
                .collect::<Result<_, String>>()?,
        )
    } else {
        None
    };
    Ok(Report {
        batch_version: VERSION.into(),
        engine_version: ENGINE_VERSION.into(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
        quality: Quality::Synthetic,
        input_sha256: sha256(&identity),
        fixture_sha256: sha256(fixture),
        rotation_sha256: sha256(rotation_source.as_bytes()),
        request_sha256: sha256(request_bytes),
        success: runs.iter().all(|run| run.success),
        complete,
        requested_runs: request.seeds.len(),
        eligible_runs,
        estimated_max_events,
        runs,
        target_statistics: summary,
        uncertainty_policy: "Approximate sample variance and standard error over the explicit distinct seeds; interpreting standard error assumes independent samples. No confidence interval, game RNG validation or game DPS is claimed.".into(),
    })
}

#[cfg(test)]
mod tests;
