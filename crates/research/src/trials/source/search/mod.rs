//! Exact empirical selection over a bounded authored domain, with trial checkpoints.
use super::*;

mod checkpoint;
pub use checkpoint::Checkpoint;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    MeanDamage,
    MeanDps,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Maximize,
    Minimize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Objective {
    pub metric: Metric,
    pub direction: Direction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub id: String,
    pub source: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    pub schema_version: u32,
    pub experiment: Request,
    pub objective: Objective,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CandidateSummary {
    pub id: String,
    pub source_sha256: String,
    pub evaluated_trials: usize,
    pub eligible: bool,
    pub mean_score: Option<ExactFraction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Rank {
    pub candidate_id: String,
    pub mean_score: ExactFraction,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchReport {
    pub input_sha256: String,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub request: SearchRequest,
    /// All planned trials were evaluated; failed trials remain explicit outcomes.
    pub complete: bool,
    /// Every candidate has a complete eligible seed sample.
    pub success: bool,
    pub candidates: Vec<CandidateSummary>,
    /// Only fully evaluated eligible candidates; ties preserve authored order.
    pub ranking: Vec<Rank>,
    pub best_candidate_ids: Vec<String>,
    pub checkpoint: Checkpoint,
    pub halt_diagnostic: Option<String>,
    pub selection_scope: String,
}

impl SearchRequest {
    fn prepare(&self) -> Result<Vec<PreparedSourceScenario>, String> {
        self.experiment.validate()?;
        if self.schema_version != 1 || self.candidates.is_empty() || self.candidates.len() > 64 {
            return Err("source search requires schema 1 and 1..=64 authored candidates".into());
        }
        let mut ids = BTreeSet::new();
        let mut source_bytes = 0_usize;
        let mut programs = Vec::new();
        let mut common = None;
        for candidate in &self.candidates {
            source_bytes = source_bytes
                .checked_add(candidate.source.len())
                .ok_or("source domain size overflow")?;
            if candidate.id.trim().is_empty()
                || candidate.id.len() > 128
                || !ids.insert(&candidate.id)
                || source_bytes > 16 * 1024 * 1024
            {
                return Err("source search requires unique bounded IDs and at most 16 MiB of authored source".into());
            }
            let program = self
                .experiment
                .prepare(&candidate.source)
                .map_err(|e| format!("candidate {}: {e}", candidate.id))?;
            let current = context(&program)?;
            if common.as_ref().is_some_and(|context| *context != current) {
                return Err("search candidate dummy/stop/combat assumption contexts differ".into());
            }
            common = Some(current);
            programs.push(program);
        }
        Ok(programs)
    }

    fn identity(&self, programs: &[PreparedSourceScenario]) -> Result<String, String> {
        // Bind compiled initial builds too, so content changes cannot reuse an identity.
        let exports = programs
            .iter()
            .map(PreparedSourceScenario::export)
            .collect::<Vec<_>>();
        Ok(identity(&[
            b"source-search-v1",
            ENGINE_SOURCE_SHA256.as_bytes(),
            &encode(self)?,
            &encode(&exports)?,
        ]))
    }
}

pub fn run(
    request: SearchRequest,
    checkpoint: Option<Checkpoint>,
    max_new_trials: usize,
) -> Result<SearchReport, String> {
    if max_new_trials == 0 || max_new_trials > 64_000 {
        return Err("max_new_trials must be in 1..=64000".into());
    }
    let programs = request.prepare()?;
    let input_sha256 = request.identity(&programs)?;
    let mut checkpoint = match checkpoint {
        Some(checkpoint) => {
            checkpoint.validate(&request, &input_sha256)?;
            checkpoint
        }
        None => Checkpoint::new(&input_sha256),
    };
    let count = request.candidates.len() * request.experiment.seeds.len();
    let mut halt_diagnostic = None;
    for _ in 0..max_new_trials {
        if checkpoint.unretained_trial_events.is_some() {
            halt_diagnostic = Some("retained receipt budget is exhausted; checkpoint cannot resume under this identity".into());
            break;
        }
        let index = checkpoint.runs.len();
        if index == count {
            break;
        }
        if checkpoint.events_charged + request.experiment.max_events_per_run
            > request.experiment.max_total_events
        {
            halt_diagnostic = Some("total event budget cannot admit another complete trial".into());
            break;
        }
        let candidate = index / request.experiment.seeds.len();
        let seed = request.experiment.seeds[index % request.experiment.seeds.len()];
        let (row, events) = run_trial(&programs[candidate], seed, &request.experiment);
        checkpoint.events_charged += events;
        let bytes = encode(&row)?.len();
        if checkpoint.retained_receipt_bytes + bytes > request.experiment.max_receipt_bytes {
            halt_diagnostic = Some("retained receipt budget exhausted".into());
            // Work was consumed, but this trial cannot become part of the retained prefix.
            checkpoint.unretained_trial_events = Some(events);
            break;
        }
        checkpoint.retained_receipt_bytes += bytes;
        checkpoint.runs.push(row);
    }
    let complete = checkpoint.runs.len() == count;
    if !complete && halt_diagnostic.is_none() {
        halt_diagnostic =
            Some("declared trial boundary reached; resume with the checkpoint".into());
    }
    let candidates = summarize(&request, &checkpoint)?;
    let ranking = rank(&request, &candidates)?;
    let best_candidate_ids = if let Some(best) = ranking.first() {
        ranking
            .iter()
            .take_while(|row| row.mean_score == best.mean_score)
            .map(|row| row.candidate_id.clone())
            .collect()
    } else {
        Vec::new()
    };
    let success = complete && candidates.iter().all(|candidate| candidate.eligible);
    checkpoint.seal()?;
    Ok(SearchReport {
        input_sha256, engine_version: ENGINE_VERSION.into(), engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
        request, complete, success, candidates, ranking, best_candidate_ids, checkpoint, halt_diagnostic,
        selection_scope: "Exact empirical means within the explicit authored candidates and common seeds only. Partial rankings contain only fully evaluated eligible candidates. No global optimum, exhaustive build coverage, statistical superiority or confirmed game evidence is claimed. Seed variation is not model uncertainty.".into(),
    })
}

fn summarize(
    request: &SearchRequest,
    checkpoint: &Checkpoint,
) -> Result<Vec<CandidateSummary>, String> {
    let mut candidates = Vec::new();
    let seeds = request.experiment.seeds.len();
    for (index, candidate) in request.candidates.iter().enumerate() {
        let start = (index * seeds).min(checkpoint.runs.len());
        let end = (start + seeds).min(checkpoint.runs.len());
        let rows = &checkpoint.runs[start..end];
        let eligible = rows.len() == seeds && rows.iter().all(|row| row.eligible);
        let mean_score = if eligible {
            let mut sum = ExactFraction::new(0, 1)?;
            for row in rows {
                let execution = row
                    .execution
                    .as_ref()
                    .ok_or("eligible search trial has no execution")?;
                let score = match request.objective.metric {
                    Metric::MeanDamage => execution.report.total_damage_exact,
                    Metric::MeanDps => dps(execution)?,
                };
                sum = sum.add(score)?;
            }
            Some(sum.div(ExactFraction::new(seeds as u128, 1)?)?)
        } else {
            None
        };
        candidates.push(CandidateSummary {
            id: candidate.id.clone(),
            source_sha256: sha256(candidate.source.as_bytes()),
            evaluated_trials: rows.len(),
            eligible,
            mean_score,
        });
    }
    Ok(candidates)
}

fn rank(request: &SearchRequest, candidates: &[CandidateSummary]) -> Result<Vec<Rank>, String> {
    let mut ranking: Vec<Rank> = Vec::new();
    for candidate in candidates {
        let Some(score) = candidate.mean_score else {
            continue;
        };
        let mut position = ranking.len();
        for (index, row) in ranking.iter().enumerate() {
            let order = score.try_cmp(row.mean_score).map_err(|e| e.to_string())?;
            let precedes = match request.objective.direction {
                Direction::Maximize => order.is_gt(),
                Direction::Minimize => order.is_lt(),
            };
            if precedes {
                position = index;
                break;
            }
        }
        ranking.insert(
            position,
            Rank {
                candidate_id: candidate.id.clone(),
                mean_score: score,
            },
        );
    }
    Ok(ranking)
}

#[cfg(test)]
mod tests;
