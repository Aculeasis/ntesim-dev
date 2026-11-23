use super::*;

#[test]
fn lacrimosa_a1_a2_damage_bonus_and_resonance_6_def_ignore() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1004".into();
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
            ability_id: "GA_Lacrimosa_QTE".into(),
            damage_effect_id: "GE_Player_Lacrimosa_QTE1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 15_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 0,
            ability_id: "GA_Lacrimosa_Skill".into(),
            damage_effect_id: "GE_Player_Lacrimosa_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 16_000_000,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 2_000_000;
    let report_a6 = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report_a6["success"], true);
    let hit0_a6 = &report_a6["hit_receipts"][0];
    assert_eq!(hit0_a6["non_crit_damage"], "4600/9");
    let hit1_a6 = &report_a6["hit_receipts"][1];
    assert_eq!(hit1_a6["non_crit_damage"], "4600/9");
    let mut req_a2 = req.clone();
    req_a2.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    let report_a2 = run_game_scenario(&req_a2, &limits).unwrap();
    let hit0_a2 = &report_a2["hit_receipts"][0];
    assert_eq!(hit0_a2["non_crit_damage"], "460/1");
    let hit1_a2 = &report_a2["hit_receipts"][1];
    assert_eq!(hit1_a2["non_crit_damage"], "460/1");
    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let report_a0 = run_game_scenario(&req_a0, &limits).unwrap();
    let hit0_a0 = &report_a0["hit_receipts"][0];
    assert_eq!(hit0_a0["non_crit_damage"], "400/1");
    let hit1_a0 = &report_a0["hit_receipts"][1];
    assert_eq!(hit1_a0["non_crit_damage"], "400/1");
}
