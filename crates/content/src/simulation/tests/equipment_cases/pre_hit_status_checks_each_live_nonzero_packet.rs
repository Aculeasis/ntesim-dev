use super::*;

#[test]
fn pre_hit_status_checks_each_live_nonzero_packet() {
    for (hp_current, atk_rate) in [(0, Ratio::one()), (10_000_000, Ratio::zero())] {
        let mut req = research_request();
        req.actors[0].character_id = "character:1071".into();
        req.actors[0].gear_procs.clear();
        req.stochastic_crit = None;
        req.targets[0].hp_current = hp_current;
        req.action_metadata.clear();
        req.actions = vec![GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Chaos071_Skill".into(),
            damage_effect_id: "GE_Player_Chaos_Skill1_Damage".into(),
            array_index: 0,
            atk_rate,
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        }];
        req.relative_duration_us = 1;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["success"], true);
        assert_eq!(
            report["final_target_states"][0]
                .get("statuses")
                .and_then(serde_json::Value::as_array)
                .map_or(0, Vec::len),
            usize::from(hp_current > 0)
        );
        // A zero primary coefficient cannot apply Warrant; the native second bite can.
        if hp_current > 0 {
            assert_eq!(report["hit_receipts"][0]["rounded_damage"], 0);
            assert_eq!(report["hit_receipts"][1]["rounded_damage"], 1131);
        }
    }
}
