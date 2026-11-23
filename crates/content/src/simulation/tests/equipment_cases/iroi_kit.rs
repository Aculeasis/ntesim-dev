use super::*;

#[test]
fn iroi_kit_execution_skill_child_hits_a2_assist_and_a5_buff() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1075".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].gear_procs.clear();

    let mut mint = req.actors[0].clone();
    mint.slot = 1;
    mint.character_id = "character:1019".into();
    mint.awakening = ntesim_data::AwakeningSelection::prefix(0);
    mint.static_stats.base_atk = 1000;
    mint.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors.push(mint);

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Oneiroi_Skill".into(),
            damage_effect_id: "GE_Player_Oneiroi_Skill_Gun_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_500_000,
            actor_index: 1,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
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

    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 12);

    // Verify Mint attack (hit index 10) under Iroi buffs:
    // Target DEF 1080 with 5% ignore -> 1026 DEF -> factor = 1080 / 2106 = 20/39
    // Resistance factor = 0.8
    // Total mitigation = (20/39) * 0.8 = 16/39
    // Mint dynamic ATK = 1000 + 200 (base buff) + 150 (A5 buff) = 1350 ATK
    // Damage = 1350 * 1.0 * (16/39) = 21600 / 39 = 553.846... -> rounds to 554
    assert_eq!(hits[10]["rounded_damage"], 554);

    let trace = report["execution"]["trace"]
        .as_array()
        .unwrap_or(report["trace"].as_array().unwrap());
    assert!(
        trace.iter().any(|e| e["event_kind"] == "iroi_skill_buff"),
        "iroi_skill_buff must be present in trace"
    );
    assert!(
        trace.iter().any(|e| e["event_kind"] == "iroi_a2_assist"),
        "iroi_a2_assist must be present in trace"
    );
    assert!(
        trace.iter().any(|e| e["event_kind"] == "iroi_a5_atk_buff"),
        "iroi_a5_atk_buff must be present in trace"
    );
}

#[test]
fn iroi_a4_lamb_damage_bonus_and_a6_revive_grant() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1075".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Oneiroi_Skill".into(),
            damage_effect_id: "GE_Player_Oneiroi_Skill_Gun_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 0,
            ability_id: "GA_Oneiroi_UltraSkill".into(),
            damage_effect_id: "GE_Player_Oneiroi_UltraSkill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 800_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 3_000_000;

    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], true);

    let trace = report["trace"].as_array().unwrap();
    assert!(
        trace
            .iter()
            .any(|e| e["event_kind"] == "iroi_a6_revive_grant"),
        "iroi_a6_revive_grant must be present in trace"
    );
}
