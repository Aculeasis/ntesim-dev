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
    req.actors[0].character_id = "character:1052".into();
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
        "GA_Jin_Melee",
        "GE_Player_Jin_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn hotori_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Jin_Melee / GE_Player_Jin_Melee1_Damage)
    // Coeffs: Lv1 = 0.200, Lv8 = 0.343, Lv9 (A3 resonance) = 0.370.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 0.200 * 0.5 = 100 / 1
    //   Lv8: 1000 * 0.343 * 0.5 = 343 / 2
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 0.370 * 0.5 = 185 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 100, 1),
        (8, 0, 343, 2),
        (8, 1, 343, 2),
        (8, 3, 185, 1),
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

    // Skill (GA_Jin_Skill / GE_Player_Jin_Skill1_Damage)
    // Delivers 7 hits in the recorded binding order: Skill3, Skill1, Skill3,
    // Skill1, Skill1, Skill1, Skill2.
    // Coeffs:
    //   Skill1: Lv1 = 0.400, Lv8 = 0.686, Lv9 = 0.740
    //   Skill3: Lv1 = 0.400, Lv8 = 0.686, Lv9 = 0.740
    //   Skill2: Lv1 = 0.601, Lv8 = 1.030, Lv9 = 1.112
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: Skill hits = 200/1 each, Skill2 = 601/2. Total = 3001/2
    //   Lv8: Skill hits = 343/1 each, Skill2 = 515/1. Total = 2573/1
    //   Lv9: Skill hits = 370/1 each, Skill2 = 556/1. Total = 2776/1
    for (level, awakening, hit0_num, hit0_den, total_num, total_den) in [
        (1, 0, 200, 1, 3001, 2),
        (8, 0, 343, 1, 2573, 1),
        (8, 3, 370, 1, 2776, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        // 11s of passive recovery funds the 60-unit Present Replay cost; the
        // A3 window that starts after the rewind must also settle in scope.
        req.relative_duration_us = 30_000_000;
        req.actions = vec![make_action(
            11_000_000,
            0,
            "GA_Jin_Skill",
            "GE_Player_Jin_Skill1_Damage",
            0,
            16_000_000,
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
            json!(format!("{}/{}", hit0_num, hit0_den))
        );
        assert_eq!(
            report["hit_receipts"][0]["damage_effect_id"],
            "GE_Player_Jin_Skill3_Damage"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
        for (index, effect) in [
            "GE_Player_Jin_Skill1_Damage",
            "GE_Player_Jin_Skill3_Damage",
            "GE_Player_Jin_Skill1_Damage",
            "GE_Player_Jin_Skill1_Damage",
            "GE_Player_Jin_Skill1_Damage",
        ]
        .iter()
        .enumerate()
        {
            let receipt = &report["hit_receipts"][index + 1];
            assert_eq!(receipt["damage_effect_id"], *effect);
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", hit0_num, hit0_den))
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
            );
        }
        assert_eq!(
            report["hit_receipts"][6]["damage_effect_id"],
            "GE_Player_Jin_Skill2_Damage"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][6])["packet"]["native_damage_source_category"],
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

    // Burst (GA_Jin_UltraSkill / GE_Player_Jin_UltraSkill1_Damage)
    // Delivers 10 hits:
    //   1x GE_Player_Jin_UltraSkill1_Damage
    //   5x GE_Player_Jin_UltraSkill1_1..5_Damage
    //   1x GE_Player_Jin_UltraSkill2_Damage
    //   1x GE_Player_Jin_UltraSkill4_Damage
    //   2x GE_Player_Jin_UltraSkill3_Damage
    // on_cast enters Time Stop (12s).
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: Hit 0 = 85/1. Total = 3999/1
    //   Lv8: Hit 0 = 291/2. Total = 6852/1
    // At A2: +20% damage bonus (6/5) during Time Stop:
    //   Lv8: Hit 0 = 873/5. Total = 41112/5
    // At A3: resonance_3 grants +1 level -> Lv9, +20% damage bonus (6/5):
    //   Lv9: Hit 0 = 189/1. Total = 44418/5
    for (level, awakening, hit0_num, hit0_den, total_num, total_den) in [
        (1, 0, 85, 1, 3999, 1),
        (8, 0, 291, 2, 6852, 1),
        (8, 2, 873, 5, 41112, 5),
        (8, 3, 189, 1, 44418, 5),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.relative_duration_us = 35_000_000;
        // World's Tide requires exactly 120 units and spends no conventional energy.
        req.actions = vec![make_action(
            20_000_000,
            0,
            "GA_Jin_UltraSkill",
            "GE_Player_Jin_UltraSkill1_Damage",
            0,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 10);
        assert_eq!(
            report["hit_receipts"][0]["damage_effect_id"],
            "GE_Player_Jin_UltraSkill1_Damage"
        );
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", hit0_num, hit0_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        for i in 1..=5 {
            assert_eq!(
                report["hit_receipts"][i]["non_crit_damage"],
                json!(format!("{}/{}", hit0_num, hit0_den))
            );
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        assert_eq!(
            report["hit_receipts"][6]["damage_effect_id"],
            "GE_Player_Jin_UltraSkill2_Damage"
        );
        assert_eq!(
            report["hit_receipts"][7]["damage_effect_id"],
            "GE_Player_Jin_UltraSkill4_Damage"
        );
        assert_eq!(
            report["hit_receipts"][8]["damage_effect_id"],
            "GE_Player_Jin_UltraSkill3_Damage"
        );
        assert_eq!(
            report["hit_receipts"][9]["damage_effect_id"],
            "GE_Player_Jin_UltraSkill3_Damage"
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
fn hotori_a6_def_ignore_during_time_stop() {
    // At A6:
    // - 30% DEF ignore during Time Stop (target DEF 1080 -> 756, DEF factor = 10/17).
    // - A2: +20% (1/5) DMG bonus during Time Stop.
    // - A5: +15% (3/20) DMG bonus on single target (target_count == 1). Total DMG bonus = 1 + 1/5 + 3/20 = 27/20.
    // - R6: +30% (3/10) of Hotori's own base ATK -> ATK = 1300.
    // - Resonance 3: Burst at Lv8 -> Lv9 = 0.315.
    // non_crit hit 0 = 1300 * 0.315 * (10 / 17) * (27 / 20) = 22113 / 68.
    // Total non_crit across 10 hits = 2598453 / 170.
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 1, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.relative_duration_us = 35_000_000;
    req.actions = vec![make_action(
        20_000_000,
        0,
        "GA_Jin_UltraSkill",
        "GE_Player_Jin_UltraSkill1_Damage",
        0,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 10);
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("22113/68")
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
    assert_eq!(sum, ExactFraction::new(2598453, 170).unwrap());
}

#[test]
fn hotori_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            11_000_000,
            0,
            "GA_Jin_Skill",
            "GE_Player_Jin_Skill1_Damage",
            0,
            16_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 11_000_010,
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
    assert_eq!(report["hits_settled"], 7);
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
fn hotori_strict_mode_rejects_missing_hypothesis_and_accepts_valid_hypothesis() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        11_000_000,
        0,
        "GA_Jin_Skill",
        "GE_Player_Jin_Skill1_Damage",
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
        crate::characters::hotori::timepiece::MODEL.into(),
        crate::characters::hotori::timepiece::MODEL
            .description()
            .into(),
    );

    req.model_assumptions.insert(
        crate::characters::hotori::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::hotori::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::HOTORI_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HOTORI_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let cycle = crate::HypothesisId::CombatPacketCycleAccrualV1;
    req.model_assumptions
        .insert(cycle.into(), cycle.description().into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 7);
}

#[test]
fn hotori_two_skill_one_burst_rotation_hit_count_and_energy() {
    // 11s Skill: 66 -> 6. Passive resumes when the fixed rewind settles at
    // 16.167475s. 36s Burst: capped at exactly 120 (4.99515 wasted), spends
    // all of it. 61s Skill: 64.900002 -> 4.900002. The 70-relative-second endpoint is absolute84.183333,
    // leaving18.015858 world seconds after the second rewind.
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.relative_duration_us = 70_000_000;
    req.actions = vec![
        make_action(
            11_000_000,
            0,
            "GA_Jin_Skill",
            "GE_Player_Jin_Skill1_Damage",
            0,
            16_000_000,
        ),
        make_action(
            36_000_000,
            0,
            "GA_Jin_UltraSkill",
            "GE_Player_Jin_UltraSkill1_Damage",
            0,
            20_000_000,
        ),
        make_action(
            61_000_000,
            0,
            "GA_Jin_Skill",
            "GE_Player_Jin_Skill1_Damage",
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
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 0);
    let timepiece = &report["final_actor_states"][0]["fractional_resources"]["hotori.timepiece"];
    assert_eq!(timepiece["current"], json!("2259903/20000"));
    assert_eq!(timepiece["spent"], json!("240/1"));
    assert_eq!(timepiece["overflow"], json!("99903/20000"));
    assert_eq!(timepiece["conserved"], true);
}

#[test]
fn hotori_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Jin_Melee1_Damage", 100, 1),
                ("GE_Player_Jin_Melee2_Damage", 353, 2),
                ("GE_Player_Jin_Melee3_Damage", 22, 1),
                ("GE_Player_Jin_Melee3_Damage", 22, 1),
                ("GE_Player_Jin_Melee9_Damage", 361, 1),
                ("GE_Player_Jin_Melee4_Damage", 113, 2),
                ("GE_Player_Jin_Melee4_Damage", 113, 2),
                ("GE_Player_Jin_Melee4_Damage", 113, 2),
                ("GE_Player_Jin_Melee4_Damage", 113, 2),
                ("GE_Player_Jin_Melee4_Damage", 113, 2),
                ("GE_Player_Jin_Melee5_Damage", 71, 1),
                ("GE_Player_Jin_Melee5_Damage", 71, 1),
                ("GE_Player_Jin_Melee6_Damage", 333, 1),
                ("GE_Player_Jin_Melee7_Damage", 243, 2),
                ("GE_Player_Jin_Melee8_Damage", 451, 2),
                ("GE_Player_Jin_Melee1_Damage", 100, 1),
            ],
            1886,
            1,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Jin_Melee1_Damage", 343, 2),
                ("GE_Player_Jin_Melee2_Damage", 605, 2),
                ("GE_Player_Jin_Melee3_Damage", 75, 2),
                ("GE_Player_Jin_Melee3_Damage", 75, 2),
                ("GE_Player_Jin_Melee9_Damage", 1237, 2),
                ("GE_Player_Jin_Melee4_Damage", 97, 1),
                ("GE_Player_Jin_Melee4_Damage", 97, 1),
                ("GE_Player_Jin_Melee4_Damage", 97, 1),
                ("GE_Player_Jin_Melee4_Damage", 97, 1),
                ("GE_Player_Jin_Melee4_Damage", 97, 1),
                ("GE_Player_Jin_Melee5_Damage", 243, 2),
                ("GE_Player_Jin_Melee5_Damage", 243, 2),
                ("GE_Player_Jin_Melee6_Damage", 1141, 2),
                ("GE_Player_Jin_Melee7_Damage", 208, 1),
                ("GE_Player_Jin_Melee8_Damage", 773, 2),
                ("GE_Player_Jin_Melee1_Damage", 343, 2),
            ],
            3232,
            1,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Jin_Melee1_Damage", 185, 1),
                ("GE_Player_Jin_Melee2_Damage", 653, 2),
                ("GE_Player_Jin_Melee3_Damage", 81, 2),
                ("GE_Player_Jin_Melee3_Damage", 81, 2),
                ("GE_Player_Jin_Melee9_Damage", 668, 1),
                ("GE_Player_Jin_Melee4_Damage", 209, 2),
                ("GE_Player_Jin_Melee4_Damage", 209, 2),
                ("GE_Player_Jin_Melee4_Damage", 209, 2),
                ("GE_Player_Jin_Melee4_Damage", 209, 2),
                ("GE_Player_Jin_Melee4_Damage", 209, 2),
                ("GE_Player_Jin_Melee5_Damage", 263, 2),
                ("GE_Player_Jin_Melee5_Damage", 263, 2),
                ("GE_Player_Jin_Melee6_Damage", 1233, 2),
                ("GE_Player_Jin_Melee7_Damage", 225, 1),
                ("GE_Player_Jin_Melee8_Damage", 835, 2),
                ("GE_Player_Jin_Melee1_Damage", 185, 1),
            ],
            3490,
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
                    "GA_Jin_Melee",
                    "GE_Player_Jin_Melee1_Damage",
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
        assert_eq!(report["hits_settled"], 16);
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
fn hotori_normal_chain_reset_and_noop_boundaries() {
    for boundary in 0..5 {
        let mut req = request();
        req.relative_duration_us = if boundary == 2 { 20_000_000 } else { 4_000_000 };
        let timestamp = match boundary {
            0 => 2_000_001,
            1 => 2_000_000,
            2 => 13_000_000,
            _ => 3000,
        };
        match boundary {
            2 => req.actions.push(make_action(
                11_000_000,
                0,
                "GA_Jin_Skill",
                "GE_Player_Jin_Skill1_Damage",
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
            "GA_Jin_Melee",
            "GE_Player_Jin_Melee1_Damage",
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
            "GE_Player_Jin_Melee9_Damage"
        } else {
            "GE_Player_Jin_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn hotori_normal_chain_strict_hypothesis_and_off_field_four_hit_delivery() {
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
        crate::characters::hotori::timepiece::MODEL.into(),
        crate::characters::hotori::timepiece::MODEL
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::hotori::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::hotori::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::HOTORI_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HOTORI_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let cycle = crate::HypothesisId::CombatPacketCycleAccrualV1;
    req.model_assumptions
        .insert(cycle.into(), cycle.description().into());
    req.actors.push(req.actors[0].clone());
    req.actions = (0..2)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Jin_Melee",
                "GE_Player_Jin_Melee1_Damage",
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
    assert_eq!(settled["hits_settled"], 5);
}

#[test]
fn hotori_multi_target_all_in_range_skill_and_burst() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Skill across 2 targets: 7 packets x 2 recipients = 14 hits, grouped in
    // stable target order per packet.
    req.actions = vec![make_action(
        11_000_000,
        0,
        "GA_Jin_Skill",
        "GE_Player_Jin_Skill1_Damage",
        0,
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
    for pair in receipts.chunks(2) {
        assert_eq!(pair[0]["target_index"], 0);
        assert_eq!(pair[1]["target_index"], 1);
        assert_eq!(pair[0]["damage_effect_id"], pair[1]["damage_effect_id"]);
    }

    // Burst across 2 targets: 10 packets x 2 recipients = 20 hits.
    let mut req_burst = request();
    let mut target2 = req_burst.targets[0].clone();
    target2.target_index = 1;
    req_burst.targets.push(target2);
    req_burst.relative_duration_us = 35_000_000;
    req_burst.actions = vec![make_action(
        20_000_000,
        0,
        "GA_Jin_UltraSkill",
        "GE_Player_Jin_UltraSkill1_Damage",
        0,
        20_000_000,
    )];
    let report_burst = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["execution_completed"], true);
    assert_eq!(report_burst["hits_settled"], 20);
    let receipts_burst = report_burst["hit_receipts"].as_array().unwrap();
    for pair in receipts_burst.chunks(2) {
        assert_eq!(pair[0]["target_index"], 0);
        assert_eq!(pair[1]["target_index"], 1);
        assert_eq!(pair[0]["damage_effect_id"], pair[1]["damage_effect_id"]);
    }
}

#[test]
fn hotori_multi_target_strict_acceptance() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        crate::simulation::energy::HOTORI_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HOTORI_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::hotori::timepiece::MODEL.into(),
        crate::characters::hotori::timepiece::MODEL
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::hotori::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::hotori::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        11_000_000,
        0,
        "GA_Jin_Skill",
        "GE_Player_Jin_Skill1_Damage",
        0,
        20_000_000,
    )];

    req.model_assumptions.insert(
        crate::characters::hotori::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::hotori::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let cycle = crate::HypothesisId::CombatPacketCycleAccrualV1;
    req.model_assumptions
        .insert(cycle.into(), cycle.description().into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 14);
}
