use super::*;

#[test]
pub(crate) fn connected_hit_callbacks_do_not_run_for_targets_already_defeated() {
    let mut req = research_request();
    req.targets[0].hp_current = 0;
    req.actors[0].gear_procs = vec![GearProcConfig::FirefliesCritDamage {
        crit_damage_per_stack: Ratio {
            numerator: 1,
            denominator: 4,
        },
        max_stacks: 2,
        duration_us: 100,
    }];
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req.relative_duration_us = 10;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();

    assert_eq!(report["hits_settled"], 0);
    assert_eq!(report["total_damage_exact"], serde_json::json!("0/1"));
    assert_eq!(report["final_actor_states"][0]["fireflies_stacks"], 0);
    assert!(!report["trace"].as_array().unwrap().iter().any(|event| {
        event["event_kind"] == "fireflies_crit_stack"
            || event["event_kind"] == "fireflies_stack_capped"
    }));
}
