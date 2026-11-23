use super::nanally_actions::make_action;
use super::*;

fn request() -> GameScenarioRequest {
    let mut req = super::nanally_actions::request();
    req.relative_duration_us = 65_000_000;
    req.actors[0].initial_cycle = 0;
    req.actors[0].static_stats.crit_rate = Ratio::one();
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].defense = 1080;
    req.stochastic_crit = None;
    let model = crate::characters::nanally::combat::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actions = [0, 20, 45]
        .map(|s| {
            make_action(
                s * 1_000_000,
                0,
                "GA_Nanally_Skill",
                "GE_Player_Nanally_Skill1_Damage",
                0,
                16_000_000,
            )
        })
        .to_vec();
    req.actions.extend([14, 18, 38, 42, 59, 63].map(|s| {
        make_action(
            s * 1_000_000,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        )
    }));
    req.actions.sort_by_key(GameActionCommand::timestamp_us);
    req.combat_transition_events = vec![
        GameActorCombatTransitionEvent {
            timestamp_us: 5_000_000,
            actor_index: 0,
            leaving: true,
        },
        GameActorCombatTransitionEvent {
            timestamp_us: 25_000_000,
            actor_index: 0,
            leaving: false,
        },
    ];
    req
}

fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    let report = run_game_scenario_report(req, &ExecutionLimits::default()).unwrap();
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    report.validate_damage_parentage().unwrap();
    report
}

#[test]
fn nanally_combat_a6_captures_fifteen_twenty_fifteen_seconds_without_transition_refresh() {
    let req = request();
    let report = run(&req);
    let normals: Vec<_> = report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Nanally_Melee1_Damage")
        .collect();
    assert_eq!(normals.len(), 6);
    // Normal1=.296*1000*1/2=148; CRIT1.5 normally,1.8 during Authority.
    // Exit5 cannot extend the initial deadline15; entry25 cannot shorten deadline40.
    for (index, hit) in normals.iter().enumerate() {
        assert_eq!(
            hit.crit_damage,
            if index % 2 == 0 {
                ExactFraction::new(1332, 5).unwrap()
            } else {
                ExactFraction::new(222, 1).unwrap()
            }
        );
    }
    assert_eq!(report.combat_transition_receipts.len(), 2);
    let replay = serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(report).unwrap(),
        serde_json::to_value(run(&replay)).unwrap()
    );
}

#[test]
fn nanally_combat_without_a6_keeps_twelve_seconds_outside_combat() {
    let mut req = request();
    req.actors[0].awakening = Default::default();
    let report = run(&req);
    for hit in report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Nanally_Melee1_Damage")
    {
        // Every selected Normal falls beyond its12s application deadline.
        assert_eq!(hit.crit_damage, ExactFraction::new(222, 1).unwrap());
    }
}

#[test]
fn nanally_combat_exit_clears_a5_and_outside_hits_cannot_regrant_the_stack_pool() {
    let mut req = request();
    req.relative_duration_us = 11_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].initial_energy = 100;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        100,
        15_000_000,
    )];
    req.actions
        .extend([4_000_000, 7_000_000, 8_500_000].map(|t| {
            make_action(
                t,
                0,
                "GA_Nanally_Melee",
                "GE_Player_Nanally_Melee1_Damage",
                0,
                0,
            )
        }));
    req.combat_transition_events = vec![
        GameActorCombatTransitionEvent {
            timestamp_us: 6_000_000,
            actor_index: 0,
            leaving: true,
        },
        GameActorCombatTransitionEvent {
            timestamp_us: 8_000_000,
            actor_index: 0,
            leaving: false,
        },
    ];
    let report = run(&req);
    let stacks: Vec<_> = report
        .trace
        .iter()
        .filter(|event| event.event_kind == "nanally_a5_atk_buff")
        .collect();
    assert_eq!(
        stacks.iter().map(|event| event.time_us).collect::<Vec<_>>(),
        [4_191_054, 8_709_023]
    );
    assert!(
        stacks
            .iter()
            .all(|event| event.data.as_ref().and_then(|d| d.stacks) == Some(1))
    );
    let outside = report
        .hit_receipts
        .iter()
        .find(|hit| {
            hit.time_us == 7_191_054 && hit.damage_effect_id == "GE_Player_Nanally_Melee1_Damage"
        })
        .unwrap();
    assert_eq!(outside.non_crit_damage, ExactFraction::new(148, 1).unwrap());
}
