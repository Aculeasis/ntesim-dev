use super::nanally_actions::make_action;
use super::*;

fn request(active: &str, source: &str) -> GameScenarioRequest {
    let source = format!("options duration=6 crit=random stamina_capacity=60 initial_stamina=60 stamina_use_efficiency=1 stamina_recovery_rate=0;
        nanally char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 start_cycle=100;
        target hp=100000000 def=1080 resist=0;
        target hp=100000000 def=1080 resist=0;
        active {active}; {source}");
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    let model = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
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
fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    run_game_scenario_report(req, &Default::default()).unwrap()
}

#[test]
fn nanally_support_clock_projects_one_delayed_contact_pause_and_distinct_switch_gate() {
    for (switch, gate) in [(false, 1_308_275), (true, 1_283_416)] {
        let mut req = request("zero", "nanally swap;");
        req.actions.push(if switch {
            GameActionCommand::SwitchActor {
                timestamp_us: gate,
                target_actor_index: 1,
                cooldown_us: 0,
            }
        } else {
            normal(gate)
        });
        let report = run(&req);
        assert!(
            report.success && report.execution_completed,
            "{:?}",
            report.halt_diagnostic
        );
        let hits: Vec<_> = report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_Nanally_QTE1_Damage")
            .collect();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|hit| hit.time_us == 666_410));
        // One250ms window at rate.2 delays recovery by200ms, regardless of two recipients.
        match &mut req.actions[1] {
            GameActionCommand::InvokeAbility { timestamp_us, .. }
            | GameActionCommand::SwitchActor { timestamp_us, .. } => *timestamp_us -= 1,
            _ => unreachable!(),
        }
        assert!(!run(&req).execution_completed);
    }
}

#[test]
fn nanally_support_clock_zero_hp_loss_keeps_nominal_recovery_and_no_energy() {
    let mut req = request("zero", "nanally swap;");
    for target in &mut req.targets {
        target.resistance = Ratio {
            numerator: 1_000_000_000,
            denominator: 1,
        };
    }
    req.actions.push(normal(1_108_275));
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    assert_eq!(report.final_actor_states[0].granted_energy, 0);
}

#[test]
fn nanally_support_clock_interruption_retains_the_actual_switch_parent_and_cancels_both_recipients()
{
    let mut req = request("zero", "nanally swap;");
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 500_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert!(
        report
            .hit_receipts
            .iter()
            .all(|hit| hit.ability_id != "GA_Nanally_QTE")
    );
    assert_eq!(report.final_actor_states[0].granted_energy, 0);
    let report = serde_json::to_value(report).unwrap();
    assert_eq!(
        report["interrupt_receipts"][0]["interrupted_command_index"],
        0
    );
    assert_eq!(
        report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        2
    );
}

#[test]
fn nanally_riposte_clock_jump_cancels_unissued_damage_without_refunding_paid_dodge() {
    let mut req = request("nanally", "nanally dash[critical=true]; nanally attack;");
    req.actions.push(GameActionCommand::PlayerInput {
        timestamp_us: 600_000,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    });
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    assert_eq!(report.hits_settled, 0);
    assert_eq!(report.final_actor_states[0].granted_energy, 0);
    let report = serde_json::to_value(report).unwrap();
    assert_eq!(
        report["combat_stamina"]["total_spent"],
        serde_json::json!("15/1")
    );
}

#[test]
fn nanally_riposte_clock_positive_contact_alone_extends_recovery_by_297ms() {
    let mut req = request("nanally", "nanally dash[critical=true]; nanally attack;");
    req.actions.push(normal(1_341_593));
    let connected = run(&req);
    assert!(!connected.execution_completed);
    assert!(
        connected
            .halt_diagnostic
            .unwrap()
            .contains("locked until 1638593")
    );
    for target in &mut req.targets {
        target.resistance = Ratio {
            numerator: 1_000_000_000,
            denominator: 1,
        };
    }
    let missed = run(&req);
    assert!(missed.success && missed.execution_completed);
    assert_eq!(missed.final_actor_states[0].granted_energy, 0);
}

#[test]
fn nanally_riposte_clock_skill_can_cancel_before_contact_but_cannot_reuse_permission() {
    let mut req = request("nanally", "nanally dash[critical=true]; nanally attack;");
    req.actions.push(make_action(
        600_000,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        0,
    ));
    req.actions.push(normal(2_000_000));
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    assert!(
        report
            .hit_receipts
            .iter()
            .all(|hit| hit.damage_effect_id != "GE_Player_Nanally_PerfectEvadeAttack_Damage")
    );
    assert!(
        report
            .hit_receipts
            .iter()
            .any(|hit| hit.damage_effect_id == "GE_Player_Nanally_Melee1_Damage")
    );
}

#[test]
fn nanally_support_clock_settled_contact_pause_continues_after_interruption() {
    let mut req = request("zero", "nanally swap;");
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 700_000,
        actor_index: 0,
        armor_damage: None,
    });
    req.actions.push(normal(700_001));
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    let hits: Vec<_> = report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Nanally_Melee1_Damage")
        .collect();
    assert_eq!(hits.len(), 2);
    // The issued pause[.726410,.976410) at rate.2 contributes200ms to the new action.
    assert!(hits.iter().all(|hit| hit.time_us == 1_091_055));
}

#[test]
fn nanally_clock_interrupt_retains_paid_dodge_command_and_already_granted_permission() {
    let mut req = request("nanally", "nanally dash[critical=true];");
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 100_000,
        actor_index: 0,
        armor_damage: None,
    });
    req.actions.push(normal(200_000));
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    assert_eq!(
        report.interrupt_receipts[0].interrupted_command_index,
        Some(0)
    );
    assert_eq!(report.interrupt_receipts[0].cancelled_hit_deliveries, 0);
    assert_eq!(report.interrupt_receipts[0].recovery_before_us, 500_000);
    assert_eq!(report.hit_receipts.len(), 2);
    assert!(report.hit_receipts.iter().all(|hit| hit.time_us == 467_562
        && hit.damage_effect_id == "GE_Player_Nanally_PerfectEvadeAttack_Damage"));
    let report = serde_json::to_value(report).unwrap();
    assert_eq!(
        report["combat_stamina"]["total_spent"],
        serde_json::json!("15/1")
    );
}

#[test]
fn nanally_secondary_clock_native_source_keeps_two_periods_and_full_report_replay() {
    let result = PreparedSourceScenario::new(
        include_str!("../../../../../presets/native/nanally-secondary-control.nte"),
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
    assert_eq!(result.report.hits_settled, 16);
    assert_eq!(result.report.final_actor_states[0].granted_energy, 22);
    assert_eq!(
        result.report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(2, 5).unwrap())
    );
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_Nanally_QTE1_Damage")
            .count(),
        2
    );
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_Nanally_PerfectEvadeAttack_Damage")
            .count(),
        2
    );
    super::assert_native_replay(&result);
}
