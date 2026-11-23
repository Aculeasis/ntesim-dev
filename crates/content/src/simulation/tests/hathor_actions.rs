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
    req.actors[0].character_id = "character:1025".into();
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
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_Melee",
        "GE_Player_Hathor_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn hathor_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Hathor_Melee / GE_Player_Hathor_Melee1_Damage)
    // Coeffs: Lv1 = 0.243, Lv8 = 0.416, Lv9 (A3 resonance) = 0.450.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 0.243 * 0.5 = 243 / 2
    //   Lv8: 1000 * 0.416 * 0.5 = 208 / 1
    // At A3: resonance_3 grants +1 level -> Lv9, plus A3 +15% dmg bonus (23/20):
    //   Lv9: 1000 * 0.450 * 0.5 * (23/20) = 1035 / 4
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 243, 2),
        (8, 0, 208, 1),
        (8, 1, 208, 1),
        (8, 3, 1035, 4),
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

    // Skill (GA_Hathor_Skill: 2x Skill1 + 1x Skill11 = 3 hits)
    // Coeffs: Lv1 = 1.000, Lv8 = 1.714, Lv9 (A3 resonance) = 1.851.
    // Unheld Skill does not start Emergency Delivery. A3 adds +15% damage
    // against this non-boss target; its +2 Power has no A5 in this loop.
    //   Lv1: 1000 * 1.000 * 0.5 = 500 / 1 (each hit) -> 3 hits total 1500 / 1
    //   Lv8: 1000 * 1.714 * 0.5 = 857 / 1 (each hit) -> 3 hits total 2571 / 1
    //   Lv9 A3: 1000 * 1.851 * 0.5 * 1.15 = 42573 / 40 per hit
    for (level, awakening, expected_num, expected_den, total_num, total_den) in [
        (1, 0, 500, 1, 1500, 1),
        (8, 0, 857, 1, 2571, 1),
        (8, 3, 42573, 40, 127719, 40),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Hathor_Skill",
            "GE_Player_Hathor_Skill1_Damage",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 3);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 3);
        assert_eq!(
            receipts[0]["damage_effect_id"],
            "GE_Player_Hathor_Skill1_Damage"
        );
        assert_eq!(
            receipts[1]["damage_effect_id"],
            "GE_Player_Hathor_Skill1_Damage"
        );
        assert_eq!(
            receipts[2]["damage_effect_id"],
            "GE_Player_Hathor_Skill11_Damage"
        );
        for receipt in receipts {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", expected_num, expected_den))
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
        }
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total_num, total_den))
        );
    }

    // Burst (GA_Hathor_UltraSkill: 2x UltraSkill1 + 1x UltraSkill2 = 3 hits)
    // Coeffs:
    //   Lv1: UltraSkill1 = 0.962, UltraSkill2 = 5.076
    //   Lv8: UltraSkill1 = 1.649, UltraSkill2 = 8.699
    //   Lv9 (A3 resonance): UltraSkill1 = 1.781, UltraSkill2 = 9.395
    // on_cast activates Emergency Delivery (+15% ATK = 23/20).
    // At A0: ATK = 1150, DEF factor = 0.5.
    //   Lv1:
    //     UltraSkill1: 1150 * 0.962 * 0.5 = 11063 / 20
    //     UltraSkill2: 1150 * 5.076 * 0.5 = 29187 / 10
    //     Total: 2 * (11063 / 20) + 29187 / 10 = 4025 / 1
    //   Lv8:
    //     UltraSkill1: 1150 * 1.649 * 0.5 = 37927 / 40
    //     UltraSkill2: 1150 * 8.699 * 0.5 = 200077 / 40
    //     Total: 2 * (37927 / 40) + 200077 / 40 = 275931 / 40
    //   Lv8 A3: resonance_3 grants Lv9, plus A3 +15% dmg bonus (23/20):
    //     UltraSkill1: 1150 * 1.781 * 0.5 * (23/20) = 942149 / 800
    //     UltraSkill2: 1150 * 9.395 * 0.5 * (23/20) = 4969955 / 800
    //     Total: 2 * (942149 / 800) + 4969955 / 800 = 6854253 / 800
    for (level, awakening, u1, u2, total) in [
        (1, 0, (11063, 20), (29187, 10), (4025, 1)),
        (8, 0, (37927, 40), (200077, 40), (275931, 40)),
        (8, 3, (942149, 800), (993991, 160), (6854253, 800)),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Hathor_UltraSkill",
            "GE_Player_Hathor_UltraSkill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 3);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 3);
        for receipt in &receipts[..2] {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", u1.0, u1.1))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                "GE_Player_Hathor_UltraSkill1_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        assert_eq!(
            receipts[2]["non_crit_damage"],
            json!(format!("{}/{}", u2.0, u2.1))
        );
        assert_eq!(
            receipts[2]["damage_effect_id"],
            "GE_Player_Hathor_UltraSkill2_Damage"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &receipts[2])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total.0, total.1))
        );
    }
}

#[test]
fn hathor_a4_waits_for_unmodeled_cyclone_strike() {
    let mut req_a4 = request();
    req_a4.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req_a4.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req_a4.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_Skill",
        "GE_Player_Hathor_Skill1_Damage",
        0,
        6_000_000,
    )];
    let report_a4 = run_game_scenario(&req_a4, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_a4["execution_completed"], true);
    assert_eq!(report_a4["hits_settled"], 3);
    for receipt in report_a4["hit_receipts"].as_array().unwrap() {
        assert_eq!(receipt["crit_rate"], json!("1/20"));
    }

    let mut req_burst = request();
    req_burst.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req_burst.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req_burst.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_UltraSkill",
        "GE_Player_Hathor_UltraSkill1_Damage",
        100,
        20_000_000,
    )];
    let report_burst = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["execution_completed"], true);
    for receipt in report_burst["hit_receipts"].as_array().unwrap() {
        assert_eq!(receipt["crit_rate"], json!("1/20"));
    }

    let mut req_melee = request();
    req_melee.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    req_melee.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req_melee.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_Melee",
        "GE_Player_Hathor_Melee1_Damage",
        0,
        0,
    )];
    let report_melee = run_game_scenario(&req_melee, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_melee["execution_completed"], true);
    assert_eq!(report_melee["hits_settled"], 1);
    assert_eq!(report_melee["hit_receipts"][0]["crit_rate"], json!("1/20"));
}

#[test]
fn hathor_power_emergency_delivery_and_field_exit_preserve_the_supported_state() {
    // Independent rank-one arithmetic, DEF factor 1/2, ATK 1000:
    // Skill at 2/4 stacks: 1000 * 1.000 * 1/2 * 1.02/1.04 = 510/520.
    // Burst retains 4 stacks: U1 1000*.962*1/2*1.15*1.04=143819/250;
    // U2 1000*5.076*1/2*1.15*1.04=379431/125. After switch, A6 ends
    // while A5's 4-stack damage bonus remains on Hathor's next Normal.
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![5, 6]).unwrap();
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.relative_duration_us = 16_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Hathor_Skill",
            "GE_Player_Hathor_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            6_000_000,
            0,
            "GA_Hathor_Skill",
            "GE_Player_Hathor_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            12_000_000,
            0,
            "GA_Hathor_UltraSkill",
            "GE_Player_Hathor_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 13_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 14_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            14_100_000,
            0,
            "GA_Hathor_Melee",
            "GE_Player_Hathor_Melee1_Damage",
            0,
            0,
        ),
    ];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 10);
    let receipts = report["hit_receipts"].as_array().unwrap();
    for receipt in &receipts[..3] {
        assert_eq!(receipt["non_crit_damage"], json!("510/1"));
    }
    for receipt in &receipts[3..6] {
        assert_eq!(receipt["non_crit_damage"], json!("520/1"));
    }
    assert_eq!(receipts[6]["non_crit_damage"], json!("143819/250"));
    assert_eq!(receipts[8]["non_crit_damage"], json!("379431/125"));
    assert_eq!(receipts[6]["crit_damage"], json!("1294371/1250"));
    assert_eq!(receipts[9]["crit_damage"], json!("9477/50"));
    assert_eq!(receipts[9]["non_crit_damage"], json!("3159/25"));

    let mut during_delivery = request();
    during_delivery.actions = vec![
        make_action(
            0,
            0,
            "GA_Hathor_UltraSkill",
            "GE_Player_Hathor_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Hathor_Skill",
            "GE_Player_Hathor_Skill1_Damage",
            0,
            6_000_000,
        ),
    ];
    let error = run_game_scenario(&during_delivery, &ExecutionLimits::default()).unwrap_err();
    assert!(
        error.contains("Hathor Aerial Command is unavailable during Emergency Delivery"),
        "{error}"
    );
}

#[test]
fn hathor_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Hathor_Skill",
            "GE_Player_Hathor_Skill1_Damage",
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
    assert_eq!(report["hits_settled"], 3);
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
fn hathor_strict_mode_rejects_missing_hypothesis_and_accepts_valid_hypothesis() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::HATHOR_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HATHOR_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_Skill",
        "GE_Player_Hathor_Skill1_Damage",
        0,
        6_000_000,
    )];
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::hathor::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::hathor::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::hathor::skill::POWER_STATE_HYPOTHESIS_ID.into(),
        crate::characters::hathor::skill::POWER_STATE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 3);
}

#[test]
fn hathor_two_skill_one_burst_rotation_hit_count_and_energy() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.relative_duration_us = 20_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Hathor_Skill",
            "GE_Player_Hathor_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            6_000_000,
            0,
            "GA_Hathor_Skill",
            "GE_Player_Hathor_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            12_000_000,
            0,
            "GA_Hathor_UltraSkill",
            "GE_Player_Hathor_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 9);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
}

#[test]
fn hathor_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Hathor_Melee1_Damage", 243, 2),
                ("GE_Player_Hathor_Melee2_Damage", 161, 2),
                ("GE_Player_Hathor_Melee2_1_Damage", 68, 1),
                ("GE_Player_Hathor_Melee2_1_Damage", 68, 1),
                ("GE_Player_Hathor_Melee2_1_Damage", 68, 1),
                ("GE_Player_Hathor_Melee2_1_Damage", 68, 1),
                ("GE_Player_Hathor_Melee3_Damage", 133, 2),
                ("GE_Player_Hathor_Melee3_Damage", 133, 2),
                ("GE_Player_Hathor_Melee4_Damage", 112, 1),
                ("GE_Player_Hathor_Melee4_1_Damage", 479, 2),
                ("GE_Player_Hathor_Melee4_2_Damage", 497, 1),
                ("GE_Player_Hathor_Melee5_Damage", 441, 2),
                ("GE_Player_Hathor_Melee5_1_Damage", 1099, 2),
                ("GE_Player_Hathor_Melee1_Damage", 243, 2),
            ],
            2347,
            1,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Hathor_Melee1_Damage", 208, 1),
                ("GE_Player_Hathor_Melee2_Damage", 138, 1),
                ("GE_Player_Hathor_Melee2_1_Damage", 233, 2),
                ("GE_Player_Hathor_Melee2_1_Damage", 233, 2),
                ("GE_Player_Hathor_Melee2_1_Damage", 233, 2),
                ("GE_Player_Hathor_Melee2_1_Damage", 233, 2),
                ("GE_Player_Hathor_Melee3_Damage", 114, 1),
                ("GE_Player_Hathor_Melee3_Damage", 114, 1),
                ("GE_Player_Hathor_Melee4_Damage", 192, 1),
                ("GE_Player_Hathor_Melee4_1_Damage", 821, 2),
                ("GE_Player_Hathor_Melee4_2_Damage", 852, 1),
                ("GE_Player_Hathor_Melee5_Damage", 378, 1),
                ("GE_Player_Hathor_Melee5_1_Damage", 1883, 2),
                ("GE_Player_Hathor_Melee1_Damage", 208, 1),
            ],
            4022,
            1,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Hathor_Melee1_Damage", 1035, 4),
                ("GE_Player_Hathor_Melee2_Damage", 3427, 20),
                ("GE_Player_Hathor_Melee2_1_Damage", 1449, 10),
                ("GE_Player_Hathor_Melee2_1_Damage", 1449, 10),
                ("GE_Player_Hathor_Melee2_1_Damage", 1449, 10),
                ("GE_Player_Hathor_Melee2_1_Damage", 1449, 10),
                ("GE_Player_Hathor_Melee3_Damage", 2829, 20),
                ("GE_Player_Hathor_Melee3_Damage", 2829, 20),
                ("GE_Player_Hathor_Melee4_Damage", 1909, 8),
                ("GE_Player_Hathor_Melee4_1_Damage", 20401, 40),
                ("GE_Player_Hathor_Melee4_2_Damage", 1058, 1),
                ("GE_Player_Hathor_Melee5_Damage", 2346, 5),
                ("GE_Player_Hathor_Melee5_1_Damage", 23391, 20),
                ("GE_Player_Hathor_Melee1_Damage", 1035, 4),
            ],
            19987,
            4,
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
                    "GA_Hathor_Melee",
                    "GE_Player_Hathor_Melee1_Damage",
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
        assert_eq!(report["hits_settled"], 14);
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
fn hathor_normal_chain_reset_and_noop_boundaries() {
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
                "GA_Hathor_Skill",
                "GE_Player_Hathor_Skill1_Damage",
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
            "GA_Hathor_Melee",
            "GE_Player_Hathor_Melee1_Damage",
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
            "GE_Player_Hathor_Melee2_1_Damage"
        } else {
            "GE_Player_Hathor_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn hathor_normal_chain_strict_hypothesis_and_off_field_five_hit_delivery() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::HATHOR_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HATHOR_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::hathor::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::hathor::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = (0..2)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Hathor_Melee",
                "GE_Player_Hathor_Melee1_Damage",
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
    assert_eq!(settled["hits_settled"], 6);
}

#[test]
fn hathor_multi_target_all_in_range_skill_and_burst() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Test Skill across 2 targets: 3 hits each = 6 hits total
    req.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_Skill",
        "GE_Player_Hathor_Skill1_Damage",
        0,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 6);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts[0]["target_index"], 0);
    assert_eq!(receipts[1]["target_index"], 0);
    assert_eq!(receipts[2]["target_index"], 0);
    assert_eq!(receipts[3]["target_index"], 1);
    assert_eq!(receipts[4]["target_index"], 1);
    assert_eq!(receipts[5]["target_index"], 1);

    // Test Burst across 2 targets: 3 hits each = 6 hits total
    let mut req_burst = request();
    let mut target2 = req_burst.targets[0].clone();
    target2.target_index = 1;
    req_burst.targets.push(target2);
    req_burst.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_UltraSkill",
        "GE_Player_Hathor_UltraSkill1_Damage",
        100,
        20_000_000,
    )];
    let report_burst = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["execution_completed"], true);
    assert_eq!(report_burst["hits_settled"], 6);
    let receipts_burst = report_burst["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts_burst[0]["target_index"], 0);
    assert_eq!(receipts_burst[1]["target_index"], 0);
    assert_eq!(receipts_burst[2]["target_index"], 0);
    assert_eq!(receipts_burst[3]["target_index"], 1);
    assert_eq!(receipts_burst[4]["target_index"], 1);
    assert_eq!(receipts_burst[5]["target_index"], 1);
}

#[test]
fn hathor_multi_target_strict_acceptance() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        crate::simulation::energy::HATHOR_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HATHOR_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::hathor::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::hathor::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::hathor::skill::POWER_STATE_HYPOTHESIS_ID.into(),
        crate::characters::hathor::skill::POWER_STATE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Hathor_Skill",
        "GE_Player_Hathor_Skill1_Damage",
        0,
        20_000_000,
    )];

    req.model_assumptions.insert(
        crate::characters::hathor::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::hathor::skill::ALL_IN_RANGE_HYPOTHESIS_ID
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
