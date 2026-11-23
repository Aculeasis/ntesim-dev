use super::*;
use crate::runtime_api::CallbackContext;
use crate::simulation::state::GameSimulatorState;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;

fn request() -> GameScenarioRequest {
    let mut req = super::fadia_clock::request(1);
    req.actions.clear();
    let model = crate::characters::fadia::effects::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actors[0].static_stats.base_hp = Some(10000);
    req.actors[0].static_stats.hp_percent = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.actors[0].static_stats.hp_flat = Ratio {
        numerator: 500,
        denominator: 1,
    };
    req.actors[0].static_stats.hp = 12500;
    req.actors[0].initial_hp_max = 12500;
    req.actors[0].initial_hp = 12500;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    req.actors.push(ally);
    req.targets[0].hp_max = 200000;
    req.targets[0].hp_current = 200000;
    req
}
#[test]
fn fadia_nova_exact_source_cap_literal_twenty_and_rollback_do_not_heal_or_damage() {
    let req = request();
    let mut state = GameSimulatorState::new(&req);
    let mut queue = BinaryHeap::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut ctx = CallbackContext {
        request: &req,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    use crate::characters::fadia::stats;
    stats::enter(&mut ctx, 0).unwrap();
    stats::enter(&mut ctx, 0).unwrap(); // Membership is idempotent.
    assert_eq!(ctx.state.actors[0].hp_max, 13500);
    assert_eq!(ctx.state.actors[1].hp_max, 13500);
    assert_eq!(ctx.state.actors[0].hp, 12500);
    for index in 1..=6 {
        stats::nova_end(&mut ctx, 0, 0).unwrap();
        let grants = index.min(5);
        assert_eq!(ctx.state.targets[0].hp_max, 200000 - 20000 * grants);
        assert_eq!(ctx.state.actors[0].hp_max, 13500 + 1000 * grants);
        assert_eq!(ctx.state.actors[1].hp_max, 13500 + 1000 * grants);
        assert_eq!(ctx.state.actors[1].hp, 12500);
    }
    assert_eq!(ctx.state.rounded_damage_total, 0);
    assert_eq!(ctx.state.hits_settled, 0);
    assert_eq!(ctx.state.actors[0].energy.current(), 0);
    stats::leave(&mut ctx, 0).unwrap();
    assert_eq!(ctx.state.targets[0].hp_max, 200000);
    assert_eq!(ctx.state.targets[0].hp_current, 100000); // Rollback does not restore lost current HP.
    for actor in &ctx.state.actors {
        assert_eq!(actor.hp_max, 12500);
        assert_eq!(actor.hp, 12500);
    }
    assert_eq!(
        ctx.state
            .structural_hp_changes
            .iter()
            .filter(|r| matches!(r.recipient, GameHpStatRecipient::Target(_)))
            .count(),
        6
    );
}
#[test]
fn fadia_small_target_limits_grant_before_rounding_and_death_keeps_membership_listener() {
    let mut req = request();
    req.targets[0].hp_max = 5001;
    req.targets[0].hp_current = 5001;
    let mut state = GameSimulatorState::new(&req);
    let mut queue = BinaryHeap::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut ctx = CallbackContext {
        request: &req,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    use crate::characters::fadia::stats;
    stats::enter(&mut ctx, 0).unwrap();
    ctx.state.actors[0].hp = 0;
    stats::nova_end(&mut ctx, 0, 0).unwrap();
    stats::nova_end(&mut ctx, 0, 0).unwrap();
    assert_eq!(ctx.state.targets[0].hp_max, 1);
    assert_eq!(ctx.state.targets[0].hp_current, 1);
    assert_eq!(ctx.state.actors[0].hp_max, 13750);
    assert_eq!(ctx.state.actors[0].hp, 0);
    assert_eq!(ctx.state.actors[1].hp_max, 13750);
    stats::leave(&mut ctx, 0).unwrap();
    assert_eq!(ctx.state.actors[0].hp, 0);
    assert_eq!(ctx.state.actors[1].hp, 12500);
}
#[test]
fn fadia_selected_a3_changes_base_before_percent_keeps_flat_and_reenters_once() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([3]).unwrap();
    req.combat_transition_events = [(1000000, true), (2000000, false)]
        .into_iter()
        .map(|(timestamp_us, leaving)| GameActorCombatTransitionEvent {
            timestamp_us,
            actor_index: 0,
            leaving,
        })
        .collect();
    let r = super::edgar_full::run(&req);
    assert_eq!(r["final_actor_states"][0]["hp_max"], 17400);
    assert_eq!(r["final_actor_states"][0]["hp"], 12500);
    assert_eq!(r["final_actor_states"][1]["hp_max"], 13500);
    assert_eq!(r["hits_settled"], 0);
    let changes = r["structural_hp_changes"].as_array().unwrap();
    assert_eq!(
        changes.iter().filter(|c| c["cause"] == "base_hp").count(),
        2
    );
}

#[test]
fn fadia_a6_target_selection_is_separate_from_execution_work_budget() {
    let mut req = super::fadia_clock::request(12);
    req.actions.clear();
    let limits = ExecutionLimits {
        max_events: 11,
        ..Default::default()
    };
    let error = run_game_scenario(&req, &limits).unwrap_err();
    assert!(
        error.contains("target count 12 exceeds execution work limit 11"),
        "{error}"
    );
}
