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
    req.actors[0].character_id = "character:1008".into();
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
        "GA_Skia_Melee",
        "GE_Player_Skia_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn skia_a3_single_locked_target_bonus_survives_an_unmarked_second_target() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Skia_Skill",
        "GE_Player_Skia_Skill1_Damage",
        0,
        0,
    )];
    let single = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let mut unmarked = req.targets[0].clone();
    unmarked.target_index = 1;
    req.targets.push(unmarked);
    let multiple = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(multiple["execution_completed"], true);
    assert_eq!(
        serde_json::json!(formula_damage_total(&single)),
        serde_json::json!(formula_damage_total(&multiple))
    );
    assert_eq!(
        multiple["final_target_states"][1]["hp_current"],
        req.targets[1].hp_current
    );
    for (single_hit, multiple_hit) in single["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .zip(multiple["hit_receipts"].as_array().unwrap())
    {
        assert_eq!(
            single_hit["non_crit_damage"],
            multiple_hit["non_crit_damage"]
        );
    }
}

#[test]
fn skia_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Skia_Melee / GE_Player_Skia_Melee1_Damage)
    // Coeffs: Lv1 = 0.304, Lv8 = 0.521, Lv9 (A3 resonance) = 0.563.
    // At A0: ATK = 1000, DEF factor = 0.5 (1080 / (1080 + 1080) = 1/2).
    //   Lv1: 1000 * 0.304 * 0.5 = 152 / 1
    //   Lv8: 1000 * 0.521 * 0.5 = 521 / 2
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 0.563 * 0.5 = 563 / 2
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 152, 1),
        (8, 0, 521, 2),
        (8, 1, 521, 2),
        (8, 3, 563, 2),
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

    // Skill (GA_Skia_Skill / Shadow Hound Chase: 4x Skill1 + 1x Skill2 + 1x Skill3 = 6 hits)
    // Coeffs:
    //   Lv1: Skill1 = 0.300, Skill2 = 0.046, Skill3 = 0.254 (Sum = 1.500)
    //   Lv8: Skill1 = 0.514, Skill2 = 0.079, Skill3 = 0.435 (Sum = 2.570)
    //   Lv9 (A3 resonance): Skill1 = 0.555, Skill2 = 0.085, Skill3 = 0.470 (Sum = 2.775)
    // At A3: before_hit applies skia.fang_thrust status, and A3 adds +8% (27/25) bonus.
    //
    // Damage at ATK=1000, DEF factor=0.5:
    //   Lv1, A0:
    //     Skill1 (x4): 1000 * 0.300 * 0.5 = 150 / 1
    //     Skill2: 1000 * 0.046 * 0.5 = 23 / 1
    //     Skill3: 1000 * 0.254 * 0.5 = 127 / 1
    //     Total: 4 * 150 + 23 + 127 = 750 / 1
    //   Lv8, A0:
    //     Skill1 (x4): 1000 * 0.514 * 0.5 = 257 / 1
    //     Skill2: 1000 * 0.079 * 0.5 = 79 / 2
    //     Skill3: 1000 * 0.435 * 0.5 = 435 / 2
    //     Total: 4 * 257 + 79/2 + 435/2 = 1285 / 1
    //   Lv8, A3 (effective Lv9 with 27/25 A3 bonus):
    //     Skill1 (x4): 1000 * (111/200) * 0.5 * (27/25) = 2997 / 10
    //     Skill2: 1000 * (17/200) * 0.5 * (27/25) = 459 / 10
    //     Skill3: 1000 * (47/100) * 0.5 * (27/25) = 1269 / 5
    //     Total: 4 * (2997/10) + 459/10 + 2538/10 = 2997 / 2 (1498.5)
    for (level, awakening, s1, s2, s3, total) in [
        (1, 0, (150, 1), (23, 1), (127, 1), (750, 1)),
        (8, 0, (257, 1), (79, 2), (435, 2), (1285, 1)),
        (8, 3, (2997, 10), (459, 10), (1269, 5), (2997, 2)),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
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
        for i in 0..4 {
            assert_eq!(
                report["hit_receipts"][i]["non_crit_damage"],
                json!(format!("{}/{}", s1.0, s1.1))
            );
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
            assert_eq!(
                report["hit_receipts"][i]["damage_effect_id"],
                "GE_Player_Skia_Skill1_Damage"
            );
        }
        assert_eq!(
            report["hit_receipts"][4]["non_crit_damage"],
            json!(format!("{}/{}", s2.0, s2.1))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][4])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        assert_eq!(
            report["hit_receipts"][4]["damage_effect_id"],
            "GE_Player_Skia_Skill2_Damage"
        );
        assert_eq!(
            report["hit_receipts"][5]["non_crit_damage"],
            json!(format!("{}/{}", s3.0, s3.1))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][5])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        assert_eq!(
            report["hit_receipts"][5]["damage_effect_id"],
            "GE_Player_Skia_Skill3_Damage"
        );
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total.0, total.1))
        );
    }

    // Burst (GA_Skia_UltraSkill / The Pack: 1x UltraSkill1 Spread + 3x UltraSkill2 Converge = 4 hits)
    // Coeffs:
    //   Lv1: UltraSkill1 = 2.902, UltraSkill2 = 0.699 (Sum = 4.999)
    //   Lv8: UltraSkill1 = 4.974, UltraSkill2 = 1.198 (Sum = 8.568)
    //   Lv9 (A3 resonance): UltraSkill1 = 5.371, UltraSkill2 = 1.294 (Sum = 9.253)
    //
    // Damage at ATK=1000, DEF factor=0.5 (no pre-existing fang_thrust status):
    //   Lv1, A0:
    //     UltraSkill1: 1000 * 2.902 * 0.5 = 1451 / 1
    //     UltraSkill2 (x3): 1000 * 0.699 * 0.5 = 699 / 2
    //     Total: 1451 + 3 * (699/2) = 4999 / 2
    //   Lv8, A0:
    //     UltraSkill1: 1000 * 4.974 * 0.5 = 2487 / 1
    //     UltraSkill2 (x3): 1000 * 1.198 * 0.5 = 599 / 1
    //     Total: 2487 + 3 * 599 = 4284 / 1
    //   Lv8, A3 (effective Lv9):
    //     UltraSkill1: 1000 * 5.371 * 0.5 = 5371 / 2
    //     UltraSkill2 (x3): 1000 * 1.294 * 0.5 = 647 / 1
    //     Total: 5371/2 + 3 * 647 = 9253 / 2
    for (level, awakening, u1, u2, total) in [
        (1, 0, (1451, 1), (699, 2), (4999, 2)),
        (8, 0, (2487, 1), (599, 1), (4284, 1)),
        (8, 3, (5371, 2), (647, 1), (9253, 2)),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Skia_UltraSkill",
            "GE_Player_Skia_UltraSkill1_Damage",
            100,
            20_000_000,
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
            json!(format!("{}/{}", u1.0, u1.1))
        );
        assert_eq!(
            report["hit_receipts"][0]["damage_effect_id"],
            "GE_Player_Skia_UltraSkill1_Damage"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        for i in 1..=3 {
            assert_eq!(
                report["hit_receipts"][i]["non_crit_damage"],
                json!(format!("{}/{}", u2.0, u2.1))
            );
            assert_eq!(
                report["hit_receipts"][i]["damage_effect_id"],
                "GE_Player_Skia_UltraSkill2_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total.0, total.1))
        );
    }
}

#[test]
fn skia_a2_crit_rate_buff_on_skill_cast() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req.actions = vec![make_action(
        0,
        0,
        "GA_Skia_Skill",
        "GE_Player_Skia_Skill1_Damage",
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
    for i in 0..6 {
        assert_eq!(report["hit_receipts"][i]["crit_rate"], json!("3/20"));
    }
}

#[test]
fn skia_a2_crit_rate_buff_absent_below_effect2() {
    // AwakeningSelection::prefix(1) grants effects [1] only; Effect1 is a damage
    // bonus on GE_Player_Skia_SkillShadowAtk_Damage and never touches crit rate.
    // per DT_SkiaEffectFigure.json Skia_Leve2CritBase = 0.10 requires Effect2.
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req.actions = vec![make_action(
        0,
        0,
        "GA_Skia_Skill",
        "GE_Player_Skia_Skill1_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 6);
    for i in 0..6 {
        assert_eq!(report["hit_receipts"][i]["crit_rate"], json!("1/20"));
    }
}

#[test]
fn skia_a2_crit_rate_buff_expires_after_15s() {
    // per DT_SkiaEffectFigure.json Skia_Leve2Duaration = 15.0 -> 15_000_000 us.
    // A second GA_Skia_Skill cast re-applies the buff at admission time, so the
    // expiry boundary is probed with a GA_Skia_Melee hit instead: cast skill at
    // t=0, then melee at t=16_000_000 (after expiry) reports base crit rate.
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req.actions = vec![
        make_action(0, 0, "GA_Skia_Skill", "GE_Player_Skia_Skill1_Damage", 0, 0),
        make_action(
            16_000_000,
            0,
            "GA_Skia_Melee",
            "GE_Player_Skia_Melee1_Damage",
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
    assert_eq!(report["hits_settled"], 7);
    let receipts = report["hit_receipts"].as_array().unwrap();
    for hit in &receipts[0..6] {
        assert_eq!(hit["crit_rate"], json!("3/20"));
    }
    let last = receipts.last().unwrap();
    assert_eq!(last["damage_effect_id"], "GE_Player_Skia_Melee1_Damage");
    assert_eq!(last["crit_rate"], json!("1/20"));
}

#[test]
fn skia_a2_crit_rate_buff_recast_inside_window_refreshes_duration() {
    // refresh_buff uses TimedBuffDeclaration::reset: recasting GA_Skia_Skill at
    // t=10_000_000 (inside the 15s window) re-arms the 15_000_000 us duration,
    // so a GA_Skia_Melee hit at t=16_000_000 still carries the +1/10 bonus.
    // Without the refresh it would have expired at t=15_000_000 (see the expiry
    // test above).
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actors[0].static_stats.crit_rate = Ratio {
        numerator: 1,
        denominator: 20,
    };
    req.actions = vec![
        make_action(0, 0, "GA_Skia_Skill", "GE_Player_Skia_Skill1_Damage", 0, 0),
        make_action(
            10_000_000,
            0,
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
            0,
            0,
        ),
        make_action(
            16_000_000,
            0,
            "GA_Skia_Melee",
            "GE_Player_Skia_Melee1_Damage",
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
    assert_eq!(report["hits_settled"], 13);
    let receipts = report["hit_receipts"].as_array().unwrap();
    let last = receipts.last().unwrap();
    assert_eq!(last["damage_effect_id"], "GE_Player_Skia_Melee1_Damage");
    assert_eq!(last["crit_rate"], json!("3/20"));
}

#[test]
fn skia_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
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
    for i in 0..6 {
        assert_eq!(report["hit_receipts"][i]["actor_index"], 0);
    }

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);
}

#[test]
fn skia_strict_mode_rejects_missing_hypothesis_and_accepts_valid_hypothesis() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::SKIA_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::SKIA_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Skia_Skill",
        "GE_Player_Skia_Skill1_Damage",
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
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID
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
fn skia_two_skill_one_burst_rotation_hit_count_and_energy() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            6_000_000,
            0,
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
            0,
            6_000_000,
        ),
        make_action(
            12_000_000,
            0,
            "GA_Skia_UltraSkill",
            "GE_Player_Skia_UltraSkill1_Damage",
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
    assert_eq!(report["hits_settled"], 16);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
}

#[test]
fn skia_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, coefficients, total) in [
        (1, 0, [304, 37, 37, 189, 321, 459, 391, 294, 554, 304], 2890),
        (8, 0, [521, 63, 63, 324, 550, 787, 670, 504, 949, 521], 4952),
        (
            8,
            3,
            [563, 68, 68, 350, 594, 850, 724, 544, 1025, 563],
            5349,
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
                    "GA_Skia_Melee",
                    "GE_Player_Skia_Melee1_Damage",
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
            serde_json::to_value(ExactFraction::new(total, 2).unwrap()).unwrap()
        );
        let stages = [1, 2, 2, 4, 5, 6, 7, 8, 9, 1];
        for (hit, stage) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(stages)
        {
            assert_eq!(
                hit["damage_effect_id"],
                format!("GE_Player_Skia_Melee{stage}_Damage")
            );
        }
    }
}

#[test]
fn skia_normal_chain_reset_and_noop_boundaries() {
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
                "GA_Skia_Skill",
                "GE_Player_Skia_Skill1_Damage",
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
            "GA_Skia_Melee",
            "GE_Player_Skia_Melee1_Damage",
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
            "GE_Player_Skia_Melee4_Damage"
        } else {
            "GE_Player_Skia_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn skia_normal_chain_strict_hypothesis_and_off_field_three_hit_delivery() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::SKIA_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::SKIA_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::skia::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::skia::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = (0..2)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Skia_Melee",
                "GE_Player_Skia_Melee1_Damage",
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
    let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(delivered["hits_settled"], 4);
    for (hit, stage) in delivered["hit_receipts"].as_array().unwrap()[1..]
        .iter()
        .zip([2, 2, 4])
    {
        assert_eq!(hit["actor_index"], 0);
        assert_eq!(
            hit["damage_effect_id"],
            format!("GE_Player_Skia_Melee{stage}_Damage")
        );
    }
}

#[test]
fn skia_hound_gnaw_attack_consumes_fang_thrust_mark_and_deals_burst_damage() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::skia::entry_state::HYPOTHESIS_ID.into(),
        crate::characters::skia::entry_state::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Skia_Skill",
        "GE_Player_Skia_Skill1_Damage",
        0,
        12_000_000,
    )];
    req.relative_duration_us = 7_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 7); // 6 skill hits + 1 shadow atk from hound
    let hits = report["hit_receipts"].as_array().unwrap();
    let shadow_hit = &hits[6];
    assert_eq!(
        shadow_hit["damage_effect_id"],
        "GE_Player_Skia_SkillShadowAtk_Damage"
    );
    assert_eq!(shadow_hit["time_us"], 1_398_984);
    // At level 1, ATK=1000, DEF factor=0.5: rate=0.55 -> non_crit_damage = 275/1
    assert_eq!(shadow_hit["non_crit_damage"], json!("275/1"));
}

#[test]
fn skia_melee_attack_consumes_fang_thrust_mark_before_hound() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::skia::entry_state::HYPOTHESIS_ID.into(),
        crate::characters::skia::entry_state::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::skia::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::skia::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    // Skill recovery finishes at 1_335_873. Cast Melee at 1_350_000 (after recovery, during sneak, before hound at 1_398_984).
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
            0,
            12_000_000,
        ),
        make_action(
            1_350_000,
            0,
            "GA_Skia_Melee",
            "GE_Player_Skia_Melee1_Damage",
            0,
            0,
        ),
    ];
    req.relative_duration_us = 3_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // 6 skill hits + 1 melee hit + 1 shadow atk = 8 hits total (no duplicate hound attack at 1_398_984)
    assert_eq!(report["hits_settled"], 8);
    let hits = report["hit_receipts"].as_array().unwrap();
    let shadow_hit = hits
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Skia_SkillShadowAtk_Damage")
        .unwrap();
    assert_eq!(shadow_hit["time_us"], 1_350_100);
    let parent = shadow_hit["parent_hit_index"].as_u64().unwrap() as usize;
    assert_eq!(hits[parent]["ability_id"], "GA_Skia_Melee");
    assert_eq!(hits[parent]["time_us"], shadow_hit["time_us"]);
    assert_eq!(hits[parent]["command_index"], shadow_hit["command_index"]);
}

#[test]
fn skia_a1_awakening_enhances_skill_shadow_attack_ratio() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.model_assumptions.insert(
        crate::characters::skia::entry_state::HYPOTHESIS_ID.into(),
        crate::characters::skia::entry_state::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Skia_Skill",
        "GE_Player_Skia_Skill1_Damage",
        0,
        12_000_000,
    )];
    req.relative_duration_us = 7_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let shadow_hit = &hits[6];
    assert_eq!(
        shadow_hit["damage_effect_id"],
        "GE_Player_Skia_SkillShadowAtk_Damage"
    );
    // At A1: +0.60 ratio. Base rate = 0.55 -> effective rate = 0.55 + 0.60 = 1.15.
    // Damage = 1000 * 1.15 * 0.5 = 575 / 1
    assert_eq!(shadow_hit["non_crit_damage"], json!("575/1"));
}

#[test]
fn skia_state_38_expires_after_5s_without_teammate_switch() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::skia::entry_state::HYPOTHESIS_ID.into(),
        crate::characters::skia::entry_state::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Skia_Skill",
        "GE_Player_Skia_Skill1_Damage",
        0,
        12_000_000,
    )];
    req.relative_duration_us = 7_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 7);
}

#[test]
fn skia_two_continuing_cycles_fixture() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::characters::skia::entry_state::HYPOTHESIS_ID.into(),
        crate::characters::skia::entry_state::HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::skia::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::skia::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::skia::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;

    // Cycle 1:
    // 0s: Skill (6 hits + Hound attack at 1.398s -> 7 hits)
    // 3s: Burst (1 spread + 3 converge = 4 hits, 100 energy spent)
    //
    // Cycle 2 (after 13s, since Skill CD is 12.15s and Burst CD is 20s):
    // 15s: Skill (6 hits + Hound attack at 16.398s -> 7 hits)
    // 25s: Burst (1 spread + 3 converge = 4 hits, 0 energy required here for cycle verification)
    req.actions = vec![
        // Cycle 1
        make_action(
            0,
            0,
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
            0,
            12_153_592,
        ),
        make_action(
            3_000_000,
            0,
            "GA_Skia_UltraSkill",
            "GE_Player_Skia_UltraSkill1_Damage",
            100,
            20_000_000,
        ),
        // Cycle 2
        make_action(
            15_000_000,
            0,
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
            0,
            12_153_592,
        ),
        make_action(
            25_000_000,
            0,
            "GA_Skia_UltraSkill",
            "GE_Player_Skia_UltraSkill1_Damage",
            0,
            20_000_000,
        ),
    ];
    req.relative_duration_us = 30_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Cycle 1: 7 hits (Skill + Hound) + 4 hits (Burst) = 11 hits
    // Cycle 2: 7 hits (Skill + Hound) + 4 hits (Burst) = 11 hits
    // Total: 22 hits settled across 2 continuous cycles
    assert_eq!(report["hits_settled"], 22);

    let hits = report["hit_receipts"].as_array().unwrap();
    let shadow_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Skia_SkillShadowAtk_Damage")
        .collect();
    assert_eq!(shadow_hits.len(), 2);
    assert_eq!(shadow_hits[0]["time_us"], 1_398_984);
    assert_eq!(shadow_hits[1]["time_us"], 16_398_984);

    let burst_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Skia_UltraSkill1_Damage")
        .collect();
    assert_eq!(burst_hits.len(), 2);
    assert_eq!(burst_hits[0]["time_us"], 3_000_100);
    assert_eq!(burst_hits[1]["time_us"], 25_000_100);
}
