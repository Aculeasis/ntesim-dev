use super::*;

#[test]
pub(crate) fn capped_stack_does_not_refresh_and_pending_expiry_retains_exact_state() {
    let mut req = research_request();
    req.actors[0].gear_procs = vec![GearProcConfig::FirefliesCritDamage {
        crit_damage_per_stack: Ratio {
            numerator: 1,
            denominator: 3,
        },
        max_stacks: 1,
        duration_us: 20,
    }];
    req.actions.truncate(2);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req.relative_duration_us = 15;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["pending_events"], 1);
    assert_eq!(
        report["final_actor_states"][0]["active_fireflies_stacks"][0],
        serde_json::json!({"expires_at_us":20,"crit_damage_bonus":"1/3"})
    );
}
