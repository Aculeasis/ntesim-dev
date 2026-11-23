use super::*;
use crate::HypothesisId;
use serde_json::{Value, json};
pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::hathor_clock::request(targets);
    req.actors[0].initial_energy = 100;
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    for model in [
        HypothesisId::HathorTapPowerV1,
        HypothesisId::HathorGazeV1,
        HypothesisId::HathorHeldSkillV1,
        HypothesisId::HathorBurstEmergencyV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req.actions.clear();
    req
}
pub(super) fn skill(time: u64) -> GameActionCommand {
    super::hathor_actions::make_action(
        time,
        0,
        "GA_Hathor_Skill",
        "GE_Player_Hathor_Skill1_Damage",
        0,
        0,
    )
}
pub(super) fn burst(time: u64) -> GameActionCommand {
    super::hathor_actions::make_action(
        time,
        0,
        "GA_Hathor_UltraSkill",
        "GE_Player_Hathor_UltraSkill1_Damage",
        100,
        0,
    )
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
fn power(r: &Value) -> &Value {
    &r["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"]
}
fn hybrid(req: &mut GameScenarioRequest) {
    let m = HypothesisId::HathorHybridEmergencyV1;
    req.model_assumptions
        .insert(m.into(), m.description().into());
}

#[test]
fn hathor_burst_has_two_collision_contacts_one_landing_and_no_future_power_snapshot() {
    let mut req = request(1);
    req.actions = vec![burst(0)];
    let r = run(&req);
    let hits = r["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 3);
    assert_eq!(
        hits.iter()
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [2_483_287, 2_883_287, 3_816_348]
    );
    assert_eq!(hits[0]["non_crit_damage"], json!("1649/2"));
    assert_eq!(hits[2]["non_crit_damage"], json!("8699/2"));
    assert_eq!(r["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(r["final_actor_states"][0]["energy"], 0);
    req.actions.push(super::hathor_clock::normal(4_530_560));
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]
            ["until_us"],
        4_530_561
    );
    *req.actions.last_mut().unwrap() = super::hathor_clock::normal(4_530_561);
    run(&req);
}
#[test]
fn hathor_burst_snapshots_actual_power_at_state_delivery_and_adds_base_atk_only() {
    let mut req = request(1);
    req.actions = vec![skill(0), burst(2_000_000)];
    let r = run(&req);
    let hits = r["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[3]["non_crit_damage"], json!("21437/20"));
    assert_eq!(hits[5]["non_crit_damage"], json!("113087/20"));
    req.actors[0].static_stats.atk = Ratio {
        numerator: 3000,
        denominator: 1,
    };
    let r = run(&req);
    assert_eq!(r["hit_receipts"][3]["non_crit_damage"], json!("54417/20")); //3300*.5*1.649.
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].awakening = serde_json::from_value(json!([2])).unwrap();
    req.actions[1] = burst(10_000_000);
    let r = run(&req); // World-10 Gaze settles before commitment: n3, ATK1450.
    assert_eq!(r["hit_receipts"][3]["non_crit_damage"], json!("47821/40"));
}
#[test]
fn hathor_burst_live_source_atk_matches_damage_zone_and_retains_the_captured_count() {
    let mut req = request(1);
    req.actors[0].static_stats.atk = Ratio {
        numerator: 3000,
        denominator: 1,
    };
    let mut state = super::super::state::GameSimulatorState::new(&req);
    crate::registry::initialize_actor(&req.actors[0], &mut state.actors[0], &req).unwrap();
    let s = state.actors[0].character.hathor_mut().unwrap();
    s.power
        .as_mut()
        .unwrap()
        .pool
        .grant(ExactFraction::new(2, 1).unwrap())
        .unwrap();
    s.express_delivery_power = 2;
    state.actors[0].locked_until_us = 4_530_561;
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
    crate::characters::hathor::burst::enter(&mut ctx, 0).unwrap();
    assert_eq!(
        ctx.live_atk(0).unwrap(),
        ExactFraction::new(3300, 1).unwrap()
    );
    let s = ctx.state.actors[0].character.hathor_mut().unwrap();
    s.power
        .as_mut()
        .unwrap()
        .pool
        .spend_up_to(ExactFraction::new(2, 1).unwrap())
        .unwrap();
    s.express_delivery_power = 0;
    assert_eq!(
        ctx.live_atk(0).unwrap(),
        ExactFraction::new(3300, 1).unwrap()
    );
    crate::characters::hathor::burst::clear(&mut ctx, 0).unwrap();
    assert_eq!(
        ctx.live_atk(0).unwrap(),
        ExactFraction::new(3000, 1).unwrap()
    );
}
#[test]
fn hathor_burst_exact_expiry_removes_atk_and_a6_before_equal_time_damage() {
    for (delta, expected) in [(0, json!("1352/5")), (1, json!("208/1"))] {
        let mut req = request(1);
        req.actors[0].awakening = serde_json::from_value(json!([6])).unwrap();
        req.actions = vec![
            skill(0),
            burst(2_000_000),
            super::hathor_clock::normal(14_301_850 + delta),
        ];
        let r = run(&req);
        let last = r["hit_receipts"].as_array().unwrap().last().unwrap();
        assert_eq!(last["non_crit_damage"], expected);
        let crit = if delta == 0 {
            json!("8788/25")
        } else {
            json!("208/1")
        };
        assert_eq!(last["crit_damage"], crit);
    }
}
#[test]
fn hathor_burst_generation_rejection_consumes_marks_without_energy_a5_or_deferred_refund() {
    let mut req = request(1);
    req.actors[0].awakening = serde_json::from_value(json!([2, 5])).unwrap();
    req.actions = vec![skill(0), burst(2_000_000)];
    req.relative_duration_us = 21_000_000;
    req.dummy_combat_events = vec![GameDummyCombatEvent::AttackStart {
        timestamp_us: 5_000_000,
        target_index: 0,
        actor_index: 0,
    }];
    let r = run(&req);
    let rejected: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_power_rejected")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(rejected, [9_518_597]);
    assert_eq!(power(&r)["granted"], json!("3/1"));
    assert_eq!(power(&r)["current"], json!("3/1"));
    assert_eq!(
        r["final_actor_states"][0]["resources"]["hathor.a5_earned"]["current"],
        3
    );
    assert_eq!(r["final_actor_states"][0]["energy"], 5);
}
#[test]
fn hathor_burst_enhanced_and_ordinary_attack_consumptions_reject_during_state() {
    let mut req = request(1);
    req.actions = vec![skill(0), burst(1_100_000)];
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            duration_us: 5_000_000,
            moving: false,
            reserved_stocks: None,
        },
    );
    req.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: 1_000_000,
        actor_index: 0,
        armor_damage: None,
    }];
    req.relative_duration_us = 12_000_000;
    req.dummy_combat_events = vec![
        GameDummyCombatEvent::AttackStart {
            timestamp_us: 3_000_000,
            target_index: 0,
            actor_index: 0,
        },
        GameDummyCombatEvent::AttackStart {
            timestamp_us: 8_100_000,
            target_index: 0,
            actor_index: 0,
        },
    ];
    let r = run(&req);
    assert_eq!(power(&r)["granted"], json!("0/1"));
    let rejected: Vec<_> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_power_rejected")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(rejected, [7_518_597, 12_618_597]);
}
#[test]
fn hathor_burst_hybrid_suspends_only_its_timer_and_interruption_resumes_five_seconds() {
    for enabled in [false, true] {
        let mut req = request(1);
        if enabled {
            hybrid(&mut req)
        }
        req.actors[0].awakening = serde_json::from_value(json!([6])).unwrap();
        req.actions = vec![burst(0), super::hathor_clock::normal(10_000_000)];
        let r = run(&req);
        let last = r["hit_receipts"].as_array().unwrap().last().unwrap();
        assert_eq!(
            last["crit_damage"],
            if enabled {
                json!("208/1")
            } else {
                json!("1352/5")
            }
        );
    }
    for (delta, crit) in [(0, json!("1352/5")), (1, json!("208/1"))] {
        let mut req = request(1);
        hybrid(&mut req);
        req.actors[0].awakening = serde_json::from_value(json!([6])).unwrap();
        req.actions = vec![burst(0), super::hathor_clock::normal(6_783_253 + delta)];
        req.interrupt_events = vec![GameActorInterruptEvent {
            timestamp_us: 2_000_000,
            actor_index: 0,
            armor_damage: None,
        }];
        let r = run(&req);
        assert_eq!(r["hits_settled"], 1);
        assert_eq!(r["hit_receipts"][0]["crit_damage"], crit);
        assert_eq!(r["final_actor_states"][0]["spent_energy"], 100);
    }
}
#[test]
fn hathor_burst_a3_counts_living_non_boss_dummies_and_r6_uses_live_defense_ignore() {
    let mut req = request(3);
    req.actors[0].awakening = serde_json::from_value(json!([3])).unwrap();
    req.targets[0].is_boss = true;
    req.targets[2].hp_max = 1;
    req.targets[2].hp_current = 1;
    req.actions = vec![
        super::hathor_clock::normal(0),
        super::hathor_clock::normal(2_000_000),
    ];
    let r = run(&req);
    let hits = r["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 5);
    for hit in &hits[..3] {
        assert_eq!(hit["non_crit_damage"], json!("1352/5"));
    }
    for hit in &hits[3..] {
        assert_eq!(hit["non_crit_damage"], json!("1196/5"));
    }
    for target in &mut req.targets {
        target.is_boss = true;
        target.hp_max = 1_000_000;
        target.hp_current = 1_000_000;
    }
    let all_boss = run(&req);
    assert!(
        all_boss["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["non_crit_damage"] == json!("208/1"))
    );
    let mut req = request(1);
    req.actors[0].awakening = serde_json::from_value(json!(6)).unwrap();
    req.actions = vec![skill(0), burst(2_000_000)];
    let r = run(&req);
    // Rank3 talent9, A3+.15, A5 earned2*.01, ATK1300, live DEFignore.10.
    assert_eq!(
        r["hit_receipts"][3]["non_crit_damage"],
        json!("2708901/1900")
    );
}
#[test]
fn hathor_burst_field_exit_and_death_end_the_snapshot_and_skill_rejects_unsupported_cyclone() {
    let mut req = request(1);
    req.actions = vec![burst(0), skill(6_000_000)];
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(
        r["rejected_action"]["reason"]["message"]
            .as_str()
            .unwrap()
            .contains("Cyclone")
    );
    req.actions[1] = skill(12_518_597);
    run(&req); // Exact expiry admits ordinary Skill using live time.
    let mut peer = req.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1020".into();
    req.actors.push(peer);
    req.actions = vec![
        skill(0),
        burst(2_000_000),
        GameActionCommand::SwitchActor {
            timestamp_us: 7_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 8_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        super::hathor_clock::normal(8_000_000),
    ];
    let r = run(&req);
    assert_eq!(
        r["hit_receipts"].as_array().unwrap().last().unwrap()["non_crit_damage"],
        json!("208/1")
    );
    req.actions = vec![burst(0)];
    let m = HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(m.into(), m.description().into());
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: 100_000,
    }];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 0);
}
#[test]
fn hathor_burst_preflight_and_cooldown_boundaries_do_not_refund_committed_energy() {
    let mut req = request(1);
    req.actions = vec![burst(0), burst(19_518_596)];
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(r["rejected_action"]["reason"]["until_us"], 19_518_597);
    assert_eq!(r["final_actor_states"][0]["spent_energy"], 100);
    *req.actions.last_mut().unwrap() = burst(19_518_597);
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(r["rejected_action"]["reason"]["until_us"].is_null());
    req.actions = vec![burst(u64::MAX - 14_000_000)];
    req.relative_duration_us = u64::MAX;
    req.absolute_limit_us = req.relative_duration_us;
    assert!(
        validate_game_request(
            &req,
            &ExecutionLimits {
                max_horizon_us: u64::MAX,
                ..Default::default()
            }
        )
        .unwrap_err()
        .contains("Burst cooldown deadline overflow")
    );
}
