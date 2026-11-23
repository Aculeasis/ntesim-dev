use super::*;

#[test]
fn hotori_a1_a2_a5_a6_and_resonance_6() {
    // Skill at 11s has 66 Timepiece (>60) and Burst at 36s has the exact 120.
    // Level-1 native packet coefficients (authored array_index 0):
    // Skill1/Skill3 .400, UltraSkill1 .170. Target DEF 1080 (factor 1/2),
    // resistance 1/5 (factor 4/5). A1 needs recorded actions (none here).
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1052".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio {
        numerator: 0,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 0,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 0,
        denominator: 1,
    };

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 11_000_000,
            actor_index: 0,
            ability_id: "GA_Jin_Skill".into(),
            damage_effect_id: "GE_Player_Jin_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 10_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 36_000_000,
            actor_index: 0,
            ability_id: "GA_Jin_UltraSkill".into(),
            damage_effect_id: "GE_Player_Jin_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 55_000_000;
    let report_a6 = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report_a6["success"], true);
    assert_eq!(report_a6["execution_completed"], true);
    assert_eq!(report_a6["hits_settled"], 17);
    // A6: 30% DEF ignore inside the Burst mode is 10/17; A2+A5 bonus 27/20,
    // R6 adds 30% of the 1000 base ATK -> 1300. Skill is outside the mode:
    // 1300*.4*(1/2)*(4/5)*(23/20) = 1196/5.
    let hit0_a6 = &report_a6["hit_receipts"][0];
    assert_eq!(hit0_a6["non_crit_damage"], "1196/5");
    // 1300*.17*(10/17)*(4/5)*(27/20) = 702/5.
    let hit1_a6 = &report_a6["hit_receipts"][7];
    assert_eq!(hit1_a6["non_crit_damage"], "702/5");

    let mut req_a5 = req.clone();
    req_a5.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    let report_a5 = run_game_scenario(&req_a5, &limits).unwrap();
    assert_eq!(report_a5["hits_settled"], 17);
    // A5 adds 15% against the sole living dummy: 160*(23/20) and 68*(27/20).
    let hit0_a5 = &report_a5["hit_receipts"][0];
    assert_eq!(hit0_a5["non_crit_damage"], "184/1");
    let hit1_a5 = &report_a5["hit_receipts"][7];
    assert_eq!(hit1_a5["non_crit_damage"], "459/5");

    let mut req_a2 = req.clone();
    req_a2.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    let report_a2 = run_game_scenario(&req_a2, &limits).unwrap();
    assert_eq!(report_a2["hits_settled"], 17);
    // A2 only applies during the Burst mode: 160 and 68*(6/5).
    let hit0_a2 = &report_a2["hit_receipts"][0];
    assert_eq!(hit0_a2["non_crit_damage"], "160/1");
    let hit1_a2 = &report_a2["hit_receipts"][7];
    assert_eq!(hit1_a2["non_crit_damage"], "408/5");

    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let report_a0 = run_game_scenario(&req_a0, &limits).unwrap();
    assert_eq!(report_a0["hits_settled"], 17);
    let hit0_a0 = &report_a0["hit_receipts"][0];
    assert_eq!(hit0_a0["non_crit_damage"], "160/1");
    let hit1_a0 = &report_a0["hit_receipts"][7];
    assert_eq!(hit1_a0["non_crit_damage"], "68/1");
}
