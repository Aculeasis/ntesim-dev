use super::*;

#[test]
fn shinku_surging_crimson_a1_a2_a6_and_resonance_6() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1076".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].energy_capacity = 60;
    req.actors[0].initial_energy = 60;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].gear_procs.clear();

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Shinku_Melee".into(),
            damage_effect_id: "GE_Player_Shinku_Melee1_1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 0,
            ability_id: "GA_Shinku_Skill".into(),
            damage_effect_id: "GE_Player_Shinku_Skill1_1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_000_000,
            actor_index: 0,
            ability_id: "GA_Shinku_UltraSkill".into(),
            damage_effect_id: "GE_Player_Shinku_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 23_250_242,
            actor_index: 0,
            ability_id: "GA_Shinku_Melee".into(),
            damage_effect_id: "GE_Player_Shinku_Melee1_1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 25_000_000;
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
    // Passive 2 Wrathscale (BT >= 4) adds +50% DMG on UltraSkill1..5 vs non-boss dummy;
    // at 23.250242s (14s world idle after 2s Burst + 7.250242s Time Stop, after 13s Surging Crimson ends),
    // Menacing Gaze (A4 cap 16 -> 12 stacks accumulated) fires Watch_Damage on the Melee hit.
    assert_eq!(
        damages_a6,
        vec![
            462, 462, 107, 368, 971, 1311, 601, 601, 601, 601, 601, 1573, 464, 520, 10592
        ]
    );
    let packet_ids: Vec<&str> = report_a6["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["damage_effect_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        packet_ids,
        vec![
            "GE_Player_Shinku_Melee1_1_Damage",
            "GE_Player_Shinku_Skill1_1_Damage",
            "GE_Player_Shinku_Skill1_2_Damage",
            "GE_Player_Shinku_Skill1_3_Damage",
            "GE_Player_Shinku_UltraSkill1_Damage",
            "GE_Player_Shinku_UltraSkill2_Damage",
            "GE_Player_Shinku_UltraSkill3_Damage",
            "GE_Player_Shinku_UltraSkill3_Damage",
            "GE_Player_Shinku_UltraSkill3_Damage",
            "GE_Player_Shinku_UltraSkill3_Damage",
            "GE_Player_Shinku_UltraSkill3_Damage",
            "GE_Player_Shinku_UltraSkill4_Damage",
            "GE_Player_Shinku_UltraSkill5_Damage",
            "GE_Player_Shinku_Melee1_1_Damage",
            "GE_Player_Shinku_Watch_Damage",
        ]
    );
    let hit0 = &report_a6["hit_receipts"][0];
    assert_eq!(hit0["crit_rate"], "3/20");
    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    let report_a0 = run_game_scenario(&req_a0, &limits).unwrap();
    let damages_a0: Vec<u64> = report_a0["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    assert_eq!(damages_a0.len(), 15);
    let packet_ids_a0: Vec<&str> = report_a0["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["damage_effect_id"].as_str().unwrap())
        .collect();
    assert_eq!(packet_ids_a0, packet_ids);
}
