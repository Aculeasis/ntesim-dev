use super::*;

#[test]
fn zero_kit_execution_attacks_skills_a1_a2_energy_grant() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1046".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].initial_energy = 50;
    req.actors[0].energy_capacity = 100;
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
            ability_id: "GA_Female046_Melee".into(),
            damage_effect_id: "GE_Player_Female046_Melee1_Damage".into(),
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
            ability_id: "GA_Female046_Skill".into(),
            damage_effect_id: "GE_Player_Female046_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 3_000_000;

    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(report["invariants_passed"], true);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 6);

    let actor0 = &report["final_actor_states"][0];
    assert_eq!(actor0["energy_conserved"], true);
    assert_eq!(actor0["initial_energy"], 50);
    // 8 by A2 cast grant + 9 by ordinary hit grants: Melee1 (1151/1000 -> 1),
    // Skill1 (587/1000 + 151/1000 carry -> 0), Skill2 (+587/1000 -> 1),
    // Skill3 (+3683/1000 -> 4), Skill4 (+3944/1000 -> 3).
    assert_eq!(actor0["granted_energy"], 17);
    assert_eq!(actor0["spent_energy"], 0);
    assert_eq!(actor0["energy"], 67);

    // Damage check:
    // Melee: 1000 ATK * 1.0 * 0.5 def * 0.8 res = 400
    // Skill packets: 1000 ATK * [0.2, 0.2, 1.256, 1.344] * 0.5 def * 0.8 res
    // A1 extra hit: 1000 ATK * 2.0 * 0.8 def (75% ignore) * 0.8 res = 1280
    let damages: Vec<u64> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    assert_eq!(damages, vec![400, 400, 80, 502, 538, 1280]);

    let a2_event = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_kind"] == "zero_a2_energy_grant")
        .expect("zero_a2_energy_grant event should be present");
    assert_eq!(a2_event["to_actor"], 0);
    assert_eq!(a2_event["energy_change"]["gross"], 8);
}

#[test]
fn zero_a3_crit_rate_a4_damage_bonus_and_a5_self_atk_buff() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1046".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();

    let mut mint = req.actors[0].clone();
    mint.character_id = "character:1019".into();
    mint.awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors.push(mint);

    req.action_metadata.clear();
    // This mixed research fixture intentionally supplies caller atk_rate=1 for each primary;
    // its Burst count/resource checks do not assert the native primary talent projection.
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Female046_QTE".into(),
            damage_effect_id: "GE_Player_Female046_QTE1_Damage".into(),
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
            ability_id: "GA_Female046_UltraSkill".into(),
            damage_effect_id: "GE_Player_Female046_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 800_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_500_000,
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
    req.relative_duration_us = 4_000_000;

    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], true);

    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 11);

    assert_eq!(
        hits.iter()
            .map(|hit| hit["damage_effect_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "GE_Player_Female046_QTE1_Damage",
            "GE_Player_Female046_QTE2_Damage",
            "GE_Player_Female046_UltraSkill1_Damage",
            "GE_Player_Female046_UltraSkill2_Damage",
            "GE_Player_Female046_UltraSkill2_Damage",
            "GE_Player_Female046_UltraSkill2_Damage",
            "GE_Player_Female046_UltraSkill2_Damage",
            "GE_Player_Female046_UltraSkill3_Damage",
            "GE_Player_Female046_UltraSkill4_Damage",
            "GE_Player_Mint_Melee1_Damage_New",
            "GE_Player_Mint_Melee2_Damage_New",
        ]
    );
    assert_eq!(
        hits.iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![440, 549, 550, 29, 29, 29, 29, 976, 927, 400, 92]
    );
    assert!(
        hits[2..9]
            .iter()
            .all(|hit| { hit["crit_rate"] == serde_json::json!("1/2") })
    );
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);

    let a5_event = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_kind"] == "zero_a5_atk_buff")
        .expect("zero_a5_atk_buff event should be present");
    assert_eq!(a5_event["to_actor"], 0);
    assert_eq!(a5_event["data"]["atk_percent"], "1/10");
}

#[test]
fn zero_a6_extra_hit_ratio_upgrade() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].character_id = "character:1046".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();

    req.action_metadata.clear();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Female046_Skill".into(),
        damage_effect_id: "GE_Player_Female046_Skill1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 500_000,
        cooldown_us: 6_000_000,
        hit_delay_us: 100_000,
    }];
    req.relative_duration_us = 2_000_000;

    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["success"], true);

    let damages: Vec<u64> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["rounded_damage"].as_u64().unwrap())
        .collect();
    // Hit 1 (Skill main): 400 dmg
    // Hit 2 (A6 extra hit): 1000 ATK * 3.0 * 0.8 def (75% ignore) * 0.8 res = 1920
    assert_eq!(damages, vec![400, 80, 502, 538, 1920]);
}
