use super::*;

#[test]
pub(crate) fn boundary_observations_capture_continuous_state_without_changing_combat_or_rng() {
    let mut value = observation_fixture();
    let report = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(report.success);
    assert_eq!(report.trace.len(), 20);
    let observations: Vec<_> = report
        .trace
        .iter()
        .filter_map(|e| e.changes.get("observation"))
        .collect();
    assert_eq!(observations.len(), 4);
    for (index, hp) in [200, 175, 150, 140].into_iter().enumerate() {
        let state = observations[index];
        assert_eq!(state["version"], "synthetic-state-observation-v1");
        assert_eq!(state["label"], "cycle_boundary");
        assert_eq!(state["targets"]["first"]["hp"]["current"], hp);
        assert_eq!(state["resources"]["energy"]["current"], 3 - index);
        assert_eq!(state["rng"]["streams"]["outcomes"]["draws"], index);
        assert_eq!(state["real_time_us"], index * 10);
        assert_eq!(state["clocks"]["0"]["local_time_us"], index * 10);
        assert_eq!(state["clocks"]["0"]["phase_millionths"], 0);
        if index > 0 {
            assert_eq!(
                state["actors"]["hero"]["actions"]["skill"]["ready_at_us"],
                index * 10 + 1
            )
        }
    }
    value["commands"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["effect"]["kind"] != "observe_state");
    let unobserved = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(report.targets, unobserved.targets);
    assert_eq!(report.resources, unobserved.resources);
    assert_eq!(report.actors, unobserved.actors);
    assert_eq!(report.rng, unobserved.rng);
}

#[test]
pub(crate) fn observation_labels_are_validated_and_no_streams_omit_rng_state() {
    for label in [" ".to_owned(), "x".repeat(129)] {
        let mut value = observation_fixture();
        value["commands"][0]["effect"]["label"] = json!(label);
        assert!(run(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut value: Value = serde_json::from_slice(EXAMPLE).unwrap();
    value["commands"] =
        json!([{"clock":0,"delay_us":0,"effect":{"kind":"observe_state","label":"initial"}}]);
    let report = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(report.success);
    assert!(report.trace[1].changes["observation"].get("rng").is_none());
}

#[test]
pub(crate) fn random_actor_hits_match_golden_words_and_keep_causal_origins() {
    let bytes = serde_json::to_vec(&random_fixture()).unwrap();
    let report = run(&bytes).unwrap();
    assert!(report.success);
    assert_eq!(report.quality, Quality::Synthetic);
    assert_eq!(report.rng_version, rng::VERSION);
    assert_eq!(report.targets["first"].hp(), 140);
    assert_eq!(report.resources["energy"].current(), 0);
    assert_eq!(report.trace.len(), 12);
    let receipts: Vec<_> = report
        .trace
        .iter()
        .filter_map(|e| e.changes.get("random_damage"))
        .collect();
    for (index, word) in [0xa15c02b7_u64, 0x7b47f409, 0xba1d3330]
        .into_iter()
        .enumerate()
    {
        assert_eq!(receipts[index]["sample"]["draws"][0]["index"], index);
        assert_eq!(receipts[index]["sample"]["draws"][0]["value"], word);
        assert_eq!(receipts[index]["sample"]["value"], word % 2);
        assert_eq!(receipts[index]["outcome_index"], word % 2);
    }
    for hit in report.trace.iter().filter(|e| e.phase == "hit") {
        let effect = report
            .trace
            .iter()
            .find(|e| Some(e.event_id) == hit.cause)
            .unwrap();
        assert!(effect.changes.get("random_damage").is_some());
        assert_eq!(effect.action_origin, hit.action_origin);
        assert_eq!(hit.action_origin.as_ref().unwrap().actor, "hero");
        assert_eq!(hit.real_time_us, effect.real_time_us + 2);
    }
    assert_eq!(
        serde_json::to_vec(&report).unwrap(),
        serde_json::to_vec(&run(&bytes).unwrap()).unwrap()
    );
    assert_eq!(
        serde_json::to_value(report.rng.unwrap()).unwrap()["streams"]["outcomes"]["draws"],
        3
    );
}

#[test]
pub(crate) fn random_dependencies_and_weights_fail_before_running() {
    for invalid in [
        "stream",
        "empty",
        "zero",
        "bound",
        "overflow",
        "target",
        "duplicate_sequence",
    ] {
        let mut value = random_fixture();
        let effect = &mut value["actors"]["hero"]["actions"]["skill"]["effects"][0];
        match invalid {
            "stream" => effect["stream"] = json!("unknown"),
            "empty" => effect["outcomes"] = json!([]),
            "zero" => effect["outcomes"][0]["weight"] = json!(0),
            "bound" => effect["outcomes"][0]["weight"] = json!(rng::WORD_RANGE),
            "overflow" => effect["outcomes"][0]["weight"] = json!(u64::MAX),
            "target" => effect["target"] = json!("unknown"),
            _ => value["rng_streams"]["other"] = json!(54),
        }
        assert!(
            run(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{invalid}"
        );
    }
}

#[test]
pub(crate) fn random_effect_scheduling_error_and_failed_admission_do_not_consume_draws() {
    let mut queue = Scheduler::default();
    queue.add_clock(0, 1_000_000).unwrap();
    let mut rng = Rng::new(42, BTreeMap::from([("outcomes".into(), 54)])).unwrap();
    let before = rng.clone();
    let error = random_hit(
        Invocation {
            index: 0,
            clock: 0,
            local_time_us: 0,
            cause: 42,
        },
        RandomHit {
            target: "first",
            stream: "outcomes",
            outcomes: &[WeightedDamage {
                weight: 1,
                amount: 10,
            }],
            delay_us: 0,
            origin: None,
        },
        &mut rng,
        &mut queue,
    )
    .unwrap_err();
    assert!(error.contains("UnknownCause(42)"));
    assert_eq!(rng, before);
    assert_eq!(queue.pending_count(), 0);
    let mut value = random_fixture();
    value["actors"]["hero"]["locked_until_us"] = json!(1);
    let report = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(!report.success);
    assert_eq!(report.diagnostics[0].code, "ACTION_ADMISSION");
    assert_eq!(report.resources["energy"].current(), 3);
    assert_eq!(
        serde_json::to_value(report.rng.unwrap()).unwrap()["streams"]["outcomes"]["draws"],
        0
    );
}

#[test]
pub(crate) fn random_draw_occurs_at_effect_even_if_hit_is_beyond_horizon() {
    let mut value = random_fixture();
    value["duration_us"] = json!(1);
    let report = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(report.success);
    assert_eq!(report.targets["first"].hp(), 200);
    assert_eq!(report.resources["energy"].current(), 2);
    assert_eq!(report.trace.len(), 3);
    assert_eq!(report.pending_events, 3);
    assert_eq!(
        serde_json::to_value(report.rng.unwrap()).unwrap()["streams"]["outcomes"]["draws"],
        1
    );
}

#[test]
pub(crate) fn unrelated_streams_do_not_shift_random_combat_and_legacy_reports_omit_rng() {
    let value = random_fixture();
    let report = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    let mut extended = value;
    extended["rng_streams"]["proc"] = json!(7);
    extended["commands"].as_array_mut().unwrap().insert(0, json!({"clock":0,"delay_us":0,"effect":{
        "kind":"random_damage","target":"first","stream":"proc","outcomes":[{"weight":1,"amount":0}],"hit_delay_us":0
    }}));
    let extended = run(&serde_json::to_vec(&extended).unwrap()).unwrap();
    assert_eq!(extended.targets["first"], report.targets["first"]);
    let final_rng = serde_json::to_value(extended.rng.unwrap()).unwrap();
    let baseline_rng = serde_json::to_value(report.rng.unwrap()).unwrap();
    assert_eq!(
        final_rng["streams"]["outcomes"],
        baseline_rng["streams"]["outcomes"]
    );
    let legacy = serde_json::to_value(run(EXAMPLE).unwrap()).unwrap();
    assert!(legacy.get("rng").is_none());
    assert_eq!(legacy["rng_version"], "unused; no random operations");
}
