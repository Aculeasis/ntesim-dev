//! Explicit finite boundary invariants; never an infinite sustainability proof.
use crate::{
    batch, rotation,
    scheduler::{ClockId, RATE_SCALE},
};
use ntesim_contracts::{Diagnostic, ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const VERSION: &str = "synthetic-finite-cycle-check-v1";

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Relation {
    Equal,
    NonDecreasing,
    Unconstrained,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceRule {
    minimum: u64,
    relation: Relation,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema_version: u32,
    label: String,
    clock: ClockId,
    interval_us: u64,
    max_phase_error_millionths: u64,
    expected_observations: usize,
    resources: BTreeMap<String, ResourceRule>,
}

#[derive(Debug, Serialize)]
pub struct Boundary {
    pub event_id: u64,
    pub command_index: usize,
    pub real_time_us: u64,
    pub total_phase_millionths: String,
    pub resources: BTreeMap<String, u64>,
    /// Relative to the previous selected boundary; absent at the first one.
    pub resource_deltas: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct Interval {
    pub from_event_id: u64,
    pub to_event_id: u64,
    pub actual_millionths: String,
    pub expected_millionths: String,
    pub error_millionths: String,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub audit_version: String,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub input_sha256: String,
    pub checks_sha256: String,
    pub quality: Quality,
    pub label: String,
    pub clock: ClockId,
    pub expected_observations: usize,
    pub observed_observations: usize,
    pub complete: bool,
    pub passed: bool,
    pub estimated_max_events: usize,
    pub boundaries: Vec<Boundary>,
    pub intervals: Vec<Interval>,
    pub diagnostics: Vec<Diagnostic>,
    pub simulation: rotation::Report,
    pub scope: String,
}

fn diagnostic(code: &str, path: String, message: String) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        path,
        message,
    }
}

pub fn run(fixture: &[u8], source: &str, checks: &[u8]) -> Result<Report, String> {
    if checks.len() > 1_048_576 {
        return Err("cycle checks exceed 1 MiB".into());
    }
    let request: Request = serde_json::from_slice(checks).map_err(|e| e.to_string())?;
    if request.schema_version != 1
        || request.label.trim().is_empty()
        || request.label.len() > 128
        || request.interval_us == 0
        || !(2..=10001).contains(&request.expected_observations)
        || request.resources.len() > 128
    {
        return Err("cycle checks require schema 1, nonblank label <=128 bytes, positive interval, 2..=10001 observations and <=128 resource rules".into());
    }
    let prepared = rotation::prepare(fixture, source)?;
    if prepared.comparison_context()["clocks"]
        .get(request.clock.to_string())
        .is_none()
    {
        return Err("unknown cycle check clock".into());
    }
    for pool in request.resources.keys() {
        if !prepared.has_resource(pool) {
            return Err(format!("unknown cycle check resource {pool}"));
        }
    }
    let estimated_max_events = prepared.max_events()?;
    if estimated_max_events > batch::MAX_EVENTS {
        return Err("cycle events exceed 1000000".into());
    }
    let simulation = prepared.run(None)?;
    let mut diagnostics = Vec::new();
    let mut boundaries: Vec<Boundary> = Vec::new();
    let mut intervals = Vec::new();
    let expected = u128::from(request.interval_us) * u128::from(RATE_SCALE);
    for event in &simulation.simulation.trace {
        let Some(observation) = event.changes.get("observation") else {
            continue;
        };
        if observation["label"] != request.label {
            continue;
        }
        let index = boundaries.len();
        let clock = &observation["clocks"][request.clock.to_string()];
        let local = clock["local_time_us"]
            .as_u64()
            .ok_or("observation clock reading missing")?;
        let fraction = clock["phase_millionths"]
            .as_u64()
            .ok_or("observation clock fraction missing")?;
        if fraction >= u64::from(RATE_SCALE) {
            return Err("invalid observation clock fraction".into());
        }
        let phase = u128::from(local) * u128::from(RATE_SCALE) + u128::from(fraction);
        let mut resources = BTreeMap::new();
        for (pool, rule) in &request.resources {
            let current = observation["resources"][pool]["current"]
                .as_u64()
                .ok_or("observation resource value missing")?;
            if current < rule.minimum {
                diagnostics.push(diagnostic(
                    "RESOURCE_FLOOR",
                    format!("observations/{index}/resources/{pool}"),
                    format!(
                        "{pool} has {current}, below declared minimum {}",
                        rule.minimum
                    ),
                ));
            }
            resources.insert(pool.clone(), current);
        }
        let resource_deltas = if let Some(previous) = boundaries.last() {
            let previous_phase = previous
                .total_phase_millionths
                .parse::<u128>()
                .map_err(|_| "invalid previous phase")?;
            let actual = phase
                .checked_sub(previous_phase)
                .ok_or("observation clock moved backwards")?;
            let error = actual.abs_diff(expected);
            if error > u128::from(request.max_phase_error_millionths) {
                diagnostics.push(diagnostic("CYCLE_INTERVAL", format!("observations/{index}/clock"), format!("interval {actual} millionths differs from {expected} by {error}, beyond declared tolerance {}", request.max_phase_error_millionths)));
            }
            intervals.push(Interval {
                from_event_id: previous.event_id,
                to_event_id: event.event_id,
                actual_millionths: actual.to_string(),
                expected_millionths: expected.to_string(),
                error_millionths: error.to_string(),
            });
            let mut deltas = BTreeMap::new();
            for (pool, rule) in &request.resources {
                let prior = previous.resources[pool];
                let current = resources[pool];
                let valid = match rule.relation {
                    Relation::Equal => current == prior,
                    Relation::NonDecreasing => current >= prior,
                    Relation::Unconstrained => true,
                };
                let delta = i128::from(current) - i128::from(prior);
                if !valid {
                    diagnostics.push(diagnostic("RESOURCE_RELATION", format!("observations/{index}/resources/{pool}"), format!("{pool} changed from {prior} to {current}, violating its declared boundary relation")));
                }
                deltas.insert(pool.clone(), delta.to_string());
            }
            Some(deltas)
        } else {
            None
        };
        boundaries.push(Boundary {
            event_id: event.event_id,
            command_index: event.command_index,
            real_time_us: event.real_time_us,
            total_phase_millionths: phase.to_string(),
            resources,
            resource_deltas,
        });
    }
    if boundaries.len() != request.expected_observations {
        diagnostics.push(diagnostic(
            "OBSERVATION_COUNT",
            "observations".into(),
            format!(
                "found {} labeled boundaries; expected {}",
                boundaries.len(),
                request.expected_observations
            ),
        ));
    }
    let executed = simulation.simulation.success && simulation.simulation.pending_events == 0;
    if !executed {
        diagnostics.push(diagnostic(
            "SIMULATION_INCOMPLETE",
            "simulation".into(),
            "execution failed or has pending events; finite checks cannot pass".into(),
        ));
    }
    let complete = executed && boundaries.len() == request.expected_observations;
    let mut identity = VERSION.as_bytes().to_vec();
    for bytes in [fixture, source.as_bytes(), checks] {
        identity.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        identity.extend_from_slice(bytes);
    }
    Ok(Report {
        audit_version: VERSION.into(),
        engine_version: ENGINE_VERSION.into(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
        input_sha256: sha256(&identity),
        checks_sha256: sha256(checks),
        quality: Quality::Synthetic,
        label: request.label,
        clock: request.clock,
        expected_observations: request.expected_observations,
        observed_observations: boundaries.len(),
        complete,
        passed: complete && diagnostics.is_empty(),
        estimated_max_events,
        boundaries,
        intervals,
        diagnostics,
        simulation,
        scope: "Only the declared resource floors/relations and clock intervals at these finite labeled boundaries are checked. Passing does not prove intervening invariants, periodic kernel state, infinite sustainability or game DPS.".into(),
    })
}

#[cfg(test)]
mod tests;
