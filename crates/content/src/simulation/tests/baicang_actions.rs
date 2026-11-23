use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;

#[test]
fn baicang_selected_a6_crit_is_separate_from_resonance_team_atk() {
    // Burst primary Lv1=4.0, Lv2=4.32. R3 raises the coefficient;
    // Effect6 grants .30 crit, R6 grants .20 * base ATK to the team.
    for (effects, damage, crit_num, crit_den) in [
        (vec![6], 2000, 3, 10),
        (vec![1, 2, 3, 4, 5], 2160, 0, 1),
        (vec![1, 2, 3, 4, 5, 6], 2592, 3, 10),
    ] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.actors[0].static_stats.crit_rate = Ratio::zero();
        req.actors[0].initial_energy = 120;
        req.actors[0].energy_capacity = 120;
        req.relative_duration_us = 1_000_000;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Cang_UltraSkill",
            "GE_Player_Cang_UltraSkill2_Damage",
            120,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", damage, 1))
        );
        assert_eq!(
            report["hit_receipts"][0]["crit_rate"],
            json!(format!("{}/{}", crit_num, crit_den))
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
    req.actors[0].character_id = "character:1023".into();
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
    req.model_assumptions.insert(
        crate::characters::packet_semantics::BAICANG_DOT_PACKET_HYPOTHESIS_ID.into(),
        crate::characters::packet_semantics::BAICANG_DOT_PACKET_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Cang_Melee",
        "GE_Player_Cang_Melee1_Damage",
        0,
        0,
    )];
    req
}

#[test]
fn baicang_normal_skill_and_burst_use_native_talent_coefficients_and_resonance_bonus() {
    // Test Normal Attack (GA_Cang_Melee / GE_Player_Cang_Melee1_Damage)
    // Coeffs: Lv1 = 0.475, Lv8 = 0.814, Lv9 (A3 resonance) = 0.879
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 0.475 * 0.5 = 475 / 2
    // Lv8: 1000 * 0.814 * 0.5 = 407 / 1
    // Lv9: 1000 * 0.879 * 0.5 = 879 / 2
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 475, 2),
        (8, 0, 407, 1),
        (8, 1, 407, 1),
        (8, 3, 46587, 100), // A2 grants +6% ATK, A3 adds resonance_3 (+1 level -> Lv9)
        (8, 4, 46587, 100),
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
            support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
    }

    // Test Skill (GA_Cang_Skill / GE_Player_Cang_Skill_Damage)
    // Coeffs: Lv1 = 1.2, Lv8 = 2.057, Lv9 (A3 resonance) = 2.221
    // At A0/A1, no power word bonus:
    // Lv1, A0: 1000 * 1.2 * 0.5 = 600 / 1
    // Lv8, A0: 1000 * 2.057 * 0.5 = 2057 / 2
    // Lv8, A1: 1000 * 2.057 * 0.5 = 2057 / 2
    // At A2, 1 Power Word stack grants +6% ATK:
    // Lv8, A2: 1000 * 1.06 ATK * 2.057 * 0.5 = 109021 / 100
    // Lv8, A3 (+1 level -> Lv9): 1000 * 1.06 ATK * 2.221 * 0.5 = 117713 / 100
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 600, 1),
        (8, 0, 2057, 2),
        (8, 1, 2057, 2),
        (8, 2, 109021, 100),
        (8, 3, 117713, 100),
    ] {
        let mut req = request();
        req.actors[0].talent_levels = Some([1, level, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Cang_Skill",
            "GE_Player_Cang_Skill_Damage",
            0,
            6_000_000,
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
            support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
        );
    }

    // Test Ultimate/Burst (GA_Cang_UltraSkill / GE_Player_Cang_UltraSkill2_Damage)
    // Coeffs: Lv1 = 4.0, Lv8 = 6.855, Lv9 (A3 resonance) = 7.404
    // At 1000 ATK, 0 RES, 0.5 DEF:
    // Lv1: 1000 * 4.0 * 0.5 = 2000 / 1
    // Lv8: 1000 * 6.855 * 0.5 = 6855 / 2
    // Lv8, A1: 1000 * 6.855 * 0.5 = 6855 / 2
    // Lv8, A3: 1000 * 7.404 * 0.5 = 3702 / 1
    for (level, awakening, expected_num, expected_den) in [
        (1, 0, 2000, 1),
        (8, 0, 6855, 2),
        (8, 1, 6855, 2),
        (8, 3, 3702, 1),
    ] {
        let mut req = request();
        req.actors[0].initial_energy = 120;
        req.actors[0].energy_capacity = 120;
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Cang_UltraSkill",
            "GE_Player_Cang_UltraSkill2_Damage",
            120,
            20_000_000,
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
            json!(format!("{}/{}", expected_num, expected_den))
        );
        assert_eq!(
            support::hit_metadata(&report, &report["hit_receipts"][0])["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
    }
}

#[test]
fn baicang_domain_packet_train_uses_effective_coordinate_and_native_metadata() {
    for (
        level,
        awakening,
        expected_ticks,
        expected_rate,
        expected_tick_damage,
        expected_primary_num,
        expected_primary_den,
    ) in [
        (1, 0, 6, json!("2/5"), 200, 2000, 1),
        (8, 0, 6, json!("343/500"), 343, 6855, 2),
        (8, 3, 6, json!("37/50"), 370, 3702, 1),
        (8, 5, 8, json!("37/50"), 370, 3702, 1),
    ] {
        let mut req = request();
        req.actors[0].initial_energy = 120;
        req.actors[0].energy_capacity = 120;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].talent_levels = Some([1, 1, level]);
        req.actions = vec![make_action(
            0,
            0,
            "GA_Cang_UltraSkill",
            "GE_Player_Cang_UltraSkill2_Damage",
            120,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1 + expected_ticks);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
        let ticks = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
            .collect::<Vec<_>>();
        assert_eq!(ticks.len(), expected_ticks);
        assert_eq!(ticks.first().unwrap()["time_us"], 4_854_959);
        assert_eq!(
            ticks.last().unwrap()["time_us"],
            if awakening >= 5 {
                18_854_959
            } else {
                14_854_959
            }
        );
        assert_eq!(ticks[0]["scaling_rate"], expected_rate);
        let tick_packet = &support::hit_metadata(&report, ticks[0])["packet"];
        assert_eq!(tick_packet["fixed_crit_rate"], json!("1/2"));
        assert_eq!(
            tick_packet["semantic_tags"],
            json!(["State.Damage", "State.Damage.Dot"])
        );
        assert_eq!(
            tick_packet["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        for tick in &ticks {
            assert_eq!(
                tick["non_crit_damage"],
                json!(format!("{}/{}", expected_tick_damage, 1))
            );
        }
        let total = expected_primary_num
            + expected_ticks as u64 * expected_tick_damage * expected_primary_den;
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", total, expected_primary_den))
        );
        assert!(
            report["callback_model_assumptions"]
                .get(crate::characters::baicang::skill::DOMAIN_HYPOTHESIS_ID.as_str())
                .is_some()
        );
    }
}

#[test]
fn baicang_domain_admission_horizon_and_unsupported_branches_are_explicit() {
    let burst = || {
        make_action(
            0,
            0,
            "GA_Cang_UltraSkill",
            "GE_Player_Cang_UltraSkill2_Damage",
            120,
            20_000_000,
        )
    };
    let mut pending = request();
    pending.actors[0].initial_energy = 120;
    pending.actors[0].energy_capacity = 120;
    pending.actions = vec![burst()];
    pending.relative_duration_us = 2_000_000;
    let at_tick = run_game_scenario(&pending, &ExecutionLimits::default()).unwrap();
    assert_eq!(at_tick["hits_settled"], 2);
    pending.relative_duration_us = 1_999_999;
    let before_tick = run_game_scenario(&pending, &ExecutionLimits::default()).unwrap();
    assert_eq!(before_tick["hits_settled"], 1);

    let mut failed = request();
    failed.actors[0].initial_energy = 0;
    failed.actions = vec![burst()];
    let failed_report = run_game_scenario(&failed, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed_report["hits_settled"], 0);
    assert!(
        failed_report["callback_model_assumptions"]
            .get("baicang.judgment_of_autumn_domain_v1")
            .is_none()
    );

    let mut execute = request();
    execute.actors[0].initial_energy = 120;
    execute.actors[0].energy_capacity = 120;
    execute.actions = vec![burst()];
    execute.targets[0].hp_max = 100_000;
    execute.targets[0].hp_current = 9_999;
    let execute_error = run_game_scenario(&execute, &ExecutionLimits::default()).unwrap_err();
    assert!(
        execute_error.contains("UNSUPPORTED_BAICANG_EXECUTE"),
        "{execute_error}"
    );

    let mut execute_after_tick = request();
    execute_after_tick.actors[0].initial_energy = 120;
    execute_after_tick.actors[0].energy_capacity = 120;
    execute_after_tick.targets[0].hp_max = 10_000;
    execute_after_tick.targets[0].hp_current = 3_000;
    execute_after_tick.actions = vec![burst()];
    let after_tick_error =
        run_game_scenario(&execute_after_tick, &ExecutionLimits::default()).unwrap_err();
    assert!(
        after_tick_error.contains("UNSUPPORTED_BAICANG_EXECUTE"),
        "{after_tick_error}"
    );

    let mut forged = request();
    forged.actors[0].initial_energy = 120;
    forged.actors[0].energy_capacity = 120;
    forged.actions = vec![make_action(
        0,
        0,
        "GA_Cang_UltraSkill",
        "GE_Player_Cang_Skill_Damage",
        120,
        20_000_000,
    )];
    let forged_error = run_game_scenario(&forged, &ExecutionLimits::default()).unwrap_err();
    assert!(
        forged_error.contains("unsupported action/packet relationship"),
        "{forged_error}"
    );

    let mut fatal = request();
    fatal.actors[0].initial_energy = 120;
    fatal.actors[0].energy_capacity = 120;
    fatal.actors[0].initial_hp = 1_000;
    fatal.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    fatal.actions = vec![burst()];
    fatal.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 500_000,
        actor_index: 0,
        delivered_damage: 1_000,
    }];
    let fatal_error = run_game_scenario(&fatal, &ExecutionLimits::default()).unwrap_err();
    assert!(
        fatal_error.contains("UNSUPPORTED_BAICANG_FATAL_IMMUNITY"),
        "{fatal_error}"
    );
}

#[test]
fn baicang_domain_endpoint_owner_death_cancels_the_due_tick() {
    let mut req = request();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.actors[0].initial_hp = 1_000;
    req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Cang_UltraSkill",
        "GE_Player_Cang_UltraSkill2_Damage",
        120,
        0,
    )];
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 12_000_000,
        actor_index: 0,
        delivered_damage: 1_000,
    }];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let ticks = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
        .collect::<Vec<_>>();
    assert_eq!(ticks.len(), 5);
    assert_eq!(ticks.last().unwrap()["time_us"], 12_854_959);
    assert_eq!(report["incoming_damage_receipts"][0]["defeated"], true);
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert!(report["trace"].as_array().unwrap().iter().any(|event| {
        event["time_us"] == 14_854_959 && event["event_kind"] == "hit_owner_unavailable"
    }));
}

#[test]
fn baicang_domain_hypotheses_are_known_and_callback_reported_in_strict_mode() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Cang_UltraSkill",
        "GE_Player_Cang_UltraSkill2_Damage",
        120,
        20_000_000,
    )];
    req.model_assumptions.insert(
        crate::characters::baicang::skill::DOMAIN_HYPOTHESIS_ID.into(),
        crate::characters::baicang::skill::DOMAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::packet_semantics::BAICANG_DOT_PACKET_HYPOTHESIS_ID.into(),
        crate::characters::packet_semantics::BAICANG_DOT_PACKET_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let accepted = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(accepted["execution_completed"], true);
    assert_eq!(accepted["hits_settled"], 7);
    assert_eq!(
        accepted["callback_model_assumptions"]
            [crate::characters::baicang::skill::DOMAIN_HYPOTHESIS_ID.as_str()],
        crate::characters::baicang::skill::DOMAIN_HYPOTHESIS_ID.description()
    );
}

#[test]
fn baicang_off_field_delivery_and_dead_target_boundary() {
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Cang_Skill",
            "GE_Player_Cang_Skill_Damage",
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
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(report["hit_receipts"][0]["actor_index"], 0);

    let mut dead_req = request();
    dead_req.targets[0].hp_current = 0;
    dead_req.actions = vec![make_action(
        0,
        0,
        "GA_Cang_Melee",
        "GE_Player_Cang_Melee1_Damage",
        0,
        0,
    )];
    let dead_report = run_game_scenario(&dead_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(dead_report["hits_settled"], 0);

    let mut domain = request();
    domain.actors[0].initial_energy = 120;
    domain.actors[0].energy_capacity = 120;
    let mut ally = domain.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    domain.actors.push(ally);
    domain.actions = vec![
        make_action(
            0,
            0,
            "GA_Cang_UltraSkill",
            "GE_Player_Cang_UltraSkill2_Damage",
            120,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 50,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let off_field = run_game_scenario(&domain, &ExecutionLimits::default()).unwrap();
    let off_field_ticks = off_field["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
        .collect::<Vec<_>>();
    assert_eq!(off_field_ticks.len(), 6);
    assert!(off_field_ticks.iter().all(|hit| hit["actor_index"] == 0));

    let mut target_dies = request();
    target_dies.actors[0].initial_energy = 120;
    target_dies.actors[0].energy_capacity = 120;
    target_dies.targets[0].hp_current = 2_500;
    target_dies.targets[0].hp_max = 2_500;
    target_dies.actions = vec![
        make_action(
            0,
            0,
            "GA_Cang_UltraSkill",
            "GE_Player_Cang_UltraSkill2_Damage",
            120,
            0,
        ),
        make_action(
            5_854_959,
            0,
            "GA_Cang_Skill",
            "GE_Player_Cang_Skill_Damage",
            0,
            0,
        ),
    ];
    let dead_domain_target = run_game_scenario(&target_dies, &ExecutionLimits::default()).unwrap();
    let dot_hits = dead_domain_target["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
        .count();
    assert_eq!(dot_hits, 1);
    assert_eq!(
        dead_domain_target["final_target_states"][0]["hp_current"],
        0
    );
}

#[test]
fn baicang_domain_recast_and_live_power_word_atk_are_sampled_at_each_tick() {
    let burst = |timestamp_us| GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: "GA_Cang_UltraSkill".into(),
        damage_effect_id: "GE_Player_Cang_UltraSkill2_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 120,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    };

    let mut recast = request();
    recast.actors[0].initial_energy = 240;
    recast.actors[0].energy_capacity = 240;
    recast.actions = vec![burst(0), burst(2_000_000)];
    let report = run_game_scenario(&recast, &ExecutionLimits::default()).unwrap();
    let times = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        times,
        vec![
            6_854_959, 8_854_959, 10_854_959, 12_854_959, 14_854_959, 16_854_959
        ]
    );

    let mut live_atk = request();
    live_atk.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    live_atk.actors[0].initial_energy = 120;
    live_atk.actors[0].energy_capacity = 120;
    live_atk.actions = vec![
        burst(0),
        make_action(
            1_000_000,
            0,
            "GA_Cang_Skill",
            "GE_Player_Cang_Skill_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&live_atk, &ExecutionLimits::default()).unwrap();
    let ticks = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
        .collect::<Vec<_>>();
    assert_eq!(ticks[0]["non_crit_damage"], json!("212/1"));
    assert_eq!(ticks[4]["time_us"], 11_000_000);
    assert_eq!(ticks[4]["non_crit_damage"], json!("200/1"));
    assert_eq!(ticks[5]["time_us"], 13_000_000);
    assert_eq!(ticks[5]["non_crit_damage"], json!("200/1"));
}

#[test]
fn baicang_normal_chain_damage_multiplicity_loop_and_resonance() {
    for (level, awakening, expected_hits, total_num, total_den) in [
        (
            1,
            0,
            vec![
                ("GE_Player_Cang_Melee1_Damage", 475, 2),
                ("GE_Player_Cang_Melee2_Damage", 423, 2),
                ("GE_Player_Cang_Melee3_Damage", 595, 1),
                ("GE_Player_Cang_Melee4_Damage", 251, 2),
                ("GE_Player_Cang_Melee4_Damage", 251, 2),
                ("GE_Player_Cang_Melee4_Damage", 251, 2),
                ("GE_Player_Cang_Melee4_Damage", 251, 2),
                ("GE_Player_Cang_Melee4_Damage", 251, 2),
                ("GE_Player_Cang_Melee4_Damage", 251, 2),
                ("GE_Player_Cang_Melee4_Damage", 251, 2),
                ("GE_Player_Cang_Melee5_Damage", 863, 2),
                ("GE_Player_Cang_Melee1_Damage", 475, 2),
            ],
            5183,
            2,
        ),
        (
            8,
            0,
            vec![
                ("GE_Player_Cang_Melee1_Damage", 407, 1),
                ("GE_Player_Cang_Melee2_Damage", 725, 2),
                ("GE_Player_Cang_Melee3_Damage", 2039, 2),
                ("GE_Player_Cang_Melee4_Damage", 215, 1),
                ("GE_Player_Cang_Melee4_Damage", 215, 1),
                ("GE_Player_Cang_Melee4_Damage", 215, 1),
                ("GE_Player_Cang_Melee4_Damage", 215, 1),
                ("GE_Player_Cang_Melee4_Damage", 215, 1),
                ("GE_Player_Cang_Melee4_Damage", 215, 1),
                ("GE_Player_Cang_Melee4_Damage", 215, 1),
                ("GE_Player_Cang_Melee5_Damage", 1479, 2),
                ("GE_Player_Cang_Melee1_Damage", 407, 1),
            ],
            8881,
            2,
        ),
        (
            8,
            3,
            vec![
                ("GE_Player_Cang_Melee1_Damage", 46587, 100),
                ("GE_Player_Cang_Melee2_Damage", 10962, 25),
                ("GE_Player_Cang_Melee3_Damage", 129977, 100),
                ("GE_Player_Cang_Melee4_Damage", 2883, 10),
                ("GE_Player_Cang_Melee4_Damage", 2883, 10),
                ("GE_Player_Cang_Melee4_Damage", 2883, 10),
                ("GE_Player_Cang_Melee4_Damage", 2883, 10),
                ("GE_Player_Cang_Melee4_Damage", 2883, 10),
                ("GE_Player_Cang_Melee4_Damage", 2883, 10),
                ("GE_Player_Cang_Melee4_Damage", 2883, 10),
                ("GE_Player_Cang_Melee5_Damage", 49507, 50),
                ("GE_Player_Cang_Melee1_Damage", 27249, 50),
            ],
            287867,
            50,
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
                    "GA_Cang_Melee",
                    "GE_Player_Cang_Melee1_Damage",
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
        assert_eq!(report["hits_settled"], 12);
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
fn baicang_normal_chain_reset_and_noop_boundaries() {
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
                "GA_Cang_Skill",
                "GE_Player_Cang_Skill_Damage",
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
            "GA_Cang_Melee",
            "GE_Player_Cang_Melee1_Damage",
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
            "GE_Player_Cang_Melee2_Damage"
        } else {
            "GE_Player_Cang_Melee1_Damage"
        };
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
            expected
        );
    }
}

#[test]
fn baicang_normal_chain_strict_hypothesis_and_off_field_seven_hit_delivery() {
    let mut req = request();
    req.model_assumptions
        .remove(crate::characters::packet_semantics::BAICANG_DOT_PACKET_HYPOTHESIS_ID.as_str());
    req.model_assumptions
        .remove(crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID.as_str());
    req.model_assumptions.insert(
        crate::simulation::energy::BAICANG_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::BAICANG_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::baicang::skill::CHAIN_HYPOTHESIS_ID.into(),
        crate::characters::baicang::skill::CHAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actors.push(req.actors[0].clone());
    req.actions = (0..4)
        .map(|index| {
            make_action(
                index * 1000,
                0,
                "GA_Cang_Melee",
                "GE_Player_Cang_Melee1_Damage",
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
    assert_eq!(pending["hits_settled"], 3);
    req.relative_duration_us = 3100;
    let settled = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(settled["hits_settled"], 10);
}

mod continuation;
mod multi_target;
mod phase2;
