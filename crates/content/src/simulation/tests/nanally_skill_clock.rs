use super::nanally_actions::make_action;
use super::*;

fn skill(at: u64) -> GameActionCommand {
    make_action(
        at,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        16_000_000,
    )
}
fn request() -> GameScenarioRequest {
    let mut req = super::nanally_actions::request();
    req.relative_duration_us = 20_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actors[0].initial_energy = 0;
    req.actors[0].initial_cycle = 0;
    req.actors[0].static_stats.crit_rate = Ratio::one();
    req.actors[0].static_stats.crit_damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.stochastic_crit = None;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].defense = 1080;
    req.actions = vec![skill(0)];
    let model = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    run_game_scenario_report(req, &Default::default()).unwrap()
}

#[test]
fn nanally_skill_clock_first_hit_precedes_authority_and_all_five_markers_are_distinct() {
    let report = run(&request());
    assert!(report.success && report.execution_completed);
    assert_eq!(
        report
            .hit_receipts
            .iter()
            .map(|hit| hit.time_us)
            .collect::<Vec<_>>(),
        [429637, 497363, 563001, 631341, 696485]
    );
    assert_eq!(
        report.hit_receipts[0].crit_damage,
        ExactFraction::new(750, 1).unwrap()
    );
    assert!(
        report.hit_receipts[1..]
            .iter()
            .all(|hit| hit.crit_damage == ExactFraction::new(900, 1).unwrap())
    );
    assert_eq!(
        report.total_damage_exact,
        ExactFraction::new(2500, 1).unwrap()
    );
    assert_eq!(report.final_actor_states[0].granted_energy, 6);
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(3, 10).unwrap())
    );
}

#[test]
fn nanally_skill_clock_interruption_keeps_only_reached_cooldown_damage_and_authority() {
    for (at, hits, cooldown, authority) in [
        (425570, 0, false, false),
        // Buff notifications settle before the explicit SettlementAndDeath interruption.
        (425571, 0, true, false),
        (427000, 0, true, false),
        (430000, 1, true, true),
    ] {
        let mut req = request();
        req.relative_duration_us = 1_000_000;
        req.interrupt_events.push(GameActorInterruptEvent {
            timestamp_us: at,
            actor_index: 0,
            armor_damage: None,
        });
        req.actions.push(make_action(
            750_000,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ));
        let report = run(&req);
        assert!(report.success && report.execution_completed);
        assert_eq!(
            report
                .hit_receipts
                .iter()
                .filter(|hit| hit.damage_effect_id == "GE_Player_Nanally_Skill1_Damage")
                .count(),
            hits as usize,
            "interrupt {at}"
        );
        let receipt = &report.interrupt_receipts[0];
        assert_eq!(receipt.interrupted_command_index, Some(0));
        assert_eq!(receipt.cancelled_hit_deliveries, 5 - hits as usize);
        assert_eq!(
            receipt.skill_cooldown_before_us,
            if cooldown { Some(16_425_571) } else { None }
        );
        assert_eq!(
            receipt.skill_cooldown_before_us,
            receipt.skill_cooldown_after_us
        );
        let probe = report
            .hit_receipts
            .iter()
            .find(|hit| hit.damage_effect_id == "GE_Player_Nanally_Melee1_Damage")
            .unwrap();
        assert_eq!(
            probe.crit_damage,
            if authority {
                ExactFraction::new(1332, 5).unwrap()
            } else {
                ExactFraction::new(222, 1).unwrap()
            }
        );
    }
}

#[test]
fn nanally_skill_clock_aoe_keeps_one_packet_energy_grant_with_a_dead_primary() {
    let mut req = request();
    let mut secondary = req.targets[0].clone();
    secondary.target_index = 1;
    req.targets.push(secondary);
    req.targets[0].hp_current = 0;
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert_eq!(report.hits_settled, 5);
    assert!(report.hit_receipts.iter().all(|hit| hit.target_index == 1));
    assert_eq!(report.final_actor_states[0].granted_energy, 6);
}

#[test]
fn nanally_skill_clock_a6_captures_combat_state_at_actual_authority_application() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
    req.combat_transition_events
        .push(GameActorCombatTransitionEvent {
            timestamp_us: 200_000,
            actor_index: 0,
            leaving: true,
        });
    let model = crate::characters::nanally::combat::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actions.push(make_action(
        16_000_000,
        0,
        "GA_Nanally_Melee",
        "GE_Player_Nanally_Melee1_Damage",
        0,
        0,
    ));
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    let probe = report
        .hit_receipts
        .iter()
        .find(|hit| hit.damage_effect_id == "GE_Player_Nanally_Melee1_Damage")
        .unwrap();
    // Exit after E commitment but before Authority application chooses20s, not15s.
    assert_eq!(probe.crit_damage, ExactFraction::new(1332, 5).unwrap());
    assert_eq!(report.combat_transition_receipts.len(), 1);
}
