use super::*;

#[test]
fn baicang_domain_a1_a2_a5_a6_and_resonance_6() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1023".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].gear_procs.clear();
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();

    req.action_metadata.clear();
    req.model_assumptions.insert(
        crate::characters::packet_semantics::BAICANG_DOT_PACKET_HYPOTHESIS_ID.into(),
        crate::characters::packet_semantics::BAICANG_DOT_PACKET_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Cang_QTE".into(),
            damage_effect_id: "GE_Player_Cang_QTE_Damage".into(),
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
            ability_id: "GA_Cang_Skill".into(),
            damage_effect_id: "GE_Player_Cang_Skill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 14_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_000_000,
            actor_index: 0,
            ability_id: "GA_Cang_UltraSkill".into(),
            damage_effect_id: "GE_Player_Cang_UltraSkill2_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 500_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 17_854_959,
            actor_index: 0,
            ability_id: "GA_Cang_Skill".into(),
            damage_effect_id: "GE_Player_Cang_Skill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 14_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 21_854_959,
            actor_index: 0,
            ability_id: "GA_Cang_Melee".into(),
            damage_effect_id: "GE_Player_Cang_Melee1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 21_000_000;

    let report_a6 = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report_a6["success"], true);
    assert_eq!(report_a6["invariants_passed"], true);
    assert_eq!(report_a6["execution_completed"], true);

    let damages_a6: Vec<u64> = report_a6["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    let domain_ticks_a6 = report_a6["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
        .collect::<Vec<_>>();
    assert_eq!(domain_ticks_a6.len(), 8);
    assert_eq!(domain_ticks_a6[6]["time_us"], 18_854_959);
    assert_eq!(domain_ticks_a6[6]["rounded_damage"], 202);
    assert_eq!(domain_ticks_a6[7]["time_us"], 20_854_959);
    assert_eq!(domain_ticks_a6[7]["rounded_damage"], 170);
    assert_eq!(
        damages_a6,
        vec![
            424, 448, 528, 211, 211, 211, 211, 192, 192, 655, 202, 170, 448
        ]
    );
    let hit2 = &report_a6["hit_receipts"][2];
    assert_eq!(hit2["crit_rate"], "3/10");
    let hit4 = &report_a6["hit_receipts"][12];
    assert_eq!(hit4["crit_rate"], "0/1");
    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let report_a0 = run_game_scenario(&req_a0, &limits).unwrap();
    let damages_a0: Vec<u64> = report_a0["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    assert_eq!(
        damages_a0,
        vec![400, 400, 400, 160, 160, 160, 160, 160, 160, 400, 400]
    );
}
