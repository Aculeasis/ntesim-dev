use super::*;
use serde_json::json;

#[test]
fn switching_to_the_active_actor_preserves_recovery_and_pending_hits() {
    let mut request = research_request();
    request.actions.truncate(2);
    request.action_metadata.retain(|index, _| *index < 2);
    if let GameActionCommand::InvokeAbility {
        recovery_us,
        hit_delay_us,
        ..
    } = &mut request.actions[0]
    {
        *recovery_us = 100;
        *hit_delay_us = 200;
    }
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut request.actions[1] {
        *timestamp_us = 100;
    }
    request.actions.insert(
        1,
        GameActionCommand::SwitchActor {
            timestamp_us: 10,
            target_actor_index: 0,
            cooldown_us: 1000,
        },
    );
    let metadata = request.action_metadata.remove(&1).unwrap();
    request.action_metadata.insert(2, metadata);
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hit_receipts"][0]["time_us"], 100);
    assert_eq!(report["hit_receipts"][1]["time_us"], 200);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "switch_actor_noop")
            .count(),
        1
    );
    // A no-op must not release the actor's existing recovery lock.
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut request.actions[2] {
        *timestamp_us = 50;
    }
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["rejected_action"]["reason"],
        json!({"kind":"recovery","until_us":100})
    );
    assert_eq!(report["rejected_action"]["command_index"], 2);
}

#[test]
fn self_switch_does_not_start_or_refresh_the_real_switch_cooldown() {
    let mut request = research_request();
    let mut other = request.actors[0].clone();
    other.slot = 1;
    other.character_id = "character:1004".into();
    request.actors.push(other);
    request.action_metadata.clear();
    request.actions = [
        (0, 0, 1000),
        (1, 1, 100),
        (2, 0, 0),
        (3, 0, 1000),
        (4, 1, 0),
    ]
    .into_iter()
    .map(
        |(timestamp_us, target_actor_index, cooldown_us)| GameActionCommand::SwitchActor {
            timestamp_us,
            target_actor_index,
            cooldown_us,
        },
    )
    .collect();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["rejected_action"],
        json!({"command_index":4,"time_us":4,"actor_index":0,"reason":{"kind":"cooldown","until_us":101}})
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "switch_actor_noop")
            .count(),
        2
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "switch_actor")
            .count(),
        2
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 100);
    assert_eq!(report["final_actor_states"][1]["energy"], 100);
}
