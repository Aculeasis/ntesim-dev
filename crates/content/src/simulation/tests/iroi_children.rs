use super::*;
use crate::simulation::tests::support::formula_damage_total;

fn request(level: u32, awakening: u32) -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1075".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].talent_levels = Some([1, level, 1]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Oneiroi_Skill".into(),
        damage_effect_id: "GE_Player_Oneiroi_Skill_Gun_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    }];
    req
}

#[test]
fn iroi_skill_train_follows_talents_resonance_and_a2_native_constant() {
    for (level, awakening, syringe, trumpet, assist) in [
        (1, 0, 179, 1250, false),
        (8, 0, 307, 2142, false),
        (8, 2, 307, 2142, true),
        (8, 3, 331, 2314, true),
    ] {
        let req = request(level, awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), if assist { 10 } else { 9 });
        // Self is excluded from the base-ATK grant, but shares5%DEF ignore:20/39 mitigation.
        assert_eq!(
            hits[0]["damage_effect_id"],
            "GE_Player_Oneiroi_Skill_Gun_Damage"
        );
        for (index, coefficient) in (1..=7).map(|index| (index, syringe)) {
            let expected = ExactFraction::new(coefficient * 20, 39).unwrap();
            assert_eq!(
                serde_json::from_value::<ExactFraction>(hits[index]["non_crit_damage"].clone())
                    .unwrap(),
                expected
            );
            assert_eq!(hits[index]["command_index"], 0);
            assert_eq!(hits[index]["time_us"], 100);
        }
        for hit in &hits[1..=7] {
            assert_eq!(
                hit["damage_effect_id"],
                "GE_Player_Oneiroi_Skill_Syringe_Damage"
            );
        }
        let trumpet_index = 8;
        assert_eq!(
            hits[trumpet_index]["damage_effect_id"],
            "GE_Player_Oneiroi_Skill_Trumpet_Damage"
        );
        assert_eq!(
            serde_json::from_value::<ExactFraction>(hits[trumpet_index]["non_crit_damage"].clone())
                .unwrap(),
            ExactFraction::new(trumpet * 20, 39).unwrap()
        );
        if assist {
            assert_eq!(hits[9]["damage_effect_id"], "GE_Damage_sheep_gun_awaken2");
            assert_eq!(
                serde_json::from_value::<ExactFraction>(hits[9]["non_crit_damage"].clone())
                    .unwrap(),
                ExactFraction::new(40_000, 39).unwrap()
            );
        }
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::characters::iroi::skill::SKILL_TRAIN_HYPOTHESIS_ID.as_str()],
            crate::characters::iroi::skill::SKILL_TRAIN_HYPOTHESIS_ID.description()
        );
        let primary = match level {
            1 => 1500,
            8 if awakening >= 3 => 2776,
            8 => 2571,
            _ => unreachable!(),
        };
        let sum_milli = primary + 7 * syringe + trumpet + if assist { 2000 } else { 0 };
        assert_eq!(
            serde_json::from_value::<ExactFraction>(
                serde_json::json!(formula_damage_total(&report)).clone()
            )
            .unwrap(),
            ExactFraction::new(sum_milli * 20, 39).unwrap()
        );
    }
}

#[test]
fn iroi_native_children_remain_pending_off_field_and_failed_casts_schedule_none() {
    let mut req = request(1, 2);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 50,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 10);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["actor_index"] == 0)
    );
    req.relative_duration_us = 99;
    let short = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(short["execution_completed"], true);
    assert_eq!(short["pending_events"], 10);
    assert_eq!(short["hits_settled"], 0);
    req = request(1, 2);
    req.actors[0].initial_energy = 0;
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
        *energy_cost = 1;
    }
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["hits_settled"], 0);
    assert!(
        !failed["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "iroi_skill_buff")
    );
}

#[test]
fn iroi_rejects_bad_native_child_coordinate_and_requires_strict_acceptance() {
    let mut req = request(1, 0);
    req.actors[0].talent_levels = None;
    if let GameActionCommand::InvokeAbility { array_index, .. } = &mut req.actions[0] {
        *array_index = 100;
    }
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
    req = request(1, 0);
    req.actors[0].talent_levels = None;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::iroi::skill::SKILL_TRAIN_HYPOTHESIS_ID.into(),
        crate::characters::iroi::skill::SKILL_TRAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::iroi::skill::SKILL_HEAL_HYPOTHESIS_ID.into(),
        crate::characters::iroi::skill::SKILL_HEAL_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::energy::IROI_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::IROI_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["hits_settled"],
        9
    );
}

#[test]
fn iroi_skill_rejects_a_child_as_the_explicit_primary_packet() {
    let mut req = request(1, 0);
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut req.actions[0]
    {
        *damage_effect_id = "GE_Player_Oneiroi_Skill_Syringe_Damage".into();
    }
    let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("modeled Gun packet"), "{error}");
}

#[test]
fn iroi_skill_train_uses_admitted_native_coefficients_over_caller_values() {
    let mut req = request(1, 0);
    if let GameActionCommand::InvokeAbility {
        array_index,
        atk_rate,
        ..
    } = &mut req.actions[0]
    {
        *array_index = 8;
        *atk_rate = Ratio {
            numerator: 999,
            denominator: 1,
        };
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 9);
    assert_eq!(
        serde_json::from_value::<ExactFraction>(
            report["hit_receipts"][0]["non_crit_damage"].clone()
        )
        .unwrap(),
        ExactFraction::new(30_000, 39).unwrap()
    );
    assert_eq!(
        serde_json::from_value::<ExactFraction>(
            report["hit_receipts"][1]["non_crit_damage"].clone()
        )
        .unwrap(),
        ExactFraction::new(3_580, 39).unwrap()
    );
}

#[test]
fn iroi_burst_models_one_direct_primary_packet_and_rejects_alternate_primary() {
    for (level, awakening, expected_num, expected_den) in
        [(1, 0, 1500, 1), (8, 0, 5141, 2), (8, 3, 5553, 2)]
    {
        let mut req = request(level, awakening);
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actions = vec![GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Oneiroi_UltraSkill".into(),
            damage_effect_id: "GE_Player_Oneiroi_UltraSkill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 0,
            cooldown_us: 20_000_000,
            hit_delay_us: 100,
        }];

        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            serde_json::json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::characters::iroi::skill::BURST_HYPOTHESIS_ID.as_str()],
            crate::characters::iroi::skill::BURST_HYPOTHESIS_ID.description()
        );
    }

    let mut req = request(1, 0);
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Oneiroi_UltraSkill".into(),
        damage_effect_id: "GE_Player_Oneiroi_UltraSkill1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 100,
        recovery_us: 0,
        cooldown_us: 20_000_000,
        hit_delay_us: 100,
    }];
    let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        error.contains("Iroi Burst requires its modeled direct UltraSkill packet as input")
            || error.contains("has no native damage row"),
        "{error}"
    );
}

#[test]
fn iroi_burst_post_cast_heal_uses_native_curve_and_settles_team_after_the_hit() {
    for (level, awakening, expected_ratio, expected_gross) in [
        (1, 0, ExactFraction::new(1413, 1000).unwrap(), 1_413),
        (8, 0, ExactFraction::new(1211, 500).unwrap(), 2_422),
        (8, 3, ExactFraction::new(327, 125).unwrap(), 2_616),
    ] {
        let mut req = request(level, awakening);
        req.actors[0].talent_levels = Some([1, 1, level]);
        if awakening == 3 {
            // Retain resonance_3 while excluding Effect3's +36% outgoing heal.
            req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 2, 4]).unwrap();
        }
        req.actors[0].slot = 2;
        req.actors[0].initial_hp = 1_000;
        req.actors[0].initial_hp_max = 10_000;
        let mut slot_zero = req.actors[0].clone();
        slot_zero.character_id = "character:1019".into();
        slot_zero.awakening = ntesim_data::AwakeningSelection::prefix(0);
        slot_zero.talent_levels = None;
        slot_zero.slot = 0;
        let mut slot_one = slot_zero.clone();
        slot_one.character_id = "character:1042".into();
        slot_one.slot = 1;
        req.actors.extend([slot_zero, slot_one]);
        req.actions = vec![GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Oneiroi_UltraSkill".into(),
            damage_effect_id: "GE_Player_Oneiroi_UltraSkill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 0,
            cooldown_us: 20_000_000,
            hit_delay_us: 100,
        }];

        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(report["hit_receipts"][0]["time_us"], 100);
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::characters::iroi::skill::BURST_HEAL_HYPOTHESIS_ID.as_str()],
            crate::characters::iroi::skill::BURST_HEAL_HYPOTHESIS_ID.description()
        );

        let trace = report["trace"].as_array().unwrap();
        let heals = trace
            .iter()
            .filter(|event| event["event_kind"] == "iroi_burst_heal")
            .collect::<Vec<_>>();
        assert_eq!(heals.len(), 3);
        assert!(
            trace
                .iter()
                .position(|event| event["event_kind"] == "hit_damage")
                .unwrap()
                < trace
                    .iter()
                    .position(|event| event["event_kind"] == "iroi_burst_heal")
                    .unwrap(),
        );
        let expected_recipients = [1, 2, 0];
        for (heal, recipient_actor_index) in heals.into_iter().zip(expected_recipients) {
            assert_eq!(heal["time_us"], 100);
            assert_eq!(heal["from_actor"], 0);
            assert_eq!(heal["to_actor"], recipient_actor_index);
            assert_eq!(heal["data"]["input"], "burst");
            assert_eq!(heal["data"]["exact_raw"], "1000/1");
            assert_eq!(
                heal["data"]["atk_ratio"],
                format!(
                    "{}/{}",
                    expected_ratio.numerator, expected_ratio.denominator
                )
            );
            assert_eq!(heal["data"]["gross"], expected_gross);
            assert_eq!(heal["data"]["applied"], expected_gross);
            assert_eq!(heal["data"]["overflow"], 0);
            assert_eq!(heal["data"]["hp_before"], 1000);
            assert_eq!(heal["data"]["hp_after"], 1000 + expected_gross);
        }
        assert!(
            !trace
                .iter()
                .any(|event| event["event_kind"] == "iroi_burst_heal_tick")
        );
    }
}

#[test]
fn iroi_burst_heal_requires_strict_hypothesis_acceptance() {
    let mut req = request(1, 0);
    req.actors[0].talent_levels = None;
    req.actions = vec![GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Oneiroi_UltraSkill".into(),
        damage_effect_id: "GE_Player_Oneiroi_UltraSkill_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 100,
        recovery_us: 0,
        cooldown_us: 20_000_000,
        hit_delay_us: 100,
    }];
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
    req.model_assumptions.insert(
        crate::simulation::energy::IROI_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::IROI_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
}

#[test]
fn iroi_awakenings_use_selected_effect_ids_while_resonance_uses_count() {
    let mut a2 = request(1, 0);
    a2.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    let a2_report = run_game_scenario(&a2, &ExecutionLimits::default()).unwrap();
    assert_eq!(a2_report["hits_settled"], 10);
    assert!(
        a2_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| { event["event_kind"] == "iroi_a2_assist" })
    );

    let mut a5 = request(1, 0);
    a5.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    let a5_report = run_game_scenario(&a5, &ExecutionLimits::default()).unwrap();
    assert_eq!(a5_report["hits_settled"], 9);
    assert!(
        a5_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| { event["event_kind"] == "iroi_a5_atk_buff" })
    );
    assert!(
        !a5_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| { event["event_kind"] == "iroi_a2_assist" })
    );

    // Three selected awakenings activate resonance_3, while Effect4 and Effect5
    // are independently selected rather than inferred from the selection count.
    let mut a4_a5_r3 = request(8, 0);
    a4_a5_r3.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 4, 5]).unwrap();
    let report = run_game_scenario(&a4_a5_r3, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 9);
    assert_eq!(
        serde_json::from_value::<ExactFraction>(
            report["hit_receipts"][0]["non_crit_damage"].clone()
        )
        .unwrap(),
        ExactFraction::new(63_848, 39).unwrap()
    );
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| { event["event_kind"] == "iroi_a5_atk_buff" })
    );

    // Ordinary Effect6 is independent of resonance_6, which needs all six
    // selected effects; this partial callback records the A6 hook only.
    let mut a6 = request(1, 0);
    a6.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
    a6.actors[0].static_stats.crit_rate = Ratio::one();
    a6.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    a6.actions[0] = GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Oneiroi_UltraSkill".into(),
        damage_effect_id: "GE_Player_Oneiroi_UltraSkill_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 100,
        recovery_us: 0,
        cooldown_us: 20_000_000,
        hit_delay_us: 100,
    };
    a6.stochastic_crit = Some(crate::simulation::ResearchCritPolicy {
        roll_policy: crate::simulation::ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Test a guaranteed eligible critical hit for awakening selection coverage."
            .into(),
        eligible_effects: std::collections::BTreeMap::from([(
            a6.actors[0].character_id.clone(),
            std::collections::BTreeSet::from(["*".into()]),
        )]),
        actor_sequences: std::collections::BTreeMap::from([(a6.actors[0].character_id.clone(), 1)]),
    });
    let a6_report = run_game_scenario(&a6, &ExecutionLimits::default()).unwrap();
    assert!(
        a6_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| { event["event_kind"] == "iroi_a6_revive_grant" })
    );

    // Ordinary Effect6 does not activate resonance_6's Crit DMG on a normal hit.
    let mut a6_crit = a6.clone();
    a6_crit.actions[0] = GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Oneiroi_Melee".into(),
        damage_effect_id: "GE_Player_Oneiroi_Melee1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    };
    let a6_crit_report = run_game_scenario(&a6_crit, &ExecutionLimits::default()).unwrap();
    let a6_base = serde_json::from_value::<ExactFraction>(
        a6_crit_report["hit_receipts"][0]["non_crit_damage"].clone(),
    )
    .unwrap();
    assert_eq!(a6_crit_report["hit_receipts"][0]["critical"], true);
    assert_eq!(
        serde_json::from_value::<ExactFraction>(
            a6_crit_report["hit_receipts"][0]["crit_damage"].clone()
        )
        .unwrap(),
        a6_base.mul(ExactFraction::new(3, 2).unwrap()).unwrap()
    );

    let mut resonance6 = a6;
    resonance6.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    resonance6.actions[0] = a6_crit.actions[0].clone();
    let resonance6_report = run_game_scenario(&resonance6, &ExecutionLimits::default()).unwrap();
    let resonance6_base = serde_json::from_value::<ExactFraction>(
        resonance6_report["hit_receipts"][0]["non_crit_damage"].clone(),
    )
    .unwrap();
    assert_eq!(resonance6_report["hit_receipts"][0]["critical"], true);
    assert_eq!(
        serde_json::from_value::<ExactFraction>(
            resonance6_report["hit_receipts"][0]["crit_damage"].clone()
        )
        .unwrap(),
        resonance6_base
            .mul(ExactFraction::new(7, 4).unwrap())
            .unwrap()
    );
}

#[test]
fn iroi_a2_ally_trigger_registers_autonomous_event() {
    let mut req = request(1, 2);
    let model = crate::characters::iroi::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 100_000,
            actor_index: 1,
            ability_id: "GA_Mint019_Skill".into(),
            damage_effect_id: "GE_Player_Mint_Skill1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 3_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["success"], true);

    let autonomous_events = report["autonomous_events"]
        .as_array()
        .expect("autonomous events");
    assert!(
        autonomous_events
            .iter()
            .any(|e| e["ability_id"] == "GA_Oneiroi_Skill" && e["actor_index"] == 0)
    );

    let hits = report["hit_receipts"].as_array().unwrap();
    let a2_hit = hits.iter().find(|h| {
        h["actor_index"] == 0
            && (h["damage_effect_id"] == "GE_Damage_sheep_gun_awaken2"
                || h["damage_effect_id"] == "GE_Damage_sheep_syringe_awaken2"
                || h["damage_effect_id"] == "GE_Damage_sheep_trumpet_awaken2")
    });
    assert!(a2_hit.is_some(), "expected an A2 sheep damage hit");
    let a2_hit = a2_hit.unwrap();
    assert!(a2_hit["autonomous_event_index"].is_number());
    assert!(a2_hit["command_index"].is_null());
    assert!(a2_hit["switch_origin"].is_null());
}
