use super::*;

#[test]
fn chaos_warrant_a2_a3_a5_and_resonance_6() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1071".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].gear_procs.clear();
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Chaos071_QTE".into(),
            damage_effect_id: "GE_Player_Chaos_QTE1_Damage".into(),
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
            ability_id: "GA_Chaos071_Skill".into(),
            damage_effect_id: "GE_Player_Chaos_Skill1_Damage".into(),
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
            ability_id: "GA_Chaos071_UltraSkill".into(),
            damage_effect_id: "GE_Player_Chaos_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 500_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 22_802_493,
            actor_index: 0,
            ability_id: "GA_Chaos071_Melee".into(),
            damage_effect_id: "GE_Player_Chaos_Melee1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 20_000_000;

    let report_a6 = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report_a6["success"], true);
    assert_eq!(report_a6["invariants_passed"], true);
    assert_eq!(report_a6["execution_completed"], true);

    // This research fixture gives the primary a manual 1.0 rate; native children still project their Burst talent.
    let damages_a6: Vec<u64> = report_a6["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    assert_eq!(
        damages_a6,
        vec![400, 750, 1631, 750, 955, 955, 955, 2182, 400]
    );
    // Native Skill second bite: 2.175 * 1000 * 5/9 * 1.5 * .9 = 1631.25.
    assert_eq!(
        report_a6["hit_receipts"][2]["non_crit_damage"],
        serde_json::json!("6525/4")
    );
    assert_eq!(
        report_a6["hit_receipts"][3]["damage_effect_id"],
        "GE_Player_Chaos_UltraSkill1_Damage"
    );
    assert_eq!(
        report_a6["hit_receipts"][7]["damage_effect_id"],
        "GE_Player_Chaos_UltraSkill2_Damage"
    );
    let hit2 = &report_a6["hit_receipts"][3];
    assert_eq!(hit2["crit_damage"], "975/1");
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
        vec![400, 520, 1131, 520, 662, 662, 662, 1513, 400]
    );
}
