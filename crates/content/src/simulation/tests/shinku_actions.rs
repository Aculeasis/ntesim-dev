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

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1076".into();
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
        "GA_Shinku_Melee",
        "GE_Player_Shinku_Melee1_1_Damage",
        0,
        0,
    )];
    req
}

fn fraction_json(numerator: u128, denominator: u128) -> serde_json::Value {
    let fraction = ExactFraction::new(numerator, denominator).unwrap();
    serde_json::to_value(fraction).unwrap()
}

fn shinku_base_packet_damage(rate_milli: u128, awakening: u32) -> serde_json::Value {
    let (def_num, def_den) = if awakening >= 1 { (25, 47) } else { (1, 2) };
    fraction_json(rate_milli * def_num, def_den)
}

fn shinku_rage_packet_damage(rate_milli: u128, awakening: u32) -> serde_json::Value {
    let (bonus_num, bonus_den, def_num, def_den) = if awakening >= 3 {
        (8, 5, 25, 47)
    } else if awakening >= 1 {
        (13, 10, 25, 47)
    } else {
        (13, 10, 1, 2)
    };
    fraction_json(rate_milli * bonus_num * def_num, bonus_den * def_den)
}

#[test]
fn shinku_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Shinku_Melee / GE_Player_Shinku_Melee1_1_Damage)
    // Coeffs: Lv1 = 0.182, Lv8 = 0.312, Lv9 (A3 resonance) = 0.337
    // Normal attack does not activate Surging Crimson; damage bonus remains 0.
    // DEF factor at A0 (0% ignore) = 1080 / (1080 + 1080) = 1/2.
    // DEF factor at A1/A3 (12% ignore) = 1080 / (1080 + 1080 * 22/25) = 25/47.
    // Lv1, A0: 1000 * 0.182 * (1/2) = 91 / 1
    // Lv8, A0: 1000 * 0.312 * (1/2) = 156 / 1
    // Lv8, A1: 1000 * 0.312 * (25/47) = 312 * 25 / 47 = 7800 / 47
    // Lv8, A3 (Lv9 = 0.337): 1000 * 0.337 * (25/47) = 337 * 25 / 47 = 8425 / 47
    // Lv8, A4 (Lv9 = 0.337): 1000 * 0.337 * (25/47) = 8425 / 47
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 91, 1),
        (8, 0, 156, 1),
        (8, 1, 7800, 47),
        (8, 3, 8425, 47),
        (8, 4, 8425, 47),
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

    // Skill (GA_Shinku_Skill / GE_Player_Shinku_Skill1_1_Damage)
    // Coeffs: Lv1 = 0.173, Lv8 = 0.296, Lv9 (A3 resonance) = 0.320
    // Ordinary Skill does not activate Surging Crimson; damage bonus remains 0.
    // Lv1, A0: 1000 * 0.173 * (1/2) = 173 / 2
    // Lv8, A0: 1000 * 0.296 * (1/2) = 148 / 1
    // Lv8, A3: 1000 * 0.320 * (25/47) = 8000 / 47
    for (level, awakening, expected_num, expected_den) in
        [(1, 0, 173, 2), (8, 0, 148, 1), (8, 3, 8000, 47)]
    {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
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
        let skill_packets = [
            ("GE_Player_Shinku_Skill1_1_Damage", 1),
            ("GE_Player_Shinku_Skill1_2_Damage", 1),
            ("GE_Player_Shinku_Skill1_3_Damage", 1),
        ];
        let skill_rates = if awakening >= 3 {
            [320, 428, 1473]
        } else if level == 1 {
            [173, 231, 796]
        } else {
            [296, 396, 1364]
        };
        for ((receipt, (packet_id, _)), rate) in receipts.iter().zip(skill_packets).zip(skill_rates)
        {
            assert_eq!(receipt["damage_effect_id"], packet_id);
            assert_eq!(receipt["actor_index"], 0);
            assert_eq!(
                receipt["non_crit_damage"],
                shinku_base_packet_damage(rate, awakening)
            );
        }
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
    }

    // Burst (GA_Shinku_UltraSkill / GE_Player_Shinku_UltraSkill1_Damage)
    // Coeffs: Lv1 = 0.185, Lv8 = 0.317, Lv9 (A3 resonance) = 0.342
    // on_cast activates Surging Crimson (13s):
    //   Base: +30% DMG bonus (13/10)
    //   At A3: +30% additional DMG bonus -> +60% total (8/5)
    // Lv1, A0: 1000 * 0.185 * (13/10) * (1/2) = 185 * 13 / 20 = 481 / 4
    // Lv8, A0: 1000 * 0.317 * (13/10) * (1/2) = 317 * 13 / 20 = 4121 / 20
    // Lv8, A3: 1000 * 0.342 * (8/5) * (25/47) = 342 * (8/5) * (25/47) = 342 * 40 / 47 = 13680 / 47
    for (level, awakening, expected_num, expected_den) in
        [(1, 0, 481, 4), (8, 0, 4121, 20), (8, 3, 13680, 47)]
    {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 9);
        let receipts = report["hit_receipts"].as_array().unwrap();
        let burst_packets = [
            ("GE_Player_Shinku_UltraSkill1_Damage", 1),
            ("GE_Player_Shinku_UltraSkill2_Damage", 1),
            ("GE_Player_Shinku_UltraSkill3_Damage", 5),
            ("GE_Player_Shinku_UltraSkill4_Damage", 1),
            ("GE_Player_Shinku_UltraSkill5_Damage", 1),
        ];
        let burst_rates = if awakening >= 3 {
            [342, 2221, 1018, 2665, 787]
        } else if level == 1 {
            [185, 1200, 550, 1440, 425]
        } else {
            [317, 2057, 943, 2468, 728]
        };
        let mut receipt_index = 0;
        for ((packet_id, multiplicity), rate) in burst_packets.into_iter().zip(burst_rates) {
            for _ in 0..multiplicity {
                let receipt = &receipts[receipt_index];
                receipt_index += 1;
                assert_eq!(receipt["damage_effect_id"], packet_id);
                assert_eq!(receipt["actor_index"], 0);
                let base_damage = shinku_rage_packet_damage(rate, awakening);
                let expected_damage = if awakening >= 2 && receipt_index > 1 {
                    let base: ExactFraction = serde_json::from_value(base_damage).unwrap();
                    fraction_json(base.numerator * 12, base.denominator * 11)
                } else {
                    base_damage
                };
                assert_eq!(receipt["non_crit_damage"], expected_damage);
            }
        }
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
fn shinku_a6_applies_defense_ignore_and_rage_skill_ratio_bonus() {
    // At A6 with resonance_6:
    // DEF ignore: 12% (A1) + 15% (resonance 6) = 27% (27/100).
    // DEF factor = 1080 / (1080 + 1080 * 73/100) = 100 / 173.
    // Ordinary Skill does not enter rage mode, so ratio is unmodified Lv9: 0.320 = 16/50.
    // Non-crit damage = 1000 * (16/50) * (100/173) = 320 * (100/173) = 32000 / 173.
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Shinku_Skill",
        "GE_Player_Shinku_Skill1_1_Damage",
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
    for (receipt, (packet_id, rate_milli)) in receipts.iter().zip([
        ("GE_Player_Shinku_Skill1_1_Damage", 320),
        ("GE_Player_Shinku_Skill1_2_Damage", 428),
        ("GE_Player_Shinku_Skill1_3_Damage", 1473),
    ]) {
        assert_eq!(receipt["damage_effect_id"], packet_id);
        assert_eq!(
            receipt["non_crit_damage"],
            fraction_json(rate_milli * 100, 173)
        );
    }
    assert_eq!(receipts[0]["non_crit_damage"], json!("32000/173"));
}

#[test]
fn shinku_burst_applies_a2_resistance_reduction_after_its_first_same_time_hit() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Shinku_UltraSkill",
        "GE_Player_Shinku_UltraSkill1_Damage",
        100,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 9);
    let receipts = report["hit_receipts"].as_array().unwrap();
    let packet_schedule = [
        ("GE_Player_Shinku_UltraSkill1_Damage", 185, 1),
        ("GE_Player_Shinku_UltraSkill2_Damage", 1200, 1),
        ("GE_Player_Shinku_UltraSkill3_Damage", 550, 5),
        ("GE_Player_Shinku_UltraSkill4_Damage", 1440, 1),
        ("GE_Player_Shinku_UltraSkill5_Damage", 425, 1),
    ];
    let mut index = 0;
    for (packet_id, rate_milli, multiplicity) in packet_schedule {
        for _ in 0..multiplicity {
            let receipt = &receipts[index];
            let base_damage = shinku_rage_packet_damage(rate_milli, 2);
            let expected = if index == 0 {
                base_damage
            } else {
                let base: ExactFraction = serde_json::from_value(base_damage).unwrap();
                fraction_json(base.numerator * 12, base.denominator * 11)
            };
            assert_eq!(receipt["time_us"], 100);
            assert_eq!(receipt["damage_effect_id"], packet_id);
            assert_eq!(receipt["non_crit_damage"], expected);
            index += 1;
        }
    }
}

#[test]
fn shinku_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.active_actor_index = 0;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            0,
            6_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 50,
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
    for (receipt, packet_id) in report["hit_receipts"].as_array().unwrap().iter().zip([
        "GE_Player_Shinku_Skill1_1_Damage",
        "GE_Player_Shinku_Skill1_2_Damage",
        "GE_Player_Shinku_Skill1_3_Damage",
    ]) {
        assert_eq!(receipt["actor_index"], 0);
        assert_eq!(receipt["damage_effect_id"], packet_id);
    }

    let mut dead_target_req = request();
    dead_target_req.actions = vec![make_action(
        0,
        0,
        "GA_Shinku_UltraSkill",
        "GE_Player_Shinku_UltraSkill1_Damage",
        100,
        20_000_000,
    )];
    dead_target_req.relative_duration_us = 100;
    dead_target_req.targets[0].hp_current = 0;
    let dead_report = run_game_scenario(&dead_target_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead_report["execution_completed"], true);
    assert_eq!(dead_report["hits_settled"], 0);
    assert_eq!(dead_report["pending_events"], 0);

    let mut pending_req = request();
    pending_req.actions = vec![make_action(
        0,
        0,
        "GA_Shinku_Skill",
        "GE_Player_Shinku_Skill1_1_Damage",
        0,
        6_000_000,
    )];
    pending_req.relative_duration_us = 99;
    let pending = run_game_scenario(&pending_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["execution_completed"], true);
    assert_eq!(pending["hits_settled"], 0);
    assert_eq!(pending["pending_events"], 3);
    pending_req.relative_duration_us = 100;
    let settled = run_game_scenario(&pending_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(settled["execution_completed"], true);
    assert_eq!(settled["hits_settled"], 3);
}

#[test]
fn shinku_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Shinku_Melee1_1_Damage", 91, 1),
                ("GE_Player_Shinku_Melee1_2_Damage", 154, 1),
                ("GE_Player_Shinku_Melee1_2_1_Damage", 205, 2),
                ("GE_Player_Shinku_Melee1_3_Damage", 147, 1),
                ("GE_Player_Shinku_Melee1_3_1_Damage", 87, 1),
                ("GE_Player_Shinku_Melee1_3_2_Damage", 197, 2),
                ("GE_Player_Shinku_Melee1_3_3_Damage", 421, 2),
                ("GE_Player_Shinku_Melee1_4_Damage", 815, 2),
                ("GE_Player_Shinku_Melee1_4_1_Damage", 249, 1),
                ("GE_Player_Shinku_Melee1_5_Damage", 1129, 2),
                ("GE_Player_Shinku_Melee1_1_Damage", 91, 1),
            ],
            4405,
            2,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Shinku_Melee1_1_Damage", 156, 1),
                ("GE_Player_Shinku_Melee1_2_Damage", 264, 1),
                ("GE_Player_Shinku_Melee1_2_1_Damage", 351, 2),
                ("GE_Player_Shinku_Melee1_3_Damage", 252, 1),
                ("GE_Player_Shinku_Melee1_3_1_Damage", 149, 1),
                ("GE_Player_Shinku_Melee1_3_2_Damage", 169, 1),
                ("GE_Player_Shinku_Melee1_3_3_Damage", 361, 1),
                ("GE_Player_Shinku_Melee1_4_Damage", 1397, 2),
                ("GE_Player_Shinku_Melee1_4_1_Damage", 853, 2),
                ("GE_Player_Shinku_Melee1_5_Damage", 1935, 2),
                ("GE_Player_Shinku_Melee1_1_Damage", 156, 1),
            ],
            3775,
            1,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Shinku_Melee1_1_Damage", 8425, 47),
                ("GE_Player_Shinku_Melee1_2_Damage", 14250, 47),
                ("GE_Player_Shinku_Melee1_2_1_Damage", 9475, 47),
                ("GE_Player_Shinku_Melee1_3_Damage", 13600, 47),
                ("GE_Player_Shinku_Melee1_3_1_Damage", 8050, 47),
                ("GE_Player_Shinku_Melee1_3_2_Damage", 9125, 47),
                ("GE_Player_Shinku_Melee1_3_3_Damage", 19475, 47),
                ("GE_Player_Shinku_Melee1_4_Damage", 37725, 47),
                ("GE_Player_Shinku_Melee1_4_1_Damage", 23050, 47),
                ("GE_Player_Shinku_Melee1_5_Damage", 52250, 47),
                ("GE_Player_Shinku_Melee1_1_Damage", 8425, 47),
            ],
            203850,
            47,
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
                    "GA_Shinku_Melee",
                    "GE_Player_Shinku_Melee1_1_Damage",
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
fn shinku_normal_chain_reset_boundaries_and_noop_switch() {
    let mut req = request();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_001,
            0,
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_1_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_1_Damage"
    );

    let mut req_cont = request();
    req_cont.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
    ];
    let report_cont = run_game_scenario(&req_cont, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_cont["hits_settled"], 3);
    let receipts_cont = report_cont["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_cont[0]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_1_Damage"
    );
    assert_eq!(
        receipts_cont[1]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_2_Damage"
    );
    assert_eq!(
        receipts_cont[2]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_2_1_Damage"
    );

    let mut req_skill_reset = request();
    req_skill_reset.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
        make_action(
            500_000,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            0,
            0,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
    ];
    let report_skill = run_game_scenario(&req_skill_reset, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_skill["hits_settled"], 5);
    let receipts_skill = report_skill["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_skill[0]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_1_Damage"
    );
    assert_eq!(
        receipts_skill[1]["damage_effect_id"],
        "GE_Player_Shinku_Skill1_1_Damage"
    );
    assert_eq!(
        receipts_skill[2]["damage_effect_id"],
        "GE_Player_Shinku_Skill1_2_Damage"
    );
    assert_eq!(
        receipts_skill[3]["damage_effect_id"],
        "GE_Player_Shinku_Skill1_3_Damage"
    );
    assert_eq!(
        receipts_skill[4]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_1_Damage"
    );

    let mut req_noop = request();
    req_noop.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
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
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
    ];
    let report_noop = run_game_scenario(&req_noop, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_noop["hits_settled"], 3);
    let receipts_noop = report_noop["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_noop[0]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_1_Damage"
    );
    assert_eq!(
        receipts_noop[1]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_2_Damage"
    );
    assert_eq!(
        receipts_noop[2]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_2_1_Damage"
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
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
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
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
    ];
    let report_exit = run_game_scenario(&req_exit, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_exit["hits_settled"], 2);
    let receipts_exit = report_exit["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_exit[0]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_1_Damage"
    );
    assert_eq!(
        receipts_exit[1]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_1_Damage"
    );
}

#[test]
fn shinku_normal_chain_strict_and_pending_final_stage() {
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
        crate::characters::shinku::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::SHINKU_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::SHINKU_HIT_ENERGY_HYPOTHESIS_ID
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
    req_exit.actions = (0..3)
        .map(|index| {
            make_action(
                index * 100_000,
                0,
                "GA_Shinku_Melee",
                "GE_Player_Shinku_Melee1_1_Damage",
                0,
                0,
            )
        })
        .collect();
    req_exit.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 200_050,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req_exit.relative_duration_us = 200_150;
    let report_exit = run_game_scenario(&req_exit, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_exit["execution_completed"], true);
    assert_eq!(report_exit["hits_settled"], 7);
    let receipts = report_exit["hit_receipts"].as_array().unwrap();
    for receipt in &receipts[3..7] {
        assert_eq!(receipt["actor_index"], 0);
    }
    assert_eq!(
        receipts[3]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_3_Damage"
    );
    assert_eq!(
        receipts[4]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_3_1_Damage"
    );
    assert_eq!(
        receipts[5]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_3_2_Damage"
    );
    assert_eq!(
        receipts[6]["damage_effect_id"],
        "GE_Player_Shinku_Melee1_3_3_Damage"
    );
}

#[test]
fn shinku_skill_and_burst_packet_hypotheses_are_required_and_strictly_validated() {
    let role_mapping = || {
        (
            ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
            ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
                .description()
                .into(),
        )
    };
    for (ability, primary, hypothesis_id, hypothesis, expected_hits) in [
        (
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            crate::characters::shinku::skill::SKILL_PACKETS_HYPOTHESIS_ID,
            crate::characters::shinku::skill::SKILL_PACKETS_HYPOTHESIS_ID.description(),
            3,
        ),
        (
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            crate::characters::shinku::skill::BURST_PACKETS_HYPOTHESIS_ID,
            crate::characters::shinku::skill::BURST_PACKETS_HYPOTHESIS_ID.description(),
            9,
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = None;
        req.actors[0].purchased_skill_level = 8;
        req.actions = vec![make_action(0, 0, ability, primary, 0, 0)];
        req.model_assumptions
            .insert(role_mapping().0, role_mapping().1);

        req.model_assumptions
            .insert(hypothesis_id.into(), hypothesis.into());
        if ability == "GA_Shinku_Skill" {
            req.model_assumptions.insert(
                crate::simulation::energy::SHINKU_HIT_ENERGY_HYPOTHESIS_ID.into(),
                crate::simulation::energy::SHINKU_HIT_ENERGY_HYPOTHESIS_ID
                    .description()
                    .into(),
            );
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], expected_hits);
        assert!(report["resolved_model_assumptions"][hypothesis_id.as_str()].is_string());

        req.model_assumptions
            .insert(hypothesis_id.into(), "incorrect packet description".into());
        let mismatch = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
        assert!(mismatch.contains("conflicting reached model hypothesis"));
    }
}

#[test]
fn shinku_crimson_judgment_and_dragonflame_inputs_deliver_one_talent_scaled_packet() {
    let packets: [(&str, [u16; 9]); 2] = [
        (
            "GE_Player_Shinku_UltraSkillPre_Rage_Damage",
            [600, 648, 700, 756, 816, 882, 952, 1028, 1111],
        ),
        (
            "GE_Player_Shinku_UltraSkill_Rage_Damage",
            [200, 216, 233, 252, 272, 294, 317, 343, 370],
        ),
    ];

    for (packet_id, rates) in packets {
        for (level, rate) in (1_u32..=9).zip(rates) {
            let mut req = request();
            req.actors[0].talent_levels = Some([1, 1, level]);
            req.actors[0].initial_energy = 0;
            req.actions = vec![make_action(0, 0, "GA_Shinku_UltraSkill", packet_id, 0, 0)];
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{} {}",
                report["halt_diagnostic"], report["rejected_action"]
            );
            assert_eq!(report["hits_settled"], 1);
            assert_eq!(
                report["hit_receipts"][0]["ability_id"],
                "GA_Shinku_UltraSkill"
            );
            assert_eq!(report["hit_receipts"][0]["damage_effect_id"], packet_id);
            assert_eq!(
                report["hit_receipts"][0]["non_crit_damage"],
                fraction_json(u128::from(rate), 2)
            );
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
            assert!(
                report["resolved_model_assumptions"]
                    .get(
                        crate::characters::shinku::skill::DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID
                            .as_str()
                    )
                    .is_some()
            );
        }
    }

    let mut sequence = request();
    sequence.actors[0].initial_energy = 100;
    sequence.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            100,
            0,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkillPre_Rage_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill_Rage_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&sequence, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 11);
    assert_eq!(
        report["hit_receipts"][9]["damage_effect_id"],
        "GE_Player_Shinku_UltraSkillPre_Rage_Damage"
    );
    assert_eq!(
        report["hit_receipts"][9]["non_crit_damage"],
        fraction_json(600 * 13, 20)
    );
    assert_eq!(
        report["hit_receipts"][10]["damage_effect_id"],
        "GE_Player_Shinku_UltraSkill_Rage_Damage"
    );
    assert_eq!(
        report["hit_receipts"][10]["non_crit_damage"],
        fraction_json(200 * 13, 20)
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
}

#[test]
fn shinku_direct_followup_packet_requires_exact_strict_acceptance() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Shinku_UltraSkill",
        "GE_Player_Shinku_UltraSkillPre_Rage_Damage",
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
        crate::characters::shinku::skill::DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID
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
fn shinku_two_continuing_cycles_preserve_resources_and_state() {
    let mut req = request();
    req.relative_duration_us = 130_000_000;
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;

    // Add teammate Actor 1 to participate in rotations
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);

    req.model_assumptions.insert(
        crate::simulation::energy::SHINKU_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::SHINKU_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::shinku::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::shinku::skill::SKILL_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::SKILL_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::shinku::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::BURST_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::shinku::skill::DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::shinku::skill::PRE_DAWN_REFRESH_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::PRE_DAWN_REFRESH_HYPOTHESIS_ID
            .description()
            .into(),
    );

    let mut actions = Vec::new();
    for cycle in 0..2 {
        let base_t = cycle * 60_000_000;

        // 1. Skill cast at base_t (3 hits, starts Surging Crimson for 13s, cooldown 12s)
        actions.push(make_action(
            base_t,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            0,
            12_000_000,
        ));

        // 2. Burst cast at base_t + 2s (100 energy, 9 hits, re-arms Surging Crimson, cooldown 20s)
        actions.push(make_action(
            base_t + 2_000_000,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            100,
            20_000_000,
        ));

        // 3. Normal attack combo loop during Surging Crimson
        for stage in 0..5 {
            actions.push(make_action(
                base_t + 6_000_000 + stage * 800_000,
                0,
                "GA_Shinku_Melee",
                "GE_Player_Shinku_Melee1_1_Damage",
                0,
                0,
            ));
        }

        // 4. Switch to teammate at base_t + 11s
        actions.push(GameActionCommand::SwitchActor {
            timestamp_us: base_t + 11_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        });

        // 5. Teammate attacks
        actions.push(GameActionCommand::InvokeAbility {
            timestamp_us: base_t + 12_000_000,
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

        // 6. Switch back to Shinku at base_t + 14s
        actions.push(GameActionCommand::SwitchActor {
            timestamp_us: base_t + 14_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        });

        // 7. Second Skill cast off cooldown at base_t + 16s
        actions.push(make_action(
            base_t + 16_000_000,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            0,
            12_000_000,
        ));

        // 8. Normal attack loops to rebuild energy to 100 for the next cycle
        for loop_idx in 0..6 {
            let loop_base = base_t + 20_000_000 + loop_idx * 5_000_000;
            for stage in 0..5 {
                actions.push(make_action(
                    loop_base + stage * 800_000,
                    0,
                    "GA_Shinku_Melee",
                    "GE_Player_Shinku_Melee1_1_Damage",
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

#[test]
fn shinku_surging_crimson_full_rotation_loop() {
    let mut req = request();
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].initial_energy = 100;
    req.model_assumptions.insert(
        crate::characters::shinku::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::shinku::skill::SKILL_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::SKILL_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::shinku::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::BURST_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::shinku::skill::DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID.into(),
        crate::characters::shinku::skill::DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID
            .description()
            .into(),
    );

    let mut actions = Vec::new();

    // 1. Enter Surging Crimson with Ultimate (Crimson Fury) at t = 0 (13s duration, 9 hits)
    actions.push(make_action(
        0,
        0,
        "GA_Shinku_UltraSkill",
        "GE_Player_Shinku_UltraSkill1_Damage",
        100,
        0,
    ));

    // 2. 3 rounds of (4 basic attacks -> build 12 Fading Reason -> Scarlet Descent)
    for round in 0..3 {
        let base_us = 1_000_000 + round * 2_500_000;
        for stage in 0..4 {
            actions.push(make_action(
                base_us + stage * 500_000,
                0,
                "GA_Shinku_Melee",
                "GE_Player_Shinku_Melee1_1_Damage",
                0,
                0,
            ));
        }
        // Scarlet Descent: consumes 12 Fading Reason, grants +1 Crimson Judgment stack (2 hits)
        actions.push(make_action(
            base_us + 2_000_000,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_Rage_Damage",
            0,
            0,
        ));
    }

    // After 3 rounds: crimson_judgment_stacks = 3. Current time reaches ~8.5s.
    // 3. At t >= 10.0s (stance started at 1.0s after initial cinematic, expires at 14.0s, remaining <= 4.0s):
    // 3 Crimson Judgment dashes
    for dash in 0..3 {
        actions.push(make_action(
            10_000_000 + dash * 500_000,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkillPre_Rage_Damage",
            0,
            0,
        ));
    }

    // 4. Stacks exhausted: Dragonflame Verdict finisher at 11.5s (10 rapid flurry + 1 finisher nuke = 11 hits)
    actions.push(make_action(
        11_500_000,
        0,
        "GA_Shinku_UltraSkill",
        "GE_Player_Shinku_UltraSkill_Rage_Damage",
        0,
        20_000_000,
    ));

    req.actions = actions;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["success"], true);

    // Hit breakdown:
    // Crimson Fury initial burst: 9 hits
    // Round 0 attacks: stage 0 (1), stage 1 (2), stage 2 (4), stage 3 (2) = 9 hits
    // Round 0 Scarlet Descent: 2 hits
    // Round 1 attacks: normal stage reset by skill -> stage 0 (1), stage 1 (2), stage 2 (4), stage 3 (2) = 9 hits
    // Round 1 Scarlet Descent: 2 hits
    // Round 2 attacks: normal stage reset by skill -> stage 0 (1), stage 1 (2), stage 2 (4), stage 3 (2) = 9 hits
    // Round 2 Scarlet Descent: 2 hits
    // 3 Crimson Judgment dashes: 3 * 4 = 12 hits
    // Dragonflame Verdict finisher: 10 + 1 = 11 hits
    // Total = 9 + 9 + 2 + 9 + 2 + 9 + 2 + 12 + 11 = 65 hits!
    assert_eq!(report["hits_settled"], 65);

    // Verify finisher nuke is present and has 685.5% scaling rate at Lv8
    let receipts = report["hit_receipts"].as_array().unwrap();
    let finisher = receipts
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Shinku_UltraSkill2_Rage_Damage")
        .expect("finisher nuke packet present");
    assert_eq!(finisher["scaling_rate"], "1371/200");
}

mod multi_target;
mod pre_dawn;
