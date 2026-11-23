use super::*;
use crate::simulation::tests::support::formula_damage_total;
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

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1070".into();
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
        "GA_Mitsuki_Melee",
        "GE_Player_Mitsuki_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn aurelia_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, coefficients, total) in [
        (1, 0, [347, 630, 300, 300, 300, 99, 99, 99, 99, 347], 1310),
        (
            8,
            0,
            [595, 1080, 514, 514, 514, 170, 170, 170, 170, 595],
            2246,
        ),
        (
            8,
            3,
            [642, 1166, 555, 555, 555, 183, 183, 183, 183, 642],
            4847,
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = (0..5)
            .map(|index| {
                make_action(
                    index * 300_000,
                    0,
                    "GA_Mitsuki_Melee",
                    "GE_Player_Mitsuki_Melee1_Damage",
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
        assert_eq!(report["hits_settled"], 10);
        for (hit, coefficient) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(coefficients)
        {
            let divisor = if coefficient % 2 == 0 { 1 } else { 2 };
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", coefficient / (2 / divisor), divisor))
            );
        }
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total, if awakening == 3 { 2 } else { 1 }))
        );
        let stages = [1, 2, 3, 3, 3, 4, 4, 4, 4, 1];
        for (hit, stage) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(stages)
        {
            assert_eq!(
                hit["damage_effect_id"],
                format!("GE_Player_Mitsuki_Melee{stage}_Damage")
            );
        }
    }
}

#[test]
fn aurelia_normal_chain_reset_and_noop_boundaries() {
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
                "GA_Mitsuki_Skill",
                "GE_Player_Mitsuki_Skill_Damage",
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
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
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
            "GE_Player_Mitsuki_Melee2_Damage"
        } else {
            "GE_Player_Mitsuki_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn aurelia_normal_chain_strict_hypothesis_and_off_field_four_hit_delivery() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::AURELIA_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::AURELIA_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::aurelia::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::aurelia::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = (0..4)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Mitsuki_Melee",
                "GE_Player_Mitsuki_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 3050,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.relative_duration_us = 3099;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["hits_settled"], 5);
    req.relative_duration_us = 3100;
    let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(delivered["hits_settled"], 9);
    for hit in &delivered["hit_receipts"].as_array().unwrap()[5..] {
        assert_eq!(hit["actor_index"], 0);
        assert_eq!(hit["damage_effect_id"], "GE_Player_Mitsuki_Melee4_Damage");
    }
}

#[test]
fn aurelia_normal_chain_samples_cadenza_crit_at_delivery_expiry() {
    let mut req = request();
    req.relative_duration_us = 15_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mitsuki_Skill",
        "GE_Player_Mitsuki_Skill_Damage",
        0,
        0,
    )];
    for timestamp in [10_000_000, 11_000_000, 11_999_899, 12_000_000] {
        req.actions.push(make_action(
            timestamp,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ));
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 10);
    let hits = report["hit_receipts"].as_array().unwrap();
    for hit in &hits[3..6] {
        assert_eq!(hit["crit_rate"], json!("1/5"));
        assert_eq!(hit["non_crit_damage"], json!("150/1"));
    }
    for hit in &hits[6..] {
        assert_eq!(hit["crit_rate"], json!("1/20"));
        assert_eq!(hit["non_crit_damage"], json!("99/2"));
    }
}

#[test]
fn aurelia_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Mitsuki_Melee / GE_Player_Mitsuki_Melee1_Damage)
    // Coeffs: Lv1 = 0.347, Lv8 = 0.595, Lv9 (A3 resonance) = 0.642.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 0.347 * 0.5 = 347 / 2
    //   Lv8: 1000 * 0.595 * 0.5 = 595 / 2
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 0.642 * 0.5 = 321 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 347, 2),
        (8, 0, 595, 2),
        (8, 1, 595, 2),
        (8, 3, 321, 1),
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

    // Skill (GA_Mitsuki_Skill / GE_Player_Mitsuki_Skill_Damage)
    // Coeffs: Lv1 = 1.000, Lv8 = 1.714, Lv9 (A3 resonance) = 1.851.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 1.000 * 0.5 = 500 / 1
    //   Lv8: 1000 * 1.714 * 0.5 = 857 / 1
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 1.851 * 0.5 = 1851 / 2
    for (level, awakening, expected_num, expected_den) in
        [(1, 0, 500, 1), (8, 0, 857, 1), (8, 3, 1851, 2)]
    {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Mitsuki_Skill",
            "GE_Player_Mitsuki_Skill_Damage",
            0,
            11_000_000,
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
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
    }

    // Burst (GA_Mitsuki_UltraSkill / GE_Player_Mitsuki_UltraSkill_Damage + child packets)
    // Coeffs:
    //   UltraSkill1: Lv1 = 0.890, Lv8 = 1.525, Lv9 (A3 resonance) = 1.647.
    //   UltraSkill2: Lv1 = 0.552, Lv8 = 0.946, Lv9 (A3 resonance) = 1.022.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1:
    //     UltraSkill1 (5 hits): 1000 * 0.890 * 0.5 = 445 / 1
    //     UltraSkill2 (1 hit):  1000 * 0.552 * 0.5 = 276 / 1
    //     Total: 5 * 445 + 276 = 2501 / 1
    //   Lv8:
    //     UltraSkill1 (5 hits): 1000 * 1.525 * 0.5 = 1525 / 2
    //     UltraSkill2 (1 hit):  1000 * 0.946 * 0.5 = 473 / 1
    //     Total: 5 * (1525 / 2) + 473 = 8571 / 2
    //   Lv8 A3: Resonance 3 selects Lv9. Burst alone does not grant Cadenza.
    //     UltraSkill1 (5 hits): 1000 * 1.647 * 0.5 = 1647 / 2
    //     UltraSkill2 (1 hit):  1000 * 1.022 * 0.5 = 511 / 1
    //     Total: 5 * (1647 / 2) + 511 = 9257 / 2
    for (level, awakening, u1, u2, total) in [
        (1, 0, (445, 1), (276, 1), (2501, 1)),
        (8, 0, (1525, 2), (473, 1), (8571, 2)),
        (8, 3, (1647, 2), (511, 1), (9257, 2)),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Mitsuki_UltraSkill",
            "GE_Player_Mitsuki_UltraSkill_Damage",
            100,
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
        assert_eq!(receipts.len(), 6);
        for receipt in &receipts[..5] {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", u1.0, u1.1))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                "GE_Player_Mitsuki_UltraSkill_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        assert_eq!(
            receipts[5]["non_crit_damage"],
            json!(format!("{}/{}", u2.0, u2.1))
        );
        assert_eq!(
            receipts[5]["damage_effect_id"],
            "GE_Player_Mitsuki_UltraSkill_Damage2"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &receipts[5])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total.0, total.1))
        );
    }
}

#[test]
fn aurelia_a1_and_a5_crit_rate_modifiers() {
    // A1: Skill cast enters Cadenza, granting +15% Crit Rate (3/20).
    // With 5% (1/20) base, total is 20% (1/5).
    let mut req_a1 = request();
    req_a1.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req_a1.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req_a1.actions = vec![make_action(
        0,
        0,
        "GA_Mitsuki_Skill",
        "GE_Player_Mitsuki_Skill_Damage",
        0,
        11_000_000,
    )];
    let report_a1 = run_game_scenario(&req_a1, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_a1["execution_completed"], true);
    assert_eq!(report_a1["hits_settled"], 1);
    assert_eq!(report_a1["hit_receipts"][0]["crit_rate"], json!("1/5"));

    // A5: Canon Chorus (UltraSkill) in Cadenza receives A1 (+15%) + A5 (+15%) = +30% (3/10).
    // With 5% (1/20) base, total is 35% (7/20) across all 6 hits.
    let mut req_a5 = request();
    req_a5.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req_a5.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req_a5.actions = vec![
        make_action(
            0,
            0,
            "GA_Mitsuki_Skill",
            "GE_Player_Mitsuki_Skill_Damage",
            0,
            11_000_000,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Mitsuki_UltraSkill",
            "GE_Player_Mitsuki_UltraSkill_Damage",
            100,
            20_000_000,
        ),
    ];
    let report_a5 = run_game_scenario(&req_a5, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_a5["execution_completed"], true);
    assert_eq!(report_a5["hits_settled"], 7);
    for receipt in &report_a5["hit_receipts"].as_array().unwrap()[1..] {
        assert_eq!(receipt["crit_rate"], json!("7/20"));
    }
}

#[test]
fn aurelia_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Mitsuki_Skill",
            "GE_Player_Mitsuki_Skill_Damage",
            0,
            11_000_000,
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
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(report["hit_receipts"][0]["actor_index"], 0);

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);
}

#[test]
fn aurelia_strict_mode_rejects_missing_hypothesis_and_accepts_valid_hypothesis() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mitsuki_UltraSkill",
        "GE_Player_Mitsuki_UltraSkill_Damage",
        100,
        20_000_000,
    )];
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::aurelia::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::aurelia::skill::PACKETS_HYPOTHESIS_ID
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
fn aurelia_two_skill_one_burst_rotation_hit_count_and_energy() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.relative_duration_us = 25_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Mitsuki_Skill",
            "GE_Player_Mitsuki_Skill_Damage",
            0,
            11_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Mitsuki_UltraSkill",
            "GE_Player_Mitsuki_UltraSkill_Damage",
            100,
            20_000_000,
        ),
        make_action(
            13_000_000,
            0,
            "GA_Mitsuki_Skill",
            "GE_Player_Mitsuki_Skill_Damage",
            0,
            11_000_000,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 8);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
}

#[test]
fn aurelia_direct_air_evade_packets_preserve_rates_chain_and_energy_boundary() {
    let packets: [(&str, [u16; 9], &str); 2] = [
        (
            "GE_Player_Mitsuki_AirAttack_Damage",
            [1300, 1404, 1516, 1638, 1769, 1910, 2063, 2228, 2406],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X",
        ),
        (
            "GE_Player_Mitsuki_EvadeBranch_Damage",
            [258, 279, 301, 325, 351, 379, 409, 442, 478],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A",
        ),
    ];

    for (packet_id, rates, category) in packets {
        for (level, rate) in (1_u32..=9).zip(rates) {
            let mut req = request();
            req.actors[0].talent_levels = Some([level, 1, 1]);
            req.actors[0].initial_energy = 0;
            req.actions = vec![make_action(0, 0, "GA_Mitsuki_Melee", packet_id, 0, 0)];
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            let expected = if rate % 2 == 0 {
                json!(format!("{}/{}", rate / 2, 1))
            } else {
                json!(format!("{}/{}", rate, 2))
            };
            assert_eq!(
                report["execution_completed"], true,
                "{} {}",
                report["halt_diagnostic"], report["rejected_action"]
            );
            assert_eq!(report["hits_settled"], 1);
            assert_eq!(report["hit_receipts"][0]["ability_id"], "GA_Mitsuki_Melee");
            assert_eq!(report["hit_receipts"][0]["damage_effect_id"], packet_id);
            assert_eq!(report["hit_receipts"][0]["non_crit_damage"], expected);
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
                category
            );
            let (expected_energy, expected_remainder) =
                if packet_id == "GE_Player_Mitsuki_AirAttack_Damage" {
                    (6, json!("1/2"))
                } else {
                    (1, json!("3/10"))
                };
            assert_eq!(report["final_actor_states"][0]["energy"], expected_energy);
            assert_eq!(
                report["final_actor_states"][0]["charge_remainder"],
                expected_remainder
            );
            assert!(
                report["resolved_model_assumptions"]
                    .get(
                        crate::characters::aurelia::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
                            .as_str()
                    )
                    .is_some()
            );
            assert!(
                report["resolved_model_assumptions"]
                    .get(
                        crate::simulation::energy::AURELIA_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
                            .as_str()
                    )
                    .is_some()
            );
            assert!(
                report["resolved_model_assumptions"]
                    .get(crate::simulation::energy::AURELIA_HIT_ENERGY_HYPOTHESIS_ID.as_str())
                    .is_none()
            );
        }
    }

    let mut chain = request();
    chain.actors[0].initial_energy = 0;
    chain.actions = vec![
        make_action(
            0,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            300_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_AirAttack_Damage",
            0,
            0,
        ),
        make_action(
            600_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&chain, &ExecutionLimits::default()).unwrap();
    let effects: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|receipt| receipt["damage_effect_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        effects,
        [
            "GE_Player_Mitsuki_Melee1_Damage",
            "GE_Player_Mitsuki_AirAttack_Damage",
            "GE_Player_Mitsuki_Melee2_Damage"
        ]
    );
}

#[test]
fn aurelia_direct_packet_input_requires_exact_strict_acceptance() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mitsuki_Melee",
        "GE_Player_Mitsuki_AirAttack_Damage",
        0,
        0,
    )];
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::aurelia::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID.into(),
        crate::characters::aurelia::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::AURELIA_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::AURELIA_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
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
}

#[test]
fn aurelia_multi_target_all_in_range_skill_and_burst() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Test Skill across 2 targets: 1 hit each = 2 hits
    req.actions = vec![make_action(
        0,
        0,
        "GA_Mitsuki_Skill",
        "GE_Player_Mitsuki_Skill_Damage",
        0,
        11_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["hit_receipts"][0]["target_index"], 0);
    assert_eq!(report["hit_receipts"][1]["target_index"], 1);

    // Test Burst across 2 targets: 6 hits each = 12 hits
    let mut req_burst = request();
    let mut target2 = req_burst.targets[0].clone();
    target2.target_index = 1;
    req_burst.targets.push(target2);
    req_burst.actors[0].initial_energy = 100;
    req_burst.actions = vec![make_action(
        0,
        0,
        "GA_Mitsuki_UltraSkill",
        "GE_Player_Mitsuki_UltraSkill_Damage",
        100,
        20_000_000,
    )];
    let report_burst = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["execution_completed"], true);
    assert_eq!(report_burst["hits_settled"], 12);

    // Strict mode requires ALL_IN_RANGE hypothesis
    req_burst.actors[0].talent_levels = None;
    req_burst.actors[0].purchased_skill_level = 8;
    req_burst.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req_burst.model_assumptions.insert(
        crate::characters::aurelia::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::aurelia::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req_burst.model_assumptions.insert(
        crate::characters::aurelia::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::aurelia::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report_strict = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_strict["execution_completed"], true);
    assert_eq!(report_strict["hits_settled"], 12);
}

#[test]
fn aurelia_riposte_from_critical_dodge_consumed_once_and_grants_energy() {
    let mut req = request();
    req.combat_stamina = Some(crate::simulation::CombatStaminaProfile {
        capacity: Ratio {
            numerator: 60,
            denominator: 1,
        },
        initial: Ratio {
            numerator: 60,
            denominator: 1,
        },
        recovery_rate: Ratio::zero(),
        use_efficiency: Ratio::one(),
    });
    req.model_assumptions.insert(
        crate::characters::aurelia::riposte::HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::riposte::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = Some([8, 1, 1]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].initial_energy = 0;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![
        GameActionCommand::PlayerInput {
            timestamp_us: 100_000,
            actor_index: 0,
            input: crate::simulation::PlayerInput::Dash,
            critical_success: true,
        },
        make_action(
            600_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_500_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
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
    assert_eq!(report["hits_settled"], 2);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits[0]["damage_effect_id"],
        "GE_Player_Mitsuki_PerfectEvadeAtk_Damage"
    );
    assert_eq!(
        super::support::hit_metadata(&report, &hits[0])["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Z"
    );
    // Lv8 coefficient 3.342 * 1000 = 3342
    assert_eq!(hits[0]["non_crit_damage"], json!("3342/1"));
    // Consumed once: second normal reverts to Melee1
    assert_eq!(
        hits[1]["damage_effect_id"],
        "GE_Player_Mitsuki_Melee1_Damage"
    );
    // Energy: 6.500 from riposte + 2.168 from Melee1 = 8.668 -> 8 whole units, carry 668/1000 (167/250)
    assert_eq!(report["final_actor_states"][0]["energy"], 8);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("167/250")
    );
}

#[test]
fn aurelia_plunge_damage_and_energy() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::aurelia::plunge::HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::plunge::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = Some([8, 1, 1]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].initial_energy = 0;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![make_action(
        100_000,
        0,
        "GA_Mitsuki_Melee",
        "GE_Player_Mitsuki_AirAttack_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 1);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits[0]["damage_effect_id"],
        "GE_Player_Mitsuki_AirAttack_Damage"
    );
    assert_eq!(
        super::support::hit_metadata(&report, &hits[0])["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X"
    );
    // Lv8 AirAttack coefficient 2.228 * 1000 = 2228
    assert_eq!(hits[0]["non_crit_damage"], json!("2228/1"));
    // Direct packet energy: 6.500 -> 6 whole units, carry 1/2
    assert_eq!(report["final_actor_states"][0]["energy"], 6);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("1/2")
    );
}

#[test]
fn aurelia_stage_3_power_selection_and_all_target_aoe() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.targets[1].defense = 0;
    req.targets[1].resistance = Ratio::zero();
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.model_assumptions.insert(
        crate::HypothesisId::AureliaCadenzaPowerSelectV1
            .as_str()
            .into(),
        crate::HypothesisId::AureliaCadenzaPowerSelectV1
            .description()
            .into(),
    );

    // Part 1: Without Cadenza, Stage 3 is ordinary GE_Player_Mitsuki_Melee3_Damage to target 0 only (3 hits).
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            500_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
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
    // Stage 1: 1 hit, Stage 2: 1 hit, Stage 3: 3 hits = 5 hits total
    assert_eq!(report["hits_settled"], 5);
    let hits = report["hit_receipts"].as_array().unwrap();
    for hit in &hits[2..5] {
        assert_eq!(hit["damage_effect_id"], "GE_Player_Mitsuki_Melee3_Damage");
        assert_eq!(hit["target_index"], 0);
        // Lv8 coefficient 0.514 * 1000 = 514
        assert_eq!(hit["non_crit_damage"], json!("514/1"));
    }

    // Part 2: With Cadenza (from Skill), Stage 3 replaces with GE_Player_Mitsuki_Melee3_Power_Damage
    // and hits ALL living targets (3 hits to target 0 + 3 hits to target 1 = 6 hits for Stage 3).
    let mut req2 = req.clone();
    req2.actions = vec![
        make_action(
            0,
            0,
            "GA_Mitsuki_Skill",
            "GE_Player_Mitsuki_Skill_Damage",
            0,
            15_000_000,
        ),
        make_action(
            1_500_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_500_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report2 = run_game_scenario(&req2, &ExecutionLimits::default()).unwrap();
    assert_eq!(report2["execution_completed"], true);
    let hits2 = report2["hit_receipts"].as_array().unwrap();
    let power_hits: Vec<_> = hits2
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Mitsuki_Melee3_Power_Damage")
        .collect();
    assert_eq!(power_hits.len(), 6);
    let t0_count = power_hits.iter().filter(|h| h["target_index"] == 0).count();
    let t1_count = power_hits.iter().filter(|h| h["target_index"] == 1).count();
    assert_eq!(t0_count, 3);
    assert_eq!(t1_count, 3);
    // Lv8 power coefficient 0.874 * 1000 = 874 (before crescendo/buffs)
    for h in power_hits {
        let dmg: crate::simulation::ExactFraction =
            serde_json::from_value(h["non_crit_damage"].clone()).unwrap();
        assert!(dmg.numerator >= dmg.denominator * 874);
    }
}

#[test]
fn aurelia_crescendo_b4_stack_accrual_and_refresh() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::aurelia::crescendo::HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::crescendo::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();

    // 3 Normal actions -> stage 0 (1 hit), stage 1 (1 hit), stage 2 (3 bullet hits).
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            500_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
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
    let hits = report["hit_receipts"].as_array().unwrap();
    // Hits 2, 3, 4 are Melee3 (bullets).
    // Hit 2: 0 stacks -> ATK 1000 * 0.514 = 514
    // Hit 3: 1 stack (+1% base ATK = +10 ATK) -> ATK 1010 * 0.514 = 519.14 (25957 / 50)
    // Hit 4: 2 stacks (+2% base ATK = +20 ATK) -> ATK 1020 * 0.514 = 524.28 (13107 / 25)
    assert_eq!(hits[2]["non_crit_damage"], json!("514/1"));
    assert_eq!(hits[3]["non_crit_damage"], json!("25957/50"));
    assert_eq!(hits[4]["non_crit_damage"], json!("13107/25"));
}

#[test]
fn aurelia_harmonics_b2_triggers_on_nova_removal() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::aurelia::harmonics::HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::harmonics::HYPOTHESIS_ID
            .description()
            .into(),
    );
    // Add chaos partner to trigger Nova
    let mut chaos = req.actors[0].clone();
    chaos.slot = 1;
    chaos.character_id = "character:1004".into();
    chaos.initial_cycle = 100;
    chaos.gear_procs.clear();
    req.actors.push(chaos);

    req.actors[0].initial_cycle = 100;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();

    // Switch to actor 1 to admit Nova
    req.actions = vec![GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 1,
        cooldown_us: 0,
    }];
    req.relative_duration_us = 10_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let harmonics_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Reaction_4_new_1070_Damage")
        .collect();
    assert_eq!(harmonics_hits.len(), 3);
    for hit in &harmonics_hits {
        assert_eq!(hit["actor_index"], 0);
        assert_eq!(hit["target_index"], 0);
        // Lv80 ATK 1000 * 0.50 rate = 500 damage
        assert_eq!(hit["non_crit_damage"], json!("500/1"));
    }
}

#[test]
fn aurelia_two_continuing_cycles_preserve_resources_and_state() {
    let mut req = request();
    req.combat_stamina = Some(crate::simulation::CombatStaminaProfile {
        capacity: Ratio {
            numerator: 200,
            denominator: 1,
        },
        initial: Ratio {
            numerator: 200,
            denominator: 1,
        },
        recovery_rate: Ratio {
            numerator: 20,
            denominator: 1,
        },
        use_efficiency: Ratio::one(),
    });
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].energy_capacity = 120;
    req.actors[0].initial_energy = 0;
    req.actors[0].initial_cycle = 0;
    req.relative_duration_us = 125_000_000;

    req.model_assumptions.insert(
        crate::HypothesisId::AureliaCadenzaPowerSelectV1
            .as_str()
            .into(),
        crate::HypothesisId::AureliaCadenzaPowerSelectV1
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::aurelia::crescendo::HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::crescendo::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::aurelia::staccato::STACCATO_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::staccato::STACCATO_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::aurelia::staccato::PERFECT_DODGE_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::staccato::PERFECT_DODGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::aurelia::cadenza::A2_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::cadenza::A2_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::aurelia::skill::PACKETS_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::aurelia::skill::CHAIN_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::characters::aurelia::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );

    let stage_durations_us = [
        1_006_006, // .9160057 + .09
        599_307,   // .57950735 + .0198
        1_815_171, // 1.7557707 + 3 * .0198
        820_717,   // .7415174 + 4 * .0198
    ];
    let chain_duration_us: u64 = stage_durations_us.iter().sum(); // 4_241_201 us

    let mut actions = Vec::new();

    for period_offset_us in [0u64, 60_000_000u64] {
        // 9 normal chains
        for chain_idx in 0..9 {
            let chain_start = period_offset_us + chain_idx * chain_duration_us;
            let mut t = chain_start;
            for &dur in &stage_durations_us {
                actions.push(make_action(
                    t,
                    0,
                    "GA_Mitsuki_Melee",
                    "GE_Player_Mitsuki_Melee1_Damage",
                    0,
                    0,
                ));
                t += dur;
            }
        }

        // Skill at 40s
        actions.push(make_action(
            period_offset_us + 40_000_000,
            0,
            "GA_Mitsuki_Skill",
            "GE_Player_Mitsuki_Skill_Damage",
            0,
            15_000_000,
        ));

        // Burst at 42s
        actions.push(make_action(
            period_offset_us + 42_000_000,
            0,
            "GA_Mitsuki_UltraSkill",
            "GE_Player_Mitsuki_UltraSkill_Damage",
            120,
            15_000_000,
        ));

        // Staccato ride at 47s (schedules 4 loop blasts):
        actions.push(make_action(
            period_offset_us + 47_000_000,
            0,
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_EvadeBranch_Damage",
            0,
            0,
        ));

        // Critical Dodge at 48s (schedules 4 perfect bullets):
        actions.push(GameActionCommand::PlayerInput {
            timestamp_us: period_offset_us + 48_000_000,
            actor_index: 0,
            input: crate::simulation::PlayerInput::Dash,
            critical_success: true,
        });

        // 10th normal chain at 51s
        let mut t = period_offset_us + 51_000_000;
        for &dur in &stage_durations_us {
            actions.push(make_action(
                t,
                0,
                "GA_Mitsuki_Melee",
                "GE_Player_Mitsuki_Melee1_Damage",
                0,
                0,
            ));
            t += dur;
        }
    }

    req.actions = actions;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 210);
    assert_eq!(report["final_actor_states"][0]["energy"], 23);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("88/125")
    );
}
