use super::*;

#[test]
pub(crate) fn observations_retain_all_clock_rates_and_fractional_phase() {
    let mut value = observation_fixture();
    value["clocks"][0]["rate_ppm"] = json!(500_001);
    value["clocks"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":1,"rate_ppm":0}));
    value["duration_us"] = json!(61);
    let report = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(report.success);
    let states: Vec<_> = report
        .trace
        .iter()
        .filter_map(|e| e.changes.get("observation"))
        .collect();
    for (index, state) in states.into_iter().enumerate() {
        assert_eq!(state["real_time_us"], index * 20);
        assert_eq!(state["clocks"]["0"]["local_time_us"], index * 10);
        assert_eq!(state["clocks"]["0"]["phase_millionths"], index * 20);
        assert_eq!(state["clocks"]["0"]["rate_ppm"], 500_001);
        assert_eq!(
            state["clocks"]["1"],
            json!({"local_time_us":0,"phase_millionths":0,"rate_ppm":0})
        );
    }
}

#[test]
pub(crate) fn actor_leaf_observation_retains_origin_and_sees_only_current_state() {
    let mut value = observation_fixture();
    let command = value["commands"][1].clone();
    value["commands"] = json!([command]);
    value["actors"]["hero"]["actions"]["skill"]["effects"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"observe_state","label":"after_roll_before_hit"}));
    let report = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(report.success);
    let event = report
        .trace
        .iter()
        .find(|e| e.changes.get("observation").is_some())
        .unwrap();
    assert_eq!(event.action_origin.as_ref().unwrap().effect_index, 1);
    let state = &event.changes["observation"];
    assert_eq!(state["targets"]["first"]["hp"]["current"], 200);
    assert_eq!(state["resources"]["energy"]["current"], 2);
    assert_eq!(state["rng"]["streams"]["outcomes"]["draws"], 1);
    assert_eq!(
        state["actors"]["hero"]["actions"]["skill"]["ready_at_us"],
        11
    );
    assert_eq!(report.targets["first"].hp(), 175);
    let admission = report
        .trace
        .iter()
        .find(|e| Some(e.event_id) == event.cause)
        .unwrap();
    assert!(admission.changes.get("admission").is_some());
}

#[test]
pub(crate) fn actor_invocations_charge_once_and_link_child_effects_and_hits() {
    let bytes = serde_json::to_vec(&actor_fixture()).unwrap();
    let result = run(&bytes).unwrap();
    assert!(result.success);
    assert_eq!(result.targets["first"].hp(), 170);
    assert_eq!(result.resources["energy"].current(), 3);
    assert_eq!(result.resources["points"].current(), 0);
    assert_eq!(result.actors["hero"].ready_at_us("skill").unwrap(), 30);
    assert_eq!(result.actors["hero"].locked_until_us(), 23);
    assert_eq!(result.trace.len(), 15);
    for hit in result.trace.iter().filter(|e| e.phase == "hit") {
        let effect = result
            .trace
            .iter()
            .find(|e| Some(e.event_id) == hit.cause)
            .unwrap();
        let invocation = result
            .trace
            .iter()
            .find(|e| Some(e.event_id) == effect.cause)
            .unwrap();
        let command = result
            .trace
            .iter()
            .find(|e| Some(e.event_id) == invocation.cause)
            .unwrap();
        assert_eq!(command.phase, "command");
        assert_eq!(
            invocation.changes["admission"]["spent"],
            json!({"energy":2,"points":1})
        );
        assert_eq!(effect.action_origin.as_ref().unwrap().effect_index, 0);
        assert_eq!(hit.action_origin, effect.action_origin);
        assert_eq!(hit.command_index, command.command_index);
    }
    assert_eq!(
        serde_json::to_vec(&result).unwrap(),
        serde_json::to_vec(&run(&bytes).unwrap()).unwrap()
    );
}

#[test]
pub(crate) fn failed_actor_admission_does_not_charge_or_schedule_its_effects() {
    for failure in ["recovery", "cooldown", "shortage"] {
        let mut value = actor_fixture();
        value["commands"][1]["delay_us"] = json!(match failure {
            "recovery" => 2,
            "cooldown" => 5,
            _ => 10,
        });
        if failure == "shortage" {
            value["resources"]["points"]["current"] = json!(1);
        }
        let result = run(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(!result.success, "{failure}");
        assert_eq!(result.diagnostics[0].code, "ACTION_ADMISSION");
        assert_eq!(result.diagnostics[0].path, "commands/1");
        assert_eq!(result.targets["first"].hp(), 190);
        assert_eq!(result.resources["energy"].current(), 5);
        assert_eq!(
            result.resources["points"].current(),
            if failure == "shortage" { 0 } else { 2 }
        );
        assert_eq!(result.actors["hero"].ready_at_us("skill").unwrap(), 10);
        assert_eq!(result.actors["hero"].locked_until_us(), 3);
        assert_eq!(result.trace.len(), 7);
        assert_eq!(result.pending_events, 1);
    }
}

#[test]
pub(crate) fn actor_dependencies_and_initial_state_are_checked_before_execution() {
    for failure in [
        "actor",
        "action",
        "clock",
        "pool",
        "child",
        "nested",
        "empty",
        "initial",
        "missing_ready",
    ] {
        let mut value = actor_fixture();
        match failure {
            "actor" => value["commands"][0]["effect"]["actor"] = json!("unknown"),
            "action" => value["commands"][0]["effect"]["action"] = json!("unknown"),
            "clock" => value["commands"][0]["clock"] = json!(0),
            "pool" => {
                value["actors"]["hero"]["actions"]["skill"]["rule"]["costs"] = json!({"unknown":0})
            }
            "child" => {
                value["actors"]["hero"]["actions"]["skill"]["effects"][0]["target"] =
                    json!("unknown")
            }
            "nested" => {
                value["actors"]["hero"]["actions"]["skill"]["effects"][0] =
                    json!({"kind":"invoke_action","actor":"hero","action":"skill"})
            }
            "empty" => value["actors"]["hero"]["actions"]["skill"]["effects"] = json!([]),
            "initial" => value["actors"]["hero"]["initial_time_us"] = json!(1),
            _ => {
                value["actors"]["hero"]["actions"]["skill"]
                    .as_object_mut()
                    .unwrap()
                    .remove("ready_at_us");
            }
        }
        assert!(
            run(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{failure}"
        );
    }
}

#[test]
pub(crate) fn formula_hit_retains_exact_audit_and_causal_delayed_settlement() {
    let result = run(&serde_json::to_vec(&formula_fixture()).unwrap()).unwrap();
    assert!(result.success);
    assert_eq!(result.targets["first"].hp(), 69);
    assert_eq!(result.trace.len(), 3);
    assert_eq!(
        result.trace[1].changes["formula"]["exact"],
        json!({"numerator":"38181","denominator":"1250"})
    );
    assert_eq!(result.trace[1].changes["formula"]["amount"], 31);
    assert_eq!(result.trace[2].real_time_us, 11);
    assert_eq!(result.trace[2].cause, Some(result.trace[1].event_id));
    assert_eq!(
        serde_json::to_vec(&result).unwrap(),
        serde_json::to_vec(&run(&serde_json::to_vec(&formula_fixture()).unwrap()).unwrap())
            .unwrap()
    );
}

#[test]
pub(crate) fn formula_errors_preserve_state_and_a_failed_effect_trace() {
    let mut bad_rational = formula_fixture();
    bad_rational["commands"][0]["effect"]["formula"]["flat"] = json!("1/0");
    assert!(
        run(&serde_json::to_vec(&bad_rational).unwrap())
            .unwrap_err()
            .contains("denominator")
    );
    for failure in ["missing", "empty_stage", "overflow"] {
        let mut value = formula_fixture();
        let effect = &mut value["commands"][0]["effect"];
        match failure {
            "missing" => effect["stats"] = json!({}),
            "empty_stage" => {
                effect["formula"]["stages"] = json!([{"id": "", "multiplier": "1/1"}]);
            }
            _ => {
                effect["stats"]["attack"] = json!(format!("{}/1", u64::MAX));
            }
        }
        let result = run(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(!result.success, "{failure}");
        assert_eq!(result.targets["first"].hp(), 100);
        assert_eq!(result.trace.len(), 2);
        assert_eq!(result.trace[1].phase, "effect");
        assert_eq!(result.diagnostics[0].code, "FORMULA_EFFECT");
        assert_eq!(result.diagnostics[0].path, "commands/0");
        assert_eq!(result.pending_events, 0);
    }
}

#[test]
pub(crate) fn formula_target_unit_and_horizon_are_explicit() {
    let mut value = formula_fixture();
    value["duration_us"] = json!(10);
    let result = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(result.success);
    assert_eq!(result.targets["first"].hp(), 100);
    assert_eq!(result.pending_events, 1);
    assert_eq!(result.trace[1].changes["formula"]["amount"], 31);
    value["commands"][0]["effect"]["target"] = json!("unknown");
    assert!(run(&serde_json::to_vec(&value).unwrap()).is_err());
    value["commands"][0]["effect"]["target"] = json!("first");
    value["commands"][0]["effect"]["formula"]["unit"] = json!("other_quantum");
    assert!(run(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
pub(crate) fn causal_trace_and_persistent_state() {
    let r = run(EXAMPLE).unwrap();
    assert!(r.success);
    assert_eq!(r.quality, Quality::Synthetic);
    assert_eq!(r.targets["first"].hp(), 60);
    assert_eq!(r.targets["second"].hp(), 90);
    assert_eq!(r.resources["energy"].current(), 3);
    for hit in r.trace.iter().filter(|e| e.phase == "hit") {
        let effect = r
            .trace
            .iter()
            .find(|e| Some(e.event_id) == hit.cause)
            .unwrap();
        assert_eq!(effect.phase, "effect");
        let command = r
            .trace
            .iter()
            .find(|e| Some(e.event_id) == effect.cause)
            .unwrap();
        assert_eq!(command.phase, "command");
    }
}

#[test]
pub(crate) fn same_input_reproduces_serialized_result() {
    assert_eq!(
        serde_json::to_vec(&run(EXAMPLE).unwrap()).unwrap(),
        serde_json::to_vec(&run(EXAMPLE).unwrap()).unwrap()
    );
}

#[test]
pub(crate) fn shortage_preserves_partial_trace_without_waiting() {
    let mut v: Value = serde_json::from_slice(EXAMPLE).unwrap();
    v["commands"][0] =
        json!({"clock":0,"delay_us":0,"effect":{"kind":"spend","pool":"energy","amount":100}});
    let r = run(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(!r.success);
    assert_eq!(r.resources["energy"].current(), 3);
    assert_eq!(r.real_time_us, 0);
    assert_eq!(r.diagnostics[0].path, "commands/0");
}

#[test]
pub(crate) fn mode_pause_changes_mode_time_without_resetting_target_state() {
    let mut v: Value = serde_json::from_slice(EXAMPLE).unwrap();
    v["commands"].as_array_mut().unwrap().insert(
        0,
        json!({"clock":0,"delay_us":0,"effect":{"kind":"set_rate","clock":1,"rate_ppm":0}}),
    );
    v["commands"].as_array_mut().unwrap().push(
        json!({"clock":0,"delay_us":50,"effect":{"kind":"set_rate","clock":1,"rate_ppm":1000000}}),
    );
    let r = run(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(r.success);
    assert_eq!(r.real_time_us, 100);
    assert_eq!(r.mode_time_us, 50);
    assert_eq!(r.targets["second"].hp(), 90);
}
