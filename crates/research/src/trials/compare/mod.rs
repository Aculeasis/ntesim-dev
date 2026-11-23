//! Paired synthetic comparisons; no inferred ranking or significance claim.
use crate::batch::{self, Statistics};
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use serde::Serialize;
use std::collections::BTreeMap;

pub const VERSION: &str = "synthetic-paired-compare-v1";

#[derive(Debug, Serialize)]
pub struct Pair {
    pub seed: u64,
    pub complete: bool,
    /// Exact signed candidate-minus-baseline differences for complete pairs.
    pub target_deltas: Option<BTreeMap<String, BTreeMap<String, String>>>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub comparison_version: String,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub input_sha256: String,
    pub quality: Quality,
    pub success: bool,
    pub complete: bool,
    pub eligible_pairs: usize,
    pub estimated_max_events: usize,
    pub direction: String,
    pub baseline: batch::Report,
    pub candidate: batch::Report,
    pub pairs: Vec<Pair>,
    pub paired_statistics: Option<BTreeMap<String, BTreeMap<String, Statistics>>>,
    pub uncertainty_policy: String,
}

fn signed_statistics(values: &[i128]) -> Result<Statistics, String> {
    let count = values.len();
    let anchor = *values.first().ok_or("empty paired sample")?;
    let sum = values.iter().try_fold(0_i128, |sum, value| {
        sum.checked_add(*value).ok_or("paired sum overflow")
    })?;
    let mut mean = 0.0;
    let mut m2 = 0.0;
    for (index, value) in values.iter().enumerate() {
        let centered = value
            .checked_sub(anchor)
            .ok_or("paired centering overflow")? as f64;
        let delta = centered - mean;
        mean += delta / (index + 1) as f64;
        m2 += delta * (centered - mean);
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
        approximate_standard_error: variance.map(|v| (v / count as f64).sqrt()),
    })
}

pub fn run(
    baseline_fixture: &[u8],
    baseline_source: &str,
    candidate_fixture: &[u8],
    candidate_source: &str,
    request: &[u8],
) -> Result<Report, String> {
    let (baseline_context, baseline_work) =
        batch::context_and_work(baseline_fixture, baseline_source, request)?;
    let (candidate_context, candidate_work) =
        batch::context_and_work(candidate_fixture, candidate_source, request)?;
    for field in [
        "duration_us",
        "mode_clock",
        "clocks",
        "targets",
        "rng_streams",
    ] {
        if baseline_context[field] != candidate_context[field] {
            return Err(format!("comparison context differs: {field}"));
        }
    }
    let estimated_max_events = baseline_work
        .checked_add(candidate_work)
        .ok_or("comparison work overflow")?;
    if estimated_max_events > batch::MAX_EVENTS {
        return Err("comparison combined events exceed 1000000".into());
    }
    let baseline = batch::run(baseline_fixture, baseline_source, request)?;
    let candidate = batch::run(candidate_fixture, candidate_source, request)?;
    let mut identity = VERSION.as_bytes().to_vec();
    for bytes in [
        baseline_fixture,
        baseline_source.as_bytes(),
        candidate_fixture,
        candidate_source.as_bytes(),
        request,
    ] {
        identity.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        identity.extend_from_slice(bytes);
    }
    let mut pairs = Vec::new();
    let mut samples: BTreeMap<String, BTreeMap<String, Vec<i128>>> = BTreeMap::new();
    let mut eligible_pairs = 0;
    for (left, right) in baseline.runs.iter().zip(&candidate.runs) {
        if left.seed != right.seed {
            return Err("comparison seed pairing mismatch".into());
        }
        let complete = left.complete && right.complete;
        let deltas = if complete {
            eligible_pairs += 1;
            let mut targets = BTreeMap::new();
            for (target, values) in &left.target_metrics {
                let counterpart = right
                    .target_metrics
                    .get(target)
                    .ok_or("comparison target missing")?;
                let mut changes = BTreeMap::new();
                for (metric, value) in values {
                    let left = value
                        .parse::<i128>()
                        .map_err(|_| "baseline metric exceeds signed range")?;
                    let right = counterpart
                        .get(metric)
                        .ok_or("candidate metric missing")?
                        .parse::<i128>()
                        .map_err(|_| "candidate metric exceeds signed range")?;
                    let delta = right
                        .checked_sub(left)
                        .ok_or("paired difference overflow")?;
                    changes.insert(metric.clone(), delta.to_string());
                    samples
                        .entry(target.clone())
                        .or_default()
                        .entry(metric.clone())
                        .or_default()
                        .push(delta);
                }
                targets.insert(target.clone(), changes);
            }
            Some(targets)
        } else {
            None
        };
        pairs.push(Pair {
            seed: left.seed,
            complete,
            target_deltas: deltas,
        });
    }
    let complete = baseline.complete && candidate.complete;
    let paired_statistics = if complete {
        Some(
            samples
                .into_iter()
                .map(|(target, values)| {
                    let stats = values
                        .into_iter()
                        .map(|(metric, values)| Ok((metric, signed_statistics(&values)?)))
                        .collect::<Result<_, String>>()?;
                    Ok((target, stats))
                })
                .collect::<Result<_, String>>()?,
        )
    } else {
        None
    };
    Ok(Report {
        comparison_version: VERSION.into(),
        engine_version: ENGINE_VERSION.into(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
        input_sha256: sha256(&identity),
        quality: Quality::Synthetic,
        success: baseline.success && candidate.success,
        complete,
        eligible_pairs,
        estimated_max_events,
        direction: "candidate-minus-baseline".into(),
        baseline,
        candidate,
        pairs,
        paired_statistics,
        uncertainty_policy: "Approximate sample variance/standard error of per-seed differences; interpretation assumes independent seed pairs. Matching streams does not guarantee matching draws after changed call order. No automatic ranking or significance claim.".into(),
    })
}

#[cfg(test)]
mod tests;
