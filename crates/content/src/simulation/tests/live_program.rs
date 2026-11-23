use super::*;
use crate::simulation::{
    GameActionCommand, GameLiveBranch, GameLiveBranchCondition, GameLiveEnergyWait,
    GameScenarioProgramRequest, PreparedGameProgram,
};

fn program(hp_below: u64) -> GameScenarioProgramRequest {
    let mut request = sample_s1_request();
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut request.actions[0] {
        *hit_delay_us = 800_000;
    }
    request.actions.insert(
        1,
        GameActionCommand::Wait {
            timestamp_us: 800_000,
            duration_us: 200_000,
        },
    );
    GameScenarioProgramRequest {
        request,
        energy_waits: Vec::new(),
        branches: vec![GameLiveBranch {
            timestamp_us: 800_000,
            after_action_index: 2,
            condition: GameLiveBranchCondition::TargetHpBelow {
                target_index: 0,
                threshold: hp_below,
            },
            wait_if_below_us: 200_000,
            wait_otherwise_us: 500_000,
            source_line: 9,
            source_column: 1,
        }],
    }
}

fn energy_wait_program(delayed_hit_us: u64, energy_at_least: u64) -> GameScenarioProgramRequest {
    let mut request = super::daffodil_actions::request();
    request.actors[0].initial_energy = 0;
    request.relative_duration_us = 20_000_000;
    let mut first = request.actions[0].clone();
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut first {
        *hit_delay_us = delayed_hit_us;
    }
    let mut suffix = first.clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        hit_delay_us,
        ..
    } = &mut suffix
    {
        *timestamp_us = 900_000;
        *hit_delay_us = 0;
    }
    request.actions = vec![
        first,
        GameActionCommand::Wait {
            timestamp_us: 500_000,
            duration_us: 0,
        },
        suffix,
    ];
    GameScenarioProgramRequest {
        request,
        branches: Vec::new(),
        energy_waits: vec![GameLiveEnergyWait {
            timestamp_us: 500_000,
            after_action_index: 2,
            actor_index: 0,
            energy_at_least,
            source_line: 12,
            source_column: 1,
        }],
    }
}

fn sequential_energy_wait_program(second_energy_threshold: u64) -> GameScenarioProgramRequest {
    let mut program = energy_wait_program(800_000, 1);
    let mut second = program.request.actions[2].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        hit_delay_us,
        ..
    } = &mut second
    {
        *timestamp_us = 900_000;
        *hit_delay_us = 800_000;
    }
    let mut final_action = second.clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        hit_delay_us,
        ..
    } = &mut final_action
    {
        *timestamp_us = 2_000_000;
        *hit_delay_us = 0;
    }
    program.request.actions[2] = second;
    program.request.actions.extend([
        GameActionCommand::Wait {
            timestamp_us: 1_500_000,
            duration_us: 0,
        },
        final_action,
    ]);
    program.energy_waits.push(GameLiveEnergyWait {
        timestamp_us: 1_500_000,
        after_action_index: 4,
        actor_index: 0,
        energy_at_least: second_energy_threshold,
        source_line: 14,
        source_column: 1,
    });
    program
}

#[test]
fn live_hp_branch_observes_same_time_settlement_and_shifts_the_suffix() {
    let limits = ExecutionLimits::default();
    let after_hit = PreparedGameProgram::new(&program(10_000_000), &limits)
        .unwrap()
        .run(42)
        .unwrap();
    let branch = &after_hit.live_branch_receipts[0];
    assert_eq!(branch.time_us, 800_000);
    assert!(branch.observed_value < 10_000_000);
    assert_eq!(branch.selected_wait_us, 200_000);
    assert_eq!(branch.source_line, 9);
    assert!(
        after_hit
            .hit_receipts
            .iter()
            .any(|hit| hit.command_index == Some(2) && hit.time_us == 1_000_000)
    );

    let otherwise = PreparedGameProgram::new(&program(0), &limits)
        .unwrap()
        .run(42)
        .unwrap();
    assert_eq!(otherwise.live_branch_receipts[0].selected_wait_us, 500_000);
    assert!(
        otherwise
            .hit_receipts
            .iter()
            .any(|hit| hit.command_index == Some(2) && hit.time_us == 1_300_000)
    );
}

#[test]
fn live_energy_wait_resumes_after_same_time_energy_settlement_and_shifts_suffix() {
    let program = energy_wait_program(800_000, 1);
    let report = PreparedGameProgram::new(&program, &ExecutionLimits::default())
        .unwrap()
        .run(42)
        .unwrap();
    let receipt = &report.live_energy_wait_receipts[0];
    assert_eq!(receipt.started_time_us, 500_000);
    assert!(receipt.energy_at_start < receipt.energy_at_least);
    assert_eq!(receipt.resumed_time_us, Some(800_000));
    assert!(receipt.energy_at_resume.unwrap() >= receipt.energy_at_least);
    assert!(
        report
            .hit_receipts
            .iter()
            .any(|hit| { hit.command_index == Some(2) && hit.time_us == 1_200_000 })
    );
    assert!(report.trace.iter().any(|event| {
        event.event_kind == "live_energy_wait_resumed" && event.time_us == 800_000
    }));
}

#[test]
fn live_energy_wait_does_not_poll_sparse_time_and_reports_unreached_threshold() {
    let short = PreparedGameProgram::new(
        &energy_wait_program(800_000, 1),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    let long = PreparedGameProgram::new(
        &energy_wait_program(8_000_000, 1),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        long.total_events_processed <= short.total_events_processed + 2,
        "sparse delay added more than two natural lifecycle events: short={}, long={}",
        short.total_events_processed,
        long.total_events_processed
    );
    assert!(
        long.total_events_processed < 10,
        "energy wait appears to poll time"
    );
    assert_eq!(
        long.live_energy_wait_receipts[0].resumed_time_us,
        Some(8_000_000)
    );

    let unresolved = PreparedGameProgram::new(
        &energy_wait_program(800_000, 120),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(!unresolved.execution_completed);
    assert_eq!(unresolved.absolute_elapsed_us, 20_000_000);
    unresolved
        .input_time
        .validate(unresolved.absolute_elapsed_us)
        .unwrap();
    assert_eq!(
        unresolved.live_energy_wait_receipts[0].resumed_time_us,
        None
    );
    assert!(unresolved.pending_events >= 2);
    assert!(
        unresolved
            .halt_diagnostic
            .as_deref()
            .unwrap()
            .contains("LIVE_ENERGY_WAIT_PENDING")
    );
    assert!(
        !unresolved
            .hit_receipts
            .iter()
            .any(|hit| hit.command_index == Some(2))
    );
}

#[test]
fn sequential_live_energy_waits_shift_each_suffix_and_gate_time() {
    let limits = ExecutionLimits::default();
    let baseline = PreparedGameProgram::new(&sequential_energy_wait_program(1), &limits)
        .unwrap()
        .run(42)
        .unwrap();
    let first_energy = baseline.live_energy_wait_receipts[0]
        .energy_at_resume
        .unwrap();
    let threshold = first_energy + 1;
    let report = PreparedGameProgram::new(&sequential_energy_wait_program(threshold), &limits)
        .unwrap()
        .run(42)
        .unwrap();
    assert_eq!(report.live_energy_wait_receipts.len(), 2);
    let first = &report.live_energy_wait_receipts[0];
    let second = &report.live_energy_wait_receipts[1];
    assert_eq!(first.started_time_us, 500_000);
    assert_eq!(first.resumed_time_us, Some(800_000));
    assert_eq!(second.started_time_us, 1_800_000);
    assert_eq!(second.resumed_time_us, Some(2_000_000));
    assert_eq!(second.source_line, 14);
    assert!(
        report
            .hit_receipts
            .iter()
            .any(|hit| { hit.command_index == Some(2) && hit.time_us == 2_000_000 })
    );
    assert!(
        report
            .hit_receipts
            .iter()
            .any(|hit| { hit.command_index == Some(4) && hit.time_us == 2_500_000 })
    );
    assert!(report.execution_completed, "{:?}", report.halt_diagnostic);
}

#[test]
fn live_energy_waits_reject_mixed_branches_and_out_of_order_gates() {
    let limits = ExecutionLimits::default();
    let mut invalid = sequential_energy_wait_program(2);
    invalid.energy_waits[1].timestamp_us = 400_000;
    assert!(
        PreparedGameProgram::new(&invalid, &limits)
            .err()
            .unwrap()
            .contains("must follow their timestamps")
    );

    let mut invalid = energy_wait_program(800_000, 1);
    invalid.branches.push(GameLiveBranch {
        timestamp_us: 400_000,
        after_action_index: 3,
        condition: GameLiveBranchCondition::TargetHpBelow {
            target_index: 0,
            threshold: 1,
        },
        wait_if_below_us: 100_000,
        wait_otherwise_us: 100_000,
        source_line: 13,
        source_column: 1,
    });
    assert!(
        PreparedGameProgram::new(&invalid, &limits)
            .err()
            .unwrap()
            .contains("earlier or equal timestamp")
    );
}

#[test]
fn live_active_energy_branch_observes_settled_grants_and_shifts_suffix() {
    let limits = ExecutionLimits::default();
    let mut program = energy_wait_program(800_000, 1);
    program.energy_waits.clear();
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut program.request.actions[2] {
        *timestamp_us = 1_000_000;
    }
    program.branches.push(GameLiveBranch {
        timestamp_us: 800_000,
        after_action_index: 2,
        condition: GameLiveBranchCondition::ActiveActorEnergyBelow { threshold: 1 },
        wait_if_below_us: 200_000,
        wait_otherwise_us: 500_000,
        source_line: 12,
        source_column: 1,
    });

    let report = PreparedGameProgram::new(&program, &limits)
        .unwrap()
        .run(42)
        .unwrap();
    let receipt = &report.live_branch_receipts[0];
    assert_eq!(receipt.time_us, 800_000);
    assert_eq!(receipt.observer_actor_index, 0);
    assert!(
        receipt.observed_value >= 1,
        "the same-time hit grants energy"
    );
    assert_eq!(receipt.selected_wait_us, 500_000);
    assert!(
        report
            .hit_receipts
            .iter()
            .any(|hit| { hit.command_index == Some(2) && hit.time_us == 1_300_000 })
    );
    assert!(report.trace.iter().any(|event| {
        event.event_kind == "live_branch"
            && event.from_actor == Some(0)
            && event.data.as_ref().and_then(|d| d.condition.as_deref())
                == Some("active_actor_energy_below")
            && event.time_us == 800_000
    }));
}

#[test]
fn unresolved_first_energy_wait_keeps_later_gates_parked() {
    let limits = ExecutionLimits::default();
    let mut program = sequential_energy_wait_program(1);
    program.energy_waits[0].energy_at_least = 120;
    let report = PreparedGameProgram::new(&program, &limits)
        .unwrap()
        .run(42)
        .unwrap();
    assert!(!report.execution_completed);
    assert_eq!(report.live_energy_wait_receipts.len(), 1);
    assert_eq!(report.live_energy_wait_receipts[0].resumed_time_us, None);
    assert!(report.pending_events >= 3);
    assert!(
        report
            .halt_diagnostic
            .as_deref()
            .unwrap()
            .contains("LIVE_ENERGY_WAIT_PENDING")
    );
}

#[test]
fn pending_energy_wait_keeps_its_diagnostic_when_next_event_exceeds_horizon() {
    let mut program = energy_wait_program(800_000, 120);
    program.request.relative_duration_us = 600_000;
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut program.request.actions[2] {
        *timestamp_us = 550_000;
    }

    let report = PreparedGameProgram::new(&program, &ExecutionLimits::default())
        .unwrap()
        .run(42)
        .unwrap();

    assert!(!report.execution_completed);
    assert_eq!(report.absolute_elapsed_us, 600_000);
    report
        .input_time
        .validate(report.absolute_elapsed_us)
        .unwrap();
    assert_eq!(report.live_energy_wait_receipts.len(), 1);
    assert_eq!(report.live_energy_wait_receipts[0].resumed_time_us, None);
    let diagnostic = report.halt_diagnostic.as_deref().unwrap();
    assert!(
        diagnostic.contains("LIVE_ENERGY_WAIT_PENDING"),
        "{diagnostic}"
    );
    assert!(
        !diagnostic.contains("SCENARIO_HORIZON_PENDING"),
        "{diagnostic}"
    );
}

#[test]
fn shifted_energy_gate_beyond_horizon_reports_pending_wait() {
    let limits = ExecutionLimits::default();
    let mut program = sequential_energy_wait_program(1);
    program.request.relative_duration_us = 1_600_000;
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut program.request.actions[4] {
        *timestamp_us = 1_550_000;
    }
    let report = PreparedGameProgram::new(&program, &limits)
        .unwrap()
        .run(42)
        .unwrap();
    assert!(!report.execution_completed);
    assert_eq!(report.live_energy_wait_receipts.len(), 1);
    assert!(
        report
            .halt_diagnostic
            .as_deref()
            .unwrap()
            .contains("LIVE_ENERGY_WAIT_PENDING: gate 1")
    );
}

#[test]
fn live_branch_rejects_invalid_suffixes_and_zero_waits() {
    let limits = ExecutionLimits::default();
    let mut invalid = program(1);
    invalid.branches[0].wait_if_below_us = 0;
    assert!(
        PreparedGameProgram::new(&invalid, &limits)
            .err()
            .unwrap()
            .contains("must both advance")
    );

    let mut invalid = program(1);
    invalid.branches[0].after_action_index = invalid.request.actions.len() + 1;
    assert!(
        PreparedGameProgram::new(&invalid, &limits)
            .err()
            .unwrap()
            .contains("boundary")
    );

    let mut invalid = program(1);
    invalid.branches[0].after_action_index = 3;
    assert!(
        PreparedGameProgram::new(&invalid, &limits)
            .err()
            .unwrap()
            .contains("cannot precede an action")
    );
}

#[test]
fn live_branches_support_multiple_sequential_observations() {
    let limits = ExecutionLimits::default();
    let mut prog = program(1); // will not satisfy hp < 1, so wait_otherwise_us (500_000) is chosen
    prog.request.relative_duration_us = 10_000_000;
    // First branch: timestamp_us = 800_000, after_action_index = 2, wait_if_below: 200_000, wait_otherwise: 500_000
    // Suffix starts after 800_000 + 200_000 = 1_000_000 (minimum wait)
    // Shift is 500_000 - 200_000 = +300_000
    // Now add an action and a second branch after it
    let mut next_act = prog.request.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        hit_delay_us,
        ..
    } = &mut next_act
    {
        *timestamp_us = 5_500_000;
        *hit_delay_us = 0;
    }
    prog.request.actions.push(next_act);
    prog.request.actions.push(GameActionCommand::Wait {
        timestamp_us: 5_500_000,
        duration_us: 100_000,
    });
    prog.branches.push(GameLiveBranch {
        timestamp_us: 5_000_000,
        after_action_index: 4,
        condition: GameLiveBranchCondition::ActiveActorEnergyBelow { threshold: 1000 },
        wait_if_below_us: 100_000,
        wait_otherwise_us: 200_000,
        source_line: 15,
        source_column: 1,
    });

    // Test ordering error first
    let mut out_of_order = prog.clone();
    out_of_order.branches[1].timestamp_us = 700_000;
    assert!(
        PreparedGameProgram::new(&out_of_order, &limits)
            .err()
            .unwrap()
            .contains("must follow their timestamps")
    );

    let report = PreparedGameProgram::new(&prog, &limits)
        .unwrap()
        .run(42)
        .unwrap();
    assert_eq!(report.live_branch_receipts.len(), 2);
    assert_eq!(report.live_branch_receipts[0].selected_wait_us, 500_000);
    assert_eq!(report.live_branch_receipts[1].selected_wait_us, 100_000);
    assert!(report.execution_completed);
}

#[test]
fn live_program_combines_branch_and_energy_wait() {
    let limits = ExecutionLimits::default();
    let mut prog = energy_wait_program(800_000, 1);
    prog.branches.push(GameLiveBranch {
        timestamp_us: 1_200_000,
        after_action_index: 3,
        condition: GameLiveBranchCondition::ActiveActorEnergyBelow { threshold: 1000 },
        wait_if_below_us: 100_000,
        wait_otherwise_us: 200_000,
        source_line: 20,
        source_column: 1,
    });
    // Add suffix wait and action after branch
    let mut suffix = prog.request.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        hit_delay_us,
        ..
    } = &mut suffix
    {
        *timestamp_us = 1_400_000;
        *hit_delay_us = 0;
    }
    prog.request.actions.extend([
        GameActionCommand::Wait {
            timestamp_us: 1_400_000,
            duration_us: 0,
        },
        suffix,
    ]);

    let prepared = PreparedGameProgram::new(&prog, &limits).unwrap();
    let report = prepared.run(42).unwrap();
    assert_eq!(report.live_energy_wait_receipts.len(), 1);
    assert_eq!(report.live_branch_receipts.len(), 1);
    assert!(report.execution_completed);
    assert!(report.success);
}

#[test]
fn native_wait_deadlines_complete_at_time_limit_even_when_wait_extends_beyond_it() {
    let mut request = sample_s1_request();
    request.relative_duration_us = 20_000_000;
    for duration_us in [0, 20_000_000, 20_000_001] {
        request.actions = vec![GameActionCommand::Wait {
            timestamp_us: 0,
            duration_us,
        }];
        let report = run_game_scenario_report(&request, &Default::default()).unwrap();
        assert!(report.success);
        assert!(report.execution_completed);
        assert_eq!(report.absolute_elapsed_us, request.relative_duration_us);
        assert_eq!(report.total_events_processed, 0);
        assert_eq!(report.pending_events, 0);
        report
            .input_time
            .validate(report.absolute_elapsed_us)
            .unwrap();
        assert!(report.halt_diagnostic.is_none());
    }
}

#[test]
fn selected_branch_waits_and_energy_shifted_final_waits_keep_actual_deadlines_and_time() {
    for horizon in [1_300_000, 1_299_999] {
        let mut program = program(0);
        program.request.actions.truncate(2);
        program.request.relative_duration_us = horizon;
        let report = run_game_program_report(&program, &Default::default()).unwrap();
        assert!(report.success);
        assert!(report.execution_completed);
        assert_eq!(report.live_branch_receipts[0].selected_wait_us, 500_000);
        assert_eq!(report.absolute_elapsed_us, horizon);
        let waits: u64 = report
            .input_time
            .intervals
            .iter()
            .filter(|interval| interval.cause == GameTimeCause::AuthoredWait)
            .map(|interval| interval.duration_us)
            .sum();
        assert_eq!(waits, horizon - 800_000);
        report.input_time.validate(horizon).unwrap();
    }
    let mut program = energy_wait_program(800_000, 1);
    program.request.actions[2] = GameActionCommand::Wait {
        timestamp_us: 900_000,
        duration_us: 19_000_000,
    };
    let report = run_game_program_report(&program, &Default::default()).unwrap();
    assert!(report.success && report.execution_completed);
    assert_eq!(report.absolute_elapsed_us, 20_000_000);
    assert_eq!(
        report.live_energy_wait_receipts[0].resumed_time_us,
        Some(800_000)
    );
    let wait = report
        .input_time
        .intervals
        .iter()
        .find(|interval| interval.command_index == Some(2))
        .unwrap();
    assert_eq!(
        (wait.start_us, wait.end_us, wait.cause),
        (1_200_000, 20_000_000, GameTimeCause::AuthoredWait)
    );
    report
        .input_time
        .validate(report.absolute_elapsed_us)
        .unwrap();
}

#[test]
fn shorter_selected_branch_wait_uses_its_declared_baseline_and_cannot_rewind_suffix() {
    let mut program = program(0);
    program.branches[0].wait_if_below_us = 500_000;
    program.branches[0].wait_otherwise_us = 200_000;
    assert!(PreparedGameProgram::new(&program, &Default::default()).is_err());
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut program.request.actions[2] {
        *timestamp_us = 1_300_000;
    }
    let report = run_game_program_report(&program, &Default::default()).unwrap();
    assert_eq!(report.live_branch_receipts[0].selected_wait_us, 200_000);
    assert!(
        report
            .hit_receipts
            .iter()
            .any(|hit| hit.command_index == Some(2) && hit.time_us == 1_000_000)
    );
    assert!(report.success && report.execution_completed);
    // The compiler's baseline Wait is replaced even when no suffix is queued.
    program.request.actions.truncate(2);
    program.request.actions[1] = GameActionCommand::Wait {
        timestamp_us: 800_000,
        duration_us: 500_000,
    };
    program.request.relative_duration_us = 1_000_000;
    let report = run_game_program_report(&program, &Default::default()).unwrap();
    assert!(report.success && report.execution_completed);
    assert_eq!(report.absolute_elapsed_us, 1_000_000);
    assert_eq!(
        report.input_time.intervals.last().unwrap().duration_us,
        200_000
    );
    program.branches[0].wait_otherwise_us = u64::MAX;
    assert!(run_game_program_report(&program, &Default::default()).is_err());
}
