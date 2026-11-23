use super::*;
use crate::simulation::{GameActorDamageEvent, GameCausalTrigger};

fn scenario() -> GameScenarioRequest {
    let mut req = phase2_request();
    let id = crate::HypothesisId::BaicangFadiaAssistV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    let mut fadia = req.actors[0].clone();
    fadia.character_id = "character:1039".into();
    fadia.slot = 1;
    fadia.initial_energy = 0;
    fadia.initial_hp = 1000;
    fadia.initial_hp_max = 100000;
    fadia.static_stats.hp = 100000;
    fadia.static_stats.base_hp = None;
    fadia.static_stats.hp_percent = Ratio::zero();
    fadia.static_stats.hp_flat = Ratio::zero();
    req.actors.push(fadia);
    req.active_actor_index = 1;
    req.actions.clear();
    req.relative_duration_us = 64_000_000;
    req
}

fn incoming(req: &mut GameScenarioRequest, time: u64, amount: u64) {
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: time,
        actor_index: 1,
        delivered_damage: amount,
    });
}

#[test]
fn fadia_autonomous_three_assists_need_no_authored_actions_and_keep_fractional_ledgers() {
    let mut req = scenario();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let report = run(&req);
    assert_eq!(report["hits_settled"], 6);
    assert_eq!(report["total_damage_exact"], json!("3348/1"));
    assert!(report["action_metadata_ledger"].is_null());
    let events = report["autonomous_events"].as_array().unwrap();
    assert_eq!(events.len(), 3);
    for (index, time) in [0, 30_000_000, 60_000_000].into_iter().enumerate() {
        assert_eq!(events[index]["time_us"], time);
        assert_eq!(events[index]["recipient_healing"]["applied"], 3240);
        if index == 0 {
            assert_eq!(events[index]["trigger"], json!({"kind":"initial_state"}));
        } else {
            assert_eq!(
                events[index]["trigger"],
                json!({"kind":"owner_deadline","autonomous_event_index":index-1})
            );
        }
    }
    for hit in report["hit_receipts"].as_array().unwrap() {
        assert!(hit["command_index"].is_null());
        assert_eq!(hit["ability_id"], "GA_Cang_WorldPas_Fadia");
        assert_eq!(
            support::hit_metadata(&report, hit)["action"]["input_role"],
            "other"
        );
        assert!(support::hit_metadata(&report, hit)["action"]["native_ability_index"].is_null());
    }
    let owner = &report["final_actor_states"][0];
    assert_eq!(owner["hp"], 9127);
    assert_eq!(owner["granted_energy"], 9);
    assert_eq!(owner["charge_remainder"], json!("3/10"));
    assert_eq!(owner["cycle"]["granted"], 15);
    assert_eq!(owner["cycle"]["carry"], json!("3/5"));
    assert_eq!(report["final_actor_states"][1]["hp"], 10720);
    assert_eq!(report["final_actor_states"][1]["energy"], 5);
    assert_eq!(
        report["final_actor_states"][1]["charge_remainder"],
        json!("29/50")
    );
}

#[test]
fn fadia_autonomous_threshold_is_inclusive_and_only_the_active_living_pair_qualifies() {
    for (hp, count) in [(20000, 1), (20001, 0), (0, 0)] {
        let mut req = scenario();
        req.relative_duration_us = 1_000_000;
        req.actors[1].initial_hp = hp;
        let report = run(&req);
        assert_eq!(report["hits_settled"], count);
    }
    let mut req = scenario();
    req.active_actor_index = 0;
    assert_eq!(run(&req)["hits_settled"], 0);
    req = scenario();
    req.actors[0].initial_hp = 0;
    assert_eq!(run(&req)["hits_settled"], 0);
    req = scenario();
    req.actors[0].character_level = 40;
    req.actors[0].purchased_qte_level = Some(1);
    req.actors[0].character_max_level = Some(40);
    assert_eq!(run(&req)["hits_settled"], 0);
    req = scenario();
    req.targets[0].hp_current = 0;
    let report = run(&req);
    assert_eq!(report["hits_settled"], 0);
    assert_eq!(report["final_actor_states"][1]["hp"], 1000);
    assert!(report["autonomous_events"].is_null());
}

#[test]
fn fadia_autonomous_damage_callbacks_and_exact_ready_deadlines_keep_their_real_causes() {
    let mut req = scenario();
    req.actors[1].initial_hp = 2000;
    req.actors[1].initial_hp_max = 10000;
    req.actors[1].static_stats.hp = 10000;
    incoming(&mut req, 29_999_999, 3500);
    let report = run(&req);
    let events = report["autonomous_events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1]["time_us"], 30_000_000);
    assert_eq!(report["final_actor_states"][1]["hp"], 4980);
    req.incoming_damage_events[0].timestamp_us = 31_000_000;
    let report = run(&req);
    assert_eq!(report["autonomous_events"][1]["time_us"], 31_000_000);
    assert_eq!(
        report["autonomous_events"][1]["trigger"],
        json!({"kind":"incoming_damage","event_index":0})
    );
}

#[test]
fn fadia_autonomous_heal_reads_base_scaled_b4_and_new_a2_without_charge_efficiency_scaling() {
    let mut req = scenario();
    req.relative_duration_us = 1_000_000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 2000,
        denominator: 1,
    };
    req.actors[0]
        .gear_procs
        .push(crate::simulation::GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator: 2,
                denominator: 1,
            },
        });
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![2]).unwrap();
    let report = run(&req);
    let heal = &report["autonomous_events"][0]["recipient_healing"];
    assert_eq!(heal["base"], json!("6102/1")); // (2000 +1000*(.20+.06))*2.7
    assert_eq!(heal["applied"], 6102);
    assert_eq!(report["final_actor_states"][1]["hp"], 7102);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 6); // resource efficiency affects Charge only
}

#[test]
fn fadia_autonomous_unbound_model_diagnoses_and_a_switch_keeps_its_authored_parent() {
    let mut req = scenario();
    req.model_assumptions
        .remove(crate::HypothesisId::BaicangFadiaAssistV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("UNSUPPORTED_BAICANG_FADIA_ASSIST")
    );
    req = scenario();
    req.active_actor_index = 0;
    req.relative_duration_us = 1_000_000;
    req.actions = vec![GameActionCommand::SwitchActor {
        timestamp_us: 1,
        target_actor_index: 1,
        cooldown_us: 0,
    }];
    let report = run(&req);
    assert_eq!(
        report["autonomous_events"][0]["trigger"],
        json!({"kind":"field_entry","command_index":0})
    );
    assert!(report["hit_receipts"][0]["command_index"].is_null());
}

#[test]
fn fadia_autonomous_heal_clips_both_healing_zones_once_to_the_captured_recipient() {
    let mut req = scenario();
    req.actors[1].initial_hp = 2000;
    req.actors[1].initial_hp_max = 10000;
    req.actors[1].static_stats.hp = 10000;
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    let id = crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    crate::registry::initialize_actor(&req.actors[0], &mut state.actors[0], &req).unwrap();
    crate::simulation::state::apply_timed_buff(
        &mut state.buffs,
        crate::simulation::state::TimedBuffDeclaration::reset(
            "test.fadia.incoming",
            "test",
            ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(1),
            2_000_000,
            ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([(
                "incoming_healing_bonus".into(),
                ExactFraction::new(1, 5).unwrap(),
            )]),
        ),
        0,
        0,
    )
    .unwrap();
    let key = state
        .origins
        .register(
            &req,
            crate::simulation::GameAutonomousEventReceipt {
                actor_index: 0,
                active_actor_index: 1,
                ability_id: crate::characters::baicang::fadia_assist::ABILITY.into(),
                time_us: 0,
                trigger: GameCausalTrigger::InitialState,
                recipient_healing: None,
            },
        )
        .unwrap();
    // An admitted heal retains Fadia after another settlement changes control/HP.
    state.active_actor_index = 0;
    state.actors[1].hp = 9000;
    let mut ctx = crate::runtime_api::CallbackContext {
        state: &mut state,
        request: &req,
        queue: &mut Default::default(),
        sequence: &mut 0,
        trace: &mut Vec::new(),
    };
    crate::characters::baicang::fadia_assist::connected(
        &mut ctx,
        0,
        key,
        None,
        crate::characters::baicang::fadia_assist::PACKET,
    )
    .unwrap();
    let proof = ctx
        .state
        .origins
        .get(0)
        .unwrap()
        .recipient_healing
        .as_ref()
        .unwrap();
    assert_eq!(
        (proof.gross, proof.applied, proof.excess),
        (5832, 1000, 4832)
    ); // 1200*2.7*1.5*1.2
    assert_eq!(ctx.state.actors[1].hp, 10000);
    ctx.state.actors[1].hp = 2000;
    crate::characters::baicang::fadia_assist::connected(
        &mut ctx,
        0,
        key,
        None,
        crate::characters::baicang::fadia_assist::PACKET,
    )
    .unwrap();
    assert_eq!(ctx.state.actors[1].hp, 2000);
}

#[test]
fn fadia_autonomous_max_hp_settlement_is_a_causal_callback_and_does_not_touch_skill_or_recovery() {
    let mut req = scenario();
    req.actors[1].initial_hp = 2500;
    req.actors[1].initial_hp_max = 10000;
    req.actors[1].static_stats.hp = 10000;
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    let before = crate::simulation::vitals::Snapshot::capture(&state).unwrap();
    state.actors[1].hp_max = 15000;
    state.actors[0].locked_until_us = 15_000_000;
    state.actors[0]
        .ability_cooldowns
        .insert("GA_Cang_Skill".into(), 14_000_000);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut seq = 42;
    let mut trace = Vec::new();
    let mut ctx = crate::runtime_api::CallbackContext {
        state: &mut state,
        request: &req,
        queue: &mut queue,
        sequence: &mut seq,
        trace: &mut trace,
    };
    before
        .settle(
            &mut ctx,
            GameCausalTrigger::VitalChange { event_sequence: 42 },
        )
        .unwrap();
    let event = ctx.queue.pop().unwrap();
    let crate::simulation::events::GameEventData::Content(event) = event.data else {
        panic!("expected owner callback");
    };
    crate::callbacks::apply(event, &mut ctx).unwrap();
    assert_eq!(
        ctx.state.origins.get(0).unwrap().trigger,
        GameCausalTrigger::VitalChange { event_sequence: 42 }
    );
    assert_eq!(
        ctx.state.actors[0]
            .ability_cooldowns
            .get("GA_Cang_Skill")
            .copied()
            .unwrap(),
        14_000_000
    );
    assert_eq!(
        ctx.state.actors[0]
            .ability_cooldowns
            .get("GA_Cang_WorldPas_Fadia")
            .copied()
            .unwrap(),
        30_000_000
    );
    assert_eq!(ctx.state.actors[0].locked_until_us, 15_000_000);
    assert_eq!(ctx.state.active_actor_index, 1);
}

#[test]
fn fadia_autonomous_word_and_break_descendants_keep_the_event_without_inventing_casts() {
    let mut req = scenario();
    req.relative_duration_us = 25_000_000;
    req.active_actor_index = 0;
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.actions = vec![
        burst(0),
        GameActionCommand::SwitchActor {
            timestamp_us: 3_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run(&req);
    let silence = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["damage_effect_id"] == "GE_Player_Cang_WorldA_Damage")
        .unwrap();
    assert_eq!(silence["autonomous_event_index"], 0);
    assert!(silence["command_index"].is_null());
    assert_eq!(
        report["action_metadata_ledger"].as_array().unwrap().len(),
        1
    );

    req = scenario();
    req.relative_duration_us = 1_000_000;
    req.targets[0].break_profile = Some(crate::simulation::GameTargetBreakProfileConfig {
        capacity: Ratio::one(),
        current_gauge: Ratio {
            numerator: 1,
            denominator: 2,
        },
        broken_duration_us: 100_000,
        discord_fraction: None,
    });
    let report = run(&req);
    let descendants = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit.get("break_contribution").is_some())
        .collect::<Vec<_>>();
    assert_eq!(descendants.len(), 2);
    for hit in descendants {
        assert_eq!(hit["autonomous_event_index"], 0);
        assert!(hit["command_index"].is_null());
    }
}
