use super::*;
use serde_json::json;

fn make_action(
    timestamp_us: u64,
    actor_index: usize,
    ability_id: &str,
    damage_effect_id: &str,
    energy_cost: u64,
    cooldown_us: u64,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: ability_id.into(),
        damage_effect_id: damage_effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost,
        recovery_us: 0,
        cooldown_us,
        hit_delay_us: 100,
    }
}

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1046".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Female046_Melee",
        "GE_Player_Female046_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn zero_a2_typed_energy_preserves_efficiency_carry_and_overflow_across_casts() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    req.actors[0].initial_energy = 98;
    req.actors[0]
        .gear_procs
        .push(GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator: 7,
                denominator: 5,
            },
        });
    req.actions = [0, 6_000_000]
        .into_iter()
        .map(|time| {
            make_action(
                time,
                0,
                "GA_Female046_Skill",
                "GE_Player_Female046_Skill1_Damage",
                0,
                6_000_000,
            )
        })
        .collect();
    req.relative_duration_us = 6_000_100;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 8);
    let a2: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "zero_a2_energy_grant")
        .collect();
    assert_eq!(a2.len(), 2);
    assert_eq!(a2[0]["energy_change"]["gross"], 11);
    assert_eq!(a2[1]["energy_change"]["gross"], 11);
    // Per cast: (8 A2 + .587 + .587 + 3.683 + 3.944 hit charge) * 1.4.
    // Two casts generate 47.0428, retaining 2 whole units above initial 98;
    // the remaining 45 overflow and 107/2500 carry survive the second cast.
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["granted_energy"], 47);
    assert_eq!(actor["overflow_energy"], 45);
    assert_eq!(actor["energy"], 100);
    assert_eq!(actor["energy_conserved"], true);
    assert_eq!(actor["charge_remainder"], json!("107/2500"));
}

#[test]
fn zero_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Test Normal Attack (GA_Female046_Melee / GE_Player_Female046_Melee1_Damage)
    // Coeffs: Lv1 = 0.23, Lv8 = 0.394, Lv9 (A3 resonance) = 0.426
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 0.23 * 0.5 = 115 / 1
    // Lv8: 1000 * 0.394 * 0.5 = 197 / 1
    // Lv9: 1000 * 0.426 * 0.5 = 213 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 115, 1),
        (8, 0, 197, 1),
        (8, 1, 197, 1),
        (8, 3, 213, 1),
        (8, 4, 213, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
    }

    // Test Skill (GA_Female046_Skill / GE_Player_Female046_Skill1_Damage)
    // Coeffs: Lv1 = 0.2, Lv8 = 0.343, Lv9 (A3 resonance) = 0.37
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 0.2 * 0.5 = 100 / 1
    // Lv8: 1000 * 0.343 * 0.5 = 343 / 2
    // Lv9: 1000 * 0.37 * 0.5 = 185 / 1
    // At A0, no A1 extra hit triggers:
    for (level, awakening, target_level, expected_num, expected_den) in [
        (1, 0, 80, 100, 1),
        (8, 0, 80, 343, 2),
        (8, 1, 81, 343, 2), // target > actor level: A1 does not fire
        (8, 3, 81, 185, 1), // resonance_3 (+1 lvl = 9), target > actor level: A1 does not fire
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.targets[0].level = target_level;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Female046_Skill",
            "GE_Player_Female046_Skill1_Damage",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 4);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|hit| hit["damage_effect_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec![
                "GE_Player_Female046_Skill1_Damage",
                "GE_Player_Female046_Skill2_Damage",
                "GE_Player_Female046_Skill3_Damage",
                "GE_Player_Female046_Skill4_Damage",
            ]
        );
    }

    {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
        req.targets[0].level = 80;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Female046_Skill",
            "GE_Player_Female046_Skill1_Damage",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], 5);
        assert_eq!(report["hit_receipts"][0]["non_crit_damage"], json!("100/1"));
        // A1 hit ignores 75% DEF: 1080 / (1080 + 1080 * 0.25) = 1080 / 1350 = 4/5. 200% * 1000 * 4/5 = 1600.
        assert_eq!(
            report["hit_receipts"][4]["non_crit_damage"],
            json!("1600/1")
        );
    }

    // Test Ultimate/Burst (GA_Female046_UltraSkill / GE_Player_Female046_UltraSkill1_Damage)
    // Coeffs: Lv1 = 1.327, Lv8 = 2.274, Lv9 (A3 resonance) = 2.456
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 1.327 * 0.5 = 1327 / 2
    // Lv8: 1000 * 2.274 * 0.5 = 1137 / 1
    // Lv9: 1000 * 2.456 * 0.5 = 1228 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 1327, 2),
        (8, 0, 1137, 1),
        (8, 1, 1137, 1),
        (8, 3, 1228, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Female046_UltraSkill",
            "GE_Player_Female046_UltraSkill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 7);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
    }
}

#[test]
fn zero_skill_train_matches_all_four_native_packets_and_resonance_projection() {
    for (level, awakening, target_level, expected) in [
        (
            1,
            0,
            80,
            vec![
                ("GE_Player_Female046_Skill1_Damage", 100, 1),
                ("GE_Player_Female046_Skill2_Damage", 100, 1),
                ("GE_Player_Female046_Skill3_Damage", 628, 1),
                ("GE_Player_Female046_Skill4_Damage", 672, 1),
            ],
        ),
        (
            8,
            0,
            80,
            vec![
                ("GE_Player_Female046_Skill1_Damage", 343, 2),
                ("GE_Player_Female046_Skill2_Damage", 343, 2),
                ("GE_Player_Female046_Skill3_Damage", 2153, 2),
                ("GE_Player_Female046_Skill4_Damage", 2303, 2),
            ],
        ),
        (
            8,
            3,
            81,
            vec![
                ("GE_Player_Female046_Skill1_Damage", 185, 1),
                ("GE_Player_Female046_Skill2_Damage", 185, 1),
                ("GE_Player_Female046_Skill3_Damage", 2325, 2),
                ("GE_Player_Female046_Skill4_Damage", 1244, 1),
            ],
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.targets[0].level = target_level;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Female046_Skill",
            "GE_Player_Female046_Skill1_Damage",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 4);
        for (receipt, (packet, numerator, denominator)) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected)
        {
            assert_eq!(receipt["damage_effect_id"], packet);
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", numerator, denominator))
            );
            assert_eq!(receipt["time_us"], 100);
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
        }
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::characters::zero::skill::SKILL_PACKETS_HYPOTHESIS_ID.as_str()],
            crate::characters::zero::skill::SKILL_PACKETS_HYPOTHESIS_ID.description()
        );
    }
}

#[test]
fn zero_off_field_delivery_and_dead_target_boundary() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors[0].talent_levels = Some([1, 1, 1]);

    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 50,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    req.relative_duration_us = 200;

    let off_field = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(off_field["hits_settled"], 1);
    assert_eq!(off_field["hit_receipts"][0]["actor_index"], 0);
    assert_eq!(
        off_field["hit_receipts"][0]["non_crit_damage"],
        json!("115/1")
    );

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead["hits_settled"], 0);

    let mut req_pending = request();
    req_pending.actions = vec![make_action(
        0,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        6_000_000,
    )];
    req_pending.relative_duration_us = 99;
    let pending = run_game_scenario(&req_pending, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["execution_completed"], true);
    assert_eq!(pending["hits_settled"], 0);
    assert_eq!(pending["pending_events"], 4);
    req_pending.relative_duration_us = 100;
    let settled = run_game_scenario(&req_pending, &ExecutionLimits::default()).unwrap();
    assert_eq!(settled["execution_completed"], true);
    assert_eq!(settled["hits_settled"], 4);

    let mut req_off_field = request();
    let mut ally = req_off_field.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req_off_field.actors.push(ally);
    req_off_field.actions = vec![
        make_action(
            0,
            0,
            "GA_Female046_Skill",
            "GE_Player_Female046_Skill1_Damage",
            0,
            6_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 50,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let off_field = run_game_scenario(&req_off_field, &ExecutionLimits::default()).unwrap();
    assert_eq!(off_field["hits_settled"], 9);
    assert!(
        off_field["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|receipt| receipt["damage_effect_id"] != "ntesim.reaction.blossom")
            .all(|receipt| {
                receipt["actor_index"] == 0
                    && receipt["ability_id"] == "GA_Female046_Skill"
                    && receipt["target_index"] == 0
            })
    );

    let mut req_dead_skill = request();
    req_dead_skill.targets[0].hp_current = 0;
    req_dead_skill.actions = req_pending.actions.clone();
    let dead_skill = run_game_scenario(&req_dead_skill, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead_skill["execution_completed"], true);
    assert_eq!(dead_skill["hits_settled"], 0);
}

#[test]
fn zero_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Female046_Melee1_Damage", 115, 1),
                ("GE_Player_Female046_Melee2_1_Damage", 112, 1),
                ("GE_Player_Female046_Melee2_Damage", 259, 2),
                ("GE_Player_Female046_Melee3_Damage", 379, 2),
                ("GE_Player_Female046_Melee4_Damage", 32, 1),
                ("GE_Player_Female046_Melee5_Damage", 385, 2),
                ("GE_Player_Female046_Melee6_Damage", 77, 1),
                ("GE_Player_Female046_Melee7_Damage", 997, 2),
                ("GE_Player_Female046_Melee8_Damage", 683, 2),
                ("GE_Player_Female046_Melee8_Damage", 683, 2),
                ("GE_Player_Female046_Melee1_Damage", 115, 1),
            ],
            2144,
            1,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Female046_Melee1_Damage", 197, 1),
                ("GE_Player_Female046_Melee2_1_Damage", 192, 1),
                ("GE_Player_Female046_Melee2_Damage", 222, 1),
                ("GE_Player_Female046_Melee3_Damage", 325, 1),
                ("GE_Player_Female046_Melee4_Damage", 55, 1),
                ("GE_Player_Female046_Melee5_Damage", 330, 1),
                ("GE_Player_Female046_Melee6_Damage", 132, 1),
                ("GE_Player_Female046_Melee7_Damage", 1709, 2),
                ("GE_Player_Female046_Melee8_Damage", 1171, 2),
                ("GE_Player_Female046_Melee8_Damage", 1171, 2),
                ("GE_Player_Female046_Melee1_Damage", 197, 1),
            ],
            7351,
            2,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Female046_Melee1_Damage", 213, 1),
                ("GE_Player_Female046_Melee2_1_Damage", 415, 2),
                ("GE_Player_Female046_Melee2_Damage", 479, 2),
                ("GE_Player_Female046_Melee3_Damage", 351, 1),
                ("GE_Player_Female046_Melee4_Damage", 59, 1),
                ("GE_Player_Female046_Melee5_Damage", 713, 2),
                ("GE_Player_Female046_Melee6_Damage", 285, 2),
                ("GE_Player_Female046_Melee7_Damage", 1845, 2),
                ("GE_Player_Female046_Melee8_Damage", 632, 1),
                ("GE_Player_Female046_Melee8_Damage", 632, 1),
                ("GE_Player_Female046_Melee1_Damage", 213, 1),
            ],
            7937,
            2,
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = (0..6)
            .map(|index| {
                make_action(
                    index * 300_000,
                    0,
                    "GA_Female046_Melee",
                    "GE_Player_Female046_Melee1_Damage",
                    0,
                    0,
                )
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 11);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 11);
        for (i, (expected_packet, exp_num, exp_den)) in expected_hits.iter().enumerate() {
            assert_eq!(receipts[i]["damage_effect_id"], *expected_packet, "hit {i}");
            assert_eq!(
                receipts[i]["non_crit_damage"],
                json!(format!("{}/{}", exp_num, exp_den)),
                "hit {i}"
            );
            assert_eq!(
                super::support::hit_metadata(&report, &receipts[i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
            );
        }
        let sum: ExactFraction = receipts
            .iter()
            .try_fold(ExactFraction::new(0, 1).unwrap(), |acc, r| {
                acc.add(
                    serde_json::from_value::<ExactFraction>(r["non_crit_damage"].clone())
                        .map_err(|err| err.to_string())?,
                )
            })
            .unwrap();
        assert_eq!(
            sum,
            ExactFraction::new(total_num as u128, total_den as u128).unwrap()
        );
    }
}

#[test]
fn zero_normal_chain_reset_boundaries_and_noop_switch() {
    let mut req = request();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_001,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Female046_Melee1_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Female046_Melee1_Damage"
    );

    let mut req_cont = request();
    req_cont.actions = vec![
        make_action(
            0,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_cont = run_game_scenario(&req_cont, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_cont["hits_settled"], 3);
    let receipts_cont = report_cont["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_cont[0]["damage_effect_id"],
        "GE_Player_Female046_Melee1_Damage"
    );
    assert_eq!(
        receipts_cont[1]["damage_effect_id"],
        "GE_Player_Female046_Melee2_1_Damage"
    );
    assert_eq!(
        receipts_cont[2]["damage_effect_id"],
        "GE_Player_Female046_Melee2_Damage"
    );

    let mut req_skill_reset = request();
    req_skill_reset.actions = vec![
        make_action(
            0,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            500_000,
            0,
            "GA_Female046_Skill",
            "GE_Player_Female046_Skill1_Damage",
            0,
            0,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_skill = run_game_scenario(&req_skill_reset, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_skill["hits_settled"], 6);
    let receipts_skill = report_skill["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_skill[0]["damage_effect_id"],
        "GE_Player_Female046_Melee1_Damage"
    );
    assert_eq!(
        receipts_skill[1]["damage_effect_id"],
        "GE_Player_Female046_Skill1_Damage"
    );
    assert_eq!(
        receipts_skill[2]["damage_effect_id"],
        "GE_Player_Female046_Skill2_Damage"
    );
    assert_eq!(
        receipts_skill[3]["damage_effect_id"],
        "GE_Player_Female046_Skill3_Damage"
    );
    assert_eq!(
        receipts_skill[4]["damage_effect_id"],
        "GE_Player_Female046_Skill4_Damage"
    );
    assert_eq!(
        receipts_skill[5]["damage_effect_id"],
        "GE_Player_Female046_Melee1_Damage"
    );

    let mut req_noop = request();
    req_noop.actions = vec![
        make_action(
            0,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 100_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            300_000,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_noop = run_game_scenario(&req_noop, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_noop["hits_settled"], 3);
    let receipts_noop = report_noop["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_noop[0]["damage_effect_id"],
        "GE_Player_Female046_Melee1_Damage"
    );
    assert_eq!(
        receipts_noop[1]["damage_effect_id"],
        "GE_Player_Female046_Melee2_1_Damage"
    );
    assert_eq!(
        receipts_noop[2]["damage_effect_id"],
        "GE_Player_Female046_Melee2_Damage"
    );

    let mut req_exit = request();
    let mut ally = req_exit.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req_exit.actors.push(ally);
    req_exit.actions = vec![
        make_action(
            0,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 100_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 200_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            800_000,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_exit = run_game_scenario(&req_exit, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_exit["execution_completed"], true);
    assert_eq!(report_exit["hits_settled"], 2);
    let receipts_exit: Vec<_> = report_exit["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Female046_Melee")
        .collect();
    assert_eq!(receipts_exit.len(), 2);
    assert_eq!(
        receipts_exit[0]["damage_effect_id"],
        "GE_Player_Female046_Melee1_Damage"
    );
    assert_eq!(
        receipts_exit[1]["damage_effect_id"],
        "GE_Player_Female046_Melee1_Damage"
    );
}

#[test]
fn zero_normal_chain_strict_and_pending_final_stage() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        "owner.action_role.character_1046_melee_v1".into(),
        "Classify this exact Zero action as Melee because the frozen character action identity names its primary attack and the effect-config row is absent; owner role is an explicit model hypothesis.".into(),
    );
    req.model_assumptions.insert(
        crate::characters::zero::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::ZERO_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::ZERO_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);

    let mut req_exit = request();
    let mut ally = req_exit.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req_exit.actors.push(ally);
    req_exit.actions = vec![
        make_action(
            0,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            100_000,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            200_000,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
    ];
    req_exit.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 200_050,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req_exit.relative_duration_us = 200_150;
    let report_exit = run_game_scenario(&req_exit, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_exit["execution_completed"], true);
    assert_eq!(report_exit["hits_settled"], 6);
    let receipts = report_exit["hit_receipts"].as_array().unwrap();
    for receipt in &receipts[3..6] {
        assert_eq!(receipt["actor_index"], 0);
    }
    assert_eq!(
        receipts[3]["damage_effect_id"],
        "GE_Player_Female046_Melee3_Damage"
    );
    assert_eq!(
        receipts[4]["damage_effect_id"],
        "GE_Player_Female046_Melee4_Damage"
    );
    assert_eq!(
        receipts[5]["damage_effect_id"],
        "GE_Player_Female046_Melee5_Damage"
    );
}

#[test]
fn zero_awakenings_use_selected_effects_and_keep_resonance_distinct() {
    let mut a2_only = request();
    a2_only.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    a2_only.actors[0].initial_energy = 0;
    a2_only.actions = vec![make_action(
        0,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        6_000_000,
    )];
    let a2_report = run_game_scenario(&a2_only, &ExecutionLimits::default()).unwrap();
    assert_eq!(a2_report["hits_settled"], 4);
    let a2_event = a2_report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["event_kind"] == "zero_a2_energy_grant")
        .unwrap();
    assert_eq!(a2_event["energy_change"]["gross"], 8);

    let mut a1_a6 = request();
    a1_a6.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 6]).unwrap();
    a1_a6.actions = vec![make_action(
        0,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        6_000_000,
    )];
    let a1_a6_report = run_game_scenario(&a1_a6, &ExecutionLimits::default()).unwrap();
    assert_eq!(a1_a6_report["hits_settled"], 5);
    assert_eq!(
        a1_a6_report["hit_receipts"][4]["damage_effect_id"],
        "GE_Player_Female046_Skill_Kill_Damage_lv2"
    );
    assert_eq!(
        a1_a6_report["hit_receipts"][4]["non_crit_damage"],
        json!("2400/1")
    );

    let make_burst = |effects: Vec<u8>| {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, 8]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.actors[0].static_stats.crit_rate = Ratio::zero();
        req.actions = vec![make_action(
            0,
            0,
            "GA_Female046_UltraSkill",
            "GE_Player_Female046_UltraSkill1_Damage",
            100,
            20_000_000,
        )];
        req
    };
    let a3_without_resonance = make_burst(vec![3]);
    let a3_report = run_game_scenario(&a3_without_resonance, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        a3_report["hit_receipts"][0]["non_crit_damage"],
        json!("1137/1")
    );
    assert_eq!(a3_report["hit_receipts"][0]["crit_rate"], json!("1/2"));

    let resonance3_without_a3 = make_burst(vec![1, 2, 4]);
    let r3_report = run_game_scenario(&resonance3_without_a3, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        r3_report["hit_receipts"][0]["non_crit_damage"],
        json!("1535/1")
    );
    assert_eq!(r3_report["hit_receipts"][0]["crit_rate"], json!("0/1"));

    assert_eq!(
        crate::characters::zero::buff_duration(
            &ntesim_data::AwakeningSelection::selected([5]).unwrap(),
            "GA_Female046_QTE"
        ),
        Some(20_000_000)
    );
    assert_eq!(
        crate::characters::zero::buff_duration(
            &ntesim_data::AwakeningSelection::selected([1, 2, 4]).unwrap(),
            "GA_Female046_QTE"
        ),
        None
    );
}

#[test]
fn zero_burst_multi_target_aoe_dummy_strict_acceptance() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].talent_levels = None;
    let role =
        crate::characters::action_roles::resolve("character:1046", "GA_Female046_UltraSkill")
            .unwrap();
    req.model_assumptions
        .insert(role.hypothesis_id.into(), role.description.into());
    req.model_assumptions.insert(
        crate::simulation::energy::ZERO_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::ZERO_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::zero::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::BURST_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::zero::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::zero::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Female046_UltraSkill",
        "GE_Player_Female046_UltraSkill1_Damage",
        100,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 14);
}

#[test]
fn zero_burst_delivery_and_recipient_limits_under_dummy_model() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;

    // 3 targets: target 0 alive, target 1 dead, target 2 alive.
    let mut target1 = req.targets[0].clone();
    target1.target_index = 1;
    target1.hp_current = 0;
    req.targets.push(target1);

    let mut target2 = req.targets[0].clone();
    target2.target_index = 2;
    target2.hp_current = 100_000;
    req.targets.push(target2);

    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Female046_Skill",
            "GE_Player_Female046_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            3_000_000,
            0,
            "GA_Female046_UltraSkill",
            "GE_Player_Female046_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
    ];
    req.relative_duration_us = 4_000_000;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let receipts = report["hit_receipts"].as_array().unwrap();
    let melee_hits: Vec<_> = receipts
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Female046_Melee")
        .collect();
    assert_eq!(melee_hits.len(), 1);
    assert_eq!(melee_hits[0]["target_index"], 0);

    let skill_hits: Vec<_> = receipts
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Female046_Skill")
        .collect();
    assert_eq!(skill_hits.len(), 4);
    assert!(skill_hits.iter().all(|hit| hit["target_index"] == 0));

    let burst_hits: Vec<_> = receipts
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Female046_UltraSkill")
        .collect();
    assert_eq!(burst_hits.len(), 14);
    assert!(burst_hits.iter().all(|hit| hit["target_index"] != 1));

    let burst_targets: Vec<_> = burst_hits
        .iter()
        .map(|hit| hit["target_index"].as_u64().unwrap())
        .collect();
    assert_eq!(burst_targets, [vec![0; 7], vec![2; 7]].concat());
}

fn zero_continuing_cycle_actions() -> Vec<GameActionCommand> {
    let mut actions = Vec::new();
    // 0.0s: Switch in Zero (triggers QTE Support)
    actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 0,
        cooldown_us: 0,
    });

    let push_chain = |actions: &mut Vec<GameActionCommand>, start_us: u64| {
        for i in 0..5 {
            actions.push(make_action(
                start_us + i * 800_000,
                0,
                "GA_Female046_Melee",
                "GE_Player_Female046_Melee1_Damage",
                0,
                0,
            ));
        }
    };

    // Cycle 1 (0.0s - 60.0s)
    push_chain(&mut actions, 1_000_000);
    actions.push(make_action(
        5_000_000,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        12_000_000,
    ));
    push_chain(&mut actions, 7_000_000);
    push_chain(&mut actions, 11_000_000);
    actions.push(make_action(
        17_000_000,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        12_000_000,
    ));
    actions.push(make_action(
        19_000_000,
        0,
        "GA_Female046_UltraSkill",
        "GE_Player_Female046_UltraSkill1_Damage",
        100,
        20_000_000,
    ));
    push_chain(&mut actions, 22_000_000);
    actions.push(make_action(
        29_000_000,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        12_000_000,
    ));
    push_chain(&mut actions, 31_000_000);
    push_chain(&mut actions, 36_000_000);
    actions.push(make_action(
        41_000_000,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        12_000_000,
    ));
    push_chain(&mut actions, 45_000_000);

    // Cycle 2 (60.0s - 120.0s): Starts seamlessly from live state at 60s
    actions.push(make_action(
        60_000_000,
        0,
        "GA_Female046_UltraSkill",
        "GE_Player_Female046_UltraSkill1_Damage",
        100,
        20_000_000,
    ));
    actions.push(make_action(
        63_000_000,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        12_000_000,
    ));
    push_chain(&mut actions, 65_000_000);
    push_chain(&mut actions, 70_000_000);
    actions.push(make_action(
        77_000_000,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        12_000_000,
    ));
    push_chain(&mut actions, 80_000_000);
    push_chain(&mut actions, 85_000_000);
    actions.push(make_action(
        90_000_000,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        12_000_000,
    ));
    actions.push(make_action(
        92_000_000,
        0,
        "GA_Female046_UltraSkill",
        "GE_Player_Female046_UltraSkill1_Damage",
        100,
        20_000_000,
    ));
    push_chain(&mut actions, 95_000_000);
    push_chain(&mut actions, 100_000_000);
    actions.push(make_action(
        104_000_000,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        12_000_000,
    ));
    push_chain(&mut actions, 107_000_000);
    push_chain(&mut actions, 112_000_000);
    actions.push(make_action(
        117_000_000,
        0,
        "GA_Female046_Skill",
        "GE_Player_Female046_Skill1_Damage",
        0,
        12_000_000,
    ));

    // This independent fixture authors World offsets. Three explicitly modeled
    // two-second Zero cinematics project later admissions onto A.
    for action in &mut actions {
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = action {
            let world = *timestamp_us;
            *timestamp_us += [19_000_000, 60_000_000, 92_000_000]
                .iter()
                .filter(|&&cast| cast < world)
                .count() as u64
                * 2_000_000;
        }
    }
    actions
}

#[test]
fn zero_two_continuing_cycles_preserve_resources_passives_and_burst_delivery() {
    let mut req = request();
    req.actors[0].character_level = 80;
    req.actors[0].character_max_level = Some(80);
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 2, 3, 4, 5]).unwrap();
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 100;
    req.actors[0].static_stats.base_atk = 1188;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 27663,
        denominator: 10,
    };

    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    ally.initial_cycle = 100;
    req.actors.push(ally);

    req.targets[0].level = 80;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();

    // Start with Ally 1 active so Zero enters with Support QTE at t=0
    req.active_actor_index = 1;
    req.actions = zero_continuing_cycle_actions();
    req.relative_duration_us = 120_000_000;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["invariants_passed"], true);

    let zero_state = &report["final_actor_states"][0];
    assert_eq!(zero_state["character_id"], "character:1046");
    assert_eq!(zero_state["energy_conserved"], true);
    // 3 Bursts paid at 100 energy each = 300 energy spent
    assert_eq!(zero_state["spent_energy"], 300);
    // Live energy pool ended at 100 cap
    assert_eq!(zero_state["energy"], 100);

    let trace = report["trace"].as_array().unwrap();
    // QTE granted A5 ATK buff
    assert!(trace.iter().any(|e| e["event_kind"] == "zero_a5_atk_buff"));
    // Skills granted A2 energy refund (8 energy per cast)
    let a2_grants = trace
        .iter()
        .filter(|e| e["event_kind"] == "zero_a2_energy_grant")
        .count();
    assert_eq!(a2_grants, 9); // 4 skills in cycle 1 + 5 skills in cycle 2

    // Receipts: A1 execution hit occurred on skills
    let a1_hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Female046_Skill_Kill_Damage_lv1")
        .count();
    assert_eq!(a1_hits, 9);

    // Burst delivered 3 * 7 = 21 hits
    let burst_hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["ability_id"] == "GA_Female046_UltraSkill")
        .count();
    assert_eq!(burst_hits, 21);
}
