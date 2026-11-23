use super::super::tests::{request, source};
use super::*;

fn search_request() -> SearchRequest {
    SearchRequest {
        schema_version: 1,
        experiment: request(),
        objective: Objective {
            metric: Metric::MeanDps,
            direction: Direction::Maximize,
        },
        candidates: vec![
            Candidate {
                id: "one".into(),
                source: source("mint attack;"),
            },
            Candidate {
                id: "two".into(),
                source: source("mint attack:2;"),
            },
            Candidate {
                id: "tie".into(),
                source: source("mint attack:2;"),
            },
        ],
    }
}

fn roundtrip(checkpoint: &Checkpoint) -> Checkpoint {
    serde_json::from_slice(&encode(checkpoint).unwrap()).unwrap()
}

#[test]
fn source_search_preserves_exact_order_ties_and_trial_boundary_resume() {
    let request = search_request();
    let full = run(request.clone(), None, 64).unwrap();
    assert!(full.complete && full.success);
    assert_eq!(
        full.ranking
            .iter()
            .map(|row| row.candidate_id.as_str())
            .collect::<Vec<_>>(),
        ["two", "tie", "one"]
    );
    assert_eq!(full.best_candidate_ids, ["two", "tie"]);
    let first = run(request.clone(), None, 1).unwrap();
    assert!(!first.complete && first.ranking.is_empty());
    let second = run(request.clone(), Some(roundtrip(&first.checkpoint)), 1).unwrap();
    assert!(!second.complete);
    assert_eq!(second.ranking[0].candidate_id, "one");
    let resumed = run(request.clone(), Some(roundtrip(&second.checkpoint)), 64).unwrap();
    assert_eq!(encode(&resumed).unwrap(), encode(&full).unwrap());
    let mut minimum = request;
    minimum.objective.direction = Direction::Minimize;
    minimum.objective.metric = Metric::MeanDamage;
    let report = run(minimum, None, 64).unwrap();
    assert_eq!(report.best_candidate_ids, ["one"]);
}

#[test]
fn source_search_never_ranks_failed_incomplete_or_budget_stopped_candidates() {
    let mut request = search_request();
    request.candidates[1].source = source("mint burst;");
    request.candidates[2].source =
        source("mint attack;").replace("duration=10", "duration=10 stop=victory");
    // A changed stop policy is incompatible, even when its execution would fail.
    assert!(
        run(request.clone(), None, 64)
            .unwrap_err()
            .contains("contexts differ")
    );
    request.candidates.pop();
    let report = run(request.clone(), None, 64).unwrap();
    assert!(report.complete && !report.success);
    assert_eq!(report.ranking.len(), 1);
    assert_eq!(report.ranking[0].candidate_id, "one");
    assert!(
        report.checkpoint.runs[2]
            .execution
            .as_ref()
            .unwrap()
            .diagnostic
            .is_some()
    );
    request.experiment.max_events_per_run = 1;
    let report = run(request.clone(), None, 64).unwrap();
    assert!(report.complete && report.ranking.is_empty());
    request = search_request();
    request.experiment.max_total_events = request.experiment.max_events_per_run;
    let report = run(request, None, 64).unwrap();
    assert_eq!(report.checkpoint.runs.len(), 1);
    assert!(!report.complete && report.ranking.is_empty());
    let mut request = search_request();
    request.experiment.max_receipt_bytes = 1;
    request.candidates[0].source = source("");
    let report = run(request.clone(), None, 64).unwrap();
    assert!(report.ranking.is_empty() && report.checkpoint.runs.is_empty());
    assert_eq!(report.checkpoint.unretained_trial_events, Some(0));
    let resumed = run(request, Some(roundtrip(&report.checkpoint)), 64).unwrap();
    assert_eq!(
        resumed.checkpoint.events_charged,
        report.checkpoint.events_charged
    );
}

#[test]
fn source_search_rejects_mismatched_and_corrupted_checkpoint_identities() {
    let request = search_request();
    let first = run(request.clone(), None, 1).unwrap();
    let mut changes = Vec::new();
    let mut changed = request.clone();
    changed.candidates[0].source.push_str(" # changed bytes");
    changes.push(changed);
    let mut changed = request.clone();
    changed.experiment.seeds.reverse();
    changes.push(changed);
    let mut changed = request.clone();
    changed.experiment.max_events_per_run += 1;
    changes.push(changed);
    let mut changed = request.clone();
    changed.objective.direction = Direction::Minimize;
    changes.push(changed);
    for changed in changes {
        assert!(
            run(changed, Some(roundtrip(&first.checkpoint)), 64)
                .unwrap_err()
                .contains("identity mismatch")
        );
    }
    let mut corrupted = roundtrip(&first.checkpoint);
    corrupted.runs[0].eligible = false;
    assert!(
        run(request.clone(), Some(corrupted), 64)
            .unwrap_err()
            .contains("corrupted")
    );
    let mut stale = roundtrip(&first.checkpoint);
    stale.engine_source_sha256 = "stale".into();
    assert!(
        run(request, Some(stale), 64)
            .unwrap_err()
            .contains("identity mismatch")
    );
}
