use super::*;

#[test]
fn skia_a1_a2_a3_damage_bonus_and_resonance_6() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1008".into();
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
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Skia_Skill".into(),
            damage_effect_id: "GE_Player_Skia_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 16_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 0,
            ability_id: "GA_Skia_UltraSkill".into(),
            damage_effect_id: "GE_Player_Skia_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 2_000_000;
    let report_a6 = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report_a6["success"], true);
    assert_eq!(report_a6["hits_settled"], 10);
    let hit0_a6 = &report_a6["hit_receipts"][0];
    assert_eq!(hit0_a6["non_crit_damage"], "2214/5");
    let hit1_a6 = &report_a6["hit_receipts"][6];
    assert_eq!(hit1_a6["non_crit_damage"], "2214/5");
    let mut req_a3 = req.clone();
    req_a3.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    let report_a3 = run_game_scenario(&req_a3, &limits).unwrap();
    assert_eq!(report_a3["hits_settled"], 10);
    let hit0_a3 = &report_a3["hit_receipts"][0];
    assert_eq!(hit0_a3["non_crit_damage"], "432/1");
    let hit1_a3 = &report_a3["hit_receipts"][6];
    assert_eq!(hit1_a3["non_crit_damage"], "432/1");
    let mut req_a1 = req.clone();
    req_a1.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    let report_a1 = run_game_scenario(&req_a1, &limits).unwrap();
    assert_eq!(report_a1["hits_settled"], 10);
    let hit0_a1 = &report_a1["hit_receipts"][0];
    assert_eq!(hit0_a1["non_crit_damage"], "400/1");
    let hit1_a1 = &report_a1["hit_receipts"][6];
    assert_eq!(hit1_a1["non_crit_damage"], "400/1");
    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let report_a0 = run_game_scenario(&req_a0, &limits).unwrap();
    assert_eq!(report_a0["hits_settled"], 10);
    let hit0_a0 = &report_a0["hit_receipts"][0];
    assert_eq!(hit0_a0["non_crit_damage"], "400/1");
    let hit1_a0 = &report_a0["hit_receipts"][6];
    assert_eq!(hit1_a0["non_crit_damage"], "400/1");
}
