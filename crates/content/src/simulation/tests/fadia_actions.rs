use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;

fn action(timestamp_us: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: "GA_Fadia_Melee".into(),
        damage_effect_id: "GE_Player_Fadia_Melee1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    }
}

fn burst(timestamp_us: u64, level: u32) -> GameActionCommand {
    let rates = [
        (39, 1000),
        (42, 1000),
        (45, 1000),
        (49, 1000),
        (53, 1000),
        (57, 1000),
        (62, 1000),
        (67, 1000),
        (72, 1000),
    ];
    let (numerator, denominator) = rates[(level - 1) as usize];
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: "GA_Fadia_UltraSkill".into(),
        damage_effect_id: "GE_Player_Fadia_UltraSkill1_Damage".into(),
        array_index: (level - 1) as usize,
        atk_rate: Ratio {
            numerator,
            denominator,
        },
        scaling_stat: crate::simulation::NativeScalingStat::Hp,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    }
}

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1039".into();
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
    req.actors[0].static_stats.elemental_bonuses.clear();
    req.actors[0].initial_hp = 10_000;
    req.actors[0].initial_hp_max = 10_000;
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![action(0)];
    req
}

#[test]
fn fadia_normal_chain_uses_native_published_coefficients_and_loops() {
    for (level, awakening, expected, chain_total, loop_total) in [
        (
            1,
            0,
            [433, 297, 360, 771, 283, 332, 273, 803, 433],
            (1776, 1),
            (3985, 2),
        ),
        (
            8,
            0,
            [742, 509, 617, 1321, 485, 569, 468, 1376, 742],
            (6087, 2),
            (6829, 2),
        ),
        (
            8,
            3,
            [801, 550, 666, 1427, 524, 615, 505, 1486, 801],
            (3287, 1),
            (7375, 2),
        ),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = (0..6).map(|index| action(index * 300_000)).collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 9);
        let receipts = report["hit_receipts"].as_array().unwrap();
        let expected_effects = [1, 2, 3, 4, 5, 6, 7, 8, 1];
        for ((receipt, coefficient), effect) in receipts.iter().zip(expected).zip(expected_effects)
        {
            let (numerator, denominator) = if coefficient % 2 == 0 {
                (coefficient / 2, 1)
            } else {
                (coefficient, 2)
            };
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", numerator, denominator))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                format!("GE_Player_Fadia_Melee{effect}_Damage")
            );
            let packet = &support::hit_metadata(&report, receipt)["packet"];
            assert_eq!(
                packet["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
            );
            assert_eq!(
                packet["native_damage_type_ex"],
                "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"
            );
        }
        let expected_total = expected.iter().sum::<u64>();
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", loop_total.0, loop_total.1))
        );
        assert_eq!(expected_total, loop_total.0 * (2 / loop_total.1));
        let mut chain_req = req;
        chain_req.actions.truncate(5);
        let chain = run_game_scenario(&chain_req, &ExecutionLimits::default()).unwrap();
        assert_eq!(chain["hits_settled"], 8);
        assert_eq!(
            serde_json::json!(formula_damage_total(&chain)),
            json!(format!("{}/{}", chain_total.0, chain_total.1))
        );
    }
}

#[test]
fn fadia_normal_chain_damage_ignores_hp_current_hp_and_character_defense() {
    let baseline = run_game_scenario(&request(), &ExecutionLimits::default()).unwrap();
    for changed in [
        {
            let mut req = request();
            req.actors[0].static_stats.hp += 50_000;
            req.actors[0].initial_hp += 50_000;
            req.actors[0].initial_hp_max += 50_000;
            req
        },
        {
            let mut req = request();
            req.actors[0].initial_hp = 1;
            req
        },
        {
            let mut req = request();
            req.actors[0].static_stats.def += 20_000;
            req
        },
    ] {
        let report = run_game_scenario(&changed, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            baseline["hit_receipts"][0]["non_crit_damage"],
            report["hit_receipts"][0]["non_crit_damage"]
        );
        assert_eq!(
            baseline["hit_receipts"][0]["damage_effect_id"],
            report["hit_receipts"][0]["damage_effect_id"]
        );
    }
}

#[test]
fn fadia_normal_chain_strict_mode_and_off_field_packet_horizon() {
    let mut req = request();
    req.model_assumptions.insert(
        crate::simulation::energy::FADIA_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::FADIA_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::fadia::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::fadia::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = vec![action(0), action(10)];
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 50,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.relative_duration_us = 99;
    let pending = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pending["hits_settled"], 0);
    assert_eq!(pending["pending_events"], 3);
    req.relative_duration_us = 110;
    let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(delivered["hits_settled"], 3);
    assert_eq!(delivered["pending_events"], 0);
    assert_eq!(delivered["final_buff_state"]["active_actor_index"], 1);
    let receipts = delivered["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts[0]["time_us"], 100);
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Fadia_Melee1_Damage"
    );
    for (receipt, (time, effect)) in receipts[1..].iter().zip([
        (110, "GE_Player_Fadia_Melee2_Damage"),
        (110, "GE_Player_Fadia_Melee3_Damage"),
    ]) {
        assert_eq!(receipt["actor_index"], 0);
        assert_eq!(receipt["time_us"], time);
        assert_eq!(receipt["damage_effect_id"], effect);
    }
    let mut mismatched = req.clone();
    mismatched.model_assumptions.insert(
        crate::characters::fadia::skill::CHAIN_HYPOTHESIS_ID.into(),
        "incorrect packet description".into(),
    );
    let mismatch = run_game_scenario(&mismatched, &ExecutionLimits::default()).unwrap_err();
    assert!(
        mismatch.contains("conflicting reached model hypothesis"),
        "unexpected mismatch error: {mismatch}"
    );
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 120,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    req.actions.push(action(130));
    req.relative_duration_us = 130;
    let queued_after_return = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(queued_after_return["hits_settled"], 3);
    assert_eq!(queued_after_return["pending_events"], 1);
    req.relative_duration_us = 230;
    let returned = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(returned["hits_settled"], 4);
    assert_eq!(
        returned["hit_receipts"][3]["damage_effect_id"],
        "GE_Player_Fadia_Melee1_Damage"
    );
}

#[test]
fn fadia_normal_chain_drops_later_packets_after_target_defeat() {
    let mut req = request();
    req.targets[0].hp_current = 300;
    req.actions = vec![action(0), action(10)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["pending_events"], 0);
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 2);
    assert_eq!(
        receipts[0]["damage_effect_id"],
        "GE_Player_Fadia_Melee1_Damage"
    );
    assert_eq!(
        receipts[1]["damage_effect_id"],
        "GE_Player_Fadia_Melee2_Damage"
    );
}

#[test]
fn fadia_normal_chain_gap_boundary_resets_only_after_two_seconds() {
    let mut req = request();
    req.actions = vec![action(0), action(2_000_000), action(4_000_001)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let effects = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|receipt| receipt["damage_effect_id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        effects,
        [
            "GE_Player_Fadia_Melee1_Damage",
            "GE_Player_Fadia_Melee2_Damage",
            "GE_Player_Fadia_Melee3_Damage",
            "GE_Player_Fadia_Melee1_Damage"
        ]
    );
}

#[test]
fn fadia_full_hp_initial_burst_uses_hp_coefficients_and_enters_lilith_window() {
    for (level, expected) in [(1, 1500), (8, 2590)] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, level, level]);
        req.actors[0].static_stats.hp = 10_000;
        req.actors[0].initial_hp = 10_000;
        req.actors[0].initial_hp_max = 10_000;
        req.actions = vec![burst(0, level)];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 6);
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", expected, 1))
        );
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(
            hits.iter()
                .map(|hit| hit["damage_effect_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "GE_Player_Fadia_UltraSkill1_Damage",
                "GE_Player_Fadia_UltraSkill2_Damage",
                "GE_Player_Fadia_UltraSkill2_Damage",
                "GE_Player_Fadia_UltraSkill2_Damage",
                "GE_Player_Fadia_UltraSkill2_Damage",
                "GE_Player_Fadia_UltraSkill3_Damage",
            ]
        );
        assert!(hits.iter().all(|hit| hit["scaling_stat"] == "hp"));

        let mut with_weapon = req.clone();
        with_weapon.actors[0].weapon_id = "weapon:fork_Arachne".into();
        with_weapon.actors[0].gear_procs = vec![GearProcConfig::EternalWaltz {
            max_hp_bonus: Ratio {
                numerator: 1,
                denominator: 5,
            },
            mental_bonus: Ratio {
                numerator: 1,
                denominator: 10,
            },
            duration_us: 10_000_000,
        }];
        with_weapon.actors[0].static_stats.hp = 12_000;
        with_weapon.actors[0].initial_hp = 12_000;
        with_weapon.actors[0].initial_hp_max = 12_000;
        let weapon_report = run_game_scenario(&with_weapon, &ExecutionLimits::default()).unwrap();
        let weapon_expected = if level == 1 {
            json!("1980/1")
        } else {
            json!("17094/5")
        };
        assert_eq!(
            serde_json::json!(formula_damage_total(&weapon_report)),
            weapon_expected
        );

        with_weapon.actors[0].static_stats.mental_damage_bonus = Ratio {
            numerator: 1,
            denominator: 10,
        };
        let set_report = run_game_scenario(&with_weapon, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            serde_json::json!(formula_damage_total(&set_report)),
            serde_json::to_value(ExactFraction::new(expected * 36, 25).unwrap()).unwrap()
        );
        assert!(
            set_report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|hit| { hit["static_mental_damage_bonus"] == json!("1/10") })
        );

        let mut r5 = req;
        r5.actors[0].weapon_id = "weapon:fork_Arachne".into();
        r5.actors[0].weapon_tier = 5;
        r5.actors[0].talent_levels = Some([1; 3]);
        r5.actions = vec![burst(0, 1)];
        r5.actors[0].gear_procs = vec![GearProcConfig::EternalWaltz {
            max_hp_bonus: Ratio {
                numerator: 8,
                denominator: 25,
            },
            mental_bonus: Ratio {
                numerator: 9,
                denominator: 50,
            },
            duration_us: 10_000_000,
        }];
        r5.actors[0].static_stats.hp = 13_200;
        r5.actors[0].initial_hp = 13_200;
        r5.actors[0].initial_hp_max = 13_200;
        let r5_report = run_game_scenario(&r5, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            serde_json::json!(formula_damage_total(&r5_report)),
            json!("11682/5")
        );
    }
}

#[test]
fn fadia_burst_rejects_wounds_and_awakening_and_lilith_controls_followups() {
    let mut wounded = request();
    wounded.actions = vec![burst(0, 1)];
    wounded.actors[0].initial_hp -= 1;
    assert!(
        run_game_scenario(&wounded, &ExecutionLimits::default())
            .unwrap_err()
            .contains("exactly full runtime HP")
    );

    let mut awakened = request();
    awakened.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    awakened.actions = vec![burst(0, 1)];
    assert!(
        run_game_scenario(&awakened, &ExecutionLimits::default())
            .unwrap_err()
            .contains("awakening 0 only")
    );

    let mut active = request();
    active.actions = vec![burst(0, 1), action(4_999_999)];
    assert!(
        run_game_scenario(&active, &ExecutionLimits::default())
            .unwrap_err()
            .contains("Lilith form window")
    );

    let mut expired = request();
    expired.actions = vec![burst(0, 1), action(9_417_366)];
    let report = run_game_scenario(&expired, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 7);
}

#[test]
fn fadia_burst_native_projection_uses_max_hp_and_requires_strict_hypothesis() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].initial_hp = 10_000;
    req.actors[0].initial_hp_max = 10_000;
    req.actions = vec![burst(0, 1)];
    if let GameActionCommand::InvokeAbility {
        atk_rate,
        scaling_stat,
        ..
    } = &mut req.actions[0]
    {
        *atk_rate = Ratio {
            numerator: 999,
            denominator: 1,
        };
        *scaling_stat = crate::simulation::NativeScalingStat::Def;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("1500/1")
    );
    let mut changed = req.clone();
    changed.actors[0].static_stats.atk = Ratio {
        numerator: 77_777,
        denominator: 1,
    };
    changed.actors[0].static_stats.def = 8888;
    changed.targets[0].defense = 999_999;
    let invariant = run_game_scenario(&changed, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        serde_json::json!(formula_damage_total(&invariant)),
        serde_json::json!(formula_damage_total(&report))
    );
    changed.actors[0].initial_hp = 20_000;
    changed.actors[0].initial_hp_max = 20_000;
    let doubled = run_game_scenario(&changed, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        serde_json::json!(formula_damage_total(&doubled)),
        json!("3000/1")
    );
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    if let GameActionCommand::InvokeAbility {
        atk_rate,
        scaling_stat,
        ..
    } = &mut req.actions[0]
    {
        *atk_rate = Ratio {
            numerator: 39,
            denominator: 1000,
        };
        *scaling_stat = crate::simulation::NativeScalingStat::Hp;
    }
    req.model_assumptions.insert(
        crate::characters::fadia::skill::BURST_HYPOTHESIS_ID.into(),
        crate::characters::fadia::skill::BURST_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let strict = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(strict["hits_settled"], 6);
}

#[test]
fn fadia_psyche_uses_its_override_while_burst_mental_uses_common_resistance() {
    let mut normal_baseline = request();
    normal_baseline.targets[0].resistance = Ratio {
        numerator: 1,
        denominator: 5,
    };
    normal_baseline.actions = vec![action(0)];
    let normal_base = run_game_scenario(&normal_baseline, &ExecutionLimits::default()).unwrap();
    let mut normal_override = normal_baseline.clone();
    normal_override.targets[0].elemental_resistances = BTreeMap::from([
        (
            "PSYCHE".into(),
            Ratio {
                numerator: 4,
                denominator: 5,
            },
        ),
        ("NATURE".into(), Ratio::zero()),
    ]);
    let normal_changed = run_game_scenario(&normal_override, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        ExactFraction::new(1, 4)
            .unwrap()
            .mul(
                serde_json::from_value(normal_base["hit_receipts"][0]["non_crit_damage"].clone())
                    .unwrap(),
            )
            .unwrap(),
        serde_json::from_value(normal_changed["hit_receipts"][0]["non_crit_damage"].clone())
            .unwrap()
    );

    let mut burst_baseline = request();
    burst_baseline.targets[0].resistance = Ratio {
        numerator: 1,
        denominator: 5,
    };
    burst_baseline.actors[0].initial_hp = 10_000;
    burst_baseline.actors[0].initial_hp_max = 10_000;
    burst_baseline.actions = vec![burst(0, 1)];
    let burst_base = run_game_scenario(&burst_baseline, &ExecutionLimits::default()).unwrap();
    let mut burst_override = burst_baseline;
    burst_override.targets[0].elemental_resistances = BTreeMap::from([(
        "PSYCHE".into(),
        Ratio {
            numerator: 4,
            denominator: 5,
        },
    )]);
    let burst_changed = run_game_scenario(&burst_override, &ExecutionLimits::default()).unwrap();
    assert_eq!(burst_base["hits_settled"], 6);
    assert_eq!(burst_changed["hits_settled"], 6);
    for (base, changed) in burst_base["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .zip(burst_changed["hit_receipts"].as_array().unwrap())
    {
        assert_eq!(base["non_crit_damage"], changed["non_crit_damage"]);
    }
}

#[test]
fn fadia_burst_multi_target_all_in_range() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].initial_hp = 10_000;
    req.actors[0].initial_hp_max = 10_000;
    req.actions = vec![burst(0, 1)];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 12);
    let receipts = report["hit_receipts"].as_array().unwrap();
    for receipt in &receipts[..6] {
        assert_eq!(receipt["target_index"], 0);
    }
    for receipt in &receipts[6..12] {
        assert_eq!(receipt["target_index"], 1);
    }
}

#[test]
fn fadia_burst_multi_target_strict_acceptance() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.actors[0].initial_hp = 10_000;
    req.actors[0].initial_hp_max = 10_000;
    req.model_assumptions.insert(
        crate::simulation::energy::FADIA_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::FADIA_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::fadia::skill::BURST_HYPOTHESIS_ID.into(),
        crate::characters::fadia::skill::BURST_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![burst(0, 1)];

    req.model_assumptions.insert(
        crate::characters::fadia::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::fadia::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 12);
}
