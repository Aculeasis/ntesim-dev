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
    req.actors[0].character_id = "character:1020".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].static_stats.base_atk = 0;
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
        "GA_Haniel_Melee",
        "GE_Player_Haniel_Melee1_Damage",
        0,
        0,
    )];
    let hootie = crate::characters::haniel::hootie::HYPOTHESIS;
    req.model_assumptions
        .insert(hootie.into(), hootie.description().into());
    req
}

#[test]
fn haniel_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, coefficients, total) in [
        (
            1,
            0,
            [419, 112, 112, 582, 156, 156, 156, 156, 528, 419],
            1398,
        ),
        (
            8,
            0,
            [718, 192, 192, 997, 267, 267, 267, 267, 905, 718],
            2395,
        ),
        (
            8,
            3,
            [776, 207, 207, 1077, 289, 289, 289, 289, 977, 776],
            2588,
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
                    "GA_Haniel_Melee",
                    "GE_Player_Haniel_Melee1_Damage",
                    0,
                    0,
                )
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}; {}",
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
            json!(format!("{}/{}", total, 1))
        );
        let stages = [1, 2, 2, 3, 4, 4, 4, 4, 5, 1];
        for (hit, stage) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(stages)
        {
            assert_eq!(
                hit["damage_effect_id"],
                format!("GE_Player_Haniel_Melee{stage}_Damage")
            );
        }
    }
}

#[test]
fn haniel_normal_chain_reset_and_noop_boundaries() {
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
                "GA_Haniel_Skill",
                "GE_Player_Haniel_Skill1_Damage",
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
            "GA_Haniel_Melee",
            "GE_Player_Haniel_Melee1_Damage",
            0,
            0,
        ));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}; {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let expected = if boundary == 1 || boundary == 4 {
            "GE_Player_Haniel_Melee2_Damage"
        } else {
            "GE_Player_Haniel_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn haniel_normal_chain_strict_hypothesis_and_off_field_four_hit_delivery() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::HANIEL_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HANIEL_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::haniel::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::haniel::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = (0..4)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Haniel_Melee",
                "GE_Player_Haniel_Melee1_Damage",
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
    assert_eq!(pending["hits_settled"], 4);
    req.relative_duration_us = 3100;
    let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(delivered["hits_settled"], 8);
    for hit in &delivered["hit_receipts"].as_array().unwrap()[4..] {
        assert_eq!(hit["actor_index"], 0);
        assert_eq!(hit["damage_effect_id"], "GE_Player_Haniel_Melee4_Damage");
    }
}

#[test]
fn haniel_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Haniel_Melee / GE_Player_Haniel_Melee1_Damage)
    // Coeffs: Lv1 = 0.419, Lv8 = 0.718, Lv9 (A3 resonance) = 0.776.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 0.419 * 0.5 = 419 / 2
    //   Lv8: 1000 * 0.718 * 0.5 = 359 / 1
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 0.776 * 0.5 = 388 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 419, 2),
        (8, 0, 359, 1),
        (8, 1, 359, 1),
        (8, 3, 388, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}; {}",
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

    // Skill (GA_Haniel_Skill / GE_Player_Haniel_Skill1_Damage + 1 child packet)
    // Coeffs: Lv1 = 1.000, Lv8 = 1.714, Lv9 (A3 resonance) = 1.851.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 1.000 * 0.5 = 500 / 1 per hit -> total 2 * 500 = 1000 / 1
    //   Lv8: 1000 * 1.714 * 0.5 = 857 / 1 per hit -> total 2 * 857 = 1714 / 1
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 1.851 * 0.5 = 1851 / 2 per hit -> total 2 * (1851 / 2) = 1851 / 1
    for (level, awakening, expected_num, expected_den, total_num, total_den) in [
        (1, 0, 500, 1, 1000, 1),
        (8, 0, 857, 1, 1714, 1),
        (8, 3, 1851, 2, 1851, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Haniel_Skill",
            "GE_Player_Haniel_Skill1_Damage",
            0,
            12_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}; {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 2);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 2);
        for receipt in receipts {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", expected_num, expected_den))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                "GE_Player_Haniel_Skill1_Damage"
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

    // Burst (GA_Haniel_UltraSkill / GE_Player_Haniel_Ultraskill1_Damage)
    // Coeffs: Lv1 = 3.000, Lv8 = 5.141, Lv9 (A3 resonance) = 5.553.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 3.000 * 0.5 = 1500 / 1
    //   Lv8: 1000 * 5.141 * 0.5 = 5141 / 2
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 5.553 * 0.5 = 5553 / 2
    for (level, awakening, expected_num, expected_den) in
        [(1, 0, 1500, 1), (8, 0, 5141, 2), (8, 3, 5553, 2)]
    {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Haniel_UltraSkill",
            "GE_Player_Haniel_Ultraskill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}; {}",
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
fn haniel_a3_crit_rate_and_a5_psyche_damage_buff_on_burst_cast() {
    let mut req_a3 = request();
    req_a3.actors[0].talent_levels = Some([1, 1, 8]);
    req_a3.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    req_a3.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req_a3.actions = vec![make_action(
        0,
        0,
        "GA_Haniel_UltraSkill",
        "GE_Player_Haniel_Ultraskill1_Damage",
        100,
        20_000_000,
    )];
    let report_a3 = run_game_scenario(&req_a3, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_a3["execution_completed"], true);
    assert_eq!(report_a3["hits_settled"], 1);
    assert_eq!(report_a3["hit_receipts"][0]["crit_rate"], json!("1/4"));

    // At A5: on_cast of GA_Haniel_UltraSkill additionally grants +30% Psyche DMG bonus
    // Lv9 coeff (A3 resonance) = 5.553.
    // 1000 * 5.553 * 0.5 * (1 + 3/10) = (5553 / 2) * (13 / 10) = 72189 / 20.
    let mut req_a5 = request();
    req_a5.actors[0].talent_levels = Some([1, 1, 8]);
    req_a5.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req_a5.actions = vec![make_action(
        0,
        0,
        "GA_Haniel_UltraSkill",
        "GE_Player_Haniel_Ultraskill1_Damage",
        100,
        20_000_000,
    )];
    let report_a5 = run_game_scenario(&req_a5, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_a5["execution_completed"], true);
    assert_eq!(report_a5["hits_settled"], 1);
    assert_eq!(
        report_a5["hit_receipts"][0]["non_crit_damage"],
        json!("72189/20")
    );
}

#[test]
fn haniel_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Haniel_Skill",
            "GE_Player_Haniel_Skill1_Damage",
            0,
            12_000_000,
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
        "{}; {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
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
fn haniel_strict_mode_rejects_missing_hypothesis_and_accepts_valid_hypothesis() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::HANIEL_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HANIEL_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Haniel_Skill",
        "GE_Player_Haniel_Skill1_Damage",
        0,
        12_000_000,
    )];
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::haniel::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::haniel::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::haniel::skill::TEAM_ATK_CLOCK_HYPOTHESIS_ID.into(),
        crate::characters::haniel::skill::TEAM_ATK_CLOCK_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::haniel::hootie::HYPOTHESIS.into(),
        crate::characters::haniel::hootie::HYPOTHESIS
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}; {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
}

#[test]
fn haniel_two_skill_one_burst_rotation_hit_count_and_energy() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.relative_duration_us = 35_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Haniel_Skill",
            "GE_Player_Haniel_Skill1_Damage",
            0,
            12_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Haniel_UltraSkill",
            "GE_Player_Haniel_Ultraskill1_Damage",
            120,
            20_000_000,
        ),
        make_action(
            16_237_614,
            0,
            "GA_Haniel_Skill",
            "GE_Player_Haniel_Skill1_Damage",
            0,
            12_000_000,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}; {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 5);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
}

#[test]
fn haniel_team_atk_uses_source_base_atk_adds_and_pauses_hootie_clock() {
    let mut req = request();
    req.relative_duration_us = 4_000_000;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1700,
        denominator: 1,
    };
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.static_stats.base_atk = 500;
    ally.static_stats.atk = Ratio {
        numerator: 900,
        denominator: 1,
    };
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Haniel_Skill",
            "GE_Player_Haniel_Skill1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Haniel_UltraSkill",
            "GE_Player_Haniel_Ultraskill1_Damage",
            0,
            0,
        ),
        make_action(
            3_000_000,
            0,
            "GA_Haniel_Melee",
            "GE_Player_Haniel_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 3_500_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}; {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 4);
    assert_eq!(hits[0]["non_crit_damage"], json!("850/1"));
    assert_eq!(hits[1]["non_crit_damage"], json!("850/1"));
    assert_eq!(hits[2]["non_crit_damage"], json!("2850/1"));
    assert_eq!(hits[3]["non_crit_damage"], json!("7961/20"));
    let snapshot = &report["final_buff_state"];
    assert_eq!(snapshot["active_actor_index"], 1);
    let instances = crate::simulation::tests::support::final_buffs(&report);
    let hootie = instances
        .iter()
        .find(|buff| buff["id"] == "haniel.hootie_team_atk")
        .unwrap();
    let ace = instances
        .iter()
        .find(|buff| buff["id"] == "haniel.paranormal_ace_team_atk")
        .unwrap();
    for buff in [hootie, ace] {
        assert_eq!(buff["owner_actor"], 0);
        assert_eq!(buff["recipient"], json!({"type":"team_all"}));
        assert_eq!(buff["stacking_scope"], "per_source");
        assert_eq!(buff["stacks"], 1);
    }
    assert_eq!(hootie["duration"]["expires_at_us"], 18_000_100);
    assert_eq!(hootie["stat_modifiers"]["atk_add"], json!("100/1"));
    assert_eq!(ace["duration"]["expires_at_us"], 12_000_000);
    assert_eq!(ace["stat_modifiers"]["atk_add"], json!("100/1"));
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::haniel::skill::TEAM_ATK_CLOCK_HYPOTHESIS_ID.as_str()],
        crate::characters::haniel::skill::TEAM_ATK_CLOCK_HYPOTHESIS_ID.description()
    );
}

#[test]
fn haniel_skill_refresh_and_burst_pause_preserve_hootie_remaining_time() {
    for (actions, expected_expiry) in [
        (
            vec![
                make_action(
                    0,
                    0,
                    "GA_Haniel_Skill",
                    "GE_Player_Haniel_Skill1_Damage",
                    0,
                    0,
                ),
                make_action(
                    2_000_000,
                    0,
                    "GA_Haniel_UltraSkill",
                    "GE_Player_Haniel_Ultraskill1_Damage",
                    0,
                    0,
                ),
            ],
            18_000_100,
        ),
        (
            vec![
                make_action(
                    0,
                    0,
                    "GA_Haniel_Skill",
                    "GE_Player_Haniel_Skill1_Damage",
                    0,
                    0,
                ),
                make_action(
                    2_000_000,
                    0,
                    "GA_Haniel_UltraSkill",
                    "GE_Player_Haniel_Ultraskill1_Damage",
                    0,
                    0,
                ),
                make_action(
                    4_000_000,
                    0,
                    "GA_Haniel_Skill",
                    "GE_Player_Haniel_Skill1_Damage",
                    0,
                    0,
                ),
            ],
            20_000_000,
        ),
        (
            vec![
                make_action(
                    0,
                    0,
                    "GA_Haniel_Skill",
                    "GE_Player_Haniel_Skill1_Damage",
                    0,
                    0,
                ),
                make_action(
                    2_000_000,
                    0,
                    "GA_Haniel_UltraSkill",
                    "GE_Player_Haniel_Ultraskill1_Damage",
                    0,
                    0,
                ),
                make_action(
                    8_000_000,
                    0,
                    "GA_Haniel_UltraSkill",
                    "GE_Player_Haniel_Ultraskill1_Damage",
                    0,
                    0,
                ),
            ],
            19_762_486,
        ),
    ] {
        let mut req = request();
        req.relative_duration_us = 9_000_000;
        req.actors[0].static_stats.base_atk = 1000;
        req.actions = actions;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let buffs = crate::simulation::tests::support::final_buffs(&report);
        let hootie = buffs
            .iter()
            .find(|buff| buff["id"] == "haniel.hootie_team_atk")
            .unwrap();
        assert_eq!(hootie["duration"]["expires_at_us"], expected_expiry);
        assert_eq!(hootie["stat_modifiers"]["atk_add"], json!("100/1"));
    }
}

#[test]
fn haniel_burst_at_hootie_expiry_does_not_resurrect_skill_grant() {
    let mut req = request();
    req.relative_duration_us = 9_000_000;
    req.actors[0].static_stats.base_atk = 1000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Haniel_Skill",
            "GE_Player_Haniel_Skill1_Damage",
            0,
            0,
        ),
        make_action(
            8_000_100,
            0,
            "GA_Haniel_UltraSkill",
            "GE_Player_Haniel_Ultraskill1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let buffs = crate::simulation::tests::support::final_buffs(&report);
    assert!(
        buffs
            .iter()
            .all(|buff| buff["id"] != "haniel.hootie_team_atk")
    );
    assert!(
        buffs
            .iter()
            .any(|buff| buff["id"] == "haniel.paranormal_ace_team_atk")
    );
}

#[test]
fn haniel_a2_extends_hootie_and_does_not_extend_paranormal_ace() {
    let mut req = request();
    req.relative_duration_us = 3_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].static_stats.base_atk = 1000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Haniel_Skill",
            "GE_Player_Haniel_Skill1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Haniel_UltraSkill",
            "GE_Player_Haniel_Ultraskill1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let buffs = crate::simulation::tests::support::final_buffs(&report);
    let hootie = buffs
        .iter()
        .find(|buff| buff["id"] == "haniel.hootie_team_atk")
        .unwrap();
    let ace = buffs
        .iter()
        .find(|buff| buff["id"] == "haniel.paranormal_ace_team_atk")
        .unwrap();
    assert_eq!(hootie["duration"]["expires_at_us"], 22_000_100);
    assert_eq!(ace["duration"]["expires_at_us"], 12_000_000);
    assert_eq!(
        buffs
            .iter()
            .find(|buff| buff["id"] == "haniel.paranormal_ace_team_atk")
            .unwrap()["duration"]["expires_at_us"],
        12_000_000
    );

    req.relative_duration_us = 5_000_000;
    req.actions.push(make_action(
        4_000_000,
        0,
        "GA_Haniel_Skill",
        "GE_Player_Haniel_Skill1_Damage",
        0,
        0,
    ));
    let refreshed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let refreshed_buffs = crate::simulation::tests::support::final_buffs(&refreshed);
    assert_eq!(
        refreshed_buffs
            .iter()
            .find(|buff| buff["id"] == "haniel.hootie_team_atk")
            .unwrap()["duration"]["expires_at_us"],
        24_000_000
    );
    assert_eq!(
        refreshed_buffs
            .iter()
            .find(|buff| buff["id"] == "haniel.paranormal_ace_team_atk")
            .unwrap()["duration"]["expires_at_us"],
        12_000_000
    );
}

#[test]
fn haniel_skill_and_burst_atk_curves_follow_effective_talent_level() {
    for (level, awakening, grant) in [(1, 0, 100), (8, 0, 170), (8, 3, 180)] {
        for (ability, packet, id) in [
            (
                "GA_Haniel_Skill",
                "GE_Player_Haniel_Skill1_Damage",
                "haniel.hootie_team_atk",
            ),
            (
                "GA_Haniel_UltraSkill",
                "GE_Player_Haniel_Ultraskill1_Damage",
                "haniel.paranormal_ace_team_atk",
            ),
        ] {
            let mut req = request();
            req.relative_duration_us = 1_000_000;
            req.actors[0].static_stats.base_atk = 1000;
            req.actors[0].talent_levels = Some([1, level, level]);
            req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
            req.actions = vec![make_action(0, 0, ability, packet, 0, 0)];
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            let buffs = crate::simulation::tests::support::final_buffs(&report);
            let buff = buffs.iter().find(|buff| buff["id"] == id).unwrap();
            assert_eq!(
                buff["stat_modifiers"]["atk_add"],
                json!(format!("{}/{}", grant, 1))
            );
            assert_eq!(
                buff["duration"]["expires_at_us"],
                if id == "haniel.hootie_team_atk" && awakening >= 2 {
                    12_000_100
                } else if id == "haniel.hootie_team_atk" {
                    8_000_100
                } else {
                    10_000_000
                }
            );
        }
    }
}

#[test]
fn haniel_failed_burst_admission_grants_no_atk_or_clock() {
    let mut req = request();
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].initial_energy = 0;
    let mut burst = make_action(
        0,
        0,
        "GA_Haniel_UltraSkill",
        "GE_Player_Haniel_Ultraskill1_Damage",
        100,
        0,
    );
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut burst {
        *energy_cost = 100;
    }
    req.actions = vec![burst];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(crate::simulation::tests::support::final_buffs(&report).is_empty());
}

#[test]
fn haniel_skill_burst_and_paused_hootie_deadlines_reject_overflow() {
    let max = u64::MAX;
    let no_delay = |mut action: GameActionCommand| {
        if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut action {
            *hit_delay_us = 0;
        }
        action
    };
    let cases = [
        (
            vec![no_delay(make_action(
                max - 5_000_000,
                0,
                "GA_Haniel_Skill",
                "GE_Player_Haniel_Skill1_Damage",
                0,
                0,
            ))],
            "Haniel Hootie deadline overflow",
        ),
        (
            vec![no_delay(make_action(
                max - 5_000_000,
                0,
                "GA_Haniel_UltraSkill",
                "GE_Player_Haniel_Ultraskill1_Damage",
                0,
                0,
            ))],
            "configured kit buff deadline overflow",
        ),
        (
            vec![
                no_delay(make_action(
                    max - 17_000_000,
                    0,
                    "GA_Haniel_Skill",
                    "GE_Player_Haniel_Skill1_Damage",
                    0,
                    0,
                )),
                no_delay(make_action(
                    max - 12_000_000,
                    0,
                    "GA_Haniel_UltraSkill",
                    "GE_Player_Haniel_Ultraskill1_Damage",
                    0,
                    0,
                )),
            ],
            "Haniel Hootie deadline overflow",
        ),
    ];
    let limits = ExecutionLimits {
        max_horizon_us: max,
        ..ExecutionLimits::default()
    };
    for (actions, message) in cases {
        let mut req = request();
        req.relative_duration_us = max;
        req.absolute_limit_us = req.relative_duration_us;
        req.actions = actions;
        let error = run_game_scenario(&req, &limits).unwrap_err();
        assert!(error.contains(message), "expected {message}, got {error}");
    }
}
