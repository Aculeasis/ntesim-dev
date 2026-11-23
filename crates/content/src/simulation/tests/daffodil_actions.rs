use super::*;
use crate::simulation::tests::support::formula_damage_total;
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
    req.actors[0].character_id = "character:1054".into();
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
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn daffodil_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Daffodill_Melee / GE_Player_Daffodill_Melee1_Damage)
    // Coeffs: Lv1 = 0.167, Lv8 = 0.286, Lv9 (A3 resonance) = 0.309.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 0.167 * 0.5 = 167 / 2
    //   Lv8: 1000 * 0.286 * 0.5 = 143 / 1
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 0.309 * 0.5 = 309 / 2
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 167, 2),
        (8, 0, 143, 1),
        (8, 1, 143, 1),
        (8, 3, 309, 2),
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

    // Skill (GA_Daffodill_Skill / GE_Player_Daffodill_Skill1_Damage + child packets)
    // Coeffs:
    //   Skill1: Lv1 = 0.409, Lv8 = 0.701, Lv9 (A3 resonance) = 0.757.
    //   Skill2: Lv1 = 0.337, Lv8 = 0.578, Lv9 (A3 resonance) = 0.624.
    //   Skill5: Lv1 = 0.498, Lv8 = 0.853, Lv9 (A3 resonance) = 0.922.
    // Multiplicities: 2x Skill1 + 5x Skill2 + 1x Skill5 (8 hits total).
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: s1 = 409/2, s2 = 337/2, s5 = 249/1 -> total = 3001/2
    //   Lv8: s1 = 701/2, s2 = 289/1, s5 = 853/2 -> total = 5145/2
    // With no admitted Support, A1 adds nothing. R3 still grants one talent level.
    for (level, awakening, s1, s2, s5, total) in [
        (1, 0, (409, 2), (337, 2), (249, 1), (3001, 2)),
        (8, 0, (701, 2), (289, 1), (853, 2), (5145, 2)),
        (8, 1, (701, 2), (289, 1), (853, 2), (5145, 2)),
        (8, 3, (757, 2), (312, 1), (461, 1), (2778, 1)),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Daffodill_Skill",
            "GE_Player_Daffodill_Skill1_Damage",
            0,
            16_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 8);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 8);
        for receipt in &receipts[..2] {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", s1.0, s1.1))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                "GE_Player_Daffodill_Skill1_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
        }
        for (receipt, row) in receipts[2..7].iter().zip([2, 6, 7, 8, 9]) {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", s2.0, s2.1))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                format!("GE_Player_Daffodill_Skill{row}_Damage")
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
        }
        assert_eq!(
            receipts[7]["non_crit_damage"],
            json!(format!("{}/{}", s5.0, s5.1))
        );
        assert_eq!(
            receipts[7]["damage_effect_id"],
            "GE_Player_Daffodill_Skill5_Damage"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &receipts[7])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total.0, total.1))
        );
    }

    // Burst (GA_Daffodill_UltraSkill / GE_Player_Daffodill_UltraSkill1_Damage + child packets)
    // Coeffs:
    //   UltraSkill1: Lv1 = 0.493, Lv8 = 0.845, Lv9 (A3 resonance) = 0.913.
    //   UltraSkill2: Lv1 = 0.253, Lv8 = 0.434, Lv9 (A3 resonance) = 0.468.
    //   UltraSkill4: Lv1 = 0.511, Lv8 = 0.876, Lv9 (A3 resonance) = 0.946.
    //   UltraSkill5: Lv1 = 2.183, Lv8 = 3.741, Lv9 (A3 resonance) = 4.041.
    // Multiplicities: Ultra1, 2x Ultra3, Ultra2, 2x Ultra6, 7x Ultra4, Ultra5.
    // Ultra6 differs from Ultra2 by .001/.001/.002 at levels 1/8/9.
    // on_cast enters Insight (+10% damage bonus -> mult = 11/10).
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: u1 = 5423/20, u2 = 2783/20, u6 = 1397/10 -> total = 4400
    //   Lv8: u1 = 1859/4, u2 = 2387/10, u6 = 957/4 -> total = 37708/5
    // At A3: resonance_3 grants +1 level -> Lv9, Insight (+10% -> 11/10):
    //   Lv9: u1 = 10043/20, u2 = 1287/5, u6 = 517/2 -> total = 16291/2
    for (level, awakening, u1, u2, u6, u4, u5, total) in [
        (
            1,
            0,
            (5423, 20),
            (2783, 20),
            (1397, 10),
            (5621, 20),
            (24013, 20),
            (4400, 1),
        ),
        (
            8,
            0,
            (1859, 4),
            (2387, 10),
            (957, 4),
            (2409, 5),
            (41151, 20),
            (37708, 5),
        ),
        (
            8,
            3,
            (10043, 20),
            (1287, 5),
            (517, 2),
            (5203, 10),
            (44451, 20),
            (16291, 2),
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Daffodill_UltraSkill",
            "GE_Player_Daffodill_UltraSkill1_Damage",
            120,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 14);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 14);
        for (receipt, row) in receipts[..3].iter().zip([1, 3, 3]) {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", u1.0, u1.1))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                format!("GE_Player_Daffodill_UltraSkill{row}_Damage")
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        for (receipt, row) in receipts[3..6].iter().zip([2, 6, 6]) {
            let expected = if row == 2 { u2 } else { u6 };
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", expected.0, expected.1))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                format!("GE_Player_Daffodill_UltraSkill{row}_Damage")
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        for receipt in &receipts[6..13] {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", u4.0, u4.1))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                "GE_Player_Daffodill_UltraSkill4_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        assert_eq!(
            receipts[13]["non_crit_damage"],
            json!(format!("{}/{}", u5.0, u5.1))
        );
        assert_eq!(
            receipts[13]["damage_effect_id"],
            "GE_Player_Daffodill_UltraSkill5_Damage"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &receipts[13])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total.0, total.1))
        );
    }
}

#[test]
fn daffodil_a6_def_ignore_on_skill() {
    // At A6: 15% DEF ignore (3/20).
    // Effective DEF = 1080 * 0.85 = 918. DEF factor = 1080 / (1080 + 918) = 20 / 37.
    // No Support stacks: Skill at Lv8, R3 -> Lv9 = 0.757, no A1 addition.
    // non_crit = 1000 * 0.757 * (20 / 37) = 15140 / 37.
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Skill1_Damage",
        0,
        16_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 8);
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("15140/37")
    );
}

#[test]
fn daffodil_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Daffodill_Skill",
            "GE_Player_Daffodill_Skill1_Damage",
            0,
            16_000_000,
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
    assert_eq!(report["hits_settled"], 8);
    for receipt in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(receipt["actor_index"], 0);
    }

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);
}

#[test]
fn daffodil_strict_mode_rejects_missing_hypothesis_and_accepts_valid_hypothesis() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Skill1_Damage",
        0,
        16_000_000,
    )];
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let resonance = crate::characters::daffodil::resonance::HYPOTHESIS;
    req.model_assumptions
        .insert(resonance.into(), resonance.description().into());
    req.model_assumptions.insert(
        crate::characters::daffodil::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::daffodil::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 8);
}

#[test]
fn daffodil_two_skill_one_burst_rotation_hit_count_and_energy() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.relative_duration_us = 35_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Daffodill_Skill",
            "GE_Player_Daffodill_Skill1_Damage",
            0,
            16_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Daffodill_UltraSkill",
            "GE_Player_Daffodill_UltraSkill1_Damage",
            120,
            20_000_000,
        ),
        make_action(
            21_099_006,
            0,
            "GA_Daffodill_Skill",
            "GE_Player_Daffodill_Skill1_Damage",
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
    assert_eq!(report["hits_settled"], 30);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
}

#[test]
fn daffodil_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Daffodill_Melee1_Damage", 167, 2),
                ("GE_Player_Daffodill_Melee2_Damage", 153, 2),
                ("GE_Player_Daffodill_Melee2_1_Damage", 153, 2),
                ("GE_Player_Daffodill_Melee3_Damage", 66, 1),
                ("GE_Player_Daffodill_Melee3_Damage", 66, 1),
                ("GE_Player_Daffodill_Melee3_1_Damage", 31, 1),
                ("GE_Player_Daffodill_Melee3_1_Damage", 31, 1),
                ("GE_Player_Daffodill_Melee3_2_Damage", 141, 2),
                ("GE_Player_Daffodill_Melee3_3_Damage", 263, 2),
                ("GE_Player_Daffodill_Melee4_Damage", 262, 1),
                ("GE_Player_Daffodill_Melee4_1_Damage", 193, 2),
                ("GE_Player_Daffodill_Melee4_2_Damage", 127, 2),
                ("GE_Player_Daffodill_Melee5_Damage", 101, 2),
                ("GE_Player_Daffodill_Melee5_Damage", 101, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 101, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 101, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 101, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 101, 2),
                ("GE_Player_Daffodill_Melee5_1_Damage", 473, 2),
                ("GE_Player_Daffodill_Melee1_Damage", 167, 2),
            ],
            3355,
            2,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Daffodill_Melee1_Damage", 143, 1),
                ("GE_Player_Daffodill_Melee2_Damage", 131, 1),
                ("GE_Player_Daffodill_Melee2_1_Damage", 131, 1),
                ("GE_Player_Daffodill_Melee3_Damage", 113, 1),
                ("GE_Player_Daffodill_Melee3_Damage", 113, 1),
                ("GE_Player_Daffodill_Melee3_1_Damage", 53, 1),
                ("GE_Player_Daffodill_Melee3_1_Damage", 53, 1),
                ("GE_Player_Daffodill_Melee3_2_Damage", 121, 1),
                ("GE_Player_Daffodill_Melee3_3_Damage", 451, 2),
                ("GE_Player_Daffodill_Melee4_Damage", 449, 1),
                ("GE_Player_Daffodill_Melee4_1_Damage", 331, 2),
                ("GE_Player_Daffodill_Melee4_2_Damage", 109, 1),
                ("GE_Player_Daffodill_Melee5_Damage", 173, 2),
                ("GE_Player_Daffodill_Melee5_Damage", 173, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 173, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 173, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 173, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 173, 2),
                ("GE_Player_Daffodill_Melee5_1_Damage", 811, 2),
                ("GE_Player_Daffodill_Melee1_Damage", 143, 1),
            ],
            5749,
            2,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Daffodill_Melee1_Damage", 309, 2),
                ("GE_Player_Daffodill_Melee2_Damage", 283, 2),
                ("GE_Player_Daffodill_Melee2_1_Damage", 283, 2),
                ("GE_Player_Daffodill_Melee3_Damage", 122, 1),
                ("GE_Player_Daffodill_Melee3_Damage", 122, 1),
                ("GE_Player_Daffodill_Melee3_1_Damage", 115, 2),
                ("GE_Player_Daffodill_Melee3_1_Damage", 115, 2),
                ("GE_Player_Daffodill_Melee3_2_Damage", 261, 2),
                ("GE_Player_Daffodill_Melee3_3_Damage", 487, 2),
                ("GE_Player_Daffodill_Melee4_Damage", 485, 1),
                ("GE_Player_Daffodill_Melee4_1_Damage", 357, 2),
                ("GE_Player_Daffodill_Melee4_2_Damage", 235, 2),
                ("GE_Player_Daffodill_Melee5_Damage", 187, 2),
                ("GE_Player_Daffodill_Melee5_Damage", 187, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 187, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 187, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 187, 2),
                ("GE_Player_Daffodill_Melee6_Damage", 187, 2),
                ("GE_Player_Daffodill_Melee5_1_Damage", 875, 2),
                ("GE_Player_Daffodill_Melee1_Damage", 309, 2),
            ],
            6209,
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
                    "GA_Daffodill_Melee",
                    "GE_Player_Daffodill_Melee1_Damage",
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
        assert_eq!(report["hits_settled"], 20);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 20);
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
fn daffodil_normal_chain_reset_boundaries_and_noop_switch() {
    let mut req = request();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_001,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Daffodill_Melee1_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Daffodill_Melee1_Damage"
    );

    let mut req_cont = request();
    req_cont.actions = vec![
        make_action(
            0,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_cont = run_game_scenario(&req_cont, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_cont["hits_settled"], 3);
    let receipts_cont = report_cont["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_cont[0]["damage_effect_id"],
        "GE_Player_Daffodill_Melee1_Damage"
    );
    assert_eq!(
        receipts_cont[1]["damage_effect_id"],
        "GE_Player_Daffodill_Melee2_Damage"
    );
    assert_eq!(
        receipts_cont[2]["damage_effect_id"],
        "GE_Player_Daffodill_Melee2_1_Damage"
    );

    let mut req_skill_reset = request();
    req_skill_reset.actions = vec![
        make_action(
            0,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            500_000,
            0,
            "GA_Daffodill_Skill",
            "GE_Player_Daffodill_Skill1_Damage",
            0,
            0,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_skill = run_game_scenario(&req_skill_reset, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_skill["hits_settled"], 10);
    let receipts_skill = report_skill["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_skill[0]["damage_effect_id"],
        "GE_Player_Daffodill_Melee1_Damage"
    );
    assert_eq!(
        receipts_skill[1]["damage_effect_id"],
        "GE_Player_Daffodill_Skill1_Damage"
    );
    assert_eq!(
        receipts_skill[9]["damage_effect_id"],
        "GE_Player_Daffodill_Melee1_Damage"
    );

    let mut req_noop = request();
    req_noop.actions = vec![
        make_action(
            0,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
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
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_noop = run_game_scenario(&req_noop, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_noop["hits_settled"], 3);
    let receipts_noop = report_noop["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_noop[0]["damage_effect_id"],
        "GE_Player_Daffodill_Melee1_Damage"
    );
    assert_eq!(
        receipts_noop[1]["damage_effect_id"],
        "GE_Player_Daffodill_Melee2_Damage"
    );
    assert_eq!(
        receipts_noop[2]["damage_effect_id"],
        "GE_Player_Daffodill_Melee2_1_Damage"
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
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
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
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_exit = run_game_scenario(&req_exit, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_exit["hits_settled"], 2);
    let receipts_exit = report_exit["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_exit[0]["damage_effect_id"],
        "GE_Player_Daffodill_Melee1_Damage"
    );
    assert_eq!(
        receipts_exit[1]["damage_effect_id"],
        "GE_Player_Daffodill_Melee1_Damage"
    );
}

#[test]
fn daffodil_normal_chain_strict_and_pending_final_stage() {
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
        crate::characters::daffodil::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::daffodil::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID
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
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            100_000,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            200_000,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            300_000,
            0,
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            0,
            0,
        ),
    ];
    req_exit.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 300_050,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req_exit.relative_duration_us = 300_150;
    let report_exit = run_game_scenario(&req_exit, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_exit["execution_completed"], true);
    assert_eq!(report_exit["hits_settled"], 12);
    let receipts = report_exit["hit_receipts"].as_array().unwrap();
    for receipt in &receipts[9..12] {
        assert_eq!(receipt["actor_index"], 0);
    }
    assert_eq!(
        receipts[9]["damage_effect_id"],
        "GE_Player_Daffodill_Melee4_Damage"
    );
    assert_eq!(
        receipts[10]["damage_effect_id"],
        "GE_Player_Daffodill_Melee4_1_Damage"
    );
    assert_eq!(
        receipts[11]["damage_effect_id"],
        "GE_Player_Daffodill_Melee4_2_Damage"
    );
}

mod multi_target;
