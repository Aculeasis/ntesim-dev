use super::*;

#[test]
fn jiuyuan_rose_pact_a1_a5_and_resonance_6() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1055".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Kuhara_QTE".into(),
            damage_effect_id: "GE_Player_Kuhara_QTE_Damage".into(),
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
            ability_id: "GA_Kuhara_Skill".into(),
            damage_effect_id: "GE_Player_Kuhara_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 16_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_000_000,
            actor_index: 0,
            ability_id: "GA_Kuhara_UltraSkill".into(),
            damage_effect_id: "GE_Player_Kuhara_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 500_000,
            cooldown_us: 15_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 32_599_441,
            actor_index: 0,
            ability_id: "GA_Kuhara_QTE".into(),
            damage_effect_id: "GE_Player_Kuhara_QTE_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 30_000_000;

    let report_a6 = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report_a6["success"], true);
    assert_eq!(report_a6["invariants_passed"], true);
    assert_eq!(report_a6["execution_completed"], true);
    assert_eq!(report_a6["hits_settled"], 20);

    let damages_a6: Vec<u64> = report_a6["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    let primary_damages_a6 = [damages_a6[0], damages_a6[1], damages_a6[7], damages_a6[19]];
    assert_eq!(primary_damages_a6, [400, 510, 510, 400]);

    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let report_a0 = run_game_scenario(&req_a0, &limits).unwrap();
    assert_eq!(report_a0["hits_settled"], 20);
    let damages_a0: Vec<u64> = report_a0["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    let primary_damages_a0 = [damages_a0[0], damages_a0[1], damages_a0[7], damages_a0[19]];
    assert_eq!(primary_damages_a0, [400, 400, 400, 400]);

    let mut req_a1 = req.clone();
    req_a1.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    let report_a1 = run_game_scenario(&req_a1, &limits).unwrap();
    assert_eq!(report_a1["hits_settled"], 20);
    let damages_a1: Vec<u64> = report_a1["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    let primary_damages_a1 = [damages_a1[0], damages_a1[1], damages_a1[7], damages_a1[19]];
    assert_eq!(primary_damages_a1, [400, 420, 420, 400]);

    let mut req_a5 = req.clone();
    req_a5.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    let report_a5 = run_game_scenario(&req_a5, &limits).unwrap();
    assert_eq!(report_a5["hits_settled"], 20);
    let damages_a5: Vec<u64> = report_a5["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    let primary_damages_a5 = [damages_a5[0], damages_a5[1], damages_a5[7], damages_a5[19]];
    assert_eq!(primary_damages_a5, [400, 454, 454, 400]);
}
