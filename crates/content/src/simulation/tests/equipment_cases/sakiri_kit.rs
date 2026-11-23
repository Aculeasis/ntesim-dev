use super::*;

#[test]
fn sakiri_kit_execution_attacks_skills_a3_a5_cooldown_reset_and_damage_bonus() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1003".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio {
        numerator: 0,
        denominator: 1,
    };
    // Target remains airborne through the Skill's delayed hit at 2.1s.
    req.targets[0].airborne_until_us = 2_500_000;

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Sagiri_Melee".into(),
            damage_effect_id: "GE_Player_Sagiri_Melee1_Damage".into(),
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
            ability_id: "GA_Sagiri_UltraSkill".into(),
            damage_effect_id: "GE_Player_Sagiri_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 800_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_000_000,
            actor_index: 0,
            ability_id: "GA_Sagiri_Skill".into(),
            damage_effect_id: "GE_Player_Sagiri_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 100_000,
        },
        // A5 must restore both energy and cooldown to admit this second Ultimate.
        GameActionCommand::InvokeAbility {
            timestamp_us: 3_000_000,
            actor_index: 0,
            ability_id: "GA_Sagiri_UltraSkill".into(),
            damage_effect_id: "GE_Player_Sagiri_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 800_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 5_000_000;

    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(report["invariants_passed"], true);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let actor0 = &report["final_actor_states"][0];
    assert_eq!(actor0["energy_conserved"], true);
    assert_eq!(actor0["initial_energy"], 100);
    assert_eq!(actor0["granted_energy"], 108); // 100 by A5 reset + 8 by ordinary hit grants
    assert_eq!(actor0["spent_energy"], 200);
    assert_eq!(actor0["energy"], 0);

    // Damage check:
    // Base hit: 1000 ATK * 1.0 rate * 0.5 def mitigation * 0.8 res factor = 400
    // Hit 1 (Melee at 0.1s): 400 dmg (Melee has no A3 bonus)
    // Hit 2 (Ultra 1 at 1.1s): A3 activates on Ultra cast (+6% = 1.06) -> 400 * 1.06 = 424 dmg
    // Hit 3 (Skill at 2.1s): A3 is active (+6% = 1.06) -> 400 * 1.06 = 424 dmg
    // Hit 4 (Ultra 2 at 3.1s): A3 is active (+6% = 1.06) -> 400 * 1.06 = 424 dmg
    let damages: Vec<u64> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    assert_eq!(
        damages,
        vec![
            400, 400, 480, 480, 480, 480, 1200, 400, 929, 400, 480, 480, 480, 480, 1200,
        ]
    );

    let a5_event = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_kind"] == "sakiri_a5_reset")
        .expect("sakiri_a5_reset event should be present");
    // The A5 reset tops up the 3 units already granted by the Skill1 ordinary hit.
    assert_eq!(a5_event["data"]["amount"], 97);
    assert_eq!(a5_event["data"]["ability_id"], "GA_Sagiri_UltraSkill");
    assert_eq!(a5_event["time_us"], 2_100_000);

    // A cast-time check would incorrectly grant A5 here: the target lands after
    // the cast at 2.0s but before the delayed hit at 2.1s.
    let mut landed = req.clone();
    landed.action_metadata.clear();
    landed.actions.truncate(3);
    landed
        .action_metadata
        .retain(|index, _| *index < landed.actions.len());
    landed.targets[0].airborne_until_us = 50_000; // World deadline: lands before the A2.1 hit after the cinematic.
    landed.relative_duration_us = 3_000_000;
    let landed_report = run_game_scenario(&landed, &limits).unwrap();
    assert_eq!(landed_report["execution_completed"], true);
    // No A5 reset here, but the ordinary Melee/Skill hits still grant 2 + 3 + 6 units.
    assert_eq!(landed_report["final_actor_states"][0]["granted_energy"], 11);
    assert!(
        !landed_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "sakiri_a5_reset")
    );
}

#[test]
fn sakiri_resonance_6_damage_bonus_scaling() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1003".into();
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

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Sagiri_Melee".into(),
            damage_effect_id: "GE_Player_Sagiri_Melee1_Damage".into(),
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
            ability_id: "GA_Sagiri_Melee".into(),
            damage_effect_id: "GE_Player_Sagiri_Melee1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 3_000_000;

    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], true);

    let damages: Vec<u64> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    // The research primary ratio remains 1.0 (400 damage). The next Normal
    // input advances stage two: native 0.138 and 0.757 * 1000 ATK * 0.4 DEF.
    assert_eq!(damages, vec![400, 55, 303]);
}
