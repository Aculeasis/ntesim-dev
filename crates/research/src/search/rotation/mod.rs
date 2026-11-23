//! Exhaustive empirical ranking over an explicit bounded synthetic candidate list.
use crate::batch;
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;

pub const VERSION: &str = "synthetic-candidate-search-v1";
pub const MAX_CANDIDATES: usize = 64;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Minimize,
    Maximize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Objective {
    pub target: String,
    pub metric: String,
    pub direction: Direction,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    id: String,
    source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema_version: u32,
    seeds: Vec<u64>,
    objective: Objective,
    candidates: Vec<Candidate>,
}

#[derive(Debug, Serialize)]
pub struct CandidateResult {
    pub id: String,
    pub source_sha256: String,
    pub batch: batch::Report,
}

#[derive(Debug, Serialize)]
pub struct Rank {
    pub candidate_id: String,
    pub mean_numerator: String,
    pub mean_denominator: usize,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub search_version: String,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub input_sha256: String,
    pub fixture_sha256: String,
    pub request_sha256: String,
    pub quality: Quality,
    pub objective: Objective,
    pub success: bool,
    pub complete: bool,
    pub eligible_candidates: usize,
    pub estimated_max_events: usize,
    pub candidates: Vec<CandidateResult>,
    pub ranking: Option<Vec<Rank>>,
    pub best_candidate_ids: Option<Vec<String>>,
    pub selection_scope: String,
}

fn ordered_scores(mut scores: Vec<(usize, u128)>, direction: Direction) -> Vec<(usize, u128)> {
    scores.sort_by(|(left_index, left), (right_index, right)| {
        let order = match direction {
            Direction::Minimize => left.cmp(right),
            Direction::Maximize => right.cmp(left),
        };
        order.then(left_index.cmp(right_index))
    });
    scores
}

pub fn run(fixture: &[u8], request_bytes: &[u8]) -> Result<Report, String> {
    if request_bytes.len() > 1_048_576 {
        return Err("search request exceeds 1 MiB".into());
    }
    let request: Request = serde_json::from_slice(request_bytes).map_err(|e| e.to_string())?;
    if request.schema_version != 1
        || request.candidates.is_empty()
        || request.candidates.len() > MAX_CANDIDATES
    {
        return Err("search requires schema 1 and 1..=64 explicit candidates".into());
    }
    let mut ids = BTreeSet::new();
    for candidate in &request.candidates {
        if candidate.id.trim().is_empty() || candidate.id.len() > 128 || !ids.insert(&candidate.id)
        {
            return Err(
                "search candidate IDs must be nonempty, unique and at most 128 bytes".into(),
            );
        }
    }
    if ![
        "final_hp",
        "hp_lost",
        "shield_absorbed",
        "overkill",
        "offered_damage",
    ]
    .contains(&request.objective.metric.as_str())
    {
        return Err("unknown search objective metric".into());
    }
    let seeds = serde_json::to_vec(&json!({"schema_version":1,"seeds":request.seeds}))
        .map_err(|e| e.to_string())?;
    let mut estimated_max_events = 0_usize;
    for candidate in &request.candidates {
        let (context, work) = batch::context_and_work(fixture, &candidate.source, &seeds)
            .map_err(|e| format!("candidate {}: {e}", candidate.id))?;
        if context["targets"].get(&request.objective.target).is_none() {
            return Err("unknown search objective target".into());
        }
        estimated_max_events = estimated_max_events
            .checked_add(work)
            .ok_or("search event estimate overflow")?;
        if estimated_max_events > batch::MAX_EVENTS {
            return Err("search combined events exceed 1000000".into());
        }
    }
    let mut candidates = Vec::new();
    let mut eligible_candidates = 0;
    let mut scores = Vec::new();
    for (index, candidate) in request.candidates.iter().enumerate() {
        let report = batch::run(fixture, &candidate.source, &seeds)
            .map_err(|e| format!("candidate {}: {e}", candidate.id))?;
        if report.complete {
            eligible_candidates += 1;
            let stats = &report.target_statistics.as_ref().unwrap()[&request.objective.target]
                [&request.objective.metric];
            if stats.mean_denominator != request.seeds.len() {
                return Err("search sample count mismatch".into());
            }
            let sum = stats
                .mean_numerator
                .parse::<u128>()
                .map_err(|_| "search score exceeds exact range")?;
            scores.push((index, sum));
        }
        candidates.push(CandidateResult {
            id: candidate.id.clone(),
            source_sha256: sha256(candidate.source.as_bytes()),
            batch: report,
        });
    }
    let complete = eligible_candidates == candidates.len();
    let (ranking, best_candidate_ids) = if complete {
        // All candidates share the same denominator, so compare exact integer sums.
        let scores = ordered_scores(scores, request.objective.direction);
        let best_score = scores[0].1;
        let best_ids = scores
            .iter()
            .take_while(|(_, sum)| *sum == best_score)
            .map(|(index, _)| candidates[*index].id.clone())
            .collect();
        let ranking = scores
            .into_iter()
            .map(|(index, sum)| Rank {
                candidate_id: candidates[index].id.clone(),
                mean_numerator: sum.to_string(),
                mean_denominator: request.seeds.len(),
            })
            .collect();
        (Some(ranking), Some(best_ids))
    } else {
        (None, None)
    };
    let mut identity = VERSION.as_bytes().to_vec();
    for bytes in [fixture, request_bytes] {
        identity.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        identity.extend_from_slice(bytes);
    }
    Ok(Report {
        search_version: VERSION.into(),
        engine_version: ENGINE_VERSION.into(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
        input_sha256: sha256(&identity),
        fixture_sha256: sha256(fixture),
        request_sha256: sha256(request_bytes),
        quality: Quality::Synthetic,
        objective: request.objective,
        success: candidates.iter().all(|c| c.batch.success),
        complete,
        eligible_candidates,
        estimated_max_events,
        candidates,
        ranking,
        best_candidate_ids,
        selection_scope: "Exact empirical means over these explicit seeds and candidates only; no statistical superiority, global optimum, build legality, sustainable cycles or game DPS is established.".into(),
    })
}

#[cfg(test)]
mod tests;
