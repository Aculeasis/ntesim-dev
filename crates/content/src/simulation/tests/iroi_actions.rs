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
    req.actors[0].character_id = "character:1075".into();
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
        "GA_Oneiroi_Melee",
        "GE_Player_Oneiroi_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn iroi_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Test Normal Attack (GA_Oneiroi_Melee / GE_Player_Oneiroi_Melee1_Damage)
    // Coeffs: Lv1 = 0.76, Lv8 = 1.303, Lv9 (A3 resonance) = 1.407
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 0.76 * 0.5 = 380 / 1
    // Lv8: 1000 * 1.303 * 0.5 = 1303 / 2
    // Lv9: 1000 * 1.407 * 0.5 = 1407 / 2
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 380, 1),
        (8, 0, 1303, 2),
        (8, 1, 1303, 2),
        (8, 3, 1407, 2),
        (8, 4, 1407, 2),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
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

    // Test Skill (GA_Oneiroi_Skill / GE_Player_Oneiroi_Skill_Gun_Damage)
    // Gun Coeffs: Lv1 = 1.5, Lv8 = 2.571, Lv9 (A3 resonance) = 2.776
    // Skill cast grants 5% DEF ignore (mitigation = 20 / 39).
    // Gun Damage:
    // Lv1: 1000 * 1.5 * (20 / 39) = 10000 / 13
    // Lv8: 1000 * 2.571 * (20 / 39) = 17140 / 13
    // Lv9: 1000 * 2.776 * (20 / 39) = 55520 / 39
    // Lv8 + A4 (+30% DMG): 1000 * 2.571 * (20 / 39) * 1.3 = 1714 / 1
    for (level, awakening, expected_hits, expected_num, expected_den) in [
        (1, 0, 9, 10000, 13),
        (8, 0, 9, 17140, 13),
        (8, 1, 9, 17140, 13),
        (8, 2, 10, 17140, 13),
        (8, 3, 10, 55520, 39),
        (8, 4, 10, 5552, 3),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Oneiroi_Skill",
            "GE_Player_Oneiroi_Skill_Gun_Damage",
            0,
            6_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(report["hits_settled"], expected_hits);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            super::support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
    }

    // Test Ultimate/Burst (GA_Oneiroi_UltraSkill / GE_Player_Oneiroi_UltraSkill_Damage)
    // Coeffs: Lv1 = 3.0, Lv8 = 5.141, Lv9 (A3 resonance) = 5.553
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 3.0 * 0.5 = 1500 / 1
    // Lv8: 1000 * 5.141 * 0.5 = 5141 / 2
    // Lv9: 1000 * 5.553 * 0.5 = 5553 / 2
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 1500, 1),
        (8, 0, 5141, 2),
        (8, 1, 5141, 2),
        (8, 3, 5553, 2),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Oneiroi_UltraSkill",
            "GE_Player_Oneiroi_UltraSkill_Damage",
            100,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
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
fn iroi_normal_chain_uses_all_five_native_stages_and_loops() {
    for (level, awakening, milli_coefficients) in [
        (1, 0, [760, 159, 103, 702, 258]),
        (8, 0, [1303, 272, 177, 1203, 442]),
        (8, 3, [1407, 294, 191, 1299, 478]),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = (0..10)
            .map(|index| {
                make_action(
                    index * 300_000,
                    0,
                    "GA_Oneiroi_Melee",
                    "GE_Player_Oneiroi_Melee1_Damage",
                    0,
                    0,
                )
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(report["hits_settled"], 32);
        let packet_ids: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| hit["damage_effect_id"].as_str().unwrap())
            .collect();
        let expected_stage_packets = [
            "GE_Player_Oneiroi_Melee1_Damage",
            "GE_Player_Oneiroi_Melee2_Damage",
            "GE_Player_Oneiroi_Melee2_Damage",
            "GE_Player_Oneiroi_Melee2_Damage",
            "GE_Player_Oneiroi_Melee3_Damage",
            "GE_Player_Oneiroi_Melee3_Damage",
            "GE_Player_Oneiroi_Melee3_Damage",
            "GE_Player_Oneiroi_Melee3_Damage",
            "GE_Player_Oneiroi_Melee3_Damage",
            "GE_Player_Oneiroi_Melee3_Damage",
            "GE_Player_Oneiroi_Melee3_Damage",
            "GE_Player_Oneiroi_Melee3_Damage",
            "GE_Player_Oneiroi_Melee4_Damage",
            "GE_Player_Oneiroi_Melee5_Damage",
            "GE_Player_Oneiroi_Melee5_Damage",
            "GE_Player_Oneiroi_Melee5_Damage",
        ];
        assert_eq!(packet_ids, expected_stage_packets.repeat(2));

        let per_stage = [1, 3, 8, 1, 3];
        let mut receipt_index = 0;
        for _ in 0..2 {
            for (stage, multiplicity) in per_stage.into_iter().enumerate() {
                for _ in 0..multiplicity {
                    let milli = milli_coefficients[stage];
                    let (numerator, denominator) = if milli % 2 == 0 {
                        (milli / 2, 1)
                    } else {
                        (milli, 2)
                    };
                    assert_eq!(
                        report["hit_receipts"][receipt_index]["non_crit_damage"],
                        json!(format!("{}/{}", numerator, denominator))
                    );
                    receipt_index += 1;
                }
            }
        }
        let sum_milli: u64 = milli_coefficients
            .into_iter()
            .zip(per_stage)
            .map(|(coefficient, count)| coefficient * count)
            .sum();
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", sum_milli, 1))
        );
    }
}

#[test]
fn iroi_normal_chain_resets_at_gap_ability_and_field_exit_but_not_noop_switch() {
    let mut exact_gap = request();
    exact_gap.relative_duration_us = 3_000_000;
    exact_gap.actions = vec![
        make_action(
            0,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_000,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&exact_gap, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 4);
    assert_eq!(
        report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Oneiroi_Melee2_Damage"
    );

    let mut req = request();
    req.relative_duration_us = 5_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            2_000_001,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 2);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| { hit["damage_effect_id"] == "GE_Player_Oneiroi_Melee1_Damage" })
    );

    let mut req = request();
    req.relative_duration_us = 5_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            1_000,
            0,
            "GA_Oneiroi_Skill",
            "GE_Player_Oneiroi_Skill_Gun_Damage",
            0,
            0,
        ),
        make_action(
            2_000,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
        "GE_Player_Oneiroi_Melee1_Damage"
    );

    let mut req = request();
    req.relative_duration_us = 5_000_000;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            3_000,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
        "GE_Player_Oneiroi_Melee1_Damage"
    );

    let mut req = request();
    req.relative_duration_us = 3_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            2_000,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["hit_receipts"][1]["damage_effect_id"],
        "GE_Player_Oneiroi_Melee2_Damage"
    );
}

#[test]
fn iroi_normal_chain_requires_explicit_strict_hypothesis() {
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
        crate::characters::iroi::skill::CHAIN_HYPOTHESIS_ID.into(),
        "Iroi chain uses one hit at each stage".into(),
    );
    let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        error.contains("conflicting reached model hypothesis"),
        "{error}"
    );
    req.model_assumptions.insert(
        crate::characters::iroi::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::iroi::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::IROI_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::IROI_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = (0..5)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Oneiroi_Melee",
                "GE_Player_Oneiroi_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 16);
}

#[test]
fn iroi_normal_chain_keeps_off_field_owner_and_pending_or_missed_hits() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
        make_action(
            300,
            0,
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 350,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 4);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Melee2_Damage")
            .all(|hit| hit["actor_index"] == 0)
    );

    let mut pending = request();
    pending.relative_duration_us = 50;
    let report = run_game_scenario(&pending, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(report["pending_events"], 1);
    assert_eq!(report["execution_completed"], true);

    let mut missed = request();
    missed.targets[0].hp_current = 0;
    let report = run_game_scenario(&missed, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 0);
}

#[test]
fn iroi_normal_chain_rejects_a_non_primary_packet() {
    let mut req = request();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_Melee",
        "GE_Player_Oneiroi_Melee2_Damage",
        0,
        0,
    )];
    let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("modeled first packet"), "{error}");
}

#[test]
fn iroi_off_field_delivery_and_dead_target_boundary() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Oneiroi_Skill",
            "GE_Player_Oneiroi_Skill_Gun_Damage",
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
    assert_eq!(report["hits_settled"], 10);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["actor_index"] == 0)
    );

    let mut dead_req = request();
    dead_req.targets[0].hp_current = 0;
    dead_req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_Melee",
        "GE_Player_Oneiroi_Melee1_Damage",
        0,
        0,
    )];
    let dead_report = run_game_scenario(&dead_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead_report["hits_settled"], 0);
}

#[test]
fn iroi_skill_heals_slots_once_at_the_effective_skill_coordinate() {
    for (skill_level, awakening, expected_gross, expected_ratio) in
        [(1, 0, 943, "943/1000"), (8, 3, 2_373, "349/200")]
    {
        // R3 level9: 1000 ATK * 1.745 * 1.36 = 2373.2 -> 2373.
        let mut req = request();
        req.actors[0].slot = 2;
        req.actors[0].initial_hp = 5_000;
        req.actors[0].initial_hp_max = 10_000;
        req.actors[0].talent_levels = Some([1, skill_level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let mut first = req.actors[0].clone();
        first.slot = 0;
        first.character_id = "character:1019".into();
        first.initial_hp = 10_000;
        first.initial_hp_max = 20_000;
        let mut dead = first.clone();
        dead.slot = 1;
        dead.character_id = "character:1042".into();
        dead.initial_hp = 0;
        dead.initial_hp_max = 10_000;
        req.actors.extend([first, dead]);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Oneiroi_Skill",
            "GE_Player_Oneiroi_Skill_Gun_Damage",
            0,
            6_000_000,
        )];

        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let heals = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "iroi_skill_heal")
            .collect::<Vec<_>>();
        assert_eq!(heals.len(), 2);
        for (heal, expected_to, applied, overflow, hp_after) in [
            (
                heals[0],
                1,
                10_000u64.min(expected_gross),
                expected_gross.saturating_sub(10_000),
                10_000 + 10_000u64.min(expected_gross),
            ),
            (heals[1], 0, expected_gross, 0, 5000 + expected_gross),
        ] {
            assert_eq!(heal["from_actor"], 0);
            assert_eq!(heal["to_actor"], expected_to);
            assert_eq!(heal["data"]["input"], "skill");
            assert_eq!(heal["data"]["exact_raw"], "1000/1");
            assert_eq!(heal["data"]["atk_ratio"], expected_ratio);
            assert_eq!(heal["data"]["gross"], expected_gross);
            assert_eq!(heal["data"]["applied"], applied);
            assert_eq!(heal["data"]["overflow"], overflow);
            assert_eq!(heal["data"]["hp_after"], hp_after);
        }
        assert!(heals.iter().all(|event| event["time_us"] == 0));
    }
}

#[test]
fn iroi_skill_healing_uses_shared_outgoing_and_incoming_modifiers() {
    let mut req = request();
    req.actors[0].initial_hp = 100;
    req.actors[0].initial_hp_max = 1_000;
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    let mut recipient = req.actors[0].clone();
    recipient.slot = 1;
    recipient.character_id = "character:1019".into();
    recipient.initial_hp = 100;
    recipient.initial_hp_max = 10_000;
    let mut modifier_source = recipient.clone();
    modifier_source.slot = 2;
    modifier_source.character_id = "character:1042".into();
    req.actors.extend([recipient, modifier_source]);

    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state
        .apply_buff(
            2,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "test.iroi_incoming_healing",
                "test",
                ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(1),
                10_000_000,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([(
                    "incoming_healing_bonus".into(),
                    crate::simulation::ExactFraction::new(1, 5).unwrap(),
                )]),
            ),
        )
        .unwrap();
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::IroiSkillHeal {
            source_actor_index: 0,
            recipient_actor_index: 1,
            atk_ratio: crate::simulation::ExactFraction::new(943, 1_000).unwrap(),
        },
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    assert_eq!(
        trace[0].data.as_ref().and_then(|d| d.gross.zip(d.applied)),
        Some((1245, 1245))
    );
    assert_eq!(state.actors[1].hp, 1_345);
    assert_eq!(
        state.execution_model_assumptions
            [crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.as_str()],
        crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description()
    );
}

#[test]
fn iroi_burst_multi_target_all_in_range() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_UltraSkill",
        "GE_Player_Oneiroi_UltraSkill_Damage",
        100,
        16_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(report["hits_settled"], 2);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts[0]["target_index"], 0);
    assert_eq!(receipts[1]["target_index"], 1);
}

#[test]
fn iroi_burst_multi_target_strict_acceptance() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        crate::simulation::energy::IROI_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::IROI_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::iroi::skill::BURST_HYPOTHESIS_ID.into(),
        crate::characters::iroi::skill::BURST_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::iroi::skill::BURST_HEAL_HYPOTHESIS_ID.into(),
        crate::characters::iroi::skill::BURST_HEAL_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_UltraSkill",
        "GE_Player_Oneiroi_UltraSkill_Damage",
        100,
        16_000_000,
    )];

    req.model_assumptions.insert(
        crate::characters::iroi::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::iroi::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(report["hits_settled"], 2);
}

fn packets<'a>(report: &'a serde_json::Value, effect: &str) -> Vec<&'a serde_json::Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|receipt| {
            super::support::hit_metadata(report, receipt)["packet"]["damage_effect_id"].as_str()
                == Some(effect)
        })
        .collect()
}

#[test]
fn iroi_riposte_from_critical_dodge_consumed_once_and_grants_energy_and_moon() {
    let source = r#"
options iteration=1 duration=8 crit=fixed stamina_capacity=60 initial_stamina=60 stamina_use_efficiency=1 stamina_recovery_rate=0;
iroi char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
target hp=100000000 def=0 resist=0;
active iroi;

iroi dash[critical=true]; wait(0.4); iroi attack;
wait(0.8); iroi attack;
"#;
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    req.model_assumptions.insert(
        crate::HypothesisId::IroiRiposteV1.as_str().into(),
        crate::HypothesisId::IroiRiposteV1.description().into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );

    let hits = packets(&report, "GE_Player_Oneiroi_PerfectEvadeAttack_Damage");
    assert_eq!(hits.len(), 1);
    let riposte_hit = hits[0];
    assert_eq!(
        super::support::hit_metadata(&report, riposte_hit)["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Z"
    );
    // At Iroi lv80 base ATK 596, 0 DEF, Lv1 coeff 1.340 -> 596 * 1.340 = 798.64 = 19966 / 25
    assert_eq!(riposte_hit["non_crit_damage"], json!("19966/25"));

    assert_eq!(packets(&report, "GE_Player_Oneiroi_Melee1_Damage").len(), 1);
}

#[test]
fn iroi_plunge_damage_category_and_moon() {
    let mut req = request();
    req.actors[0].talent_levels = Some([1, 1, 1]);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_Melee",
        "GE_Player_Oneiroi_AirAttack_Damage",
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
    let hit = &report["hit_receipts"][0];
    assert_eq!(
        hit["damage_effect_id"],
        "GE_Player_Oneiroi_AirAttack_Damage"
    );
    assert_eq!(
        super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X"
    );
    assert_eq!(hit["non_crit_damage"], json!("180/1"));
}

#[test]
fn iroi_daydream_branch_spends_moon_and_falls_back_on_insufficient_moon() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_Melee",
        "GE_Player_Oneiroi_Branch1_Damage",
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
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Oneiroi_Melee1_Damage"
    );

    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_Melee",
        "GE_Player_Oneiroi_Branch1_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = packets(&report, "GE_Player_Oneiroi_Branch1_Damage");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["non_crit_damage"], json!("184/1"));

    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_Melee",
        "GE_Player_Oneiroi_Branch5_Damage",
        0,
        0,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = packets(&report, "GE_Player_Oneiroi_Branch5_Damage");
    assert_eq!(hits.len(), 35);
    assert_eq!(hits[0]["non_crit_damage"], json!("237/2"));
}

#[test]
fn iroi_dream_state_ivory_vs_horn_selection_and_pulses() {
    let mut req = request();
    req.relative_duration_us = 20_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(1);
    req.model_assumptions.insert(
        crate::HypothesisId::IroiDreamStateV1.as_str().into(),
        crate::HypothesisId::IroiDreamStateV1.description().into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_UltraSkill",
        "GE_Player_Oneiroi_UltraSkill_Damage",
        100,
        16_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let direct_hits = packets(&report, "GE_Player_Oneiroi_UltraSkill_Damage");
    assert_eq!(direct_hits.len(), 1);
    let pulse_hits = packets(&report, "GE_Player_Oneiroi_UltraSkill1_Damage");
    assert_eq!(
        pulse_hits.len(),
        12,
        "expected 12 pulses, got {}",
        pulse_hits.len()
    );

    let mut req = request();
    req.relative_duration_us = 20_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.model_assumptions.insert(
        crate::HypothesisId::IroiDreamStateV1.as_str().into(),
        crate::HypothesisId::IroiDreamStateV1.description().into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Oneiroi_UltraSkill",
        "GE_Player_Oneiroi_UltraSkill_Damage",
        100,
        16_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert!(packets(&report, "GE_Player_Oneiroi_UltraSkill_Damage").is_empty());
    assert!(packets(&report, "GE_Player_Oneiroi_UltraSkill1_Damage").is_empty());
}

#[test]
fn iroi_b2_blossom_bud_mirror_image_ratio_and_cap() {
    let mut req = crate::simulation::tests::research_request();
    req.stochastic_crit = None;
    req.actions.clear();
    req.action_metadata.clear();
    req.relative_duration_us = 15_000_000;

    let mut actor0 = req.actors[0].clone();
    actor0.slot = 0;
    actor0.character_id = "character:1046".into();
    actor0.initial_cycle = 100;
    actor0.gear_procs.clear();
    req.actors[0] = actor0;

    let mut iroi = req.actors[0].clone();
    iroi.slot = 1;
    iroi.character_id = "character:1075".into();
    iroi.awakening = ntesim_data::AwakeningSelection::prefix(2);
    iroi.initial_cycle = 0;
    iroi.static_stats.cycle_intensity = Ratio {
        numerator: 600,
        denominator: 1,
    };
    iroi.gear_procs.clear();
    req.actors.push(iroi);

    req.model_assumptions.insert(
        crate::HypothesisId::CombatEsperCycleReactionsV1
            .as_str()
            .into(),
        crate::HypothesisId::CombatEsperCycleReactionsV1
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::HypothesisId::IroiBudCopyB2V1.as_str().into(),
        crate::HypothesisId::IroiBudCopyB2V1.description().into(),
    );

    req.actions = vec![GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 1,
        cooldown_us: 0,
    }];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|receipt| {
            super::support::hit_metadata(&report, receipt)["packet"]["damage_effect_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("ntesim.reaction.blossom"))
        })
        .collect();
    assert_eq!(hits.len(), 25);
    assert_eq!(hits[0]["non_crit_damage"], json!("7200/1"));
    let copied_hits: Vec<_> = hits
        .iter()
        .filter(|hit| hit["non_crit_damage"] == json!("2700/1"))
        .collect();
    assert_eq!(copied_hits.len(), 20);

    // Four generations reach the copy boundary before the first copy's own
    // ten-second lifetime ends. Replacing a parent must not delete its copy;
    // the fourth copy replaces only the oldest copy and its remaining shots.
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let snapshot = crate::simulation::reactions::ReactionSnapshot {
        command_index: 0,
        switch_origin: crate::simulation::GameSwitchOrigin::EsperCycle {
            outgoing_actor_index: 0,
            incoming_actor_index: 1,
        },
        owner_index: 1,
        level: 80,
        cycle_intensity: ExactFraction::new(600, 1).unwrap(),
        element_key: "NATURE".into(),
        crit_damage: ExactFraction::new(0, 1).unwrap(),
    };
    for id in 0..4 {
        let birth = id * 3_000_000;
        state.advance_unpaused_test_clock(birth);
        crate::simulation::reactions::create_blossom(
            &mut crate::simulation::context::EventContext {
                request: &req,
                prepared_packets: &[],
                state: &mut state,
                queue: &mut queue,
                sequence: &mut sequence,
                trace: &mut trace,
                rng: &mut None,
                hits: &mut Vec::new(),
                action_metadata_ledger: &mut Vec::new(),
            },
            birth,
            snapshot.clone(),
        )
        .unwrap();
        state.advance_unpaused_test_clock(birth + 3_000_000);
        crate::characters::iroi::bud_copy::on_bud_copy_check(
            &mut crate::runtime_api::CallbackContext {
                state: &mut state,
                request: &req,
                queue: &mut queue,
                sequence: &mut sequence,
                trace: &mut trace,
            },
            id,
            snapshot.clone(),
        )
        .unwrap();
    }
    assert_eq!(
        state.actors[1].character.iroi_mut().unwrap().bud_copy_ids,
        std::collections::VecDeque::from([1, 2, 3])
    );
    let mut ctx = crate::runtime_api::CallbackContext {
        state: &mut state,
        request: &req,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    crate::simulation::reactions::expire_bud(&mut ctx, 1, false).unwrap();
    crate::simulation::reactions::expire_bud(&mut ctx, 0, true).unwrap(); // Stale replaced-copy expiry.
    assert_eq!(
        ctx.state.actors[1]
            .character
            .iroi_mut()
            .unwrap()
            .bud_copy_ids
            .len(),
        3
    );
    crate::simulation::reactions::expire_bud(&mut ctx, 1, true).unwrap();
    assert_eq!(
        ctx.state.actors[1]
            .character
            .iroi_mut()
            .unwrap()
            .bud_copy_ids,
        std::collections::VecDeque::from([2, 3])
    );
    assert!(!queue.iter().any(|event| matches!(&event.data,
        crate::simulation::events::GameEventData::ReactionPistil(pistil)
            if pistil.is_copy && pistil.bud_id < 2)));
}

fn make_melee_chain(start_us: u64) -> Vec<GameActionCommand> {
    (0..5)
        .map(|step| {
            make_action(
                start_us + step * 500_000,
                0,
                "GA_Oneiroi_Melee",
                "GE_Player_Oneiroi_Melee1_Damage",
                0,
                0,
            )
        })
        .collect()
}

fn cycle_actions(offset_us: u64) -> Vec<GameActionCommand> {
    let mut actions = Vec::new();
    for t_sec in [0, 5, 10, 15, 20, 25] {
        actions.extend(make_melee_chain(offset_us + t_sec * 1_000_000));
    }
    actions.push(make_action(
        offset_us + 30_000_000,
        0,
        "GA_Oneiroi_Skill",
        "GE_Player_Oneiroi_Skill_Gun_Damage",
        0,
        12_000_000,
    ));
    actions.push(make_action(
        offset_us + 34_000_000,
        0,
        "GA_Oneiroi_UltraSkill",
        "GE_Player_Oneiroi_UltraSkill_Damage",
        100,
        16_000_000,
    ));
    actions.extend(make_melee_chain(offset_us + 56_000_000));
    actions
}

#[test]
fn iroi_two_continuing_cycles_preserve_resources_and_state() {
    let mut req = request();
    req.relative_duration_us = 150_000_000;
    req.actors[0].character_level = 80;
    req.actors[0].character_max_level = Some(80);
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 100;

    req.targets[0].hp_current = 24_000_000;
    req.targets[0].hp_max = 24_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio {
        numerator: 1,
        denominator: 5,
    }; // 0.20

    req.model_assumptions.insert(
        crate::HypothesisId::IroiDreamStateV1.as_str().into(),
        crate::HypothesisId::IroiDreamStateV1.description().into(),
    );

    let mut actions = cycle_actions(0);
    actions.extend(cycle_actions(75_000_000));
    req.actions = actions;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(report["invariants_passed"], true);

    // 270 damage packets settled over 150s (135 per 75s period)
    // 268 damage packets settled over 150s (134 per 75s period: 112 Normals + 9 Skill + 1 Ivory Q + 12 pulses)
    assert_eq!(report["hits_settled"], 268);

    let direct_bursts = packets(&report, "GE_Player_Oneiroi_UltraSkill_Damage");
    assert_eq!(direct_bursts.len(), 2);

    let pulses = packets(&report, "GE_Player_Oneiroi_UltraSkill1_Damage");
    assert_eq!(pulses.len(), 24); // 12 pulses per cycle * 2 cycles

    let final_state = &report["final_actor_states"][0];
    assert_eq!(final_state["character_id"], "character:1075");
    assert_eq!(final_state["spent_energy"], 200); // 2 Bursts paid at 100 energy each
}
