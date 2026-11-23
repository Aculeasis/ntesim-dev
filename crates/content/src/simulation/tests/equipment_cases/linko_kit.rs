use super::*;
use serde_json::json;

#[test]
fn linko_resonance_field_a2_a5_a6_and_resonance_6() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1072".into();
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
            ability_id: "GA_Radio072_Melee".into(),
            damage_effect_id: "GE_Player_Radio072_Melee1_Damage".into(),
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
            ability_id: "GA_Radio072_UltraSkill".into(),
            damage_effect_id: "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 500_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_000_000,
            actor_index: 0,
            ability_id: "GA_Radio072_Skill".into(),
            damage_effect_id: "GE_Player_Radio072_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 12_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 15_000_000,
            actor_index: 0,
            ability_id: "GA_Radio072_Skill".into(),
            damage_effect_id: "GE_Player_Radio072_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 16_000_000;

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
    assert_eq!(
        damages_a6,
        vec![480, 811, 864, 119, 206, 385, 720, 99, 171, 321]
    );
    let hits = report_a6["hit_receipts"].as_array().unwrap();
    for hit in &hits[2..6] {
        assert_eq!(hit["crit_rate"], json!("1/4"));
    }
    for hit in &hits[6..10] {
        assert_eq!(hit["crit_rate"], json!("1/4"));
    }
    let skill_child_damage = [
        json!("14904/125"),
        json!("25704/125"),
        json!("48168/125"),
        json!("2484/25"),
        json!("4284/25"),
        json!("8028/25"),
    ];
    for (hit, expected) in hits[3..6]
        .iter()
        .chain(&hits[7..10])
        .zip(skill_child_damage)
    {
        assert_eq!(hit["non_crit_damage"], expected);
    }
    let hit0 = &report_a6["hit_receipts"][0];
    assert_eq!(hit0["crit_rate"], "0/1");
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
        vec![400, 400, 400, 55, 95, 178, 400, 55, 95, 178]
    );
}
