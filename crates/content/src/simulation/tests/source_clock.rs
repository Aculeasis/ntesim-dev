use super::*;

fn run(options: &str, hp: u64, rotation: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(
        &format!(
            "options iteration=1 crit=fixed {options};
            hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp={hp} def=0 resist=0; active hotori; {rotation}"
        ),
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
}

#[test]
fn source_clock_time_and_victory_keep_zero_relative_rate_unavailable_and_replay() {
    let time = run("duration=20 stop=time", 1, "hotori burst;");
    let victory = run("duration=20 stop=victory", 1, "hotori burst; wait(99);");
    let unbounded = run("stop=victory", 1, "hotori burst; wait(.duration);");
    assert_eq!(time.report.absolute_elapsed_us, 2_316_080);
    for result in [&time, &victory, &unbounded] {
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        assert_eq!(result.report.hits_settled, 1);
        assert_eq!(result.report.relative_elapsed_us, 0);
        assert!(!result.report.game_dps_computed);
        assert_eq!(result.report.damage_analysis().unwrap().total.dps, None);
        result.report.validate_clocks().unwrap();
        assert_native_replay(result);
    }
    // The first Burst contact defeats the dummy before recovery. No suffix is evaluated.
    assert_eq!(victory.report.absolute_elapsed_us, 2_316_080);
    assert_eq!(unbounded.report.absolute_elapsed_us, 2_316_080);
    assert_eq!(victory.occurrences.len(), 1);
    assert_eq!(unbounded.occurrences.len(), 1);
    assert!(victory.report.pending_events > 0);
    // A later same-time authored input is outside the victorious causal closure.
    let mut request = victory.request.clone();
    request.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2_316_080,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report =
        crate::simulation::run_game_scenario_report(&request, &Default::default()).unwrap();
    assert!(report.success && report.execution_completed);
    assert_eq!(report.absolute_elapsed_us, 2_316_080);
    assert_eq!(report.final_buff_state.active_actor_index, 0);
    assert!(report.rejected_action.is_none());
}

#[test]
fn source_clock_victory_horizon_and_early_rotation_exhaustion_never_complete() {
    let result = run("duration=20 stop=victory", 100_000_000, "hotori burst;");
    assert!(result.report.success && !result.report.execution_completed);
    assert_eq!(result.report.absolute_elapsed_us, 14_183_333);
    assert!(
        result
            .report
            .halt_diagnostic
            .as_ref()
            .unwrap()
            .contains("rotation and pending effects ended")
    );
    assert!(
        result
            .report
            .halt_diagnostic
            .as_ref()
            .unwrap()
            .contains("limit: 3600 seconds")
    );
    assert_native_replay(&result);
    for rotation in ["wait(.duration); wait(1);", "wait(.duration + 0.000001);"] {
        let result = run("stop=victory", 100_000_000, rotation);
        assert!(result.report.success && !result.report.execution_completed);
        assert_eq!(result.report.absolute_elapsed_us, 3_600_000_000);
        assert!(result.report.halt_diagnostic.as_ref().unwrap().contains(
            "combat-time limit of 3600 seconds reached before all targets were defeated"
        ));
        assert_eq!(
            result.diagnostic.as_ref().unwrap().code,
            crate::dsl::validation::DiagnosticCode::Horizon
        );
        assert_native_replay(&result);
    }
    // Native pending events can cross the safety horizon before a cast finishes.
    let result = PreparedSourceScenario::new(
        "options stop=victory; hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120; target hp=100000000 def=0 resist=0; active hotori; hotori burst;",
        Default::default(),
        &ExecutionLimits { max_horizon_us: 1_000_001, ..Default::default() },
    ).unwrap().run(42).unwrap();
    assert!(!result.report.execution_completed);
    assert!(
        result
            .report
            .halt_diagnostic
            .as_ref()
            .unwrap()
            .contains("combat-time limit of 1.000001 seconds reached")
    );
    assert_native_replay(&result);
}

#[test]
fn source_clock_time_stop_clips_action_wait_and_implicit_switch_without_evaluating_suffix() {
    for rotation in [
        "hotori attack;",
        "mint attack;",
        "wait(21);",
        "wait_until(21);",
        "while 1 {hotori attack;}",
    ] {
        let result = run(
            "duration=0.1",
            100_000_000,
            &format!("{rotation} let invalid = 1 / 0;"),
        );
        assert!(
            result.report.success && result.report.execution_completed,
            "{rotation}: {:?}",
            result.diagnostic
        );
        assert!(result.diagnostic.is_none());
        assert!(result.request.source_halt.is_none());
        assert_eq!(result.report.relative_elapsed_us, 100_000);
        assert_eq!(result.report.absolute_elapsed_us, 100_000);
        assert_eq!(result.occurrences.len(), 1);
        result.report.input_time.validate(100_000).unwrap();
        assert_native_replay(&result);
    }
    let result = PreparedSourceScenario::new(
        "options duration=0.1 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint;
        zero attack; let invalid = 1 / 0;",
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
    assert!(result.diagnostic.is_none());
    assert_eq!(result.report.relative_elapsed_us, 100_000);
    // The stop occurs during the real Support entry, before Zero's attack.
    assert_eq!(result.request.actions.len(), 1);
    assert!(matches!(
        result.request.actions[0],
        GameActionCommand::SwitchActor { .. }
    ));
    assert_native_replay(&result);
}

#[test]
fn source_clock_exhaustion_drains_burst_and_retains_valid_time_stop_at_boundary() {
    let result = run("duration=5", 100_000_000, "hotori burst;");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.hits_settled, 10);
    assert_eq!(result.report.absolute_elapsed_us, 19_183_333);
    assert_eq!(result.report.relative_elapsed_us, 5_000_000);
    assert!(result.report.halt_diagnostic.is_none());
    assert_native_replay(&result);
    let window = run(
        "duration=5.1",
        100_000_000,
        "hotori burst; hotori attack[window=attack];",
    );
    assert!(window.report.success && window.report.execution_completed);
    assert_eq!(window.report.relative_elapsed_us, 5_100_000);
    assert_eq!(window.report.hits_settled, 11);
    assert_native_replay(&window);
    let truncated = run("duration=1", 100_000_000, "hotori burst;");
    assert!(truncated.report.execution_completed);
    assert_eq!(truncated.report.absolute_elapsed_us, 15_183_333);
    assert_eq!(truncated.report.relative_elapsed_us, 1_000_000);
    assert!(truncated.diagnostic.is_none());
    assert_native_replay(&truncated);
}

#[test]
fn source_clock_victory_requires_every_dummy_and_accepts_exact_horizon_contact() {
    let base = run("duration=2.316080 stop=victory", 1, "hotori burst;");
    assert!(base.report.success && base.report.execution_completed);
    assert_eq!(base.report.absolute_elapsed_us, 2_316_080);
    for hp in [1, 100_000_000] {
        let source = base.source.replace(
            "target hp=1 def=0 resist=0;",
            &format!("target hp=1 def=0 resist=0; target hp={hp} def=0 resist=0;"),
        );
        let result = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap();
        assert!(result.report.success);
        assert_eq!(result.report.execution_completed, hp == 1);
        assert_native_replay(&result);
    }
}

#[test]
fn supported_ultimate_routes_consume_a_positive_owned_cinematic_binding() {
    for owner in [
        "adler",
        "aurelia",
        "baicang",
        "blackbird",
        "chaos",
        "chiz",
        "daffodil",
        "edgar",
        "fadia",
        "haniel",
        "hathor",
        "hotori",
        "iroi",
        "jiuyuan",
        "lacrimosa",
        "linko",
        "mint",
        "nanally",
        "sakiri",
        "shinku",
        "skia",
        "zankou",
        "zero",
    ] {
        let resource = if owner == "hotori" {
            "start_timepiece=120"
        } else {
            ""
        };
        let source = format!(
            "options duration=1 crit=fixed; {owner} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {resource}; target hp=1000000000 def=0 resist=0; active {owner}; {owner} burst;"
        );
        let program = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
            .unwrap_or_else(|error| panic!("{owner}: {error:?}"));
        let result = program
            .run(42)
            .unwrap_or_else(|error| panic!("{owner}: {error}"));
        assert!(
            result.report.success && result.report.execution_completed,
            "{owner}: {:?}",
            result.diagnostic
        );
        assert_eq!(result.report.relative_elapsed_us, 1_000_000, "{owner}");
        assert!(
            result.report.absolute_elapsed_us > result.report.relative_elapsed_us,
            "{owner}"
        );
        assert_eq!(result.report.timer_bindings.len(), 1, "{owner}");
        assert!(
            !result.report.timer_pauses.as_ref().unwrap().is_empty(),
            "{owner}"
        );
        result.report.validate_clocks().unwrap();
        assert_native_replay(&result);
    }
}

#[test]
fn live_time_freezes_across_absolute_wait_and_continuing_rotation_reaches_relative_target() {
    let result = run(
        "duration=120",
        1_000_000_000,
        "hotori burst; if .time != 0 {let assertion_failure=1/0;} wait(1); if .time != 0 {let assertion_failure=1/0;} while .time < .duration {wait(.duration-.time);}",
    );
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.relative_elapsed_us, 120_000_000);
    assert_eq!(result.report.absolute_elapsed_us, 134_183_333);
    assert_native_replay(&result);
    let value = serde_json::to_value(&result.request).unwrap();
    for field in [
        "clock_contract",
        "timer_policy",
        "relative_duration_us",
        "absolute_limit_us",
    ] {
        let mut obsolete = value.clone();
        obsolete.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<GameScenarioRequest>(obsolete).is_err(),
            "{field}"
        );
    }
    let mut malformed = result.report;
    malformed.relative_elapsed_us += 1;
    assert!(malformed.validate_clocks().is_err());
    malformed.relative_elapsed_us -= 1;
    malformed.timer_pauses.as_mut().unwrap()[0].end_absolute_us = malformed.absolute_elapsed_us + 1;
    assert!(malformed.validate_clocks().is_err());
}

#[test]
fn source_clock_live_cooldown_and_modifier_queries_freeze_through_an_absolute_wait() {
    let source = "options duration=40 iteration=1 crit=fixed;
        sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120;
        target hp=1000000000 def=0 resist=0; active sakiri;
        sakiri burst; let before=.time; let cooldown=.sakiri.burst.cd;
        let buff=.sakiri.mods.team_atk; hotori burst;
        if .time!=before || .sakiri.burst.cd!=cooldown || .sakiri.mods.team_atk!=buff {let assertion_failure=1/0;}
        wait(1);
        if .time!=before || .sakiri.burst.cd!=cooldown || .sakiri.mods.team_atk!=buff {let assertion_failure=1/0;}";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.relative_elapsed_us, 40_000_000);
    assert_native_replay(&result);
    assert!(!result.report.timer_catalog.as_ref().unwrap().is_empty());
    let mut invalid = result.report.clone();
    invalid.timer_catalog.as_mut().unwrap()[0].clock_basis =
        ntesim_core::kernel::timing::ClockDomain::Absolute;
    assert!(invalid.validate_clocks().is_err());
    let mut missing = result.report.clone();
    missing.state_timeline.last_mut().unwrap().timers = None;
    assert!(missing.validate_clocks().is_err());
    let mut invalid = result.report.clone();
    invalid.state_timeline[0].timers = Some(vec![crate::simulation::GameTimerRef {
        timer_def_id: 0,
        expires_at_us: 0,
    }]);
    assert!(invalid.validate_clocks().is_err());
}

#[test]
fn source_clock_paused_execution_keeps_absolute_event_and_operation_limits_independent() {
    let source = "options duration=20 iteration=1 crit=fixed;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120;
        target hp=1000000000 def=0 resist=0; active hotori; hotori burst;";
    for (limits, expected_absolute, message) in [
        (
            ExecutionLimits {
                max_horizon_us: 1_000_000,
                ..Default::default()
            },
            1_000_000,
            "absolute",
        ),
        (
            ExecutionLimits {
                max_events: 2,
                ..Default::default()
            },
            2_183_333,
            "event",
        ),
    ] {
        let result = PreparedSourceScenario::new(source, Default::default(), &limits)
            .unwrap()
            .run(42)
            .unwrap();
        assert!(!result.report.execution_completed);
        assert_eq!(result.report.relative_elapsed_us, 0);
        assert_eq!(result.report.absolute_elapsed_us, expected_absolute);
        assert!(
            result
                .report
                .halt_diagnostic
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains(message),
            "{:?}",
            result.diagnostic
        );
        assert_native_replay(&result);
    }
    let program = PreparedSourceScenario::new(
        &format!("{source} while .time==0 {{let value=1;}}"),
        crate::dsl::compiler::CompileLimits {
            max_steps: 200,
            ..Default::default()
        },
        &Default::default(),
    )
    .unwrap();
    let result = program.run(42).unwrap();
    assert!(!result.report.success && !result.report.execution_completed);
    assert_eq!(result.report.relative_elapsed_us, 0);
    assert_eq!(result.report.absolute_elapsed_us, 4_604_238);
    assert!(
        result
            .report
            .halt_diagnostic
            .as_ref()
            .unwrap()
            .contains("limit 200 operations")
    );
    assert_native_replay(&result);

    let declarations = source
        .replace(
            "target hp=",
            "hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=",
        )
        .replace(
            "active hotori;",
            "incoming hotori at=25 damage=0;
        dummy attack_start at=25 actor=hotori;
        interrupt hotori at=25; combat hathor at=26 leaving=true; active hotori;",
        );
    let result =
        PreparedSourceScenario::new(&declarations, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.relative_elapsed_us, 20_000_000);
    assert_eq!(result.report.absolute_elapsed_us, 34_183_333);
    assert_eq!(result.report.interrupt_receipts[0].time_us, 25_000_000);
    assert_eq!(
        result.report.combat_transition_receipts[0].time_us,
        26_000_000
    );
    assert!(result.report.incoming_damage_receipts.is_empty()); // World delay lies beyond R=20.
    assert_native_replay(&result);
    let later = declarations
        .replace("at=25", "at=4500")
        .replace("at=26", "at=4501");
    let larger = PreparedSourceScenario::new(
        &later,
        Default::default(),
        &ExecutionLimits {
            max_horizon_us: 5_000_000_000,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(larger.export().request.absolute_limit_us, 5_000_000_000);
    assert!(
        PreparedSourceScenario::new(
            &declarations,
            Default::default(),
            &ExecutionLimits {
                max_horizon_us: 24_000_000,
                ..Default::default()
            }
        )
        .is_err()
    );
}

#[test]
fn source_clock_queue_retains_periodic_phase_and_new_timer_inside_overlap() {
    use crate::simulation::events::{GameEventData, ScheduledGameEvent};
    use ntesim_core::kernel::timing::{ClockDomain, ModePauseKind, PauseOwner, RuntimeModeClock};
    let mut clock = RuntimeModeClock::default();
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let event = |time_us, seq| ScheduledGameEvent {
        time_us,
        seq,
        priority: crate::execution::CausalPriority::BuffTickOrExpiry,
        data: GameEventData::ReactionRemoraEnd {
            target_index: 0,
            generation: seq,
        },
    };
    queue.push(event(250_000, 1));
    assert_eq!(queue.peek().unwrap().time_us, 250_000);
    let first = PauseOwner {
        owner: 0,
        generation: 1,
    };
    let second = PauseOwner {
        owner: 1,
        generation: 2,
    };
    clock
        .start_pause(first, ModePauseKind::ActiveStop, 0)
        .unwrap();
    queue.sync(&clock).unwrap();
    assert!(queue.peek().is_none()); // Same-timestamp pause invalidates the projected head.
    queue.push(event(0, 3)); // Already-due causal work still settles at entry.
    assert_eq!(queue.peek().unwrap().time_us, 0);
    assert_eq!(queue.pop().unwrap().seq, 3);
    assert!(queue.peek().is_none());
    clock.advance_to(5_000_000).unwrap();
    queue.sync(&clock).unwrap();
    queue.push(event(7_000_000, 2)); // A new two-second timer inside the pause.
    queue.push(ScheduledGameEvent {
        time_us: 7_000_000,
        seq: 4,
        priority: crate::execution::CausalPriority::BuffTickOrExpiry,
        data: GameEventData::Content(crate::callbacks::ContentEvent::JiuyuanPactTick {
            actor_index: 0,
            target_index: 0,
            planted_us: 5_000_000,
        }),
    });
    clock
        .start_pause(second, ModePauseKind::Cinematic, 5_000_000)
        .unwrap();
    queue.sync(&clock).unwrap();
    clock.end_pause(first, 10_000_000).unwrap();
    queue.sync(&clock).unwrap();
    assert!(queue.peek().is_none());
    clock.end_pause(second, 12_000_000).unwrap();
    queue.sync(&clock).unwrap();
    assert_eq!(queue.peek().unwrap().time_us, 12_250_000);
    assert_eq!(queue.pop().unwrap().time_us, 12_250_000);
    assert_eq!(queue.peek().unwrap().time_us, 14_000_000);
    assert_eq!(queue.pop().unwrap().time_us, 14_000_000);
    let pact = queue.pop().unwrap();
    assert_eq!(pact.time_us, 14_000_000);
    assert!(matches!(
        pact.data,
        GameEventData::Content(crate::callbacks::ContentEvent::JiuyuanPactTick {
            planted_us: 12_000_000,
            ..
        })
    ));
    assert_eq!(clock.elapsed_us(ClockDomain::World), Some(0));
    assert!(queue.is_empty());
}

#[test]
fn source_clock_finite_switch_gates_finish_through_their_own_cinematic() {
    let source = "options duration=20 iteration=1 crit=fixed;
        daffodil char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        chaos char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=1000000000 def=0 resist=0; active daffodil;
        daffodil burst; mint swap; chaos burst; wait(0.029547); mint swap;";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.relative_elapsed_us, 20_000_000);
    assert_eq!(result.report.timer_bindings.len(), 2);
    assert_native_replay(&result);
}
