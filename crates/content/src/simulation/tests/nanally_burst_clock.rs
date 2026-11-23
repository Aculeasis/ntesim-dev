use super::nanally_actions::make_action;
use super::*;
use crate::simulation::tests::support::formula_damage_total;
fn burst(at: u64) -> GameActionCommand {
    make_action(
        at,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        100,
        15_000_000,
    )
}
fn normal(at: u64) -> GameActionCommand {
    make_action(
        at,
        0,
        "GA_Nanally_Melee",
        "GE_Player_Nanally_Melee1_Damage",
        0,
        0,
    )
}
fn request() -> GameScenarioRequest {
    let mut req = super::nanally_actions::request();
    req.relative_duration_us = 20_000_000;
    req.actors[0].awakening = Default::default();
    req.actors[0].talent_levels = Some([1, 1, 8]);
    req.actors[0].initial_energy = 100;
    req.actors[0].initial_cycle = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].defense = 1080;
    req.actions = vec![burst(0)];
    let model = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    run_game_scenario_report(req, &Default::default()).unwrap()
}

#[test]
fn nanally_burst_clock_seven_source_markers_keep_independent_damage_and_no_free_cat() {
    let report = run(&request());
    assert!(report.success && report.execution_completed);
    assert_eq!(
        report
            .hit_receipts
            .iter()
            .map(|hit| hit.time_us)
            .collect::<Vec<_>>(),
        [
            2300192, 2382032, 2468449, 2547640, 2634465, 2716783, 3015465
        ]
    );
    assert_eq!(
        formula_damage_total(&report),
        ExactFraction::new(8569, 1).unwrap()
    );
    assert_eq!(report.final_actor_states[0].spent_energy, 100);
    assert_eq!(report.final_actor_states[0].granted_energy, 0);
    assert!(
        report
            .hit_receipts
            .iter()
            .all(|hit| !hit.damage_effect_id.contains("Cat_"))
    );
}

#[test]
fn nanally_burst_clock_interrupt_before_birth_cancels_only_unissued_hits_and_keeps_payment() {
    let mut req = request();
    req.actions.push(normal(2_500_000));
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 2_400_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert_eq!(
        report
            .hit_receipts
            .iter()
            .filter(|hit| hit.ability_id == "GA_Nanally_UltraSkill")
            .count(),
        2
    );
    assert!(
        report
            .hit_receipts
            .iter()
            .all(|hit| !hit.damage_effect_id.contains("Cat_"))
    );
    assert_eq!(report.interrupt_receipts[0].cancelled_hit_deliveries, 5);
    assert_eq!(
        report.interrupt_receipts[0].interrupted_command_index,
        Some(0)
    );
    assert_eq!(report.final_actor_states[0].spent_energy, 100);
    assert_eq!(report.final_actor_states[0].energy, 1);
}

#[test]
fn nanally_burst_clock_birth_precedes_equal_time_interrupt_and_cooldown_survives() {
    for (at, cat) in [(3_415_869, false), (3_415_870, true)] {
        let mut req = request();
        req.actors[0].initial_energy = 200;
        req.actors[0].energy_capacity = 200;
        req.interrupt_events.push(GameActorInterruptEvent {
            timestamp_us: at,
            actor_index: 0,
            armor_damage: None,
        });
        req.actions.push(normal(3_415_871));
        let report = run(&req);
        assert!(report.success && report.execution_completed);
        assert_eq!(
            report
                .hit_receipts
                .iter()
                .filter(|hit| hit.damage_effect_id.contains("Cat_"))
                .count(),
            usize::from(cat)
        );
        assert_eq!(report.final_actor_states[0].spent_energy, 100);
        req.actions.push(burst(14_999_999));
        let denied = run(&req);
        assert!(!denied.execution_completed);
        assert!(
            denied
                .halt_diagnostic
                .unwrap()
                .contains(&format!("on cooldown until {}", 15_000_000 + at))
        );
        assert_eq!(denied.final_actor_states[0].spent_energy, 100);
    }
}

#[test]
fn nanally_burst_clock_switch_and_movement_retain_distinct_source_gates() {
    for (switch, gate) in [(true, 3_583_025), (false, 3_756_871)] {
        let mut req = request();
        let mut ally = req.actors[0].clone();
        ally.slot = 1;
        ally.character_id = "character:1019".into();
        req.actors.push(ally);
        req.actions.push(if switch {
            GameActionCommand::SwitchActor {
                timestamp_us: gate,
                target_actor_index: 1,
                cooldown_us: 0,
            }
        } else {
            GameActionCommand::PlayerInput {
                timestamp_us: gate,
                actor_index: 0,
                input: PlayerInput::Special,
                critical_success: false,
            }
        });
        let report = run(&req);
        assert!(
            report.success && report.execution_completed,
            "{:?}",
            report.halt_diagnostic
        );
        match &mut req.actions[1] {
            GameActionCommand::SwitchActor { timestamp_us, .. }
            | GameActionCommand::PlayerInput { timestamp_us, .. } => *timestamp_us -= 1,
            _ => unreachable!(),
        }
        assert!(!run(&req).execution_completed);
    }
}

#[test]
fn nanally_burst_clock_source_funds_two_periods_without_resetting_energy_or_cooldowns() {
    let result = PreparedSourceScenario::new(
        include_str!("../../../../../presets/native/nanally-burst-control.nte"),
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.hits_settled, 212);
    let actor = &result.report.final_actor_states[0];
    assert_eq!(
        (
            actor.energy,
            actor.granted_energy,
            actor.spent_energy,
            actor.overflow_energy
        ),
        (100, 211, 200, 11)
    );
    assert_eq!(
        actor.charge_remainder,
        Some(ExactFraction::new(197, 250).unwrap())
    );
    super::assert_native_replay(&result);
}
