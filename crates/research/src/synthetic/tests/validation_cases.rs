use super::*;

#[test]
pub(crate) fn actor_invocations_wait_on_local_deadlines_across_a_real_time_pause() {
    let mut value = actor_fixture();
    value["commands"].as_array_mut().unwrap().extend([
        json!({"clock":0,"delay_us":1,"effect":{"kind":"set_rate","clock":1,"rate_ppm":0}}),
        json!({"clock":0,"delay_us":11,"effect":{"kind":"set_rate","clock":1,"rate_ppm":1000000}}),
    ]);
    let result = run(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(result.success);
    let starts: Vec<_> = result
        .trace
        .iter()
        .filter(|e| e.changes.get("admission").is_some())
        .map(|e| (e.real_time_us, e.local_time_us))
        .collect();
    assert_eq!(starts, [(0, 0), (20, 10)]);
    assert_eq!(result.mode_time_us, 15);
    assert_eq!(result.pending_events, 1);
    assert_eq!(result.targets["first"].hp(), 180);
    assert_eq!(result.actors["hero"].ready_at_us("skill").unwrap(), 20);
}

#[test]
pub(crate) fn game_data_and_unknown_targets_are_rejected() {
    let mut v: Value = serde_json::from_slice(EXAMPLE).unwrap();
    v["content_id"] = json!("ntesim-current");
    assert!(run(&serde_json::to_vec(&v).unwrap()).is_err());
    v["content_id"] = json!("synthetic-v1");
    v["commands"][0]["effect"]["target"] = json!("missing");
    assert!(run(&serde_json::to_vec(&v).unwrap()).is_err());
}

#[test]
fn all_target_and_resource_effects_reject_unknown_ids_before_execution() {
    for effect in [
        json!({"kind":"heal","target":"missing","amount":1}),
        json!({"kind":"shield","target":"missing","id":"shield","source":"test","amount":1}),
        json!({"kind":"damage","target":"missing","amount":1,"hit_delay_us":0}),
        json!({"kind":"gain","pool":"missing","amount":1}),
        json!({"kind":"spend","pool":"missing","amount":1}),
    ] {
        let mut value: Value = serde_json::from_slice(EXAMPLE).unwrap();
        value["commands"][0]["effect"] = effect;
        let error = run(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(
            error.contains("unknown target") || error.contains("unknown resource"),
            "{error}"
        );
    }
}
