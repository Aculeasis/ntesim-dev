use super::*;
use crate::characters::haniel::stats;
use crate::runtime_api::CallbackContext;
use crate::simulation::state::GameSimulatorState;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use serde_json::json;

pub(super) fn request() -> GameScenarioRequest {
    let mut req =
        super::reactions::request(&[(0, "character:1004", 100, 0), (1, "character:1020", 100, 0)]);
    req.model_assumptions
        .insert(stats::MODEL.into(), stats::MODEL.description().into());
    req.actors[1].character_level = 80;
    req.actors[1].character_max_level = Some(80);
    req.actors[1].static_stats.base_atk = 1000;
    // Deliberately different from base: extraction must not read final ATK.
    req.actors[1].static_stats.atk = Ratio {
        numerator: 2400,
        denominator: 1,
    };
    req.targets[0].original_atk = Some(1000);
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].resistance = Ratio::zero();
    req.actions.clear();
    req.relative_duration_us = 12_000_000;
    req
}

fn f(n: u128, d: u128) -> ExactFraction {
    ExactFraction::new(n, d).unwrap()
}

#[test]
fn haniel_nova_base_capture_source_cap_and_rollback_are_exact_and_not_combat_loss() {
    let req = request();
    let mut state = GameSimulatorState::new(&req);
    let mut queue = BinaryHeap::new();
    let mut sequence = 0;
    let mut trace = vec![];
    let mut ctx = CallbackContext {
        request: &req,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    stats::enter(&mut ctx, 1).unwrap();
    stats::enter(&mut ctx, 1).unwrap();
    for generation in 0..3 {
        stats::nova_end(&mut ctx, 1, 0, generation).unwrap();
        assert_eq!(
            ctx.state.targets[0].attack,
            Some(f(if generation == 0 { 960 } else { 920 }, 1))
        );
    }
    let buff = ctx
        .state
        .buffs
        .instances()
        .iter()
        .find(|b| b.id == "haniel.friendship_atk")
        .unwrap();
    assert_eq!(
        buff.stat_modifiers["atk_add"],
        Ratio {
            numerator: 160,
            denominator: 1
        }
    );
    assert_eq!(ctx.state.atk_transfers.len(), 2);
    assert_eq!(ctx.state.atk_transfers[0].party_flat_atk_after, f(80, 1));
    assert_eq!(ctx.state.atk_transfers[1].party_flat_atk_after, f(160, 1));
    assert_eq!(ctx.state.hits_settled, 0);
    assert_eq!(ctx.state.rounded_damage_total, 0);
    assert!(ctx.state.structural_hp_changes.is_empty());
    assert!(ctx.queue.is_empty());
    stats::leave(&mut ctx, 1).unwrap();
    stats::leave(&mut ctx, 1).unwrap();
    assert_eq!(ctx.state.targets[0].attack, Some(f(1000, 1)));
    assert!(ctx.state.buffs.instances().is_empty());
    assert_eq!(ctx.state.atk_transfers.len(), 3);
    assert_eq!(ctx.state.atk_transfers[2].party_flat_atk_after, f(0, 1));
    for (runtime, config) in ctx.state.actors.iter().zip(&req.actors) {
        assert_eq!(runtime.hp, config.initial_hp);
        assert_eq!(runtime.energy.current(), config.initial_energy);
        assert_eq!(runtime.cycle.current(), config.initial_cycle);
    }
}

#[test]
fn haniel_nova_target_cap_is_twenty_percent_and_source_cap_does_not_reset_on_target_change() {
    let mut req = request();
    req.targets[0].original_atk = Some(101);
    let mut other = req.targets[0].clone();
    other.target_index = 1;
    other.original_atk = Some(1000);
    req.targets.push(other);
    let mut state = GameSimulatorState::new(&req);
    let mut queue = BinaryHeap::new();
    let mut sequence = 0;
    let mut trace = vec![];
    let mut ctx = CallbackContext {
        request: &req,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    stats::enter(&mut ctx, 1).unwrap();
    stats::nova_end(&mut ctx, 1, 0, 0).unwrap();
    stats::nova_end(&mut ctx, 1, 0, 1).unwrap();
    // Initial101 permits loss20.2 and per-ally gain40.4, without integer rounding.
    assert_eq!(ctx.state.targets[0].attack, Some(f(404, 5)));
    assert_eq!(ctx.state.atk_transfers.len(), 1);
    assert_eq!(ctx.state.atk_transfers[0].party_flat_atk_after, f(202, 5));
    stats::nova_end(&mut ctx, 1, 1, 2).unwrap();
    stats::nova_end(&mut ctx, 1, 1, 3).unwrap();
    assert_eq!(ctx.state.atk_transfers[2].party_flat_atk_after, f(160, 1));
    assert_eq!(ctx.state.targets[1].attack, Some(f(4701, 5)));
    stats::leave(&mut ctx, 1).unwrap();
    assert_eq!(ctx.state.targets[0].attack, Some(f(101, 1)));
    assert_eq!(ctx.state.targets[1].attack, Some(f(1000, 1)));
    assert_eq!(
        ctx.state.atk_transfers.last().unwrap().party_flat_atk_after,
        f(0, 1)
    );
}

#[test]
fn haniel_nova_refresh_and_expiry_are_once_per_generation_and_survive_source_death() {
    let mut req = request();
    let incoming = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(incoming.into(), incoming.description().into());
    req.actions = vec![
        super::reactions::switch(0, 1),
        super::reactions::switch(5_050_000, 0),
    ];
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 5_060_000,
        actor_index: 1,
        delivered_damage: 1_000_000,
    });
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        r["execution_completed"], true,
        "{} {}",
        r["halt_diagnostic"], r["rejected_action"]
    );
    assert_eq!(r["final_actor_states"][1]["hp"], 0);
    assert_eq!(
        r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.nova")
            .count(),
        2
    );
    let rows = r["atk_transfers"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["time_us"], 5_050_000);
    assert_eq!(rows[1]["time_us"], 10_150_000);
    assert_ne!(rows[0]["nova_generation"], rows[1]["nova_generation"]);
    assert_eq!(rows[1]["party_flat_atk_after"], json!("160/1"));
    assert_eq!(r["final_target_states"][0]["attack"], json!("920/1"));
    assert!(
        crate::simulation::tests::support::final_buffs(&r)
            .iter()
            .any(|b| b["id"] == "haniel.friendship_atk")
    );
}

#[test]
fn haniel_nova_combat_exit_rolls_back_and_reentry_opens_a_fresh_membership() {
    let mut req = request();
    req.actions = vec![
        super::reactions::switch(0, 1),
        super::reactions::switch(7_000_000, 0),
    ];
    req.relative_duration_us = 13_000_000;
    req.combat_transition_events = [(6_000_000, true), (6_500_000, false)]
        .into_iter()
        .map(|(timestamp_us, leaving)| GameActorCombatTransitionEvent {
            timestamp_us,
            actor_index: 1,
            leaving,
        })
        .collect();
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        r["execution_completed"], true,
        "{} {}",
        r["halt_diagnostic"], r["rejected_action"]
    );
    let rows = r["atk_transfers"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1]["cause"], "combat_exit");
    assert_eq!(rows[1]["target_atk_after"], json!("1000/1"));
    assert_eq!(rows[2]["party_flat_atk_after"], json!("80/1"));
    assert_eq!(r["final_target_states"][0]["attack"], json!("960/1"));
}

#[test]
fn haniel_nova_missing_attack_rejects_before_switch_spend_but_explicit_zero_is_valid() {
    for target in 0..2 {
        let mut req = request();
        req.targets.push(req.targets[0].clone());
        req.targets[1].target_index = 1;
        req.aim_targets.insert(0, target);
        req.actions = vec![super::reactions::switch(0, 1)];
        req.targets[target].original_atk = None;
        let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
        assert!(error.contains("original_atk"), "{error}");
        req.targets[target].original_atk = Some(0);
        req.targets[1 - target].original_atk = None;
        let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            r["execution_completed"], true,
            "{} {}",
            r["halt_diagnostic"], r["rejected_action"]
        );
        assert_eq!(r["hits_settled"], 1);
        assert!(r["atk_transfers"].is_null());
        assert_eq!(r["final_target_states"][target]["attack"], json!("0/1"));
    }
}

#[test]
fn haniel_nova_dead_target_and_locked_breakthrough_have_no_extraction() {
    let mut req = request();
    req.actions = vec![super::reactions::switch(0, 1)];
    req.targets[0].hp_current = 1;
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(r["atk_transfers"].is_null());
    assert_eq!(r["final_target_states"][0]["attack"], json!("1000/1"));
    req.targets[0].hp_current = 100_000_000;
    req.actors[1].character_level = 1;
    req.actors[1].purchased_qte_level = Some(1);
    req.actors[1].character_max_level = Some(20);
    req.targets[0].original_atk = None;
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        r["execution_completed"], true,
        "{} {}",
        r["halt_diagnostic"], r["rejected_action"]
    );
    assert!(r["atk_transfers"].is_null());
}

#[test]
fn haniel_nova_flat_grant_changes_live_damage_without_reapplying_static_atk_percent() {
    let mut req = request();
    req.actors[1].static_stats.elemental_bonus = Ratio::zero();
    req.actors[1].talent_levels = Some([1; 3]);
    req.actions = vec![
        super::reactions::switch(0, 1),
        super::haniel_actions::make_action(
            6_000_000,
            1,
            "GA_Haniel_Melee",
            "GE_Player_Haniel_Melee1_Damage",
            0,
            0,
        ),
    ];
    let r = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        r["execution_completed"], true,
        "{} {}",
        r["halt_diagnostic"], r["rejected_action"]
    );
    let hit = &r["hit_receipts"][1];
    // Static2400 + captured base1000*.08 =2480; .419 coefficient, half DEF, zero RES.
    assert_eq!(hit["non_crit_damage"], json!("12989/25"));
}

#[test]
fn haniel_nova_multiple_sources_share_target_loss_limit_and_rollback_only_their_own_gain() {
    let mut req = request();
    let mut second = req.actors[1].clone();
    second.slot = 2;
    req.actors.push(second);
    req.targets[0].original_atk = Some(300); // Target cap60, less than two full40 drains.
    let mut state = GameSimulatorState::new(&req);
    let mut queue = BinaryHeap::new();
    let mut sequence = 0;
    let mut trace = vec![];
    let mut ctx = CallbackContext {
        request: &req,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
    };
    for owner in [1, 2] {
        stats::enter(&mut ctx, owner).unwrap();
    }
    stats::nova_end(&mut ctx, 1, 0, 0).unwrap();
    stats::nova_end(&mut ctx, 2, 0, 0).unwrap();
    assert_eq!(ctx.state.targets[0].attack, Some(f(240, 1)));
    assert_eq!(ctx.state.atk_transfers[1].party_flat_atk_after, f(40, 1));
    stats::leave(&mut ctx, 1).unwrap();
    assert_eq!(ctx.state.targets[0].attack, Some(f(280, 1)));
    let buffs = ctx.state.buffs.instances();
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs[0].owner_actor, 2);
    assert_eq!(
        buffs[0].stat_modifiers["atk_add"],
        Ratio {
            numerator: 40,
            denominator: 1
        }
    );
    stats::leave(&mut ctx, 2).unwrap();
    assert_eq!(ctx.state.targets[0].attack, Some(f(300, 1)));
}
