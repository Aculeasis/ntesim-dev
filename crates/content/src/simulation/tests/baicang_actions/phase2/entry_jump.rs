use super::*;

fn scenario() -> GameScenarioRequest {
    let mut req = phase2_request();
    dodge(&mut req);
    req.actions.clear();
    for id in [
        crate::HypothesisId::BaicangLacrimosaEntryJumpV1,
        crate::HypothesisId::LacrimosaBatEvadeExitV1,
        crate::HypothesisId::BaicangQualifiedSupportV1,
        crate::HypothesisId::LacrimosaFormsV1,
        crate::HypothesisId::InputsSpecialDodgeTimingV1,
        crate::HypothesisId::InputsCriticalDodgeAssumedSuccessV1,
    ] {
        req.model_assumptions
            .insert(id.into(), id.description().into());
    }
    let mut lacrimosa = req.actors[0].clone();
    lacrimosa.slot = 1;
    lacrimosa.character_id = "character:1004".into();
    lacrimosa.initial_energy = 0;
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors.push(lacrimosa);
    req.active_actor_index = 1;
    req.relative_duration_us = 2_000_000;
    req.actions = vec![
        GameActionCommand::PlayerInput {
            timestamp_us: 0,
            actor_index: 1,
            input: PlayerInput::Dash,
            critical_success: false,
        },
        switch(100_000, 0),
    ];
    req
}

fn switch(time: u64, actor: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us: time,
        target_actor_index: actor,
        cooldown_us: 0,
    }
}

fn jumps(report: &serde_json::Value) -> Vec<&serde_json::Value> {
    report["autonomous_events"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|event| event["ability_id"] == "Buff_Cang_JumpZ")
        .collect()
}

#[test]
fn lacrimosa_entry_jump_is_a_real_non_damage_effect_and_does_not_invent_an_input() {
    let mut req = scenario();
    req.relative_duration_us = 150_000;
    let report = run(&req);
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(report["total_damage_dealt"].as_f64(), Some(0.0));
    assert!(report["action_metadata_ledger"].is_null());
    assert_eq!(report["player_input_receipts"].as_array().unwrap().len(), 1);
    let entries = jumps(&report);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["time_us"], 100_000);
    assert_eq!(
        entries[0]["trigger"],
        json!({"kind":"field_entry","command_index":1})
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
    let buffs = crate::simulation::tests::support::final_buffs(&report);
    let effect = buffs
        .iter()
        .find(|buff| buff["id"] == "Buff_Cang_JumpZ")
        .unwrap();
    assert_eq!(effect["duration"]["expires_at_us"], 300_000);
    assert_eq!(effect["stat_modifiers"]["jump_multiplier"], json!("6/5"));
}

#[test]
fn lacrimosa_entry_jump_rechecks_dodge_end_retained_tag_passive_and_real_recovery() {
    for (time, expected) in [(499_999, 1), (500_000, 0)] {
        let mut req = scenario();
        req.actions[1] = switch(time, 0);
        assert_eq!(jumps(&run(&req)).len(), expected);
    }
    let mut req = scenario();
    req.actions.remove(0);
    assert!(jumps(&run(&req)).is_empty());
    req = scenario();
    req.actions[0] = GameActionCommand::PlayerInput {
        timestamp_us: 0,
        actor_index: 1,
        input: PlayerInput::Dash,
        critical_success: true,
    };
    req.actions[1] = switch(500_000, 0);
    assert!(jumps(&run(&req)).is_empty());
    req = scenario();
    req.actors[0].character_level = 40;
    req.actors[0].purchased_qte_level = Some(1);
    req.actors[0].character_max_level = Some(40);
    assert!(jumps(&run(&req)).is_empty());
    req = scenario();
    req.actions.extend([
        switch(200_000, 1),
        switch(600_000, 0),
        switch(700_000, 1),
        switch(1_600_000, 0),
    ]);
    let report = run(&req);
    assert_eq!(
        jumps(&report)
            .iter()
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![100_000, 600_000]
    );
    // Returning during the source1.5s tag does not extend its original expiry.
    assert_eq!(
        report["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn lacrimosa_entry_plunge_requires_authored_landing_and_never_multiplies_damage_by_jump_stat() {
    let mut req = scenario();
    req.actions.push(make_action(
        200_000,
        0,
        "GA_Cang_Melee",
        "GE_Player_Cang_AirAttack_Damage",
        0,
        0,
    ));
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("ENTRY_LANDING")
    );
    req.plunge_inputs.insert(
        2,
        crate::simulation::GamePlungeInput {
            landing_delay_us: 600_000,
            height_bonus: Ratio {
                numerator: 1,
                denominator: 2,
            },
        },
    );
    let report = run(&req);
    let hit = &report["hit_receipts"][0];
    assert_eq!(hit["time_us"], 849_876);
    // 1200*.418*1.5*halfDEF =376.2; JumpZ1.2 changes movement, not this coefficient.
    assert_eq!(hit["damage"], json!("1881/5"));
    assert_eq!(hit["command_index"], 2);
    assert!(hit["autonomous_event_index"].is_null());
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 1);
}

#[test]
fn lacrimosa_entry_jump_does_not_bypass_switch_dodge_cooldowns_or_incoming_recovery() {
    let mut req = scenario();
    req.actions[1] = GameActionCommand::SwitchActor {
        timestamp_us: 100_000,
        target_actor_index: 0,
        cooldown_us: 1_000_000,
    };
    req.actions.extend([switch(200_000, 1), switch(300_000, 0)]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["reason"]["kind"], "cooldown");
    assert_eq!(jumps(&report).len(), 1);
    req = scenario();
    req.actions.extend([
        switch(200_000, 1),
        GameActionCommand::PlayerInput {
            timestamp_us: 200_001,
            actor_index: 1,
            input: PlayerInput::Dash,
            critical_success: false,
        },
    ]);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(
        report["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
    assert_eq!(jumps(&report).len(), 1);

    let mut state = crate::simulation::state::GameSimulatorState::new(&scenario());
    state.advance_unpaused_test_clock(100_000);
    state.actors[0].locked_until_us = 1_000_000;
    state.actors[1].locked_until_us = 500_000;
    state.actors[1]
        .character
        .lacrimosa_mut()
        .unwrap()
        .bat_evade_until_us = 500_000;
    assert!(
        crate::characters::baicang::entry_jump::select(&scenario(), &state, 1, 0, false)
            .unwrap()
            .is_none()
    );
}

#[test]
fn lacrimosa_entry_jump_requires_both_models_and_preserves_a_qualified_support() {
    let mut req = scenario();
    req.model_assumptions
        .remove(crate::HypothesisId::BaicangLacrimosaEntryJumpV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("UNSUPPORTED_BAICANG_LACRIMOSA_ENTRY")
    );
    req = scenario();
    req.model_assumptions
        .remove(crate::HypothesisId::LacrimosaBatEvadeExitV1.as_str());
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["execution_completed"], false);
    assert!(jumps(&rejected).is_empty());
    req = scenario();
    let id = crate::HypothesisId::CombatEsperCycleReactionsV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.actors[1].initial_cycle = 100;
    req.relative_duration_us = 20_000_000;
    req.actions.push(make_action(
        2_000_000,
        0,
        "GA_Cang_Melee",
        "GE_Player_Cang_AirAttack_Damage",
        0,
        0,
    ));
    req.plunge_inputs.insert(
        2,
        crate::simulation::GamePlungeInput {
            landing_delay_us: 600_000,
            height_bonus: Ratio {
                numerator: 1,
                denominator: 2,
            },
        },
    );
    let report = run(&req);
    assert_eq!(jumps(&report).len(), 1);
    assert_eq!(
        report["action_metadata_ledger"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["ability_id"] == "GA_Cang_QTE")
            .count(),
        1
    );
    assert_eq!(report["final_actor_states"][1]["cycle"]["spent"], 100);
    let id = crate::HypothesisId::BaicangQualifiedSupportV1;
    assert_eq!(
        support::receipt_assumption(&report, &report["action_metadata_ledger"][0], id.as_str()),
        Some(id.description())
    );
    assert!(
        report["callback_model_assumptions"][crate::HypothesisId::CombatSupportDispatchV4.as_str()]
            .is_null()
    );

    req.actions = vec![switch(100_000, 0)]; // ordinary qualified entry also retains Support
    req.plunge_inputs.clear();
    let plain = run(&req);
    assert!(jumps(&plain).is_empty());
    assert_eq!(plain["action_metadata_ledger"].as_array().unwrap().len(), 1);
    req.actors[1].initial_cycle = 99;
    let empty = run(&req);
    assert!(empty["action_metadata_ledger"].is_null());
}
