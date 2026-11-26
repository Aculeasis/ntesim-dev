use super::*;
use serde_json::json;

pub(super) fn make_action(
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

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1072".into();
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
        "GA_Radio072_Melee",
        "GE_Player_Radio072_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn linko_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Radio072_Melee / GE_Player_Radio072_Melee1_Damage)
    // Coeffs: Lv1 = 0.496, Lv8 = 0.850.
    // Resonance 3 modifies QTE, Skill, UltraSkill — but not Melee. Melee stays Lv8.
    // At A0: ATK = 1000. DEF factor = 0.5.
    //   Lv1: 1000 * 0.496 * 0.5 = 248 / 1
    //   Lv8: 1000 * 0.850 * 0.5 = 425 / 1
    // At A3: Resonance 3 adds +20% ATK -> ATK = 1200.
    //   Lv8: 1200 * 0.850 * 0.5 = 510 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 248, 1),
        (8, 0, 425, 1),
        (8, 1, 425, 1),
        (8, 3, 510, 1),
        (8, 4, 510, 1),
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

    for (
        level,
        awakening,
        expected_total_num,
        expected_total_den,
        expected_damage,
        expected_exact,
    ) in [
        (
            1,
            0,
            500,
            1,
            [89, 69, 119, 223],
            [(89, 1), (69, 1), (119, 1), (223, 1)],
        ),
        (
            8,
            0,
            857,
            1,
            [153, 119, 204, 382],
            [(305, 2), (237, 2), (204, 1), (382, 1)],
        ),
        (
            8,
            3,
            5553,
            5,
            [197, 153, 265, 496],
            [(987, 5), (153, 1), (1323, 5), (2478, 5)],
        ),
        (
            8,
            5,
            16659,
            10,
            [296, 230, 397, 743],
            [(2961, 10), (459, 2), (3969, 10), (3717, 5)],
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Radio072_Skill",
            "GE_Player_Radio072_Skill1_Damage",
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
        let effects = [
            "GE_Player_Radio072_Skill1_Damage",
            "GE_Player_Radio072_Skill2_Damage",
            "GE_Player_Radio072_Skill3_Damage",
            "GE_Player_Radio072_Skill4_Damage",
        ];
        let receipts = report["hit_receipts"].as_array().unwrap();
        let mut total = ExactFraction::new(0, 1).unwrap();
        for (index, receipt) in receipts.iter().enumerate() {
            assert_eq!(receipt["damage_effect_id"], effects[index]);
            assert_eq!(receipt["target_index"], 0);
            assert_eq!(receipt["rounded_damage"], expected_damage[index]);
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!(
                    "{}/{}",
                    expected_exact[index].0, expected_exact[index].1
                ))
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
            let damage = &receipt["non_crit_damage"];
            total = total
                .add(serde_json::from_value::<ExactFraction>(damage.clone()).unwrap())
                .unwrap();
        }
        assert_eq!(
            total,
            ExactFraction::new(expected_total_num, expected_total_den).unwrap()
        );
        assert!(report["resolved_model_assumptions"]["linko.skill_four_hits_v1"].is_string());
    }

    // Burst (GA_Radio072_UltraSkill / GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage)
    // Coeffs: Lv1 = 1.0, Lv8 = 1.714, Lv9 (A3 resonance) = 1.851
    // At A0: ATK = 1000.
    //   Lv1: 1000 * 1.0 * 0.5 = 500 / 1
    //   Lv8: 1000 * 1.714 * 0.5 = 857 / 1
    // At A2: A2 adds +30% ratio (3/10) to UltraSkill -> ratio = 1.714 + 0.3 = 2.014:
    //   Lv8: 1000 * 2.014 * 0.5 = 1007 / 1
    // At A3: ATK = 1200 (+20% ATK), Lv9 = 1.851, A2 ratio +30% -> ratio = 2.151:
    //   Lv9: 1200 * 2.151 * 0.5 = 6453 / 5
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 500, 1),
        (8, 0, 857, 1),
        (8, 2, 1007, 1),
        (8, 3, 6453, 5),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Radio072_UltraSkill",
            "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
            100,
            20_000_000,
        )];
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
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
    }
}

#[test]
fn linko_a6_crit_rate_and_resonance_field_buff() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Radio072_Skill",
        "GE_Player_Radio072_Skill1_Damage",
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
    for receipt in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(receipt["crit_rate"], json!("41/50"));
    }

    let mut burst_req = request();
    burst_req.actors[0].talent_levels = Some([1, 1, 8]);
    burst_req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    burst_req.actions = vec![make_action(
        0,
        0,
        "GA_Radio072_UltraSkill",
        "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
        100,
        20_000_000,
    )];
    let burst_report = run_game_scenario(&burst_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(burst_report["execution_completed"], true);
    assert_eq!(burst_report["hits_settled"], 1);
}

#[test]
fn linko_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Radio072_Skill",
            "GE_Player_Radio072_Skill1_Damage",
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
    assert_eq!(report["hits_settled"], 4);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["actor_index"] == 0)
    );

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);
}

#[test]
fn linko_skill_train_strict_primary_and_horizon_boundaries() {
    let mut req = request();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Radio072_Skill",
        "GE_Player_Radio072_Skill1_Damage",
        0,
        6_000_000,
    )];
    req.relative_duration_us = 99;
    let before_impact = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(before_impact["hits_settled"], 0);
    req.relative_duration_us = 100;
    let at_impact = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(at_impact["hits_settled"], 4);
    req.model_assumptions.insert(
        crate::simulation::energy::LINKO_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::LINKO_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::linko::skill::SKILL_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::linko::skill::SKILL_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let accepted = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(accepted["execution_completed"], true);
    assert_eq!(accepted["hits_settled"], 4);
}

#[test]
fn linko_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Radio072_Melee1_Damage", 248, 1),
                ("GE_Player_Radio072_Melee2_Damage", 85, 1),
                ("GE_Player_Radio072_Melee2_Damage", 85, 1),
                ("GE_Player_Radio072_Melee3_Damage", 202, 1),
                ("GE_Player_Radio072_Melee4_1_Damage", 133, 2),
                ("GE_Player_Radio072_Melee4_2_Damage", 149, 1),
                ("GE_Player_Radio072_Melee4_3_Damage", 139, 2),
                ("GE_Player_Radio072_Melee4_3_Damage", 139, 2),
                ("GE_Player_Radio072_Melee4_3_Damage", 139, 2),
                ("GE_Player_Radio072_Melee4_3_Damage", 139, 2),
                ("GE_Player_Radio072_Melee4_3_Damage", 139, 2),
                ("GE_Player_Radio072_Melee4_Damage", 172, 1),
                ("GE_Player_Radio072_Melee5_Damage", 370, 1),
                ("GE_Player_Radio072_Melee1_Damage", 248, 1),
            ],
            1973,
            1,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Radio072_Melee1_Damage", 425, 1),
                ("GE_Player_Radio072_Melee2_Damage", 291, 2),
                ("GE_Player_Radio072_Melee2_Damage", 291, 2),
                ("GE_Player_Radio072_Melee3_Damage", 346, 1),
                ("GE_Player_Radio072_Melee4_1_Damage", 114, 1),
                ("GE_Player_Radio072_Melee4_2_Damage", 511, 2),
                ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
                ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
                ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
                ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
                ("GE_Player_Radio072_Melee4_3_Damage", 119, 1),
                ("GE_Player_Radio072_Melee4_Damage", 295, 1),
                ("GE_Player_Radio072_Melee5_Damage", 634, 1),
                ("GE_Player_Radio072_Melee1_Damage", 425, 1),
            ],
            6761,
            2,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Radio072_Melee1_Damage", 510, 1),
                ("GE_Player_Radio072_Melee2_Damage", 873, 5),
                ("GE_Player_Radio072_Melee2_Damage", 873, 5),
                ("GE_Player_Radio072_Melee3_Damage", 2076, 5),
                ("GE_Player_Radio072_Melee4_1_Damage", 684, 5),
                ("GE_Player_Radio072_Melee4_2_Damage", 1533, 5),
                ("GE_Player_Radio072_Melee4_3_Damage", 714, 5),
                ("GE_Player_Radio072_Melee4_3_Damage", 714, 5),
                ("GE_Player_Radio072_Melee4_3_Damage", 714, 5),
                ("GE_Player_Radio072_Melee4_3_Damage", 714, 5),
                ("GE_Player_Radio072_Melee4_3_Damage", 714, 5),
                ("GE_Player_Radio072_Melee4_Damage", 354, 1),
                ("GE_Player_Radio072_Melee5_Damage", 3804, 5),
                ("GE_Player_Radio072_Melee1_Damage", 510, 1),
            ],
            20283,
            5,
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = (0..6)
            .map(|index| {
                make_action(
                    index * 500_000,
                    0,
                    "GA_Radio072_Melee",
                    "GE_Player_Radio072_Melee1_Damage",
                    0,
                    0,
                )
            })
            .collect();
        req.relative_duration_us = 3_000_000;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 14);
        let receipts = report["hit_receipts"].as_array().unwrap();
        for (i, (expected_effect, exp_num, exp_den)) in expected_hits.iter().enumerate() {
            assert_eq!(
                receipts[i]["damage_effect_id"], *expected_effect,
                "hit {i} effect mismatch"
            );
            assert_eq!(
                receipts[i]["non_crit_damage"],
                json!(format!("{}/{}", exp_num, exp_den)),
                "hit {i} damage mismatch"
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
fn linko_normal_chain_resets_and_strict() {
    let mut req = request();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Radio072_Melee",
            "GE_Player_Radio072_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_001,
            0,
            "GA_Radio072_Melee",
            "GE_Player_Radio072_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Radio072_Melee1_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Radio072_Melee1_Damage"
    );

    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    let mut swap_req = request();
    swap_req.actors.push(ally);
    swap_req.actions = vec![
        make_action(
            0,
            0,
            "GA_Radio072_Melee",
            "GE_Player_Radio072_Melee1_Damage",
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
            300_000,
            0,
            "GA_Radio072_Melee",
            "GE_Player_Radio072_Melee1_Damage",
            0,
            0,
        ),
    ];
    let swap_report = run_game_scenario(&swap_req, &ExecutionLimits::default()).unwrap();
    let swap_receipts = swap_report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        swap_receipts[0]["damage_effect_id"],
        "GE_Player_Radio072_Melee1_Damage"
    );
    assert_eq!(
        swap_receipts[1]["damage_effect_id"],
        "GE_Player_Radio072_Melee1_Damage"
    );

    let mut strict_req = request();
    strict_req.model_assumptions.insert(
        crate::simulation::energy::LINKO_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::LINKO_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    strict_req.actors[0].talent_levels = None;
    strict_req.actors[0].purchased_skill_level = 1;
    strict_req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    strict_req.model_assumptions.insert(
        crate::characters::linko::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::linko::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&strict_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);
}

#[test]
fn linko_burst_multi_target_all_in_range() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Radio072_UltraSkill",
        "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
        100,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts[0]["target_index"], 0);
    assert_eq!(receipts[1]["target_index"], 1);
}

#[test]
fn linko_burst_multi_target_strict_acceptance() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        crate::simulation::energy::LINKO_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::LINKO_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Radio072_UltraSkill",
        "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
        100,
        20_000_000,
    )];

    req.model_assumptions.insert(
        crate::characters::linko::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::linko::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
}

#[test]
fn linko_pet_deployment_and_autonomous_pulses() {
    let mut req = request();
    req.relative_duration_us = 10_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.model_assumptions.insert(
        crate::characters::linko::skill::PET_HYPOTHESIS_ID.into(),
        crate::characters::linko::skill::PET_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Radio072_Skill",
        "GE_Player_Radio072_Skill1_Damage",
        0,
        12_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Four canonical rockets plus one A1 pulse; no alternate Throw packets.
    assert_eq!(report["hits_settled"], 5);

    let effects: Vec<&str> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["damage_effect_id"].as_str().unwrap())
        .collect();

    assert_eq!(
        effects,
        vec![
            "GE_Player_Radio072_Skill1_Damage",
            "GE_Player_Radio072_Skill2_Damage",
            "GE_Player_Radio072_Skill3_Damage",
            "GE_Player_Radio072_Skill4_Damage",
            "GE_Xiaozhen_Skill1_Damage",
        ]
    );
    // L1: four rockets total 1.000 ATK and the pulse is .500 ATK.
    // At 1000 ATK and .5 DEF, their independent damage totals are 500 and 250.
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts[4]["time_us"], 6_000_100);
    assert_eq!(receipts[4]["non_crit_damage"], json!("250/1"));
    assert_eq!(
        receipts[..4]
            .iter()
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .sum::<u64>(),
        500
    );
}

#[test]
fn linko_pet_pulse_uses_the_deploying_skills_effective_talent() {
    // L8 pulse: .857 * 1000 * .5 = 428.5. A3 raises Skill to L9 and ATK to1200:
    // .925 * 1200 * .5 = 555; the purchased L8 coordinate must not be reused.
    for (awakening, numerator, denominator) in [(0, 857, 2), (3, 555, 1)] {
        let mut req = request();
        req.relative_duration_us = 9_000_000;
        req.actors[0].talent_levels = Some([1, 8, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.model_assumptions.insert(
            crate::characters::linko::skill::PET_HYPOTHESIS_ID.into(),
            crate::characters::linko::skill::PET_HYPOTHESIS_ID
                .description()
                .into(),
        );
        req.actions = vec![make_action(
            0,
            0,
            "GA_Radio072_Skill",
            "GE_Player_Radio072_Skill1_Damage",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["hit_receipts"][4]["non_crit_damage"],
            json!(format!("{}/{}", numerator, denominator))
        );
    }
}

#[test]
fn linko_recast_replaces_the_pending_companion_cadence() {
    let mut req = request();
    req.relative_duration_us = 23_000_000;
    req.model_assumptions.insert(
        crate::characters::linko::skill::PET_HYPOTHESIS_ID.into(),
        crate::characters::linko::skill::PET_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = [0, 6_000_000]
        .into_iter()
        .map(|time| {
            make_action(
                time,
                0,
                "GA_Radio072_Skill",
                "GE_Player_Radio072_Skill1_Damage",
                0,
                6_000_000,
            )
        })
        .collect();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let pulses = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Xiaozhen_Skill1_Damage")
        .map(|hit| {
            assert_eq!(
                hit["command_index"], 1,
                "latest actual deployment owns continuing pulses"
            );
            assert_eq!(
                super::support::hit_metadata(&report, hit)["binding"]["kind"],
                "native_direct"
            );
            hit["time_us"].as_u64().unwrap()
        })
        .collect::<Vec<_>>();
    // Neither the old8s pulse nor its16s continuation survives the6s redeployment.
    assert_eq!(pulses, [14_000_100, 22_000_100]);
}

#[test]
fn linko_explicit_throw_retains_its_two_packets_and_deploys_once() {
    let mut req = request();
    req.relative_duration_us = 9_000_000;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Radio072_Skill",
        "GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 3);
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage2"
    );
    assert_eq!(receipts[2]["damage_effect_id"], "GE_Xiaozhen_Skill1_Damage");
    assert_eq!(receipts[2]["time_us"], 8_000_100);
}

#[test]
fn linko_lte_permission_is_half_open_at_the_exact_oversync_deadline() {
    for (time, admitted) in [(16_999_999, true), (17_000_000, false)] {
        let mut req = request();
        req.relative_duration_us = 18_000_000;
        req.actors[0].initial_energy = 100;
        req.actors[0].energy_capacity = 100;
        req.actions = vec![
            make_action(
                0,
                0,
                "GA_Radio072_UltraSkill",
                "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
                100,
                0,
            ),
            make_action(
                time,
                0,
                "GA_Radio072_UltraSkill",
                "GE_Player_Radio072_LTE_Damage",
                0,
                0,
            ),
        ];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["success"], admitted, "{time}: {report}");
        if admitted {
            // Admission before expiry retains its already committed delivery afterwards.
            assert_eq!(report["hit_receipts"][1]["time_us"], 17_000_099);
            assert_eq!(report["hits_settled"], 3);
        } else {
            assert_eq!(
                report["rejected_action"]["reason"]["message"],
                "oversync_required"
            );
        }
    }
}

#[test]
fn linko_defeat_invalidates_the_old_pulse_generation_and_clears_permission() {
    let mut state = crate::simulation::state::GameSimulatorState::new(&request());
    let actor = &mut state.actors[0];
    let owner = actor.character.linko_mut().unwrap();
    owner.pet_active = true;
    owner.pulse_generation = 1;
    owner.next_pulse_us = Some(8_000_000);
    owner.oversync_until_us = Some(15_000_000);
    owner.hexed_targets.insert(0, 15_000_000);
    crate::characters::linko::skill::on_defeat(actor);
    let owner = actor.character.linko().unwrap();
    assert!(!owner.pet_active);
    assert_eq!(owner.next_pulse_us, None);
    assert_eq!(owner.pulse_command_index, None);
    assert_eq!(owner.oversync_until_us, None);
    assert!(owner.hexed_targets.is_empty());
    // A later redeployment increments again; it must never reuse pre-defeat generation 1.
    assert_ne!(owner.pulse_generation, 1);
    assert_ne!(owner.pulse_generation.wrapping_add(1), 1);
}

#[test]
fn linko_burst_oversync_and_lte_synced_strike() {
    let mut req = request();
    req.relative_duration_us = 16_000_000;
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;

    // Test 1: Casting LTE without Oversync fails admission
    let mut fail_req = req.clone();
    fail_req.actions = vec![make_action(
        0,
        0,
        "GA_Radio072_UltraSkill",
        "GE_Player_Radio072_LTE_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&fail_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["success"], false);
    assert_eq!(report["rejected_action"]["reason"]["kind"], "other");
    assert_eq!(
        report["rejected_action"]["reason"]["message"],
        "oversync_required"
    );

    // Test 2: Burst initiates Oversync, allowing LTE Synced Strike
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Radio072_UltraSkill",
            "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
            100,
            0,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Radio072_UltraSkill",
            "GE_Player_Radio072_LTE_Damage",
            0,
            20_000_000,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 1 primary Burst hit + 2 LTE Synced Strike hits (LTE_Damage + LTE_Xiaozhen_Damage) = 3 hits
    assert_eq!(report["hits_settled"], 3);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Radio072_LTE_Damage"
    );
    assert_eq!(
        receipts[2]["damage_effect_id"],
        "GE_Player_Radio072_LTE_Xiaozhen_Damage"
    );
}

#[test]
fn linko_hexed_reaction_triggered_by_teammate() {
    let mut req = request();
    req.relative_duration_us = 10_000_000;
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;

    // Add a teammate (Actor 1)
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into(); // Mint
    req.actors.push(ally);

    req.actions = vec![
        // Linko casts Burst to apply Hexed
        make_action(
            0,
            0,
            "GA_Radio072_UltraSkill",
            "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
            100,
            20_000_000,
        ),
        // Switch to Mint
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        // Mint lands a hit on the Hexed target
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_500_000,
            actor_index: 1,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100,
        },
    ];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 1 Linko Burst hit + 2 Mint Melee hits + 2 Hexed reaction follow-up hits = 5 hits
    assert_eq!(report["hits_settled"], 5);

    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage"
    );
    let hexed_hits: Vec<_> = receipts
        .iter()
        .filter(|r| r["damage_effect_id"] == "GE_Player_Radio072_LingZhou_Damage")
        .collect();
    assert_eq!(hexed_hits.len(), 2);
    for h in hexed_hits {
        assert_eq!(h["actor_index"], 0);
        assert!(h.get("command_index").is_none());
        assert!(h.get("switch_origin").is_none());
        let event =
            &report["autonomous_events"][h["autonomous_event_index"].as_u64().unwrap() as usize];
        assert_eq!(event["trigger"]["kind"], "settled_hit");
        let parent = &receipts[event["trigger"]["hit_index"].as_u64().unwrap() as usize];
        assert_eq!(parent["actor_index"], 1);
        assert_eq!(parent["time_us"], h["time_us"]);
        let h_meta = super::support::hit_metadata(&report, h);
        assert_eq!(h_meta["binding"]["kind"], "native_direct");
        assert_eq!(h["non_crit_damage"], json!("65/1"));
        assert_eq!(h_meta["packet"]["semantic_tags"], json!(["hexed_reaction"]));
    }
    let event: crate::simulation::GameAutonomousEventReceipt =
        serde_json::from_value(report["autonomous_events"][0].clone()).unwrap();
    let hits = serde_json::from_value::<Vec<crate::simulation::GameHitReceipt>>(
        report["hit_receipts"].clone(),
    )
    .unwrap();
    let mut origins = crate::simulation::origin::Origins::default();
    assert!(origins.register(&req, event.clone()).is_err());
    assert!(origins.register_from_hit(&req, event.clone(), &[]).is_err());
    let mut future = event.clone();
    future.time_us += 1;
    assert!(origins.register_from_hit(&req, future, &hits).is_err());
    assert!(origins.register_from_hit(&req, event, &hits).is_ok());
}

#[test]
fn linko_two_continuing_cycles_preserve_resources_and_state() {
    let mut req = request();
    req.relative_duration_us = 130_000_000;
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;

    // Add teammate Actor 1 to participate in team rotations
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);

    req.model_assumptions.insert(
        crate::simulation::energy::LINKO_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::LINKO_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::linko::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::linko::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::linko::skill::SKILL_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::linko::skill::SKILL_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::linko::skill::PET_HYPOTHESIS_ID.into(),
        crate::characters::linko::skill::PET_HYPOTHESIS_ID
            .description()
            .into(),
    );

    let mut actions = Vec::new();
    for cycle in 0..2 {
        let base_t = cycle * 60_000_000;

        // 1. Burst cast (commits 100 energy, initiates Oversync, deploys Resonance Field, applies Hexed)
        actions.push(make_action(
            base_t,
            0,
            "GA_Radio072_UltraSkill",
            "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
            100,
            0,
        ));

        // 2. LTE Synced Strike during Oversync puts UltraSkill on cooldown
        actions.push(make_action(
            base_t + 2_000_000,
            0,
            "GA_Radio072_UltraSkill",
            "GE_Player_Radio072_LTE_Damage",
            0,
            20_000_000,
        ));

        // 3. Canonical four-rocket Skill starts the companion pulse schedule.
        actions.push(make_action(
            base_t + 4_000_000,
            0,
            "GA_Radio072_Skill",
            "GE_Player_Radio072_Skill1_Damage",
            0,
            12_000_000,
        ));

        // 4. Teammate switch and hit to trigger Hexed reaction
        actions.push(GameActionCommand::SwitchActor {
            timestamp_us: base_t + 7_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        });
        actions.push(GameActionCommand::InvokeAbility {
            timestamp_us: base_t + 7_500_000,
            actor_index: 1,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100,
        });

        // 5. Switch back to Linko
        actions.push(GameActionCommand::SwitchActor {
            timestamp_us: base_t + 8_500_000,
            target_actor_index: 0,
            cooldown_us: 0,
        });

        // 6. Normal attack loops regenerate Ultimate energy.
        for loop_idx in 0..3 {
            let loop_base = base_t + 10_000_000 + loop_idx * 5_000_000;
            for stage in 0..5 {
                actions.push(make_action(
                    loop_base + stage * 800_000,
                    0,
                    "GA_Radio072_Melee",
                    "GE_Player_Radio072_Melee1_Damage",
                    0,
                    0,
                ));
            }
        }

        // 7. Second Skill cast off CD at base_t + 28s
        actions.push(make_action(
            base_t + 28_000_000,
            0,
            "GA_Radio072_Skill",
            "GE_Player_Radio072_Skill1_Damage",
            0,
            12_000_000,
        ));

        // 8. Further Normal loops to top off energy to 100 for the next cycle
        for loop_idx in 0..3 {
            let loop_base = base_t + 32_000_000 + loop_idx * 5_000_000;
            for stage in 0..5 {
                actions.push(make_action(
                    loop_base + stage * 800_000,
                    0,
                    "GA_Radio072_Melee",
                    "GE_Player_Radio072_Melee1_Damage",
                    0,
                    0,
                ));
            }
        }
    }

    req.actions = actions;

    let report1 = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report1["execution_completed"], true);
    assert_eq!(report1["invariants_passed"], true);
    assert_eq!(report1["success"], true);

    // Verify deterministic replay
    let report2 = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report1["hits_settled"], report2["hits_settled"]);
    assert_eq!(report1["hit_receipts"], report2["hit_receipts"]);
    assert_eq!(report1["final_actor_states"], report2["final_actor_states"]);
}
