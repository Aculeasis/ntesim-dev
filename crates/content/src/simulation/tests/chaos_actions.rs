use super::*;
use serde_json::json;

#[test]
fn chaos_selected_awakenings_separate_defense_damage_crit_and_resonance() {
    // Native Burst1 Lv1 = 1.273, Lv2 = 1.375. B4 Warrant adds .3;
    // A5 adds another .2 within that same zone, A2 gives Fdef=5/9,
    // A3 adds .3 Crit DMG independently of resonance_3's talent level.
    for (selection, num, den, crit_num, crit_den) in [
        (vec![2], 16549, 18, 3, 2),
        (vec![3], 16549, 20, 9, 5),
        (vec![5], 3819, 4, 3, 2),
        (vec![6], 16549, 20, 3, 2),
        (vec![1, 4, 5], 4125, 4, 3, 2),
    ] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(selection).unwrap();
        req.actors[0].static_stats.crit_damage_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
        req.actions = vec![make_action(
            0,
            0,
            "GA_Chaos071_UltraSkill",
            "GE_Player_Chaos_UltraSkill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 5);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", num, den))
        );
        let crit = ExactFraction::new(num, den)
            .unwrap()
            .mul(ExactFraction::new(crit_num, crit_den).unwrap())
            .unwrap();
        assert_eq!(
            report["hit_receipts"][0]["crit_damage"],
            serde_json::to_value(crit).unwrap()
        );
    }
}

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
    req.actors[0].character_id = "character:1071".into();
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
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn chaos_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Chaos071_Melee / GE_Player_Chaos_Melee1_Damage)
    // Coeffs: Lv1 = 0.182, Lv8 = 0.312, Lv9 (A3 resonance) = 0.337.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 0.182 * 0.5 = 91 / 1
    //   Lv8: 1000 * 0.312 * 0.5 = 156 / 1
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 0.337 * 0.5 = 337 / 2
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 91, 1),
        (8, 0, 156, 1),
        (8, 1, 156, 1),
        (8, 3, 337, 2),
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

    // Skill (GA_Chaos071_Skill / GE_Player_Chaos_Skill1_Damage) delivers two bites:
    // Skill1 coeffs: Lv1 = 0.825, Lv8 = 1.414, Lv9 (A3 resonance) = 1.527.
    // Skill2 (child) coeffs: Lv1 = 2.175, Lv8 = 3.728, Lv9 (A3 resonance) = 4.026.
    // B4 Warrant applies before damage, multiplying 1000 ATK by 1.3 and
    // half DEF. At A3, Resonance 3 selects Lv9 and A2 changes DEF to 5/9.
    for (level, awakening, bite1_num, bite1_den, bite2_num, bite2_den) in [
        (1, 0, 2145, 4, 5655, 4),
        (8, 0, 9191, 10, 12116, 5),
        (8, 3, 6617, 6, 8723, 3),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Chaos071_Skill",
            "GE_Player_Chaos_Skill1_Damage",
            0,
            16_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 2);
        assert_eq!(
            report["hit_receipts"][0]["damage_effect_id"],
            "GE_Player_Chaos_Skill1_Damage"
        );
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", bite1_num, bite1_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        assert_eq!(
            report["hit_receipts"][1]["damage_effect_id"],
            "GE_Player_Chaos_Skill2_Damage"
        );
        assert_eq!(
            report["hit_receipts"][1]["non_crit_damage"],
            json!(format!("{}/{}", bite2_num, bite2_den))
        );
    }

    // Burst (GA_Chaos071_UltraSkill / GE_Player_Chaos_UltraSkill1_Damage)
    // Coeffs: Lv1 = 1.273, Lv8 = 2.182, Lv9 (A3 resonance) = 2.356.
    // B4 Warrant applies +30% DMG. A3 raises the talent coordinate once
    // through Resonance 3 and includes the A2 DEF factor of 5/9.
    for (level, awakening, first_num, first_den, final_num, final_den) in [
        (1, 0, 16549, 20, 37817, 20),
        (8, 0, 14183, 10, 32409, 10),
        (8, 3, 15314, 9, 34996, 9),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Chaos071_UltraSkill",
            "GE_Player_Chaos_UltraSkill1_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 5);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 5);
        for (index, receipt) in receipts.iter().enumerate() {
            let (packet, expected_num, expected_den) = if index < 4 {
                ("GE_Player_Chaos_UltraSkill1_Damage", first_num, first_den)
            } else {
                ("GE_Player_Chaos_UltraSkill2_Damage", final_num, final_den)
            };
            assert_eq!(receipt["damage_effect_id"], packet, "hit {index}");
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", expected_num, expected_den)),
                "hit {index}"
            );
            assert_eq!(receipt["actor_index"], 0);
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
    }
}

#[test]
fn chaos_a2_def_ignore_and_a5_single_target() {
    // At A2: 20% DEF ignore when attacking target with Warrant.
    // Target DEF 1080 * 0.8 = 864 -> DEF factor = 1080 / (1080 + 864) = 5 / 9.
    // Lv8 at B4: 1000 * (1.414 or 3.728) * (5/9) * 1.3.
    let mut req_a2 = request();
    req_a2.actors[0].talent_levels = Some([1, 8, 1]);
    req_a2.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req_a2.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_Skill",
        "GE_Player_Chaos_Skill1_Damage",
        0,
        16_000_000,
    )];
    let report_a2 = run_game_scenario(&req_a2, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_a2["execution_completed"], true);
    assert_eq!(report_a2["hits_settled"], 2);
    assert_eq!(
        report_a2["hit_receipts"][0]["non_crit_damage"],
        json!("9191/9")
    );
    assert_eq!(
        report_a2["hit_receipts"][1]["non_crit_damage"],
        json!("24232/9")
    );

    // A5 adds .20 against a single marked target for a total zone of 1.50.
    // Lv9 Skill at 1000 ATK: (1.527 or 4.026) * 1000 * (5/9) * 1.5.
    let mut req_a5 = request();
    req_a5.actors[0].talent_levels = Some([1, 8, 1]);
    req_a5.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(5);
    req_a5.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_Skill",
        "GE_Player_Chaos_Skill1_Damage",
        0,
        16_000_000,
    )];
    let report_a5 = run_game_scenario(&req_a5, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_a5["execution_completed"], true);
    assert_eq!(report_a5["hits_settled"], 2);
    assert_eq!(
        report_a5["hit_receipts"][0]["non_crit_damage"],
        json!("2545/2")
    );
    assert_eq!(
        report_a5["hit_receipts"][1]["non_crit_damage"],
        json!("3355/1")
    );
}

#[test]
fn chaos_suppression_pending_dead_target_and_strict_hypothesis() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 1, 8]);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_UltraSkill",
        "GE_Player_Chaos_UltraSkill1_Damage",
        100,
        20_000_000,
    )];
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    ally.talent_levels = None;
    req.actors.push(ally);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 50,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.relative_duration_us = 49;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["execution_completed"], true);
    assert_eq!(pending["hits_settled"], 0);
    assert_eq!(pending["pending_events"], 5);

    req.relative_duration_us = 100;
    let settled = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(settled["execution_completed"], true);
    assert_eq!(settled["hits_settled"], 5);
    assert!(
        settled["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["actor_index"] == 0)
    );

    req.targets[0].hp_current = 0;
    let dead = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead["hits_settled"], 0);

    req.targets[0].hp_current = 1_000_000;
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::chaos::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::chaos::skill::BURST_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(report["resolved_model_assumptions"]["chaos.suppression_packets_v1"].is_string());
    req.model_assumptions.insert(
        crate::characters::chaos::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        "An unreviewed packet model.".into(),
    );
    let mismatch = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        mismatch.contains("conflicting reached model hypothesis"),
        "{mismatch}"
    );
}

#[test]
fn chaos_suppression_applies_a2_a5_and_a6_to_each_packet() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].talent_levels = Some([1, 1, 8]);
    req.targets[0].resistance = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_UltraSkill",
        "GE_Player_Chaos_UltraSkill1_Damage",
        100,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 5);
    for index in 0..4 {
        assert_eq!(
            report["hit_receipts"][index]["non_crit_damage"],
            json!("1767/1"),
            "initial Suppression packet {index}"
        );
    }
    assert_eq!(
        report["hit_receipts"][4]["non_crit_damage"],
        json!("4038/1")
    );
}

#[test]
fn chaos_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Chaos071_Skill",
            "GE_Player_Chaos_Skill1_Damage",
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
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["hit_receipts"][0]["actor_index"], 0);
    assert_eq!(report["hit_receipts"][1]["actor_index"], 0);

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);
}

#[test]
fn chaos_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Chaos_Melee1_Damage", 91, 1),
                ("GE_Player_Chaos_Melee2_Damage", 151, 2),
                ("GE_Player_Chaos_Melee2_1_Damage", 195, 1),
                ("GE_Player_Chaos_Melee3_Damage", 72, 1),
                ("GE_Player_Chaos_Melee3_Damage", 72, 1),
                ("GE_Player_Chaos_Melee4_Damage", 134, 1),
                ("GE_Player_Chaos_Melee4_1_Damage", 51, 1),
                ("GE_Player_Chaos_Melee4_1_Damage", 51, 1),
                ("GE_Player_Chaos_Melee4_2_Damage", 430, 1),
                ("GE_Player_Chaos_Melee5_Damage", 249, 2),
                ("GE_Player_Chaos_Melee5_Damage", 249, 2),
                ("GE_Player_Chaos_Melee5_1_Damage", 399, 1),
                ("GE_Player_Chaos_Melee1_Damage", 91, 1),
            ],
            3821,
            2,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Chaos_Melee1_Damage", 156, 1),
                ("GE_Player_Chaos_Melee2_Damage", 259, 2),
                ("GE_Player_Chaos_Melee2_1_Damage", 334, 1),
                ("GE_Player_Chaos_Melee3_Damage", 247, 2),
                ("GE_Player_Chaos_Melee3_Damage", 247, 2),
                ("GE_Player_Chaos_Melee4_Damage", 459, 2),
                ("GE_Player_Chaos_Melee4_1_Damage", 175, 2),
                ("GE_Player_Chaos_Melee4_1_Damage", 175, 2),
                ("GE_Player_Chaos_Melee4_2_Damage", 737, 1),
                ("GE_Player_Chaos_Melee5_Damage", 427, 2),
                ("GE_Player_Chaos_Melee5_Damage", 427, 2),
                ("GE_Player_Chaos_Melee5_1_Damage", 684, 1),
                ("GE_Player_Chaos_Melee1_Damage", 156, 1),
            ],
            3275,
            1,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Chaos_Melee1_Damage", 337, 2),
                ("GE_Player_Chaos_Melee2_Damage", 279, 2),
                ("GE_Player_Chaos_Melee2_1_Damage", 361, 1),
                ("GE_Player_Chaos_Melee3_Damage", 267, 2),
                ("GE_Player_Chaos_Melee3_Damage", 267, 2),
                ("GE_Player_Chaos_Melee4_Damage", 248, 1),
                ("GE_Player_Chaos_Melee4_1_Damage", 189, 2),
                ("GE_Player_Chaos_Melee4_1_Damage", 189, 2),
                ("GE_Player_Chaos_Melee4_2_Damage", 796, 1),
                ("GE_Player_Chaos_Melee5_Damage", 461, 2),
                ("GE_Player_Chaos_Melee5_Damage", 461, 2),
                ("GE_Player_Chaos_Melee5_1_Damage", 1477, 2),
                ("GE_Player_Chaos_Melee1_Damage", 337, 2),
            ],
            3537,
            1,
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
                    "GA_Chaos071_Melee",
                    "GE_Player_Chaos_Melee1_Damage",
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
        assert_eq!(report["hits_settled"], 13);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 13);
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
fn chaos_normal_chain_reset_boundaries_and_noop_switch() {
    let mut req = request();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_001,
            0,
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Chaos_Melee1_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Chaos_Melee1_Damage"
    );

    let mut req_cont = request();
    req_cont.actions = vec![
        make_action(
            0,
            0,
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_cont = run_game_scenario(&req_cont, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_cont["hits_settled"], 3);
    let receipts_cont = report_cont["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_cont[0]["damage_effect_id"],
        "GE_Player_Chaos_Melee1_Damage"
    );
    assert_eq!(
        receipts_cont[1]["damage_effect_id"],
        "GE_Player_Chaos_Melee2_Damage"
    );
    assert_eq!(
        receipts_cont[2]["damage_effect_id"],
        "GE_Player_Chaos_Melee2_1_Damage"
    );

    let mut req_skill_reset = request();
    req_skill_reset.actions = vec![
        make_action(
            0,
            0,
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            500_000,
            0,
            "GA_Chaos071_Skill",
            "GE_Player_Chaos_Skill1_Damage",
            0,
            0,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_skill = run_game_scenario(&req_skill_reset, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_skill["hits_settled"], 4);
    let receipts_skill = report_skill["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_skill[0]["damage_effect_id"],
        "GE_Player_Chaos_Melee1_Damage"
    );
    assert_eq!(
        receipts_skill[1]["damage_effect_id"],
        "GE_Player_Chaos_Skill1_Damage"
    );
    assert_eq!(
        receipts_skill[2]["damage_effect_id"],
        "GE_Player_Chaos_Skill2_Damage"
    );
    assert_eq!(
        receipts_skill[3]["damage_effect_id"],
        "GE_Player_Chaos_Melee1_Damage"
    );

    let mut req_noop = request();
    req_noop.actions = vec![
        make_action(
            0,
            0,
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
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
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_noop = run_game_scenario(&req_noop, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_noop["hits_settled"], 3);
    let receipts_noop = report_noop["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_noop[0]["damage_effect_id"],
        "GE_Player_Chaos_Melee1_Damage"
    );
    assert_eq!(
        receipts_noop[1]["damage_effect_id"],
        "GE_Player_Chaos_Melee2_Damage"
    );
    assert_eq!(
        receipts_noop[2]["damage_effect_id"],
        "GE_Player_Chaos_Melee2_1_Damage"
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
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
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
            "GA_Chaos071_Melee",
            "GE_Player_Chaos_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report_exit = run_game_scenario(&req_exit, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_exit["hits_settled"], 2);
    let receipts_exit = report_exit["hit_receipts"].as_array().unwrap();
    assert_eq!(
        receipts_exit[0]["damage_effect_id"],
        "GE_Player_Chaos_Melee1_Damage"
    );
    assert_eq!(
        receipts_exit[1]["damage_effect_id"],
        "GE_Player_Chaos_Melee1_Damage"
    );
}

#[test]
fn chaos_normal_chain_strict_and_pending_final_stage() {
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
        crate::characters::chaos::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::chaos::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::CHAOS_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::CHAOS_HIT_ENERGY_HYPOTHESIS_ID
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
    req_exit.actions = (0..4)
        .map(|index| {
            make_action(
                index * 100_000,
                0,
                "GA_Chaos071_Melee",
                "GE_Player_Chaos_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    req_exit.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 300_050,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req_exit.relative_duration_us = 300_150;
    let report_exit = run_game_scenario(&req_exit, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_exit["execution_completed"], true);
    assert_eq!(report_exit["hits_settled"], 9);
    let receipts = report_exit["hit_receipts"].as_array().unwrap();
    for receipt in &receipts[5..9] {
        assert_eq!(receipt["actor_index"], 0);
    }
    assert_eq!(
        receipts[5]["damage_effect_id"],
        "GE_Player_Chaos_Melee4_Damage"
    );
    assert_eq!(
        receipts[6]["damage_effect_id"],
        "GE_Player_Chaos_Melee4_1_Damage"
    );
    assert_eq!(
        receipts[7]["damage_effect_id"],
        "GE_Player_Chaos_Melee4_1_Damage"
    );
    assert_eq!(
        receipts[8]["damage_effect_id"],
        "GE_Player_Chaos_Melee4_2_Damage"
    );
}

#[test]
fn chaos_multi_target_all_in_range_skill_and_burst() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Test Skill across 2 targets: 2 hits each = 4 hits
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_Skill",
        "GE_Player_Chaos_Skill1_Damage",
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
    let target_0_hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["target_index"] == 0)
        .count();
    let target_1_hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["target_index"] == 1)
        .count();
    assert_eq!(target_0_hits, 2);
    assert_eq!(target_1_hits, 2);

    // Test Burst across 2 targets: 5 hits each = 10 hits
    let mut req_burst = request();
    let mut target2 = req_burst.targets[0].clone();
    target2.target_index = 1;
    req_burst.targets.push(target2);
    req_burst.actors[0].initial_energy = 100;
    req_burst.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_UltraSkill",
        "GE_Player_Chaos_UltraSkill1_Damage",
        100,
        20_000_000,
    )];
    let report_burst = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["execution_completed"], true);
    assert_eq!(report_burst["hits_settled"], 10);

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
        crate::characters::chaos::skill::BURST_PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::chaos::skill::BURST_PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req_burst.model_assumptions.insert(
        crate::characters::chaos::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::chaos::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report_strict = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_strict["execution_completed"], true);
    assert_eq!(report_strict["hits_settled"], 10);
}
