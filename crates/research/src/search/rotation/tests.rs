use super::*;
use serde_json::Value;
const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-random.json"
));
const REQUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-search.json"
));
fn request() -> Value {
    serde_json::from_slice(REQUEST).unwrap()
}

#[test]
fn exhaustive_candidate_ranking_matches_exact_complete_batch_means() {
    let report = run(FIXTURE, REQUEST).unwrap();
    assert!(report.success && report.complete);
    assert_eq!(report.eligible_candidates, 3);
    assert_eq!(report.estimated_max_events, 192);
    assert_eq!(
        report.best_candidate_ids.as_ref().unwrap(),
        &["three_actions"]
    );
    let ranks = report.ranking.as_ref().unwrap();
    assert_eq!(
        ranks
            .iter()
            .map(|r| r.candidate_id.as_str())
            .collect::<Vec<_>>(),
        ["three_actions", "two_actions", "one_action"]
    );
    for rank in ranks {
        let candidate = report
            .candidates
            .iter()
            .find(|c| c.id == rank.candidate_id)
            .unwrap();
        let sum: u128 = candidate
            .batch
            .runs
            .iter()
            .map(|r| {
                r.target_metrics["first"]["hp_lost"]
                    .parse::<u128>()
                    .unwrap()
            })
            .sum();
        assert_eq!(rank.mean_numerator, sum.to_string());
        assert_eq!(rank.mean_denominator, 8);
    }
    assert_eq!(
        serde_json::to_vec(&report).unwrap(),
        serde_json::to_vec(&run(FIXTURE, REQUEST).unwrap()).unwrap()
    );
}

#[test]
fn objective_direction_and_exact_ties_are_explicit() {
    let mut value = request();
    value["objective"]["direction"] = json!("minimize");
    let report = run(FIXTURE, &serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(report.best_candidate_ids.unwrap(), ["one_action"]);
    value["objective"]["metric"] = json!("final_hp");
    let report = run(FIXTURE, &serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(report.best_candidate_ids.unwrap(), ["three_actions"]);
    value["objective"]["metric"] = json!("hp_lost");
    value["objective"]["direction"] = json!("maximize");
    let mut tied = value["candidates"][2].clone();
    tied["id"] = json!("three_copy");
    value["candidates"].as_array_mut().unwrap().push(tied);
    let report = run(FIXTURE, &serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(
        report.best_candidate_ids.unwrap(),
        ["three_actions", "three_copy"]
    );
    assert_eq!(report.ranking.unwrap()[0].candidate_id, "three_actions");
}

#[test]
fn failing_or_pending_candidates_prevent_an_empirical_winner() {
    let mut value = request();
    value["candidates"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"four_actions","source":"clock 0; repeat 4 { do attack; wait 10; }"}));
    let report = run(FIXTURE, &serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(!report.success && !report.complete);
    assert_eq!(report.eligible_candidates, 3);
    assert!(report.ranking.is_none() && report.best_candidate_ids.is_none());
    assert!(
        report.candidates[3]
            .batch
            .runs
            .iter()
            .all(|r| !r.diagnostics.is_empty())
    );
    value["candidates"][3]["source"] = json!("clock 0; wait 30; do attack;");
    let report = run(FIXTURE, &serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(report.success);
    assert!(!report.complete);
    assert!(report.ranking.is_none());
    assert_eq!(report.candidates[3].batch.runs[0].pending_events, 1);
}

#[test]
fn request_objective_candidate_seed_and_global_work_limits_are_checked() {
    for invalid in [
        "schema",
        "empty",
        "duplicate_id",
        "metric",
        "target",
        "direction",
        "seed",
        "source",
        "count",
        "unknown_field",
    ] {
        let mut value = request();
        match invalid {
            "schema" => value["schema_version"] = json!(2),
            "empty" => value["candidates"] = json!([]),
            "duplicate_id" => value["candidates"][1]["id"] = json!("one_action"),
            "metric" => value["objective"]["metric"] = json!("dps"),
            "target" => value["objective"]["target"] = json!("unknown"),
            "direction" => value["objective"]["direction"] = json!("better"),
            "seed" => value["seeds"] = json!([42, 42]),
            "source" => {
                value["candidates"][2]["source"] = json!("clock 0; repeat 0 { do attack; }")
            }
            "count" => {
                value["candidates"] = json!(
                    (0..65)
                        .map(|i| json!({"id":i.to_string(),"source":"clock 0; do attack;"}))
                        .collect::<Vec<_>>()
                )
            }
            _ => value["typo"] = json!(true),
        }
        assert!(
            run(FIXTURE, &serde_json::to_vec(&value).unwrap()).is_err(),
            "{invalid}"
        );
    }
    let mut value = request();
    value["seeds"] = json!((0..1000).collect::<Vec<_>>());
    value["candidates"] = json!(
        (0..64)
            .map(
                |i| json!({"id":i.to_string(),"source":"clock 0; repeat 4 { do attack; wait 10; }"})
            )
            .collect::<Vec<_>>()
    );
    assert!(
        run(FIXTURE, &serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .contains("combined events")
    );
    assert!(
        run(FIXTURE, &vec![b' '; 1_048_577])
            .unwrap_err()
            .contains("1 MiB")
    );
}

#[test]
fn scores_rank_exactly_above_float_precision_and_ties_keep_input_order() {
    let anchor = u128::from(u64::MAX);
    let scores = vec![
        (0, anchor),
        (1, anchor + 1),
        (2, anchor - 1),
        (3, anchor + 1),
    ];
    assert_eq!(
        ordered_scores(scores.clone(), Direction::Maximize)
            .iter()
            .map(|(i, _)| *i)
            .collect::<Vec<_>>(),
        [1, 3, 0, 2]
    );
    assert_eq!(
        ordered_scores(scores, Direction::Minimize)
            .iter()
            .map(|(i, _)| *i)
            .collect::<Vec<_>>(),
        [2, 0, 1, 3]
    );
}

#[test]
fn search_identity_binds_candidate_order_inline_source_and_exact_bytes() {
    let report = run(FIXTURE, REQUEST).unwrap();
    let mut changed = request();
    changed["candidates"].as_array_mut().unwrap().reverse();
    let reordered = run(FIXTURE, &serde_json::to_vec(&changed).unwrap()).unwrap();
    assert_ne!(report.input_sha256, reordered.input_sha256);
    assert_eq!(report.best_candidate_ids, reordered.best_candidate_ids);
    changed["candidates"][0]["source"] = json!(
        changed["candidates"][0]["source"]
            .as_str()
            .unwrap()
            .to_owned()
            + " # comment"
    );
    let changed = run(FIXTURE, &serde_json::to_vec(&changed).unwrap()).unwrap();
    assert_ne!(changed.input_sha256, reordered.input_sha256);
    assert_ne!(
        changed.candidates[0].source_sha256,
        reordered.candidates[0].source_sha256
    );
    assert_eq!(
        changed.ranking.unwrap()[0].mean_numerator,
        reordered.ranking.unwrap()[0].mean_numerator
    );
}
