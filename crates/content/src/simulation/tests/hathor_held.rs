use super::*;
use crate::HypothesisId;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};
fn skill(time: u64) -> GameActionCommand {
    super::hathor_actions::make_action(
        time,
        0,
        "GA_Hathor_Skill",
        "GE_Player_Hathor_Skill1_Damage",
        0,
        0,
    )
}
fn hold(duration_us: u64) -> GameHoldInput {
    GameHoldInput {
        duration_us,
        moving: false,
        reserved_stocks: None,
    }
}
fn request(targets: usize, duration: u64) -> GameScenarioRequest {
    let mut req = super::hathor_clock::request(targets);
    for model in [
        HypothesisId::HathorTapPowerV1,
        HypothesisId::HathorGazeV1,
        HypothesisId::HathorHeldSkillV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req.actions = vec![skill(0)];
    req.hold_inputs.insert(0, hold(duration));
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
fn power(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"]
}
fn incoming(req: &mut GameScenarioRequest, time: u64, damage: u64, source: bool) {
    let model = HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let index = req.incoming_damage_events.len();
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: time,
        actor_index: 0,
        delivered_damage: damage,
    });
    if source {
        req.incoming_dummy_sources.insert(index, 0);
    }
}
#[test]
fn hathor_held_full_train_has_seventeen_real_ticks_ten_end_contacts_and_exact_resources() {
    for targets in [1, 3] {
        let mut req = request(targets, 5_000_000);
        req.relative_duration_us = 15_000_000;
        let r = run(&req);
        assert_eq!(r["hits_settled"], 27 * targets);
        assert_eq!(
            serde_json::json!(formula_damage_total(&r)),
            json!(format!("{}/{}", 11865 * targets, 1))
        );
        let owner = &r["final_actor_states"][0];
        assert_eq!(owner["energy"], 33);
        assert_eq!(owner["charge_remainder"], json!("419/500"));
        assert_eq!(owner["cycle"]["current"], 56);
        assert_eq!(owner["cycle"]["carry"], json!("219/500"));
        let times: Vec<_> = r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["target_index"] == 0)
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            &times[..17],
            &(0..17).map(|k| 936_667 + 250_000 * k).collect::<Vec<_>>()
        );
        assert_eq!(
            &times[17..],
            &[
                5_349_932, 5_549_778, 5_766_217, 5_899_329, 6_115_847, 6_698_764, 6_748_859,
                6_798_516, 6_848_317, 6_898_070
            ]
        );
        req.actions.push(super::hathor_clock::normal(7_214_816));
        let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(early["rejected_action"]["reason"]["until_us"], 7_214_817);
        *req.actions.last_mut().unwrap() = super::hathor_clock::normal(7_214_817);
        run(&req);
    }
}
#[test]
fn hathor_held_release_thresholds_and_maximum_hold_do_not_emit_ghost_ticks() {
    for (duration, count) in [
        (219_999, 3),
        (220_000, 3),
        (936_667, 3),
        (1_000_000, 4),
        (1_000_001, 11),
        (5_000_000, 27),
        (9_000_000, 27),
    ] {
        let req = request(1, duration);
        let r = run(&req);
        assert_eq!(r["hits_settled"], count, "hold{duration}");
        assert_eq!(
            power(&r)["current"],
            json!(format!(
                "{}/{}",
                if duration <= 1_000_000 { 2 } else { 0 },
                1
            ))
        );
    }
}
#[test]
fn hathor_held_short_end_cd_replaces_only_an_unreached_power_marker() {
    for (duration, deadline) in [
        (230_000, 16_246_368),
        (236_652, 16_253_020),
        (236_653, 16_236_652),
    ] {
        let mut req = request(1, duration);
        req.actions.push(skill(deadline - 1));
        let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            early["rejected_action"]["reason"]["until_us"], deadline,
            "hold{duration}"
        );
        *req.actions.last_mut().unwrap() = skill(deadline);
        run(&req);
    }
}
#[test]
fn hathor_held_a1_requires_actual_birth_and_cancellation_keeps_committed_cd_and_rewards() {
    for (interrupt, expected_hits, expected_power, expected_cd) in [
        (236_651, 0, 0, 3_236_651),
        (236_652, 0, 0, 16_236_652),
        (686_666, 0, 0, 16_236_652),
        (686_667, 0, 6, 16_236_652),
        (1_100_000, 1, 6, 16_236_652),
    ] {
        let mut req = request(1, 5_000_000);
        req.actors[0].awakening = serde_json::from_value(json!([1])).unwrap();
        req.interrupt_events = vec![GameActorInterruptEvent {
            timestamp_us: interrupt,
            actor_index: 0,
            armor_damage: None,
        }];
        let r = run(&req);
        assert_eq!(r["hits_settled"], expected_hits);
        assert_eq!(
            power(&r)["current"],
            json!(format!("{}/{}", expected_power, 1))
        );
        assert_eq!(
            r["interrupt_receipts"][0]["skill_cooldown_after_us"],
            expected_cd
        );
    }
}
#[test]
fn hathor_held_a1_once_marker_is_consumed_at_cap_and_survives_field_exit() {
    let mut req = request(1, 5_000_000);
    req.relative_duration_us = 90_000_000;
    req.actors[0].awakening = serde_json::from_value(json!([1])).unwrap();
    req.actions = vec![
        skill(0),
        skill(20_000_000),
        skill(40_000_000),
        skill(60_000_000),
    ];
    req.hold_inputs.clear();
    req.hold_inputs.insert(3, hold(5_000_000));
    let mut peer = req.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1020".into();
    req.actors.push(peer);
    req.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 70_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 71_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        skill(80_000_000),
    ]);
    req.hold_inputs.insert(6, hold(5_000_000));
    let r = run(&req);
    assert_eq!(power(&r)["granted"], json!("12/1"));
    assert_eq!(power(&r)["overflow"], json!("6/1"));
    let first: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_power_gain" && e["data"]["cause"] == "first_held")
        .collect();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0]["time_us"], 60_686_667);
    assert_eq!(first[0]["energy_change"]["gross"], 0);
}
#[test]
fn hathor_held_gaze_ex_never_refreshes_and_a2_pays_only_natural_expiry() {
    let mut req = request(1, 5_000_000);
    req.relative_duration_us = 10_000_000;
    req.actors[0].awakening = serde_json::from_value(json!([2])).unwrap();
    let r = run(&req);
    let marks: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_gaze")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(marks, [936_667, 5_936_667]);
    assert_eq!(power(&r)["current"], json!("3/1"));
    req.dummy_combat_events = vec![GameDummyCombatEvent::AttackStart {
        timestamp_us: 5_500_000,
        target_index: 0,
        actor_index: 0,
    }];
    let r = run(&req);
    assert_eq!(power(&r)["granted"], json!("3/1"));
    assert!(
        r["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["time_us"] != 5_936_667 || e["event_kind"] != "hathor_power_gain")
    );
}

#[test]
fn hathor_held_replacing_ordinary_gaze_invalidates_its_natural_award() {
    let mut req = request(1, 5_000_000);
    req.relative_duration_us = 15_000_000;
    req.actions[0] = skill(5_000_000);
    req.actors[0].awakening = serde_json::from_value(json!([2])).unwrap();
    let r = run(&req);
    let marks: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_gaze")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(marks, [5_000_000, 5_936_667, 10_936_667]);
    assert_eq!(power(&r)["granted"], json!("3/1"));
    assert_eq!(r["final_actor_states"][0]["energy"], 48);
}

#[test]
fn hathor_held_source_death_cancels_actor_and_end_without_posthumous_gaze_awards() {
    let mut req = request(1, 5_000_000);
    req.relative_duration_us = 10_000_000;
    req.actors[0].awakening = serde_json::from_value(json!([1, 2, 5])).unwrap();
    incoming(&mut req, 1_100_000, 100_000, true);
    let r = run(&req);
    assert_eq!(r["hits_settled"], 1);
    assert_eq!(power(&r)["granted"], json!("6/1"));
    assert_eq!(power(&r)["spent"], json!("6/1"));
    assert_eq!(power(&r)["current"], json!("0/1"));
    assert_eq!(power(&r)["conserved"], true);
    assert_eq!(
        r["final_actor_states"][0]["resources"]["hathor.a5_earned"]["current"],
        6
    );
}
#[test]
fn hathor_held_positive_dummy_contact_and_attack_start_consume_one_generation() {
    let mut req = request(1, 5_000_000);
    req.relative_duration_us = 10_000_000;
    req.actors[0].awakening = serde_json::from_value(json!([2])).unwrap();
    incoming(&mut req, 1_000_000, 1, true);
    req.dummy_combat_events = vec![GameDummyCombatEvent::AttackStart {
        timestamp_us: 1_000_000,
        target_index: 0,
        actor_index: 0,
    }];
    let r = run(&req);
    assert_eq!(power(&r)["granted"], json!("6/1"));
    let gains: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_power_gain")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(gains, [1_000_000, 6_186_667]);
    assert_eq!(r["final_actor_states"][0]["energy"], 63);
}
#[test]
fn hathor_held_tick_marks_even_zero_hp_loss_and_an_interruption_cancels_later_requests() {
    let mut req = request(1, 5_000_000);
    req.relative_duration_us = 10_000_000;
    req.actors[0].static_stats.atk = Ratio::one();
    req.actors[0].static_stats.base_atk = 1;
    req.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: 1_100_000,
        actor_index: 0,
        armor_damage: None,
    }];
    req.dummy_combat_events = vec![GameDummyCombatEvent::AttackStart {
        timestamp_us: 1_000_000,
        target_index: 0,
        actor_index: 0,
    }];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 1);
    assert_eq!(r["hit_receipts"][0]["applied_hp_loss"], 0);
    assert_eq!(power(&r)["current"], json!("3/1"));
    assert_eq!(r["final_actor_states"][0]["energy"], 15);
}
#[test]
fn hathor_held_unattributed_or_zero_incoming_damage_cannot_consume_enhanced_gaze() {
    for (amount, source) in [(1, false), (0, true)] {
        let mut req = request(1, 5_000_000);
        req.relative_duration_us = 10_000_000;
        incoming(&mut req, 1_000_000, amount, source);
        req.dummy_combat_events = vec![GameDummyCombatEvent::AttackStart {
            timestamp_us: 5_500_000,
            target_index: 0,
            actor_index: 0,
        }];
        let r = run(&req);
        let gains: Vec<_> = r["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "hathor_power_gain")
            .map(|e| e["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(gains, [5_500_000]);
    }
}
#[test]
fn hathor_held_incoming_shield_contact_consumes_exactly_once() {
    let mut req = request(1, 5_000_000);
    incoming(&mut req, 0, 100, true);
    let mut state = super::super::state::GameSimulatorState::new(&req);
    crate::registry::initialize_actor(&req.actors[0], &mut state.actors[0], &req).unwrap();
    state.actors[0]
        .shields
        .push(super::super::state::ActorShieldRuntime {
            id: "test.shield".into(),
            source: "test".into(),
            remaining_hp: 100,
            max_hp: 100,
            expires_at_us: 10_000_000,
        });
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut ctx = crate::runtime_api::CallbackContext {
        state: &mut state,
        request: &req,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    crate::characters::hathor::gaze::initialize(&mut ctx).unwrap();
    crate::characters::hathor::gaze::enhance(&mut ctx, 0, 0).unwrap();
    let receipt = super::super::incoming::settle(&mut ctx, 0, 0, 100).unwrap();
    assert_eq!(receipt.actual_hp_loss, 0);
    assert_eq!(receipt.absorbed_by_shields[0].amount, 100);
    assert_eq!(ctx.state.actors[0].energy.current(), 15);
    // The consumed generation is absent: a repeated observer cannot pay again.
    crate::characters::hathor::gaze::contact(&mut ctx, 0).unwrap();
    assert_eq!(ctx.state.actors[0].energy.current(), 15);
}
#[test]
fn hathor_held_hp_cost_does_not_consume_a_mark() {
    let mut req = request(1, 5_000_000);
    let model = HypothesisId::CombatHpCostFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.hp_cost_events = vec![GameActorHpCostEvent {
        timestamp_us: 1_000_000,
        actor_index: 0,
        source: "test paid HP".into(),
        amount: 50,
    }];
    req.dummy_combat_events = vec![GameDummyCombatEvent::AttackStart {
        timestamp_us: 5_500_000,
        target_index: 0,
        actor_index: 0,
    }];
    let r = run(&req);
    let gains: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_power_gain")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(gains, [5_500_000]);
}

#[test]
fn hathor_held_validates_hold_and_incoming_provenance_before_execution() {
    let req = request(1, 5_000_000);
    for bad_hold in [
        GameHoldInput {
            moving: true,
            ..hold(1)
        },
        GameHoldInput {
            reserved_stocks: Some(1),
            ..hold(1)
        },
    ] {
        let mut bad = req.clone();
        bad.hold_inputs.insert(0, bad_hold);
        assert!(validate_game_request(&bad, &ExecutionLimits::default()).is_err());
    }
    for (event, target) in [(0, 0), (0, 1)] {
        let mut bad = req.clone();
        bad.incoming_dummy_sources.insert(event, target);
        assert!(validate_game_request(&bad, &ExecutionLimits::default()).is_err());
    }
    let mut bad = req;
    incoming(&mut bad, 1, 1, true);
    bad.model_assumptions
        .remove(HypothesisId::HathorHeldSkillV1.as_str());
    bad.hold_inputs.clear();
    assert!(
        validate_game_request(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("contact observer")
    );
}
