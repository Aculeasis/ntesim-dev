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

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1004".into();
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
        "GA_Lacrimosa_Melee",
        "GE_Player_Lacrimosa_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn lacrimosa_connected_packets_preserve_native_cycle_and_fractional_carry() {
    for targets in [1, 3] {
        let mut req = request();
        req.actors[0].initial_cycle = 0;
        req.actors[0].cycle_gain_efficiency = Ratio::one();
        req.targets[0].hp_current = 100_000_000;
        req.targets[0].hp_max = 100_000_000;
        for index in 1..targets {
            let mut target = req.targets[0].clone();
            target.target_index = index;
            req.targets.push(target);
        }
        let mut second = req.actions[0].clone();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut second {
            *timestamp_us = 6_000_000;
        }
        req.actions.push(second);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        // Frozen Global Melee1 HeterochromeAdd=2.376, twice, once per AoE group.
        assert_eq!(
            report["final_actor_states"][0]["cycle"],
            json!({
                "initial":0,"current":4,"capacity":100,"granted":4,"spent":0,
                "overflow":0,"carry":"94/125","conserved":true
            })
        );
    }
}

#[test]
fn lacrimosa_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_Lacrimosa_Melee / GE_Player_Lacrimosa_Melee1_Damage)
    // Coeffs: Lv1 = 0.371, Lv8 = 0.636, Lv9 (A3 resonance) = 0.687.
    // At A0: ATK = 1000, DEF factor = 0.5 (1080 / (1080 + 1080) = 1/2).
    //   Lv1: 1000 * 0.371 * 0.5 = 371 / 2
    //   Lv8: 1000 * 0.636 * 0.5 = 318 / 1
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 0.687 * 0.5 = 687 / 2
    // At A6: resonance_6 (Lullaby) ignores 20% DEF -> DEF factor = 1080 / (1080 + 864) = 5/9:
    //   Lv9: 1000 * 0.687 * (5/9) = 1145 / 3
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 371, 2),
        (8, 0, 318, 1),
        (8, 1, 318, 1),
        (8, 3, 687, 2),
        (8, 6, 1145, 3),
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

    // Skill (GA_Lacrimosa_Skill / GE_Player_Lacrimosa_Skill1_Damage)
    // Coeffs: Lv1 = 3.000, Lv8 = 5.141, Lv9 (A3 resonance) = 5.553.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 3.000 * 0.5 = 1500 / 1
    //   Lv8: 1000 * 5.141 * 0.5 = 5141 / 2
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 5.553 * 0.5 = 5553 / 2
    // At A6: resonance_6 (Lullaby) ignores 20% DEF -> DEF factor = 5/9:
    //   Lv9: 1000 * 5.553 * (5/9) = 3085 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 1500, 1),
        (8, 0, 5141, 2),
        (8, 3, 5553, 2),
        (8, 6, 3085, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
            0,
            16_000_000,
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

    // Burst (GA_Lacrimosa_UltraSkill / GE_Player_Lacrimosa_UltraSkill1_Damage)
    // Delivers 5 hits under lacrimosa.skill_burst_packets_v1:
    //   4x GE_Player_Lacrimosa_UltraSkill1_Damage
    //   1x GE_Player_Lacrimosa_UltraSkill2_Damage
    // Coeffs:
    //   UltraSkill1: Lv1 = 0.942, Lv8 = 1.614, Lv9 = 1.744
    //   UltraSkill2: Lv1 = 6.233, Lv8 = 10.682, Lv9 = 11.537
    // At A0 (DEF factor = 0.5):
    //   Lv1: U1 = 471/1 (x4 = 1884/1), U2 = 6233/2. Total = 10001/2
    //   Lv8: U1 = 807/1 (x4 = 3228/1), U2 = 5341/1. Total = 8569/1
    // At A3 (resonance_3 grants Lv9, DEF factor = 0.5):
    //   Lv9: U1 = 872/1 (x4 = 3488/1), U2 = 11537/2. Total = 18513/2
    // At A6 (resonance_6 ignores 20% DEF, DEF factor = 5/9):
    //   Lv9: U1 = 8720/9 (x4 = 34880/9), U2 = 57685/9. Total = 92565/9 = 10285/1
    for (level, awakening, u1_num, u1_den, u2_num, u2_den, total_num, total_den) in [
        (1, 0, 471, 1, 6233, 2, 10001, 2),
        (8, 0, 807, 1, 5341, 1, 8569, 1),
        (8, 3, 872, 1, 11537, 2, 18513, 2),
        (8, 6, 8720, 9, 57685, 9, 10285, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Lacrimosa_UltraSkill",
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            120,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 5);

        for i in 0..4 {
            assert_eq!(
                report["hit_receipts"][i]["damage_effect_id"],
                "GE_Player_Lacrimosa_UltraSkill1_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, &report["hit_receipts"][i])["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
            assert_eq!(
                report["hit_receipts"][i]["non_crit_damage"],
                json!(format!("{}/{}", u1_num, u1_den))
            );
        }
        assert_eq!(
            report["hit_receipts"][4]["damage_effect_id"],
            "GE_Player_Lacrimosa_UltraSkill2_Damage"
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][4])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        assert_eq!(
            report["hit_receipts"][4]["non_crit_damage"],
            json!(format!("{}/{}", u2_num, u2_den))
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
fn lacrimosa_a2_damage_buff_after_qte() {
    // Lacrimosa A2 (Rise and Shine): Increases Lacrimosa's damage dealt by 15% (3/20) for 15s after QTE
    // QTE at timestamp 0, followed by Skill at timestamp 1_000_000 (within 15s window).
    // Skill Lv8: 1000 * 5.141 * 0.5 * (1 + 3/20) = 1000 * (5141/1000) * (1/2) * (23/20) = 118243 / 40.
    for (effects, expected) in [(vec![2], json!("118243/40")), (vec![1, 3], json!("5141/2"))] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 8, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.actions = vec![
            make_action(
                0,
                0,
                "GA_Lacrimosa_QTE",
                "GE_Player_Lacrimosa_QTE1_Damage",
                0,
                15_000_000,
            ),
            make_action(
                1_000_000,
                0,
                "GA_Lacrimosa_Skill",
                "GE_Player_Lacrimosa_Skill1_Damage",
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
        assert_eq!(report["hits_settled"], 2);
        assert_eq!(report["hit_receipts"][1]["non_crit_damage"], expected);
    }
}

#[test]
fn lacrimosa_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
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
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(report["hit_receipts"][0]["actor_index"], 0);

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);
}

#[test]
fn lacrimosa_burst_hypothesis_accepted_in_strict_mode() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.model_assumptions = std::collections::BTreeMap::from([
        (
            ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
            ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
                .description()
                .into(),
        ),
        (
            crate::characters::lacrimosa::skill::PACKETS_HYPOTHESIS_ID.into(),
            crate::characters::lacrimosa::skill::PACKETS_HYPOTHESIS_ID
                .description()
                .into(),
        ),
    ]);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Lacrimosa_UltraSkill",
        "GE_Player_Lacrimosa_UltraSkill1_Damage",
        120,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 5);
}

#[test]
fn lacrimosa_qte_skill_burst_rotation_settles_expected_hits() {
    let mut req = request();
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.relative_duration_us = 25_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Lacrimosa_QTE",
            "GE_Player_Lacrimosa_QTE1_Damage",
            0,
            15_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
            0,
            16_000_000,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Lacrimosa_UltraSkill",
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            120,
            20_000_000,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 7);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
}

#[test]
fn lacrimosa_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, coefficients, total) in [
        (
            1,
            0,
            [371, 160, 160, 160, 1139, 158, 158, 158, 1937, 1239, 371],
            6011,
        ),
        (
            8,
            0,
            [636, 274, 274, 274, 1952, 271, 271, 271, 3320, 2123, 636],
            10302,
        ),
        (
            8,
            3,
            [687, 296, 296, 296, 2108, 292, 292, 292, 3585, 2293, 687],
            11124,
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
                    "GA_Lacrimosa_Melee",
                    "GE_Player_Lacrimosa_Melee1_Damage",
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
        let stages = [1, 2, 2, 2, 3, 4, 4, 4, 6, 5, 1];
        for (hit, stage) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(stages)
        {
            assert_eq!(
                hit["damage_effect_id"],
                format!("GE_Player_Lacrimosa_Melee{stage}_Damage")
            );
        }
    }
}

#[test]
fn lacrimosa_normal_chain_reset_and_noop_boundaries() {
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
                "GA_Lacrimosa_Skill",
                "GE_Player_Lacrimosa_Skill1_Damage",
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
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_Melee1_Damage",
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
            "GE_Player_Lacrimosa_Melee2_Damage"
        } else {
            "GE_Player_Lacrimosa_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn lacrimosa_normal_chain_strict_hypothesis_and_off_field_four_hit_delivery() {
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
        crate::characters::lacrimosa::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::lacrimosa::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::LACRIMOSA_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::LACRIMOSA_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.model_assumptions.insert(
        crate::HypothesisId::CombatPacketCycleAccrualV1.into(),
        crate::HypothesisId::CombatPacketCycleAccrualV1
            .description()
            .into(),
    );
    req.actions = (0..4)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Lacrimosa_Melee",
                "GE_Player_Lacrimosa_Melee1_Damage",
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
    for (hit, stage) in delivered["hit_receipts"].as_array().unwrap()[5..]
        .iter()
        .zip([4, 4, 4, 6])
    {
        assert_eq!(hit["actor_index"], 0);
        assert_eq!(
            hit["damage_effect_id"],
            format!("GE_Player_Lacrimosa_Melee{stage}_Damage")
        );
    }
}

#[test]
fn lacrimosa_burst_multi_target_all_in_range() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Test ranged Burst across 2 targets: 5 hits each = 10 hits total
    req.actions = vec![make_action(
        0,
        0,
        "GA_Lacrimosa_UltraSkill",
        "GE_Player_Lacrimosa_UltraSkill1_Damage",
        100,
        16_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 10);
    let receipts = report["hit_receipts"].as_array().unwrap();
    for receipt in &receipts[..5] {
        assert_eq!(receipt["target_index"], 0);
    }
    for receipt in &receipts[5..10] {
        assert_eq!(receipt["target_index"], 1);
    }
}

#[test]
fn lacrimosa_burst_multi_target_strict_acceptance() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        crate::simulation::energy::LACRIMOSA_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::LACRIMOSA_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::lacrimosa::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::lacrimosa::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Lacrimosa_UltraSkill",
        "GE_Player_Lacrimosa_UltraSkill1_Damage",
        100,
        16_000_000,
    )];

    req.model_assumptions.insert(
        crate::characters::lacrimosa::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::lacrimosa::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 10);
}

#[test]
fn lacrimosa_two_continuing_cycles_preserve_resources_and_state() {
    let mut req = request();
    let shatter_model = crate::HypothesisId::LacrimosaBrokenShatterV1;
    req.model_assumptions
        .insert(shatter_model.into(), shatter_model.description().into());
    req.relative_duration_us = 260_000_000;
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;

    req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 100,
            denominator: 1,
        },
        current_gauge: Ratio {
            numerator: 0,
            denominator: 1,
        },
        broken_duration_us: 150_000_000,
        discord_fraction: Some(Ratio {
            numerator: 1,
            denominator: 5,
        }),
    });

    for hypothesis in [
        crate::simulation::energy::LACRIMOSA_HIT_ENERGY_HYPOTHESIS_ID,
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
        crate::characters::lacrimosa::skill::FORM_HYPOTHESIS_ID,
        crate::characters::lacrimosa::skill::CHAIN_HYPOTHESIS_ID,
        crate::characters::lacrimosa::skill::PACKETS_HYPOTHESIS_ID,
        crate::characters::lacrimosa::skill::ALL_IN_RANGE_HYPOTHESIS_ID,
        crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID,
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID,
    ] {
        req.model_assumptions
            .insert(hypothesis.into(), hypothesis.description().into());
    }

    let mut actions = Vec::new();
    for cycle in 0..2 {
        let base_t = cycle * 75_000_000;
        // Mode A Normal attacks: Stages 0..4
        for stage in 0..5 {
            actions.push(make_action(
                base_t + stage * 800_000,
                0,
                "GA_Lacrimosa_Melee",
                "GE_Player_Lacrimosa_Melee1_Damage",
                0,
                0,
            ));
        }
        // One ordinary Skill starts its sixteen-second cooldown.
        actions.push(make_action(
            base_t + 5_000_000,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
            0,
            16_000_000,
        ));
        // Stance switch to Mode B (Dual-Blade)
        actions.push(make_action(
            base_t + 10_000_000,
            0,
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_SwitchModB_Damage",
            0,
            0,
        ));
        // Mode B Normal attacks: Stages 0..4 (Stage 3 delivers B_Melee3_Explode -> Shatter & Discord)
        for stage in 0..5 {
            actions.push(make_action(
                base_t + 11_000_000 + stage * 1_000_000,
                0,
                "GA_Lacrimosa_Melee",
                "GE_Player_Lacrimosa_Melee1_Damage",
                0,
                0,
            ));
        }
        // Five admitted sections earn one extra; the original timer survives.
        actions.push(make_action(
            base_t + 18_000_000,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
            0,
            16_000_000,
        ));
        // Mode B Burst at t=20s
        actions.push(make_action(
            base_t + 20_000_000,
            0,
            "GA_Lacrimosa_UltraSkill",
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            0,
            16_000_000,
        ));
        // A section after ordinary expiry cannot earn an extra.
        actions.push(make_action(
            base_t + 22_000_000,
            0,
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_Melee1_Damage",
            0,
            0,
        ));
        // A new ordinary Skill after the prior deadline.
        actions.push(make_action(
            base_t + 25_000_000,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
            0,
            16_000_000,
        ));
        // Stance switch back to Mode A
        actions.push(make_action(
            base_t + 30_000_000,
            0,
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_Pan_Damage",
            0,
            0,
        ));
        // Mode A Burst at t=50s
        actions.push(make_action(
            base_t + 50_000_000,
            0,
            "GA_Lacrimosa_UltraSkill",
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            0,
            16_000_000,
        ));
    }

    req.actions = actions;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(report["hits_settled"].as_u64().unwrap() > 50);
    let shatter_metadata: Vec<_> = report["metadata_catalog"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|metadata| metadata["packet"]["semantic_tags"] == json!(["shatter"]))
        .collect();
    assert!(!shatter_metadata.is_empty());
    let provenance = json!({
        "kind": "accepted_model_hypothesis",
        "hypothesis_id": shatter_model.as_str(),
    });
    for metadata in shatter_metadata {
        assert_eq!(metadata["packet"]["semantic_provenance"], provenance);
        assert_eq!(metadata["binding"]["kind"], "owner_child");
        assert_eq!(metadata["binding"]["provenance"], provenance);
        assert_eq!(
            metadata["binding"]["initiating_ability_id"],
            metadata["action"]["ability_id"],
        );
    }
    assert_eq!(
        report["resolved_model_assumptions"][shatter_model.as_str()],
        shatter_model.description(),
    );
}
