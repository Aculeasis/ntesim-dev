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

fn blackbird_energy_basis(charge_milli: u64) -> serde_json::Value {
    let mut left = charge_milli;
    let mut right = 1000;
    while right != 0 {
        (left, right) = (right, left % right);
    }
    json!(format!("{}/{}", charge_milli / left, 1000 / left))
}

fn blackbird_with_recovery(mut action: GameActionCommand, recovery_us: u64) -> GameActionCommand {
    if let GameActionCommand::InvokeAbility {
        recovery_us: action_recovery_us,
        hit_delay_us,
        ..
    } = &mut action
    {
        *action_recovery_us = recovery_us;
        // Match the authored DSL runner, which settles these modeled action
        // packets at their caller-supplied input timestamps.
        *hit_delay_us = 0;
    }
    action
}

fn assert_blackbird_continuation_timing(report: &serde_json::Value) {
    let continuation_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["command_index"].as_u64().unwrap() >= 32)
        .collect();
    assert_eq!(continuation_hits.len(), 17);
    for (command_index, time_us) in
        (32..37).zip([27_200_000, 28_000_000, 28_800_000, 29_600_000, 30_400_000])
    {
        assert!(
            continuation_hits
                .iter()
                .any(|hit| { hit["command_index"] == command_index && hit["time_us"] == time_us })
        );
    }
}

pub(super) fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1042".into();
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
        "GA_BlackBird_Melee",
        "GE_Player_BlackBird_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn blackbird_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Normal attack (GA_BlackBird_Melee / GE_Player_BlackBird_Melee1_Damage)
    // Coeffs: Lv1 = 0.237, Lv8 = 0.406, Lv9 (A3 resonance) = 0.439.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 0.237 * 0.5 = 237 / 2
    //   Lv8: 1000 * 0.406 * 0.5 = 203 / 1
    // At A3: resonance_3 grants +1 level -> Lv9:
    //   Lv9: 1000 * 0.439 * 0.5 = 439 / 2
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 237, 2),
        (8, 0, 203, 1),
        (8, 1, 203, 1),
        (8, 3, 439, 2),
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

    // Skill (GA_BlackBird_Skill / GE_Player_BlackBird_Skill_Damage: 4 hits)
    // Coeffs: Lv1 = 1.000, Lv8 = 1.714, Lv9 (A3 resonance) = 1.851.
    // At A0: ATK = 1000, DEF factor = 0.5.
    //   Lv1: 1000 * 1.000 * 0.5 = 500 / 1 (each hit) -> 4 hits total 2000 / 1
    //   Lv8: 1000 * 1.714 * 0.5 = 857 / 1 (each hit) -> 4 hits total 3428 / 1
    // A2 doubles the native coefficient: Lv8 1.714*2 = 3.428,
    // Lv9 (R3 investment) 1.851*2 = 3.702. It is CoefModify,
    // separate from Malice's additive coefficient on its own packet.
    for (level, awakening, expected_num, expected_den, total_num, total_den) in [
        (1, 0, 500, 1, 2000, 1),
        (8, 0, 857, 1, 3428, 1),
        (8, 2, 1714, 1, 6856, 1),
        (8, 3, 1851, 1, 7404, 1),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_BlackBird_Skill",
            "GE_Player_BlackBird_Skill_Damage",
            0,
            16_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 4);
        let receipts = report["hit_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 4);
        for receipt in receipts {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", expected_num, expected_den))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                "GE_Player_BlackBird_Skill_Damage"
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

    // Burst (GA_BlackBird_UltraSkill / GE_Player_BlackBird_UltraSkill_Damage: 9 hits)
    // Coeffs: Lv1 = 0.467, Lv8 = 0.800, Lv9 (A3 resonance) = 0.864.
    // on_cast refreshes Requies Animarum team ATK buff (+20% ATK -> 1200 ATK).
    // At A0: ATK = 1200, DEF factor = 0.5.
    //   Lv1: 1200 * 0.467 * 0.5 = 1401 / 5 (each hit) -> 9 hits total 12609 / 5
    //   Lv8: 1200 * 0.800 * 0.5 = 480 / 1 (each hit) -> 9 hits total 4320 / 1
    // R3 doubles the transformation coefficient after its +1 talent selection:
    // Lv9 .864*2 = 1.728; 1200*1.728*.5 = 1036.8 per hit.
    for (level, awakening, expected_num, expected_den, total_num, total_den) in [
        (1, 0, 1401, 5, 12609, 5),
        (8, 0, 480, 1, 4320, 1),
        (8, 3, 5184, 5, 46656, 5),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_BlackBird_UltraSkill",
            "GE_Player_BlackBird_UltraSkill_Damage",
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
        assert_eq!(receipts.len(), 9);
        for receipt in receipts {
            assert_eq!(
                receipt["non_crit_damage"],
                json!(format!("{}/{}", expected_num, expected_den))
            );
            assert_eq!(
                receipt["damage_effect_id"],
                "GE_Player_BlackBird_UltraSkill_Damage"
            );
            assert_eq!(
                super::support::hit_metadata(&report, receipt)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total_num, total_den))
        );
    }
}

#[test]
fn blackbird_off_field_delivery_and_target_boundary_checks() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_BlackBird_Skill",
            "GE_Player_BlackBird_Skill_Damage",
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
    assert_eq!(report["hits_settled"], 4);
    for i in 0..4 {
        assert_eq!(report["hit_receipts"][i]["actor_index"], 0);
    }

    let mut req_dead = request();
    req_dead.targets[0].hp_current = 0;
    let report_dead = run_game_scenario(&req_dead, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_dead["execution_completed"], true);
    assert_eq!(report_dead["hits_settled"], 0);
}

#[test]
fn blackbird_strict_mode_rejects_missing_hypothesis_and_accepts_valid_hypothesis() {
    let mut req = request();
    let cycle = crate::HypothesisId::CombatPacketCycleAccrualV1;
    req.model_assumptions
        .insert(cycle.into(), cycle.description().into());
    req.model_assumptions.insert(
        crate::simulation::energy::BLACKBIRD_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::BLACKBIRD_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 8;
    req.actions = vec![make_action(
        0,
        0,
        "GA_BlackBird_Skill",
        "GE_Player_BlackBird_Skill_Damage",
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
        crate::characters::blackbird::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::blackbird::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let fill = crate::HypothesisId::BlackbirdSkillCycleFillV1;
    req.model_assumptions
        .insert(fill.into(), fill.description().into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 4);
}

#[test]
fn blackbird_two_skill_one_burst_rotation_hit_count_and_energy() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 8, 8]);
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.relative_duration_us = 40_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_BlackBird_Skill",
            "GE_Player_BlackBird_Skill_Damage",
            0,
            16_000_000,
        ),
        make_action(
            16_000_000,
            0,
            "GA_BlackBird_Skill",
            "GE_Player_BlackBird_Skill_Damage",
            0,
            16_000_000,
        ),
        make_action(
            32_000_000,
            0,
            "GA_BlackBird_UltraSkill",
            "GE_Player_BlackBird_UltraSkill_Damage",
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
    assert_eq!(report["hits_settled"], 17);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
}

#[test]
fn blackbird_empty_start_normal_skill_burst_rotation_reconciles_damage_and_energy() {
    let mut req = request();
    req.actors.truncate(1);
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 100;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.relative_duration_us = 36_000_000;
    req.actions = (0..30)
        .map(|index| {
            blackbird_with_recovery(
                make_action(
                    index * 800_000,
                    0,
                    "GA_BlackBird_Melee",
                    "GE_Player_BlackBird_Melee1_Damage",
                    0,
                    0,
                ),
                800_000,
            )
        })
        .collect();
    req.actions.push(blackbird_with_recovery(
        make_action(
            24_000_000,
            0,
            "GA_BlackBird_Skill",
            "GE_Player_BlackBird_Skill_Damage",
            0,
            6_000_000,
        ),
        1_200_000,
    ));
    req.actions.push(blackbird_with_recovery(
        make_action(
            25_200_000,
            0,
            "GA_BlackBird_UltraSkill",
            "GE_Player_BlackBird_UltraSkill_Damage",
            100,
            20_000_000,
        ),
        2_000_000,
    ));
    req.actions.extend((0..5).map(|index| {
        blackbird_with_recovery(
            make_action(
                27_200_000 + index * 800_000,
                0,
                "GA_BlackBird_Melee",
                "GE_Player_BlackBird_Melee1_Damage",
                0,
                0,
            ),
            800_000,
        )
    }));

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 132);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("41840/1")
    );

    let mut expected_effects = Vec::new();
    let mut expected_damage = Vec::new();
    for _ in 0..6 {
        for (stage, count, numerator, denominator) in [
            (1, 1, 203, 1),
            (2, 2, 835, 2),
            (3, 4, 258, 1),
            (4, 5, 221, 1),
            (5, 5, 312, 1),
        ] {
            for _ in 0..count {
                expected_effects.push(format!("GE_Player_BlackBird_Melee{stage}_Damage"));
                expected_damage.push((numerator, denominator));
            }
        }
    }
    expected_effects.extend(std::iter::repeat_n(
        "GE_Player_BlackBird_Skill_Damage".to_owned(),
        4,
    ));
    expected_damage.extend(std::iter::repeat_n((857, 1), 4));
    expected_effects.extend(std::iter::repeat_n(
        "GE_Player_BlackBird_UltraSkill_Damage".to_owned(),
        9,
    ));
    expected_damage.extend(std::iter::repeat_n((480, 1), 9));
    for (stage, count, numerator, denominator) in [
        (1, 1, 203, 1),
        (2, 2, 835, 2),
        (3, 4, 258, 1),
        (4, 5, 221, 1),
        (5, 5, 312, 1),
    ] {
        for _ in 0..count {
            expected_effects.push(format!("GE_Player_BlackBird_Melee{stage}_Damage"));
            expected_damage.push((numerator * 6, denominator * 5));
        }
    }
    for (hit, (effect, (numerator, denominator))) in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .zip(expected_effects.iter().zip(expected_damage))
    {
        assert_eq!(hit["damage_effect_id"].as_str().unwrap(), effect.as_str());
        assert_eq!(
            hit["non_crit_damage"],
            serde_json::to_value(ExactFraction::new(numerator, denominator).unwrap()).unwrap()
        );
    }
    assert_blackbird_continuation_timing(&report);

    // Frozen ChargeAdd values are milli-energy. Seven chains yield 136,948m;
    // Skill adds 7,560m. The owner gets 144 whole energy, caps at 100 with 24
    // overflow, spends 100, then ends at 20 with 127/250 carry. The ally gets
    // 3/5 of the same source total: 86 whole energy plus 881/1250 carry.
    let melee_charge_milli = [
        838, 1726, 1726, 1066, 1066, 1066, 1066, 913, 913, 913, 913, 913, 1289, 1289, 1289, 1289,
        1289,
    ];
    let expected_charge_milli: Vec<u64> = (0..6)
        .flat_map(|_| melee_charge_milli)
        .chain(std::iter::repeat_n(1890, 4))
        .chain(melee_charge_milli)
        .collect();
    let mut packet_index = [0; 2];
    for event in report["trace"].as_array().unwrap() {
        let fanout = &event["energy_fanout"];
        if fanout["source"] != "blackbird_hit" {
            continue;
        }
        let recipients = fanout["recipients"].as_array().unwrap();
        assert_eq!(recipients.len(), 2);
        for entry in recipients {
            let recipient = entry["recipient_actor_index"].as_u64().unwrap() as usize;
            let index = packet_index[recipient];
            assert!(
                index < expected_charge_milli.len(),
                "unexpected Blackbird hit-energy receipt after Normal and Skill packets; Burst grants none"
            );
            let charge = expected_charge_milli[index];
            packet_index[recipient] += 1;
            assert_eq!(fanout["base"], blackbird_energy_basis(charge));
            assert_eq!(
                entry["distribution"],
                if recipient == 0 {
                    json!("1/1")
                } else {
                    json!("3/5")
                }
            );
        }
    }
    assert_eq!(packet_index, [123, 123]);
    assert_eq!(report["final_actor_states"][0]["energy"], 20);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 144);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 24);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("127/250")
    );
    assert_eq!(report["final_actor_states"][1]["energy"], 86);
    assert_eq!(report["final_actor_states"][1]["granted_energy"], 86);
    assert_eq!(
        report["final_actor_states"][1]["charge_remainder"],
        json!("881/1250")
    );
}

#[test]
fn blackbird_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, coefficients, total) in [
        (
            1,
            0,
            [
                237, 487, 487, 301, 301, 301, 301, 258, 258, 258, 258, 258, 364, 364, 364, 364,
                364, 237,
            ],
            5762,
        ),
        (
            8,
            0,
            [
                406, 835, 835, 516, 516, 516, 516, 442, 442, 442, 442, 442, 624, 624, 624, 624,
                624, 406,
            ],
            9876,
        ),
        (
            8,
            3,
            [
                439, 901, 901, 557, 557, 557, 557, 478, 478, 478, 478, 478, 674, 674, 674, 674,
                674, 439,
            ],
            10668,
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
                    "GA_BlackBird_Melee",
                    "GE_Player_BlackBird_Melee1_Damage",
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
        assert_eq!(report["hits_settled"], 18);
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
        let stages = [1, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 1];
        for (hit, stage) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(stages)
        {
            assert_eq!(
                hit["damage_effect_id"],
                format!("GE_Player_BlackBird_Melee{stage}_Damage")
            );
        }
    }
}

#[test]
fn blackbird_normal_chain_reset_and_noop_boundaries() {
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
                "GA_BlackBird_Skill",
                "GE_Player_BlackBird_Skill_Damage",
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
            "GA_BlackBird_Melee",
            "GE_Player_BlackBird_Melee1_Damage",
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
            "GE_Player_BlackBird_Melee2_Damage"
        } else {
            "GE_Player_BlackBird_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn blackbird_normal_chain_strict_hypothesis_and_off_field_five_hit_delivery() {
    let mut req = request();
    let cycle = crate::HypothesisId::CombatPacketCycleAccrualV1;
    req.model_assumptions
        .insert(cycle.into(), cycle.description().into());
    req.model_assumptions.insert(
        crate::simulation::energy::BLACKBIRD_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::BLACKBIRD_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::blackbird::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::blackbird::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = (0..4)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_BlackBird_Melee",
                "GE_Player_BlackBird_Melee1_Damage",
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
    assert_eq!(pending["hits_settled"], 7);
    req.relative_duration_us = 3100;
    let delivered = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(delivered["hits_settled"], 12);
    for hit in &delivered["hit_receipts"].as_array().unwrap()[7..] {
        assert_eq!(hit["actor_index"], 0);
        assert_eq!(hit["damage_effect_id"], "GE_Player_BlackBird_Melee4_Damage");
    }
}

#[test]
fn blackbird_multi_target_all_in_range_skill_and_burst() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Test Skill across 2 targets: 4 hits each = 8 hits
    req.actions = vec![make_action(
        0,
        0,
        "GA_BlackBird_Skill",
        "GE_Player_BlackBird_Skill_Damage",
        0,
        6_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 8);
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
    assert_eq!(target_0_hits, 4);
    assert_eq!(target_1_hits, 4);

    // Test Burst across 2 targets: 9 hits each = 18 hits
    let mut req_burst = request();
    let mut target2 = req_burst.targets[0].clone();
    target2.target_index = 1;
    req_burst.targets.push(target2);
    req_burst.actors[0].initial_energy = 100;
    req_burst.actions = vec![make_action(
        0,
        0,
        "GA_BlackBird_UltraSkill",
        "GE_Player_BlackBird_UltraSkill_Damage",
        100,
        20_000_000,
    )];
    let report_burst = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["execution_completed"], true);
    assert_eq!(report_burst["hits_settled"], 18);

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
        crate::characters::blackbird::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::blackbird::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req_burst.model_assumptions.insert(
        crate::characters::blackbird::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::blackbird::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let groups = crate::characters::blackbird::skill::GROUPS_HYPOTHESIS_ID;
    req_burst
        .model_assumptions
        .insert(groups.into(), groups.description().into());
    let report_strict = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_strict["execution_completed"], true);
    assert_eq!(report_strict["hits_settled"], 18);
}
