use super::daffodil_actions::{make_action, request};
use super::*;
use crate::characters::daffodil::insight;
use crate::simulation::{
    break_damage::{self, BreakPacket},
    state::GameSimulatorState,
};
use serde_json::json;

fn setup(effects: &[u8]) -> GameScenarioRequest {
    let mut req = request();
    req.actors[0].awakening =
        ntesim_data::AwakeningSelection::selected(effects.iter().copied()).unwrap();
    req.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 30,
            denominator: 1,
        },
        current_gauge: Ratio::one(),
        broken_duration_us: 1000,
        discord_fraction: None,
    });
    req
}

fn snapshot(
    req: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    target: usize,
) -> Vec<BreakPacket> {
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    break_damage::schedule(req, state, &mut queue, &mut 0, 0, target).unwrap();
    std::iter::from_fn(|| queue.pop())
        .map(|event| match event.data {
            crate::simulation::events::GameEventData::TargetBreakDamage(packet) => *packet,
            _ => unreachable!(),
        })
        .collect()
}

#[test]
fn daffodil_insight_a4_a5_capture_exact_target_bonus_and_own_repeats_only() {
    // Lv80 base3603, original cap30, DEF1/2, RES0. No ordinary ATK/crit zones.
    for (effects, marks, expected, repeats) in [
        (vec![4, 5], 0, (21618, 1), 1),
        (vec![4, 5], 1, (97281, 4), 2),
        (vec![3, 4, 5], 2, (54045, 2), 3),
        (vec![3, 5], 2, (21618, 1), 3),
        (vec![3, 4], 2, (54045, 2), 1),
    ] {
        let mut req = setup(&effects);
        let mut ally = req.actors[0].clone();
        ally.slot = 1;
        ally.character_id = "character:1019".into();
        ally.awakening = Default::default();
        req.actors.push(ally);
        let mut state = GameSimulatorState::new(&req);
        state.active_actor_index = 1;
        for _ in 0..marks {
            insight::apply_target(&mut state.buffs, 0, 0, effects.contains(&3), 0).unwrap();
        }
        let packets = snapshot(&req, &mut state, 0);
        assert_eq!(
            packets.iter().map(|p| p.actor_index).collect::<Vec<_>>(),
            std::iter::once(0)
                .chain(std::iter::once(1))
                .chain(std::iter::repeat_n(0, repeats - 1))
                .collect::<Vec<_>>()
        );
        let own: Vec<_> = packets.iter().filter(|p| p.actor_index == 0).collect();
        assert_eq!(own.len(), repeats);
        assert!(
            own.iter()
                .all(|p| p.damage == ExactFraction::new(expected.0, expected.1).unwrap())
        );
        assert_eq!(packets[1].damage, ExactFraction::new(18015, 1).unwrap());
        assert_eq!(
            own.iter()
                .try_fold(ExactFraction::new(0, 1).unwrap(), |sum, p| sum
                    .add(p.damage))
                .unwrap(),
            ExactFraction::new(expected.0 * repeats as u128, expected.1).unwrap()
        );
    }
}

#[test]
fn daffodil_insight_is_per_source_target_and_expiry_not_self_eyes() {
    let mut req = setup(&[3, 4, 5]);
    let mut other_source = req.actors[0].clone();
    other_source.slot = 1;
    other_source.awakening = Default::default();
    req.actors.push(other_source);
    let mut second = req.targets[0].clone();
    second.target_index = 1;
    req.targets.push(second);
    let mut state = GameSimulatorState::new(&req);
    // Two foreign marks on target0, two own marks on target1: no bonus leaks.
    for _ in 0..2 {
        insight::apply_target(&mut state.buffs, 1, 0, true, 0).unwrap();
        insight::apply_target(&mut state.buffs, 0, 1, true, 0).unwrap();
    }
    assert_eq!(
        snapshot(&req, &mut state, 0)
            .iter()
            .filter(|p| p.actor_index == 0)
            .count(),
        1
    );
    let captured: Vec<_> = snapshot(&req, &mut state, 1)
        .into_iter()
        .filter(|p| p.actor_index == 0)
        .collect();
    assert_eq!(captured.len(), 3);
    assert_eq!(captured[0].damage, ExactFraction::new(54045, 2).unwrap());
    // Expiry is half-open; captured contributions keep their original values.
    state.advance_unpaused_test_clock(9_999_000_000);
    assert_eq!(
        snapshot(&req, &mut state, 1)
            .iter()
            .filter(|p| p.actor_index == 0)
            .count(),
        1
    );
    assert_eq!(captured[2].damage, ExactFraction::new(54045, 2).unwrap());
    state.actors[0].hp = 0;
    assert!(
        snapshot(&req, &mut state, 1)
            .iter()
            .all(|p| p.actor_index != 0)
    );
}

#[test]
fn daffodil_a3_does_not_extend_self_eyes_to_target_insight_lifetime() {
    for (time, expected) in [(35_098_905, (1837, 20)), (35_098_906, (167, 2))] {
        let mut req = request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([3]).unwrap();
        req.relative_duration_us = 31_000_000;
        req.actions = vec![
            make_action(
                0,
                0,
                "GA_Daffodill_UltraSkill",
                "GE_Player_Daffodill_UltraSkill1_Damage",
                120,
                20_000_000,
            ),
            make_action(
                time,
                0,
                "GA_Daffodill_Melee",
                "GE_Player_Daffodill_Melee1_Damage",
                0,
                0,
            ),
        ];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let normal = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["command_index"] == 1)
            .unwrap();
        assert_eq!(
            normal["non_crit_damage"],
            json!(format!("{}/{}", expected.0, expected.1))
        );
    }
}

#[test]
fn daffodil_insight_refresh_caps_stacks_and_refreshes_one_shared_expiry() {
    for (effects, awakened, duration, count) in [
        (vec![4, 5], false, 45_000_000, 2),
        (vec![3, 4, 5], true, 9_999_000_000, 3),
    ] {
        let req = setup(&effects);
        let mut state = GameSimulatorState::new(&req);
        for time in [0, 10, 20] {
            insight::apply_target(&mut state.buffs, 0, 0, awakened, time).unwrap();
        }
        state.advance_unpaused_test_clock(duration + 19);
        assert_eq!(snapshot(&req, &mut state, 0).len(), count);
        state.advance_unpaused_test_clock(duration + 20);
        assert_eq!(snapshot(&req, &mut state, 0).len(), 1);
    }
}

#[test]
fn daffodil_insight_clears_on_direct_or_break_defeat_and_pending_repeats_cancel() {
    for hp in [1, 10_000] {
        let mut req = setup(&[3, 4, 5]);
        req.targets[0].hp_current = hp;
        req.targets[0].break_profile.as_mut().unwrap().current_gauge = Ratio {
            numerator: 3,
            denominator: 1,
        };
        req.relative_duration_us = 5_000_000;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Daffodill_UltraSkill",
            "GE_Player_Daffodill_UltraSkill1_Damage",
            120,
            20_000_000,
        )];
        for second in 1..5 {
            req.actions.push(make_action(
                second * 1_000_000,
                0,
                "GA_Daffodill_Melee",
                "GE_Player_Daffodill_Melee1_Damage",
                0,
                0,
            ));
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["final_target_states"][0]["hp_current"], 0);
        assert!(
            report["final_target_states"][0]["statuses"]
                .as_array()
                .is_none_or(|statuses| {
                    statuses.iter().all(|status| {
                        !status["id"]
                            .as_str()
                            .unwrap()
                            .starts_with("ntesim.daffodil.insight")
                    })
                })
        );
        let breaks = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit.get("break_contribution").is_some())
            .count();
        assert_eq!(breaks, if hp == 1 { 0 } else { 1 });
    }
}

#[test]
fn daffodil_insight_strict_execution_requires_the_reached_owner_model() {
    let mut req = setup(&[4, 5]);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_UltraSkill",
        "GE_Player_Daffodill_UltraSkill1_Damage",
        120,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    req.model_assumptions =
        serde_json::from_value(report["resolved_model_assumptions"].clone()).unwrap();
    req.model_assumptions.insert(
        insight::HYPOTHESIS.into(),
        insight::HYPOTHESIS.description().into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
}
