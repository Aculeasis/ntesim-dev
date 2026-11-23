use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;

#[test]
fn jiuyuan_selected_awakenings_separate_effects_from_resonance() {
    // Native Lv1 Skill1 = .36, Lv2 = .389. Only A1 changes ATK;
    // only A5 adds .08 damage. Three selections raise the skill coordinate.
    for (selection, numerator, denominator) in [
        (vec![5], 972, 5),
        (vec![6], 180, 1),
        (vec![2, 3, 4], 389, 2),
        (vec![1, 4, 5], 220563, 1000),
    ] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(selection).unwrap();
        req.actions = vec![make_action(
            0,
            0,
            "GA_Kuhara_Skill",
            "GE_Player_Kuhara_Skill1_Damage",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 6);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", numerator, denominator))
        );
    }
}

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
    req.actors[0].character_id = "character:1055".into();
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
        "GA_Kuhara_Melee",
        "GE_Player_Kuhara_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn jiuyuan_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Kuhara_Melee / GE_Player_Kuhara_Melee1_Damage)
    // Coeffs: Lv1 = 0.218, Lv8 = 0.374, Lv9 (A3 resonance) = 0.404
    // Does not apply Pact; ATK remains 1000.
    // Lv1: 1000 * 0.218 * 0.5 = 109 / 1
    // Lv8: 1000 * 0.374 * 0.5 = 187 / 1
    // Lv9 (A3): 1000 * 0.404 * 0.5 = 202 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 109, 1),
        (8, 0, 187, 1),
        (8, 1, 187, 1),
        (8, 3, 202, 1),
        (8, 4, 202, 1),
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

    // Skill (GA_Kuhara_Skill / GE_Player_Kuhara_Skill1_Damage)
    // Delivers 6 hits: 1x GE_Player_Kuhara_Skill1_Damage, 4x GE_Player_Kuhara_Skill2_Damage, 1x GE_Player_Kuhara_Skill3_Damage
    // Coeffs:
    //   Skill1: Lv1 = 0.360, Lv8 = 0.617, Lv9 = 0.666
    //   Skill2: Lv1 = 0.382, Lv8 = 0.655, Lv9 = 0.707
    //   Skill3: Lv1 = 1.114, Lv8 = 1.909, Lv9 = 2.062
    // On hit, before_hit applies Rose Pact.
    // At A0, A1 is inactive -> ATK = 1000.
    //   Lv1: Hit 0 = 180/1, Hits 1..4 = 191/1, Hit 5 = 557/1. Total = 1501/1
    //   Lv8: Hit 0 = 617/2, Hits 1..4 = 655/2, Hit 5 = 1909/2. Total = 2573/1
    // At A3, A1 (+5% ATK) is active -> ATK = 1050.
    //   Lv9 (A3): Hit 0 = 6993/20, Hits 1..4 = 14847/40, Hit 5 = 21651/20. Total = 29169/10
    for (level, awakening, hit0_num, hit0_den, total_num, total_den) in [
        (1, 0, 180, 1, 1501, 1),
        (8, 0, 617, 2, 2573, 1),
        (8, 3, 6993, 20, 29169, 10),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Kuhara_Skill",
            "GE_Player_Kuhara_Skill1_Damage",
            0,
            16_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 6);
        assert_eq!(
            report["hit_receipts"][0]["damage_effect_id"],
            "GE_Player_Kuhara_Skill1_Damage"
        );
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", hit0_num, hit0_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        for i in 1..=4 {
            assert_eq!(
                report["hit_receipts"][i]["damage_effect_id"],
                "GE_Player_Kuhara_Skill2_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
        }
        assert_eq!(
            report["hit_receipts"][5]["damage_effect_id"],
            "GE_Player_Kuhara_Skill3_Damage"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][5])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );

        let sum: ExactFraction = report["hit_receipts"]
            .as_array()
            .unwrap()
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

    // Burst (GA_Kuhara_UltraSkill / GE_Player_Kuhara_UltraSkill1_Damage)
    // Delivers 12 hits:
    //   1x GE_Player_Kuhara_UltraSkill1_Damage
    //   3x GE_Player_Kuhara_UltraSkill2_Damage
    //   7x GE_Player_Kuhara_UltraSkill3_Damage
    //   1x GE_Player_Kuhara_UltraSkill4_Damage
    // Coeffs:
    //   UltraSkill1: Lv1 = 0.618, Lv8 = 1.059, Lv9 = 1.144
    //   UltraSkill2: Lv1 = 0.784, Lv8 = 1.344, Lv9 = 1.451
    //   UltraSkill3: Lv1 = 0.244, Lv8 = 0.418, Lv9 = 0.452
    //   UltraSkill4: Lv1 = 1.321, Lv8 = 2.264, Lv9 = 2.445
    // On hit, before_hit applies Rose Pact.
    // At A0, A1 is inactive -> ATK = 1000.
    //   Lv1: Hit 0 = 309/1. Total = 5999/2
    //   Lv8: Hit 0 = 1059/2. Total = 10281/2
    // At A3, A1 (+5% ATK) is active -> ATK = 1050.
    //   Lv9 (A3): Hit 0 = 3003/5. Total = 116613/20
    for (level, awakening, hit0_num, hit0_den, total_num, total_den) in [
        (1, 0, 309, 1, 5999, 2),
        (8, 0, 1059, 2, 10281, 2),
        (8, 3, 3003, 5, 116613, 20),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].initial_energy = 100;
        req.actors[0].energy_capacity = 100;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Kuhara_UltraSkill",
            "GE_Player_Kuhara_UltraSkill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 12);
        assert_eq!(
            report["hit_receipts"][0]["damage_effect_id"],
            "GE_Player_Kuhara_UltraSkill1_Damage"
        );
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", hit0_num, hit0_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        for i in 1..=3 {
            assert_eq!(
                report["hit_receipts"][i]["damage_effect_id"],
                "GE_Player_Kuhara_UltraSkill2_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        for i in 4..=10 {
            assert_eq!(
                report["hit_receipts"][i]["damage_effect_id"],
                "GE_Player_Kuhara_UltraSkill3_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        assert_eq!(
            report["hit_receipts"][11]["damage_effect_id"],
            "GE_Player_Kuhara_UltraSkill4_Damage"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][11])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );

        let sum: ExactFraction = report["hit_receipts"]
            .as_array()
            .unwrap()
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
fn jiuyuan_actions_respect_off_field_delivery_and_dead_target_boundaries() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Kuhara_Skill",
            "GE_Player_Kuhara_Skill1_Damage",
            0,
            6_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 10,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 6);
    for receipt in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(receipt["actor_index"], 0);
    }

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);

    let mut req_horizon = request();
    req_horizon.relative_duration_us = 50;
    let report_horizon = run_game_scenario(&req_horizon, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_horizon["execution_completed"], true);
    assert_eq!(report_horizon["hits_settled"], 0);
}

#[test]
fn jiuyuan_strict_mode_rejects_missing_hypothesis_and_accepts_valid_hypothesis() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::JIUYUAN_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::JIUYUAN_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Kuhara_Skill",
        "GE_Player_Kuhara_Skill1_Damage",
        0,
        16_000_000,
    )];
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::HypothesisId::CombatPacketCycleAccrualV1.into(),
        crate::HypothesisId::CombatPacketCycleAccrualV1
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::jiuyuan::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::jiuyuan::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 6);
}

#[test]
fn jiuyuan_two_skill_one_burst_rotation_hit_count_and_energy() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.relative_duration_us = 35_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Kuhara_Skill",
            "GE_Player_Kuhara_Skill1_Damage",
            0,
            16_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Kuhara_UltraSkill",
            "GE_Player_Kuhara_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
        make_action(
            21_599_441,
            0,
            "GA_Kuhara_Skill",
            "GE_Player_Kuhara_Skill1_Damage",
            0,
            16_000_000,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 24);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 7);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 14);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 7);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("1/5")
    );
}

#[test]
fn jiuyuan_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Kuhara_Melee1_Damage", 109, 1),
                ("GE_Player_Kuhara_Melee2_Damage", 241, 2),
                ("GE_Player_Kuhara_Melee2_1_Damage", 93, 2),
                ("GE_Player_Kuhara_Melee3_Damage", 335, 2),
                ("GE_Player_Kuhara_Melee3_1_Damage", 51, 2),
                ("GE_Player_Kuhara_Melee3_2_Damage", 259, 2),
                ("GE_Player_Kuhara_Melee4_Damage", 63, 1),
                ("GE_Player_Kuhara_Melee4_1_Damage", 245, 2),
                ("GE_Player_Kuhara_Melee4_2_Damage", 175, 2),
                ("GE_Player_Kuhara_Melee4_3_Damage", 51, 1),
                ("GE_Player_Kuhara_Melee5_Damage", 380, 1),
                ("GE_Player_Kuhara_Melee1_Damage", 109, 1),
            ],
            2823,
            2,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Kuhara_Melee1_Damage", 187, 1),
                ("GE_Player_Kuhara_Melee2_Damage", 413, 2),
                ("GE_Player_Kuhara_Melee2_1_Damage", 159, 2),
                ("GE_Player_Kuhara_Melee3_Damage", 287, 1),
                ("GE_Player_Kuhara_Melee3_1_Damage", 87, 2),
                ("GE_Player_Kuhara_Melee3_2_Damage", 222, 1),
                ("GE_Player_Kuhara_Melee4_Damage", 108, 1),
                ("GE_Player_Kuhara_Melee4_1_Damage", 210, 1),
                ("GE_Player_Kuhara_Melee4_2_Damage", 150, 1),
                ("GE_Player_Kuhara_Melee4_3_Damage", 175, 2),
                ("GE_Player_Kuhara_Melee5_Damage", 1303, 2),
                ("GE_Player_Kuhara_Melee1_Damage", 187, 1),
            ],
            4839,
            2,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Kuhara_Melee1_Damage", 202, 1),
                ("GE_Player_Kuhara_Melee2_Damage", 223, 1),
                ("GE_Player_Kuhara_Melee2_1_Damage", 86, 1),
                ("GE_Player_Kuhara_Melee3_Damage", 310, 1),
                ("GE_Player_Kuhara_Melee3_1_Damage", 47, 1),
                ("GE_Player_Kuhara_Melee3_2_Damage", 479, 2),
                ("GE_Player_Kuhara_Melee4_Damage", 233, 2),
                ("GE_Player_Kuhara_Melee4_1_Damage", 453, 2),
                ("GE_Player_Kuhara_Melee4_2_Damage", 162, 1),
                ("GE_Player_Kuhara_Melee4_3_Damage", 189, 2),
                ("GE_Player_Kuhara_Melee5_Damage", 29547, 40),
                ("GE_Player_Kuhara_Melee1_Damage", 2121, 10),
            ],
            106311,
            40,
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
                    "GA_Kuhara_Melee",
                    "GE_Player_Kuhara_Melee1_Damage",
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
        assert_eq!(report["hits_settled"], 12);
        for (hit, (packet, num, den)) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected_hits)
        {
            assert_eq!(hit["damage_effect_id"], packet);
            assert_eq!(hit["non_crit_damage"], json!(format!("{}/{}", num, den)));
        }
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            serde_json::to_value(ExactFraction::new(total_num, total_den).unwrap()).unwrap()
        );
    }
}

#[test]
fn jiuyuan_normal_chain_reset_and_noop_boundaries() {
    for boundary in 0..5 {
        let mut req = request();
        req.relative_duration_us = 4_000_000;
        let timestamp = match boundary {
            0 => 2_000_001,
            1 => 2_000_000,
            _ => 3000,
        };
        match boundary {
            2 => req.actions.push(make_action(
                1000,
                0,
                "GA_Kuhara_Skill",
                "GE_Player_Kuhara_Skill1_Damage",
                0,
                0,
            )),
            3 => {
                req.actors.push(req.actors[0].clone());
                req.actions.push(GameActionCommand::SwitchActor {
                    timestamp_us: 1000,
                    target_actor_index: 1,
                    cooldown_us: 0,
                });
                req.actions.push(GameActionCommand::SwitchActor {
                    timestamp_us: 2000,
                    target_actor_index: 0,
                    cooldown_us: 0,
                });
            }
            4 => req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: 1000,
                target_actor_index: 0,
                cooldown_us: 0,
            }),
            _ => {}
        }
        req.actions.push(make_action(
            timestamp,
            0,
            "GA_Kuhara_Melee",
            "GE_Player_Kuhara_Melee1_Damage",
            0,
            0,
        ));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let expected = if boundary == 1 || boundary == 4 {
            "GE_Player_Kuhara_Melee2_1_Damage"
        } else {
            "GE_Player_Kuhara_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn jiuyuan_normal_chain_strict_hypothesis_and_off_field_two_hit_delivery() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::JIUYUAN_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::JIUYUAN_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::jiuyuan::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::jiuyuan::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::HypothesisId::CombatPacketCycleAccrualV1.into(),
        crate::HypothesisId::CombatPacketCycleAccrualV1
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = (0..2)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Kuhara_Melee",
                "GE_Player_Kuhara_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1050,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.relative_duration_us = 1099;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["hits_settled"], 1);
    req.relative_duration_us = 1100;
    let settled = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(settled["hits_settled"], 3);
}

#[test]
fn jiuyuan_multi_target_skill_and_burst_delivery() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Kuhara_Skill",
            "GE_Player_Kuhara_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Kuhara_UltraSkill",
            "GE_Player_Kuhara_UltraSkill1_Damage",
            0,
            20_000_000,
        ),
    ];
    req.relative_duration_us = 3_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    // Skill: 6 hits * 2 targets = 12 hits
    // Burst: 12 hits * 2 targets = 24 hits
    // Total = 36 hits settled
    assert_eq!(report["hits_settled"], 36);

    let hits = report["hit_receipts"].as_array().unwrap();
    let skill_hits = hits
        .iter()
        .filter(|h| h["ability_id"] == "GA_Kuhara_Skill")
        .collect::<Vec<_>>();
    assert_eq!(skill_hits.len(), 12);
    assert_eq!(
        skill_hits.iter().filter(|h| h["target_index"] == 0).count(),
        6
    );
    assert_eq!(
        skill_hits.iter().filter(|h| h["target_index"] == 1).count(),
        6
    );

    let burst_hits = hits
        .iter()
        .filter(|h| h["ability_id"] == "GA_Kuhara_UltraSkill")
        .collect::<Vec<_>>();
    assert_eq!(burst_hits.len(), 24);
    assert_eq!(
        burst_hits.iter().filter(|h| h["target_index"] == 0).count(),
        12
    );
    assert_eq!(
        burst_hits.iter().filter(|h| h["target_index"] == 1).count(),
        12
    );

    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::jiuyuan::skill::ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::characters::jiuyuan::skill::ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
}
