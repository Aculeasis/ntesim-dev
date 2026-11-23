use super::*;
use serde_json::Value;
fn request() -> GameScenarioRequest {
    let mut req = super::edgar_full::request(1);
    req.actors[0].character_id = "character:1039".into();
    req.actors[0].initial_hp = 100;
    req.active_actor_index = 1;
    req.actions.clear();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.initial_hp = 10000;
    req.actors.push(ally);
    for model in [
        crate::characters::fadia::incoming::MODEL,
        crate::HypothesisId::CombatIncomingDamageFixtureV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req
}
fn event(time: u64, actor_index: usize, delivered_damage: u64) -> GameActorDamageEvent {
    GameActorDamageEvent {
        timestamp_us: time,
        actor_index,
        delivered_damage,
    }
}
fn accounted(receipt: &Value) -> u64 {
    let mut total = receipt["actual_hp_loss"].as_u64().unwrap()
        + receipt["overflow"].as_u64().unwrap()
        + receipt["prevented_hp_loss"].as_u64().unwrap_or(0);
    for key in ["absorbed_by_shields", "absorbed_by_effects"] {
        if let Some(entries) = receipt[key].as_array() {
            total += entries
                .iter()
                .map(|e| e["amount"].as_u64().unwrap())
                .sum::<u64>();
        }
    }
    if let Some(children) = receipt["redirected"].as_array() {
        for child in children {
            assert!(child["redirected"].is_null());
            assert_eq!(child["event_index"], receipt["event_index"]);
            total += accounted(child);
        }
    }
    assert_eq!(total, receipt["delivered_damage"].as_u64().unwrap());
    total
}
#[test]
fn fadia_intercepts_before_shields_and_preserves_one_hp_without_a_reverse_pass_mid_hit() {
    let mut req = request();
    let mut ally = req.actors[1].clone();
    ally.slot = 2;
    req.actors.push(ally);
    req.incoming_damage_events = vec![event(0, 1, 1000), event(1_000_000, 0, 1000)];
    let r = super::edgar_full::run(&req);
    let first = &r["incoming_damage_receipts"][0];
    accounted(first);
    assert_eq!(first["actual_hp_loss"], 500);
    assert_eq!(first["redirected"][0]["actual_hp_loss"], 99);
    assert_eq!(first["redirected"][0]["prevented_hp_loss"], 401);
    let second = &r["incoming_damage_receipts"][1];
    accounted(second);
    assert_eq!(second["actual_hp_loss"], 0);
    assert_eq!(second["prevented_hp_loss"], 500);
    assert_eq!(second["redirected"][0]["actual_hp_loss"], 250);
    assert_eq!(second["redirected"][1]["actual_hp_loss"], 250);
    assert_eq!(r["final_actor_states"][0]["hp"], 1);
    assert_eq!(r["final_actor_states"][1]["hp"], 9250);
    assert_eq!(r["final_actor_states"][2]["hp"], 9750);
}

#[test]
fn incoming_defense_precedes_fadia_shares_and_never_mitigates_children_again() {
    let mut req = request();
    for (actor, def) in req.actors.iter_mut().zip([108000, 1080]) {
        actor.static_stats.base_def = Some(def);
        actor.static_stats.def = def;
        actor.static_stats.def_percent = Ratio::zero();
        actor.static_stats.def_flat = Ratio::zero();
    }
    let model = crate::HypothesisId::CombatIncomingDefenseV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.incoming_damage_events = vec![event(0, 1, 1000)];
    req.incoming_dummy_sources.insert(0, 0);
    req.incoming_damage_basis
        .insert(0, GameIncomingDamageBasis::BeforeDefense);
    req.targets[0].level = 80;
    let r = super::edgar_full::run(&req);
    let receipt = &r["incoming_damage_receipts"][0];
    assert_eq!(accounted(receipt), 500);
    assert_eq!(receipt["actual_hp_loss"], 250);
    assert_eq!(receipt["redirected"][0]["delivered_damage"], 250);
    assert_eq!(receipt["redirected"][0]["actual_hp_loss"], 99);
    assert_eq!(receipt["redirected"][0]["prevented_hp_loss"], 151);
    assert!(receipt["redirected"][0]["defense_mitigation"].is_null());
    assert_eq!(r["final_actor_states"][1]["hp"], 9750);
}
#[test]
fn fadia_a1_replaces_the_total_share_and_odd_damage_preserves_the_original_pool() {
    for (a1, damage, original, redirected) in [
        (false, 1000, 500, 500),
        (true, 1000, 250, 750),
        (false, 1, 1, 0),
    ] {
        let mut req = request();
        req.actors[0].initial_hp = 1000;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(u32::from(a1));
        req.incoming_damage_events = vec![event(0, 1, damage)];
        let r = super::edgar_full::run(&req);
        let receipt = &r["incoming_damage_receipts"][0];
        accounted(receipt);
        assert_eq!(receipt["actual_hp_loss"], original);
        assert_eq!(receipt["redirected"][0]["actual_hp_loss"], redirected);
    }
}
#[test]
fn fadia_cannot_intercept_at_one_hp_and_dies_without_another_living_party_member() {
    let mut req = request();
    req.actors[0].initial_hp = 1;
    req.incoming_damage_events = vec![event(0, 1, 1000)];
    let r = super::edgar_full::run(&req);
    assert!(r["incoming_damage_receipts"][0]["redirected"].is_null());
    assert_eq!(r["final_actor_states"][1]["hp"], 9000);
    req.actors[1].initial_hp = 0;
    req.actors[0].initial_hp = 100;
    req.active_actor_index = 0;
    req.incoming_damage_events = vec![event(0, 0, 1000)];
    let r = super::edgar_full::run(&req);
    let receipt = &r["incoming_damage_receipts"][0];
    accounted(receipt);
    assert_eq!(receipt["actual_hp_loss"], 100);
    assert_eq!(receipt["overflow"], 900);
    assert_eq!(r["final_actor_states"][0]["hp"], 0);
}

#[test]
fn fadia_captures_survival_for_one_hit_and_rechecks_living_allies_on_the_next() {
    let mut req = request();
    req.actors[1].initial_hp = 100;
    req.incoming_damage_events = vec![event(0, 1, 1000), event(1_000_000, 0, 1000)];
    let r = super::edgar_full::run(&req);
    let first = &r["incoming_damage_receipts"][0];
    accounted(first);
    assert_eq!(first["overflow"], 400);
    assert_eq!(first["redirected"][0]["hp_after"], 1);
    let next = &r["incoming_damage_receipts"][1];
    accounted(next);
    assert_eq!(next["actual_hp_loss"], 1);
    assert_eq!(next["overflow"], 999);
    assert!(next["redirected"].is_null());
}

#[test]
fn fadia_does_not_redistribute_explicit_paid_hp_costs() {
    let mut req = request();
    let model = crate::HypothesisId::CombatHpCostFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.hp_cost_events = vec![GameActorHpCostEvent {
        timestamp_us: 0,
        actor_index: 1,
        source: "test.cost".into(),
        amount: 50,
    }];
    let r = super::edgar_full::run(&req);
    assert_eq!(r["final_actor_states"][0]["hp"], 100);
    assert_eq!(r["final_actor_states"][1]["hp"], 9950);
    assert_eq!(r["hp_cost_receipts"][0]["actual_hp_loss"], 50);
    assert!(r["incoming_damage_receipts"].is_null());
}
#[test]
fn fadia_children_each_apply_their_own_shield_once() {
    let req = request();
    let mut state = super::super::state::GameSimulatorState::new(&req);
    state.actors[0].hp = 1;
    state.actors[1]
        .shields
        .push(super::super::state::ActorShieldRuntime {
            id: "test.shield".into(),
            source: "test".into(),
            remaining_hp: 100,
            max_hp: 100,
            expires_at_us: 1_000_000,
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
    let receipt = super::super::incoming::settle(&mut ctx, 0, 0, 1000).unwrap();
    let receipt = serde_json::to_value(receipt).unwrap();
    accounted(&receipt);
    assert_eq!(receipt["prevented_hp_loss"], 500);
    assert_eq!(
        receipt["redirected"][0]["absorbed_by_shields"][0]["amount"],
        100
    );
    assert_eq!(receipt["redirected"][0]["actual_hp_loss"], 400);
}
#[test]
fn fadia_does_not_redirect_a_dodge_immune_original_hit() {
    let mut req = super::edgar_full::request(1);
    let mut fadia = request().actors.remove(0);
    fadia.slot = 1;
    req.actors.push(fadia);
    let model = crate::characters::fadia::incoming::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    super::blackbird_secondary::stamina(&mut req, 100);
    req.actions = vec![super::blackbird_secondary::dash(0, false)];
    req.incoming_damage_events = vec![event(100000, 0, 1000)];
    let r = super::edgar_full::run(&req);
    let receipt = &r["incoming_damage_receipts"][0];
    accounted(receipt);
    assert!(receipt["redirected"].is_null());
    assert_eq!(receipt["prevented_hp_loss"], 1000);
    assert_eq!(r["final_actor_states"][1]["hp"], 100);
}

#[test]
fn fadia_redirected_hp_observations_feed_edgar_actual_loss_instead_of_the_requested_share() {
    let mut req = request();
    let mut edgar = super::edgar_full::request(1).actors.remove(0);
    edgar.slot = 2;
    edgar.awakening = ntesim_data::AwakeningSelection::prefix(6);
    super::edgar_actions::set_a5_vitals(&mut edgar, 10000);
    req.actors.push(edgar);
    req.incoming_damage_events = vec![event(0, 1, 1000)];
    let r = super::edgar_full::run(&req);
    accounted(&r["incoming_damage_receipts"][0]);
    // Fadia loses99 and Mint500, both at10000MaxHP: (99+500)/10000/.03.
    assert_eq!(
        r["final_actor_states"][2]["fractional_resources"]["edgar.pending_charge"]["current"],
        serde_json::json!("599/300")
    );
}
