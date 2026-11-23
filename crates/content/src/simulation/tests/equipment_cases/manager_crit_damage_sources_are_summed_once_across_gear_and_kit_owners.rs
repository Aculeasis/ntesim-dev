use super::*;

#[test]
fn manager_crit_damage_sources_are_summed_once_across_gear_and_kit_owners() {
    let mut req = sample_s1_request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.actors[0].gear_procs = vec![GearProcConfig::FirefliesCritDamage {
        crit_damage_per_stack: Ratio {
            numerator: 1,
            denominator: 4,
        },
        max_stacks: 1,
        duration_us: 10_000_000,
    }];
    let mut haniel = req.actors[0].clone();
    haniel.slot = 1;
    haniel.character_id = "character:1020".into();
    haniel.awakening = ntesim_data::AwakeningSelection::prefix(6);
    haniel.gear_procs.clear();
    req.actors.push(haniel);
    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Mint019_Skill".into(),
            damage_effect_id: "GE_Player_Mint_Skill1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 4_000_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 1,
            ability_id: "GA_Haniel_UltraSkill".into(),
            damage_effect_id: "GE_Player_Haniel_Ultraskill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 5_000_000,
            target_actor_index: 0,
            cooldown_us: 4_000_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 5_000_000,
            actor_index: 0,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
    ];
    req.relative_duration_us = 5_000_001;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["success"], true);
    let hit = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["command_index"] == 4)
        .unwrap();
    let exact = |fraction: &serde_json::Value| {
        serde_json::from_value::<crate::simulation::ExactFraction>(fraction.clone())
            .unwrap()
            .to_f64()
    };
    let multiplier = exact(&hit["crit_damage"]) / exact(&hit["non_crit_damage"]);
    // 1 + base 1/2 + Fireflies 1/4 + Mint A5 1/4 + Haniel A6 3/25.
    assert!((multiplier - 2.12).abs() < 1e-9, "{multiplier}");
}
