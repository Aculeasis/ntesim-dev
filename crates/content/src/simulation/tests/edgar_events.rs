use super::edgar_full::{burst, heals, hold, normal, request, run, skill};
use super::*;
use serde_json::json;
fn ally(req: &mut GameScenarioRequest) {
    let mut actor = req.actors[0].clone();
    actor.slot = 1;
    actor.character_id = "character:1019".into();
    actor.awakening = Default::default();
    actor.initial_hp = 1000;
    actor.initial_hp_max = 1000;
    actor.static_stats.hp = 1000;
    req.actors.push(actor);
}
fn incoming(req: &mut GameScenarioRequest) {
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
}
#[test]
fn edgar_channel_interrupt_settles_only_live_elapsed_and_a2_caps_only_extra_cd() {
    for (at, hits, ready) in [(500000, 0, 3500000), (1000000, 1, 17335200)] {
        let mut req = request(1);
        req.actions = vec![skill(0), skill(ready - 1)];
        hold(&mut req, 0, 4900000);
        req.interrupt_events = vec![GameActorInterruptEvent {
            timestamp_us: at,
            actor_index: 0,
            armor_damage: None,
        }];
        let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(r["execution_completed"], false);
        assert_eq!(r["hits_settled"], hits);
        assert_eq!(r["rejected_action"]["reason"]["until_us"], ready);
        req.actions[1] = skill(ready);
        run(&req);
    }
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    req.actions = vec![skill(0), skill(23996740)];
    hold(&mut req, 0, 4900000);
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(r["rejected_action"]["reason"]["until_us"], 23996741);
    req.actions[1] = skill(23996741);
    run(&req);
}
#[test]
fn edgar_channel_retargets_only_on_the_next_pulse_and_heals_without_enemy_hits() {
    let mut req = request(2);
    req.targets[0].hp_current = 100;
    req.targets[0].hp_max = 100;
    req.targets[1].hp_current = 100;
    req.targets[1].hp_max = 100;
    req.actions = vec![skill(0)];
    hold(&mut req, 0, 4900000);
    let r = run(&req);
    assert_eq!(r["hits_settled"], 2);
    assert_eq!(r["hit_receipts"][0]["target_index"], 0);
    assert_eq!(r["hit_receipts"][1]["target_index"], 1);
    assert_eq!(heals(&r, "edgar_skill_heal").len(), 7);
    assert_eq!(r["final_actor_states"][0]["hp"], 4080);
}
#[test]
fn edgar_full_hp_skill_requests_still_grant_one_key_and_no_heal_callback() {
    let mut req = request(1);
    req.actors[0].initial_hp = 10000;
    req.actions = vec![skill(0)];
    hold(&mut req, 0, 4900000);
    let r = run(&req);
    assert_eq!(heals(&r, "edgar_skill_heal").len(), 0);
    assert_eq!(
        r["final_actor_states"][0]["resources"]["edgar.keys_of_truth"]["current"],
        1
    );
}
#[test]
fn edgar_field_follows_active_recipient_and_source_death_cancels_remaining_ticks() {
    let mut req = request(1);
    ally(&mut req);
    req.actors[1].initial_hp = 100;
    req.actions = vec![
        burst(0),
        GameActionCommand::SwitchActor {
            timestamp_us: 3500000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let r = run(&req);
    assert_eq!(heals(&r, "edgar_field_heal").len(), 10);
    assert_eq!(r["final_actor_states"][0]["hp"], 1000);
    assert_eq!(r["final_actor_states"][1]["hp"], 1000);
    incoming(&mut req);
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 4000000,
        actor_index: 0,
        delivered_damage: 2000,
    }];
    let dead = run(&req);
    // Incoming damage uses World time: four pulses precede the World-4 defeat.
    assert_eq!(heals(&dead, "edgar_field_heal").len(), 4);
    assert_eq!(dead["final_actor_states"][0]["hp"], 0);
}
#[test]
fn edgar_r6_combines_fractional_team_hp_loss_then_pays_before_ce_refund() {
    let mut req = request(1);
    ally(&mut req);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    super::edgar_actions::set_a5_vitals(&mut req.actors[0], 10000);
    req.actors[0].gear_procs = vec![GearProcConfig::ChargeEfficiency {
        multiplier: Ratio {
            numerator: 3,
            denominator: 2,
        },
    }];
    incoming(&mut req);
    req.incoming_damage_events = vec![
        GameActorDamageEvent {
            timestamp_us: 0,
            actor_index: 1,
            delivered_damage: 15,
        },
        GameActorDamageEvent {
            timestamp_us: 1000,
            actor_index: 1,
            delivered_damage: 15,
        },
        GameActorDamageEvent {
            timestamp_us: 200000,
            actor_index: 1,
            delivered_damage: 15,
        },
    ];
    req.actions = vec![burst(100000)];
    let r = run(&req);
    let a = &r["final_actor_states"][0];
    assert_eq!(a["spent_energy"], 100);
    assert_eq!(a["energy"], 1);
    assert_eq!(a["charge_remainder"], json!("1/2"));
    let pool = &a["fractional_resources"]["edgar.pending_charge"];
    assert_eq!(pool["current"], json!("1/2"));
    assert_eq!(pool["spent"], json!("1/1"));
    assert_eq!(pool["conserved"], true);
    assert_eq!(heals(&r, "edgar_r6_refund").len(), 1);
    req.actors[0].initial_energy = 99;
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(early["execution_completed"], false);
    assert_eq!(heals(&early, "edgar_r6_refund").len(), 0);
}
#[test]
fn edgar_paid_critical_dodge_grants_one_riposte_and_can_queue_normal_five() {
    let mut req = request(2);
    super::blackbird_secondary::stamina(&mut req, 100);
    req.actions = vec![
        super::blackbird_secondary::dash(0, true),
        normal(1004912),
        normal(2203793),
    ];
    let r = run(&req);
    let hits = r["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 4);
    assert_eq!(
        hits[0]["damage_effect_id"],
        "GE_Player_Edgar_PerfectEvadeAttack_Damage"
    );
    assert_eq!(hits[0]["time_us"], 1522643);
    assert_eq!(hits[2]["damage_effect_id"], "GE_Player_Edgar_Melee5_Damage");
    assert_eq!(r["combat_stamina"]["spends"].as_array().unwrap().len(), 1);
}
#[test]
fn edgar_plunge_uses_landing_height_and_one_packet_resource_group() {
    for height in [0, 1] {
        let mut req = request(3);
        req.actors[0].initial_energy = 0;
        req.actions = vec![super::edgar_full::action(
            0,
            "GA_Edgar_Melee",
            "GE_Player_Edgar_AirAttack_Damage",
            0,
        )];
        req.plunge_inputs.insert(
            0,
            GamePlungeInput {
                landing_delay_us: 600000,
                height_bonus: Ratio {
                    numerator: height,
                    denominator: 1,
                },
            },
        );
        let r = run(&req);
        assert_eq!(r["hits_settled"], 3);
        assert_eq!(r["hit_receipts"][0]["time_us"], 649876);
        assert_eq!(
            r["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", 156 * (1 + height), 1))
        );
        assert_eq!(r["final_actor_states"][0]["energy"], 2);
    }
}
#[test]
fn edgar_combat_leave_removes_field_without_an_extra_terminal_heal() {
    let mut req = request(1);
    req.actions = vec![burst(0)];
    req.combat_transition_events = vec![
        GameActorCombatTransitionEvent {
            timestamp_us: 5402223,
            actor_index: 0,
            leaving: true,
        },
        GameActorCombatTransitionEvent {
            timestamp_us: 7402224,
            actor_index: 0,
            leaving: false,
        },
    ];
    let r = run(&req);
    assert_eq!(heals(&r, "edgar_field_heal").len(), 1);
}

#[test]
fn edgar_charge_replaces_chiz_fourteen_then_suppresses_the_guarded_recipient() {
    let mut req=crate::dsl::compiler::compile_scenario("options duration=12; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100; edgar char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100; hathor char lvl=20/20 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint;",Default::default()).unwrap().request;
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 3_000_000,
            target_actor_index: 2,
            cooldown_us: 0,
        },
    ];
    req.actors[1].character_id = "character:1021".into();
    let model = crate::characters::edgar::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let mut chiz = req.actors[0].clone();
    chiz.slot = 3;
    chiz.character_id = "character:1073".into();
    chiz.initial_cycle = 0;
    req.actors.push(chiz);
    req.actors[2].gear_procs = vec![GearProcConfig::ChargeEfficiency {
        multiplier: Ratio {
            numerator: 3,
            denominator: 2,
        },
    }];
    let r = run(&req);
    assert_eq!(
        r["final_actor_states"][1]["resources"]["edgar.keys_of_truth"]["current"],
        1
    );
    let grants = heals(&r, "reaction_charge_energy");
    assert_eq!(grants.len(), 2);
    assert_eq!(grants[0]["energy_change"]["charge_basis"], json!("120/1"));
    assert_eq!(grants[0]["energy_change"]["gross"], 180);
    assert_eq!(grants[1]["energy_change"]["charge_basis"], json!("0/1"));
    assert_eq!(grants[1]["energy_change"]["gross"], 0);
    // The guard is recipient-owned; another living recipient may receive120.
    let mut state = super::super::state::GameSimulatorState::new(&req);
    let mut assumptions = Default::default();
    state.active_actor_index = 2;
    assert_eq!(
        crate::registry::charge_reaction_override(&req, &mut state, &mut assumptions).unwrap(),
        Some(120)
    );
    state.advance_unpaused_test_clock(29_999_999);
    assert_eq!(
        crate::registry::charge_reaction_override(&req, &mut state, &mut assumptions).unwrap(),
        Some(0)
    );
    state.active_actor_index = 0;
    assert_eq!(
        crate::registry::charge_reaction_override(&req, &mut state, &mut assumptions).unwrap(),
        Some(120)
    );
    state.advance_unpaused_test_clock(30_000_000);
    state.active_actor_index = 2;
    assert_eq!(
        crate::registry::charge_reaction_override(&req, &mut state, &mut assumptions).unwrap(),
        Some(120)
    );
}

#[test]
fn edgar_a1_observes_one_real_teammate_support_while_off_field() {
    for (a1, keys) in [(false, 0), (true, 1)] {
        let mut req = super::daffodil_secondary::request(1);
        let mut edgar = request(1).actors.remove(0);
        edgar.slot = 1;
        edgar.awakening = ntesim_data::AwakeningSelection::prefix(u32::from(a1));
        req.actors.push(edgar);
        let mut bird = req.actors[0].clone();
        bird.slot = 2;
        bird.character_id = "character:1042".into();
        bird.initial_cycle = 100;
        req.actors.push(bird);
        req.active_actor_index = 2;
        let model = crate::characters::edgar::clock::MODEL;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.actions = vec![GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 0,
            cooldown_us: 0,
        }];
        let r = run(&req);
        assert_eq!(
            r["final_actor_states"][1]["resources"]["edgar.keys_of_truth"]["current"],
            keys
        );
    }
}

#[test]
fn edgar_loss_listener_ignores_stat_clamps_and_retains_healed_costs_at_each_loss_time_max() {
    use crate::simulation::vitals::Snapshot;
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    super::edgar_actions::set_a5_vitals(&mut req.actors[0], 10000);
    let mut state = super::super::state::GameSimulatorState::new(&req);
    crate::registry::initialize_actor(&req.actors[0], &mut state.actors[0], &req).unwrap();
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let snapshot = Snapshot::capture(&state).unwrap();
    state.actors[0].hp_max_base = ExactFraction::new(8000, 1).unwrap();
    state.actors[0].hp_max = 10000;
    state.actors[0].hp = 10000;
    let mut ctx = crate::runtime_api::CallbackContext {
        request: &req,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    snapshot
        .settle(
            &mut ctx,
            GameCausalTrigger::VitalChange { event_sequence: 1 },
        )
        .unwrap();
    assert!(
        ctx.state.actors[0]
            .character
            .edgar_mut()
            .unwrap()
            .pending_charge
            .as_ref()
            .unwrap()
            .current()
            .is_zero()
    );
    let snapshot = Snapshot::capture(ctx.state).unwrap();
    // Two paid150HP costs surround a stat-only MaxHP change; each is healed in the same event.
    let actor = &mut ctx.state.actors[0];
    actor.hp -= 150;
    actor.record_hp_loss(150, 10000);
    actor.hp += 150;
    actor.hp_max_base = ExactFraction::new(18000, 1).unwrap();
    actor.hp_max = 20000;
    actor.hp -= 150;
    actor.record_hp_loss(150, 20000);
    actor.hp += 150;
    snapshot
        .settle(
            &mut ctx,
            GameCausalTrigger::VitalChange { event_sequence: 2 },
        )
        .unwrap();
    assert_eq!(
        ctx.state.actors[0]
            .character
            .edgar_mut()
            .unwrap()
            .pending_charge
            .as_ref()
            .unwrap()
            .current(),
        ExactFraction::new(3, 4).unwrap()
    );
    assert!(
        ctx.state.actors[0]
            .combat_hp_losses
            .as_ref()
            .unwrap()
            .is_empty()
    );
    Snapshot::capture(ctx.state)
        .unwrap()
        .settle(
            &mut ctx,
            GameCausalTrigger::VitalChange { event_sequence: 3 },
        )
        .unwrap();
    assert_eq!(
        ctx.state.actors[0]
            .character
            .edgar_mut()
            .unwrap()
            .pending_charge
            .as_ref()
            .unwrap()
            .current(),
        ExactFraction::new(3, 4).unwrap()
    );
}
