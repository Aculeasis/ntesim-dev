use super::daffodil_actions::{make_action, request};
use super::*;
use crate::characters::daffodil::broken;
use crate::simulation::{
    break_state,
    state::{GameSimulatorState, target_resistance_reduction},
};
use serde_json::json;

const CHAOS: &str = "ECharacterElementType::CHARACTER_ELEMENT_TYPE_CHAOS";

pub(super) fn scenario() -> GameScenarioRequest {
    let mut req = request();
    req.relative_duration_us = 5000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].talent_levels = Some([8; 3]);
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.targets[0].elemental_resistances.clear();
    req.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 30,
            denominator: 1,
        },
        current_gauge: Ratio {
            numerator: 1,
            denominator: 1000,
        },
        broken_duration_us: 1000,
        discord_fraction: None,
    });
    for (slot, id) in [(1, "character:1004"), (2, "character:1019")] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = id.into();
        ally.awakening = Default::default();
        ally.talent_levels = Some([1; 3]);
        req.actors.push(ally);
    }
    req
}

fn normal(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Melee1_Damage",
        0,
        0,
    )
}

fn swap(time: u64, actor: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us: time,
        target_actor_index: actor,
        cooldown_us: 0,
    }
}

#[test]
fn daffodil_r6_off_field_aura_is_post_trigger_target_scoped_and_expires_at_recovery() {
    let mut req = scenario();
    req.actions = vec![
        normal(0),
        swap(1, 1),
        swap(200, 0),
        normal(300),
        swap(1200, 1),
        swap(1300, 0),
        normal(2000),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    let hits = report["hit_receipts"].as_array().unwrap();
    // Source A6 DEF factor20/37, talent9 first Normal .309.
    // .8 RES factor on causal/recovered hit; .95 on the live Broken hit.
    for (command, numerator) in [(0, 4944), (3, 5871), (6, 4944)] {
        let hit = hits
            .iter()
            .find(|hit| hit["command_index"] == command && hit.get("break_contribution").is_none())
            .unwrap();
        assert_eq!(
            hit["non_crit_damage"],
            json!(format!("{}/{}", numerator, 37))
        );
    }
    let contributions: Vec<_> = hits
        .iter()
        .filter(|hit| hit.get("break_contribution").is_some())
        .collect();
    assert_eq!(contributions.len(), 3);
    // Own Light Prism/A6, a different Chaos owner (Lacrimosa), and Nature (Mint).
    for (slot, numerator, denominator) in [(0, 821484, 37), (1, 68457, 4), (2, 14412, 1)] {
        let hit = contributions
            .iter()
            .find(|hit| hit["actor_index"] == slot)
            .unwrap();
        assert_eq!(
            hit["damage"],
            json!(format!("{}/{}", numerator, denominator))
        );
    }
    let kinds: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["event_kind"].as_str().unwrap())
        .collect();
    let position = |kind| kinds.iter().position(|value| *value == kind).unwrap();
    assert!(position("target_broken") < position("daffodil_broken_chaos_applied"));
    assert!(position("daffodil_broken_chaos_applied") < position("break_damage"));
    assert!(
        report["final_target_states"][0]["statuses"]
            .as_array()
            .is_none_or(|statuses| statuses
                .iter()
                .all(|status| !status["id"].as_str().unwrap().contains("broken_chaos")))
    );
}

#[test]
fn daffodil_r6_existing_snapshot_follows_rose_extension_after_source_death() {
    let mut req = scenario();
    let mut untouched = req.targets[0].clone();
    untouched.target_index = 1;
    req.targets.push(untouched);
    req.relative_duration_us = 4_000_000;
    let mut state = GameSimulatorState::new(&req);
    let mut queue = Default::default();
    let mut sequence = 0;
    let mut trace = Vec::new();
    assert!(
        break_state::apply_unbalance(
            &req,
            &mut state,
            &mut queue,
            &mut sequence,
            &mut trace,
            1,
            0,
            ExactFraction::new(1, 1).unwrap()
        )
        .unwrap()
    );
    assert_eq!(
        target_resistance_reduction(&state.buffs, 1, CHAOS).unwrap(),
        ExactFraction::new(0, 1).unwrap()
    );
    state.advance_unpaused_test_clock(200);
    state.actors[0].hp = 0;
    assert!(
        break_state::extend_broken_for_rose(
            &req,
            &mut state,
            &mut queue,
            &mut sequence,
            &mut trace,
            0,
            3_000_000
        )
        .unwrap()
    );
    state.advance_unpaused_test_clock(1000);
    state.buffs.advance_time(1000);
    break_state::recover(&mut state, &mut trace, 0, 1).unwrap();
    assert_eq!(
        target_resistance_reduction(&state.buffs, 0, CHAOS).unwrap(),
        ExactFraction::new(3, 20).unwrap()
    );
    assert_eq!(
        state.targets[0]
            .break_state
            .as_ref()
            .unwrap()
            .broken_until_us,
        Some(3_001_000)
    );
    state.advance_unpaused_test_clock(3_001_000);
    state.buffs.advance_time(state.current_time_us);
    break_state::recover(&mut state, &mut trace, 0, 1).unwrap();
    assert_eq!(
        target_resistance_reduction(&state.buffs, 0, CHAOS).unwrap(),
        ExactFraction::new(0, 1).unwrap()
    );
    assert_eq!(
        state.targets[0]
            .break_state
            .as_ref()
            .unwrap()
            .broken_until_us,
        None
    );
}

#[test]
fn daffodil_r6_is_removed_by_direct_and_break_defeat_before_its_deadline() {
    for (hp, expected_break_hits) in [(30_000, 2), (54_000, 3)] {
        let mut req = scenario();
        req.relative_duration_us = 500;
        req.targets[0].hp_current = hp;
        req.actions = vec![normal(0), swap(1, 1), swap(200, 0), normal(300)];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["final_target_states"][0]["hp_current"], 0);
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit.get("break_contribution").is_some())
                .count(),
            expected_break_hits
        );
        assert!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["event_kind"] == "daffodil_broken_chaos_applied")
        );
        assert!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .all(|event| event["event_kind"] != "target_break_recovered")
        );
        assert!(
            report["final_target_states"][0]["statuses"]
                .as_array()
                .is_none_or(|statuses| statuses
                    .iter()
                    .all(|status| !status["id"].as_str().unwrap().contains("broken_chaos")))
        );
    }
}

#[test]
fn daffodil_r6_has_no_fresh_dead_provider_no_bar_or_zero_efficiency_fallback() {
    for case in 0..3 {
        let mut req = scenario();
        if case == 1 {
            req.targets[0].break_profile = None;
        }
        if case == 2 {
            req.actors[1].static_stats.break_efficiency_multiplier = Ratio::zero();
        }
        let mut state = GameSimulatorState::new(&req);
        if case == 0 {
            state.actors[0].hp = 0;
        }
        let transitioned = break_state::apply_unbalance(
            &req,
            &mut state,
            &mut Default::default(),
            &mut 0,
            &mut Vec::new(),
            1,
            0,
            ExactFraction::new(1, 1).unwrap(),
        )
        .unwrap();
        assert_eq!(transitioned, case == 0);
        assert_eq!(
            target_resistance_reduction(&state.buffs, 0, CHAOS).unwrap(),
            ExactFraction::new(0, 1).unwrap()
        );
        assert!(
            !state
                .execution_model_assumptions
                .contains_key(broken::HYPOTHESIS.as_str())
        );
    }
}

#[test]
fn daffodil_r6_strict_execution_requires_its_reached_target_model() {
    let mut req = scenario();
    for actor in &mut req.actors {
        actor.talent_levels = None;
        actor.purchased_skill_level = 8;
    }
    req.actions = vec![normal(0)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    req.model_assumptions =
        serde_json::from_value(report["resolved_model_assumptions"].clone()).unwrap();
    req.model_assumptions.insert(
        broken::HYPOTHESIS.into(),
        broken::HYPOTHESIS.description().into(),
    );
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()["execution_completed"],
        true
    );
}
