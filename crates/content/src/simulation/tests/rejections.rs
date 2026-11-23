use super::*;
use serde_json::json;

#[test]
fn rejections_preserve_gate_facts_command_identity_and_accepted_prefix() {
    for (recovery, cooldown, cost, expected) in [
        (100, 0, 0, json!({"kind":"recovery","until_us":100})),
        (0, 100, 0, json!({"kind":"cooldown","until_us":100})),
        (
            0,
            0,
            101,
            json!({"kind":"energy","required":101,"available":100}),
        ),
    ] {
        let mut request = research_request();
        if let GameActionCommand::InvokeAbility {
            recovery_us,
            cooldown_us,
            ..
        } = &mut request.actions[0]
        {
            *recovery_us = recovery;
            *cooldown_us = cooldown;
        }
        if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut request.actions[1] {
            *energy_cost = cost;
        }
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["rejected_action"],
            json!({"command_index":1,"time_us":10,"actor_index":0,"reason":expected})
        );
        assert_eq!(report["success"], false);
        assert_eq!(report["execution_completed"], false);
        assert_eq!(
            report["action_metadata_ledger"].as_array().unwrap().len(),
            1
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 100);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 0);
    }
    let request = research_request();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert!(report.get("rejected_action").is_none());
}

#[test]
fn rejected_switches_and_inactive_actors_keep_their_distinct_commands() {
    let mut request = research_request();
    let mut other = request.actors[0].clone();
    other.slot = 1;
    other.character_id = "character:1003".into();
    request.actors.push(other);
    if let GameActionCommand::InvokeAbility { actor_index, .. } = &mut request.actions[1] {
        *actor_index = 1;
    }
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["rejected_action"],
        json!({"command_index":1,"time_us":10,"actor_index":1,"reason":{"kind":"actor_not_active","active_actor_index":0}})
    );
    request.actions[1] = GameActionCommand::SwitchActor {
        timestamp_us: 10,
        target_actor_index: 1,
        cooldown_us: 0,
    };
    request.action_metadata.remove(&1);
    if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut request.actions[0] {
        *recovery_us = 100;
    }
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["rejected_action"],
        json!({"command_index":1,"time_us":10,"actor_index":0,"reason":{"kind":"recovery","until_us":100}})
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "switch_actor")
            .count(),
        0
    );
}
