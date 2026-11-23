use super::*;
use serde_json::{Value, json};
fn request(amount: u64) -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actions.clear();
    req.relative_duration_us = 2_000_000;
    let actor = &mut req.actors[0];
    actor.gear_procs.clear();
    actor.weapon_id.clear();
    actor.set_id = None;
    actor.awakening = Default::default();
    actor.initial_hp = 10000;
    actor.initial_hp_max = 10000;
    actor.static_stats.hp = 10000;
    actor.static_stats.base_def = Some(1000);
    actor.static_stats.def = 1600;
    actor.static_stats.def_percent = Ratio {
        numerator: 1,
        denominator: 2,
    };
    actor.static_stats.def_flat = Ratio {
        numerator: 100,
        denominator: 1,
    };
    req.targets[0].level = 80;
    for model in [
        crate::HypothesisId::CombatIncomingDamageFixtureV1,
        crate::HypothesisId::CombatIncomingDefenseV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 500_000,
        actor_index: 0,
        delivered_damage: amount,
    }];
    req.incoming_dummy_sources.insert(0, 0);
    req.incoming_damage_basis
        .insert(0, GameIncomingDamageBasis::BeforeDefense);
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
#[test]
fn incoming_defense_uses_exact_compiled_base_percent_and_flat_once() {
    let r = run(&request(1000));
    let receipt = &r["incoming_damage_receipts"][0];
    // DEF=1000*(1+.5)+100=1600;1000*1080/(1080+1600)=27000/67 ->403.
    assert_eq!(receipt["delivered_damage"], 403);
    assert_eq!(receipt["actual_hp_loss"], 403);
    assert_eq!(
        receipt["defense_mitigation"],
        json!({"damage_before_defense":1000,"source_target_index":0,"source_level":80,"recipient_defense":"1600/1"})
    );
    assert_eq!(r["final_actor_states"][0]["hp"], 9597);
}
#[test]
fn incoming_defense_samples_live_base_zone_then_shields_and_expiry_without_polling() {
    use ntesim_core::buffs::dispatch::*;
    let mut req = request(1000);
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 1_000_000,
        actor_index: 0,
        delivered_damage: 1000,
    });
    req.incoming_dummy_sources.insert(1, 0);
    req.incoming_damage_basis
        .insert(1, GameIncomingDamageBasis::BeforeDefense);
    let mut state = super::super::state::GameSimulatorState::new(&req);
    state
        .buffs
        .apply_buff(
            BuffDefinition {
                id: "test.incoming_def".into(),
                source: "test".into(),
                recipient: BuffRecipient::TeamAll,
                source_lifetime: BuffSourceLifetime::Snapshot,
                stacking_scope: BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                duration_us: Some(1_000_000),
                refresh_policy: RefreshPolicy::Disallow,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: std::collections::BTreeMap::from([(
                    "def_percent".into(),
                    Ratio::one(),
                )]),
            },
            0,
            0,
        )
        .unwrap();
    state.actors[0]
        .shields
        .push(super::super::state::ActorShieldRuntime {
            id: "test.shield".into(),
            source: "test".into(),
            remaining_hp: 200,
            max_hp: 200,
            expires_at_us: 2_000_000,
        });
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut ctx = crate::runtime_api::CallbackContext {
        request: &req,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    ctx.state.advance_unpaused_test_clock(500_000);
    let first = super::super::incoming::settle(&mut ctx, 0, 0, 1000).unwrap();
    // Live100% adds base1000, not static1600: DEF2600;27000/92 ->293.
    assert_eq!(
        first.defense_mitigation.unwrap().recipient_defense,
        ExactFraction::new(2600, 1).unwrap()
    );
    assert_eq!(first.delivered_damage, 293);
    assert_eq!(first.absorbed_by_shields[0].amount, 200);
    assert_eq!(first.actual_hp_loss, 93);
    ctx.state.advance_unpaused_test_clock(1_000_000);
    let second = super::super::incoming::settle(&mut ctx, 1, 0, 1000).unwrap();
    assert_eq!(
        second.defense_mitigation.unwrap().recipient_defense,
        ExactFraction::new(1600, 1).unwrap()
    );
    assert_eq!(second.delivered_damage, 403);
    assert_eq!(second.actual_hp_loss, 403);
    assert!(queue.is_empty());
    assert_eq!(state.actors[0].hp, 9504);
}
#[test]
fn incoming_defense_rounds_once_with_upward_ties_and_uses_the_authored_source_level() {
    let mut req = request(1);
    req.actors[0].static_stats.base_def = Some(1080);
    req.actors[0].static_stats.def = 1080;
    req.actors[0].static_stats.def_percent = Ratio::zero();
    req.actors[0].static_stats.def_flat = Ratio::zero();
    assert_eq!(
        run(&req)["incoming_damage_receipts"][0]["delivered_damage"],
        1
    ); // .5 ->1.
    req.targets[0].level = 1;
    assert_eq!(
        run(&req)["incoming_damage_receipts"][0]["delivered_damage"],
        0
    ); //606/1686 <.5.
    req.incoming_damage_events[0].delivered_damage = 0;
    assert_eq!(
        run(&req)["incoming_damage_receipts"][0]["delivered_damage"],
        0
    );
}
#[test]
fn incoming_defense_omitted_and_explicit_external_basis_keep_the_declared_final_amount() {
    for explicit in [false, true] {
        let mut req = request(1000);
        req.incoming_dummy_sources.clear();
        req.incoming_damage_basis.clear();
        if explicit {
            req.incoming_damage_basis
                .insert(0, GameIncomingDamageBasis::ExternallyMitigated);
        }
        let r = run(&req);
        let receipt = &r["incoming_damage_receipts"][0];
        assert_eq!(receipt["delivered_damage"], 1000);
        assert_eq!(receipt["actual_hp_loss"], 1000);
        assert!(receipt["defense_mitigation"].is_null());
    }
}
#[test]
fn incoming_defense_preflight_requires_model_source_positive_level_and_valid_indices() {
    let req = request(1000);
    let mut bad = req.clone();
    bad.model_assumptions
        .remove(crate::HypothesisId::CombatIncomingDefenseV1.as_str());
    assert!(
        validate_game_request(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("incoming_defense_v1")
    );
    let mut bad = req.clone();
    bad.incoming_dummy_sources.clear();
    assert!(
        validate_game_request(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("explicit dummy source")
    );
    let mut bad = req.clone();
    bad.targets[0].level = 0;
    assert!(
        validate_game_request(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("positive declared level")
    );
    let mut bad = req;
    bad.incoming_damage_basis
        .insert(1, GameIncomingDamageBasis::BeforeDefense);
    assert!(
        validate_game_request(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("basis event index")
    );
}
