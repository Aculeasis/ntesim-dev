//! Scheduler acceptance with an explicit internal caller-timing fixture. Public
//! scripts cannot override hit delays; owner source bindings are checked apart.
use super::*;

#[test]
fn source_clock_relative_horizon_observes_twenty_seconds_of_actual_pause_without_polling() {
    let mut request = crate::simulation::tests::research_request();
    request.relative_duration_us = 120_000_000;
    request.actors[0].character_id = "character:1046".into();
    request.actions.clear();
    request.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 140_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let incoming = crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID;
    request
        .model_assumptions
        .insert(incoming.into(), incoming.description().into());
    request
        .incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 141_000_000,
            actor_index: 0,
            delivered_damage: 1,
        });
    request.source_exhausted = true;
    request.action_admission_order =
        crate::simulation::GameActionAdmissionOrder::AfterSettledEvents;
    let limits = ExecutionLimits::default();
    let (mut execution, lifecycle) = PreparedExecution::new(&request, 42, &[], &[]).unwrap();
    execution.state.actors[0].locked_until_us = 20_000_000;
    crate::simulation::pause::committed(
        &mut execution.state,
        &request,
        &mut execution.queue,
        &mut execution.seq,
        0,
        0,
        Some("GA_Zero_UltraSkill"),
    )
    .unwrap();
    execution
        .process_events(&request, &limits, &[], &[], &[])
        .unwrap();
    let report = execution.into_report(&request, 42, lifecycle).unwrap();
    assert!(report.success && report.execution_completed);
    assert_eq!(report.relative_elapsed_us, 120_000_000);
    assert_eq!(report.absolute_elapsed_us, 140_000_000);
    assert_eq!(report.total_events_processed, 1); // The actual release; no timer polling.
    assert_eq!(report.final_buff_state.active_actor_index, 0); // No endpoint input.
    assert_eq!(report.incoming_damage_receipts.len(), 0);
    assert!(report.pending_events >= 2); // Endpoint input and later damage.
    report.validate_clocks().unwrap();
}

#[test]
fn source_victory_settles_delayed_contact_during_wait_and_energy_wait() {
    for wait in ["wait(10)", "wait_for_energy(120)"] {
        let source = format!(
            "options duration=5 stop=victory crit=fixed;
            sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
            target hp=1 def=0 resist=0; active sakiri;
            sakiri attack; {wait}; sakiri attack;"
        );
        let mut program =
            PreparedSourceScenario::new(&source, Default::default(), &Default::default()).unwrap();
        let descriptor = program.config.actors[0]
            .projection
            .public_actions
            .get_mut("attack")
            .unwrap();
        let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut descriptor.command else {
            panic!("ordinary single-contact fixture")
        };
        *hit_delay_us = 2_000_000;
        let result = program.run(42).unwrap();
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        assert_eq!(result.report.absolute_elapsed_us, 2_000_000);
        assert_eq!(result.occurrences.len(), 2);
        assert_eq!(result.occurrences[1].completed_us, Some(2_000_000));
        crate::simulation::tests::assert_native_replay(&result);
    }
}

#[test]
fn source_resonance_phantom_queries_follow_native_handle_and_self_eyes_expiry() {
    let source = "options duration=35 crit=fixed;
        daffodil char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_resonance=2;
        target hp=100000000 def=0 resist=0; active daffodil;
        daffodil burst;
        if .daffodil.resources.resonance!=2 || .daffodil.resources.phantom_steps!=2
            || !.daffodil.phantom_handle.active || !.daffodil.eyes.active
            || .daffodil.mods.phantom_handle!=30-.time || .daffodil.mods.eyes!=30-.time {let assertion_failure=1/0;}
        while .daffodil.mods.phantom_handle>0.000001 {wait(.daffodil.mods.phantom_handle-0.000001);}
        if .daffodil.resources.phantom_steps!=2 || .daffodil.mods.phantom_handle!=0.000001 {let assertion_failure=1/0;}
        wait(0.000001);
        if .daffodil.resources.resonance!=2 || .daffodil.resources.phantom_steps!=0
            || .daffodil.phantom_handle.active || .daffodil.eyes.active
            || .daffodil.mods.phantom_handle!=0 || .daffodil.mods.eyes!=0 {let assertion_failure=1/0;}";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    crate::simulation::tests::assert_native_replay(&result);
}

#[test]
fn source_next_normal_follows_the_native_evolved_blackbird_chain() {
    let source = "options duration=30 crit=fixed;
        blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active blackbird;
        blackbird burst;
        if !.blackbird.transformed.active {let assertion_failure=1/0;}
        for let i=1; i<=5; i=i+1 {
            if .blackbird.normal!=i {let assertion_failure=1/0;}
            blackbird attack;
        }
        if .blackbird.normal!=1 || .blackbird.mods.skill_permission<=0 {let assertion_failure=1/0;}
        wait(21.5-.time);
        if .blackbird.transformed.active || .blackbird.mods.skill_permission!=0 {let assertion_failure=1/0;}";
    let result =
        PreparedSourceScenario::new(source, Default::default(), &ExecutionLimits::default())
            .unwrap()
            .run(42)
            .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    for (index, count) in [2, 5, 6, 1, 6].into_iter().enumerate() {
        // 20 primary packets; the initial 20 Malice also funds all 20 extras.
        for suffix in ["Damage", "Extra_Damage"] {
            let effect = format!("GE_Player_BlackBird_Evolve_Melee{}_{suffix}", index + 1);
            assert_eq!(
                result
                    .report
                    .hit_receipts
                    .iter()
                    .filter(|hit| hit.command_index == Some(index + 1)
                        && hit.damage_effect_id == effect)
                    .count(),
                count,
                "{effect}"
            );
        }
    }
    let replay =
        crate::simulation::run_game_scenario_report(&result.request, &Default::default()).unwrap();
    let actual = serde_json::to_value(&result.report).unwrap();
    let replay = serde_json::to_value(replay).unwrap();
    for (key, value) in actual.as_object().unwrap() {
        assert!(
            value == &replay[key],
            "evolved Blackbird replay field {key}"
        );
    }
}

#[test]
fn source_crime_flag_settles_native_dread_echo_final_tick_before_cleanup() {
    let source = "options duration=20 crit=fixed;
        chaos char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_crime=100;
        target hp=100000000 def=0 resist=0; active chaos;
        if .chaos.dread_echo.active {let assertion_failure=1/0;}
        chaos burst;
        if !.chaos.dread_echo.active {let assertion_failure=1/0;}
        while .time<12 {wait(12-.time);}
        if .chaos.dread_echo.active || .chaos.resources.crime!=1000 {let assertion_failure=1/0;}";
    let program =
        PreparedSourceScenario::new(source, Default::default(), &ExecutionLimits::default())
            .unwrap();
    let result = program.run(42).unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let actor = serde_json::to_value(&result.report.final_actor_states[0]).unwrap();
    let pool = &actor["fractional_resources"]["chaos.crime"];
    assert_eq!(pool["granted"], serde_json::json!("1800/1"));
    assert_eq!(pool["overflow"], serde_json::json!("900/1"));
}

#[test]
fn source_energy_gate_suspends_until_delayed_grants_then_rechecks_the_branch() {
    let source = "options duration=20 crit=fixed;
        sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=100000000 def=0 resist=0; active sakiri;
        sakiri attack; wait_for_energy(1); if .time==2 && .actor.energy>=1 {wait(0.3);} else {wait(9);}";
    let mut program =
        PreparedSourceScenario::new(source, Default::default(), &ExecutionLimits::default())
            .unwrap();
    let descriptor = program.config.actors[0]
        .projection
        .public_actions
        .get_mut("attack")
        .unwrap();
    let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut descriptor.command else {
        panic!("normal fixture")
    };
    *hit_delay_us = 2_000_000;
    let result = program.run(42).unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let gate = &result.occurrences[1];
    assert_eq!(gate.started_us, 800_000);
    assert_eq!(gate.completed_us, Some(2_000_000));
    let branch = &result.occurrences[2];
    assert_eq!(branch.started_us, 2_000_000);
    assert_eq!(branch.completed_us, Some(2_300_000));
    assert_eq!(result.report.total_events_processed, 2);
    let interval = result
        .report
        .input_time
        .intervals
        .iter()
        .find(|interval| interval.cause == crate::simulation::GameTimeCause::EnergyGate)
        .unwrap();
    assert_eq!(
        (interval.start_us, interval.end_us, interval.duration_us),
        (800_000, 2_000_000, 1_200_000)
    );
    assert_eq!(interval.energy.as_ref().unwrap().required, 1);
    assert_eq!(interval.energy.as_ref().unwrap().available, 0);
    assert_eq!(interval.source.as_ref().unwrap().loc, gate.loc);
    crate::simulation::tests::assert_native_replay(&result);
}

#[test]
fn long_native_recovery_is_not_a_player_idle_warning() {
    let source = "options duration=6 crit=fixed;
        sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active sakiri; sakiri attack;";
    let mut program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let descriptor = program.config.actors[0]
        .projection
        .public_actions
        .get_mut("attack")
        .unwrap();
    let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut descriptor.command else {
        panic!("caller-timed fixture")
    };
    *recovery_us = 6_000_000;
    let result = program.run(42).unwrap();
    assert!(result.report.success && result.report.execution_completed);
    assert!(result.report.input_time.warnings.is_empty());
    assert_eq!(result.report.input_time.totals.len(), 1);
    assert_eq!(
        result.report.input_time.totals[0].cause,
        crate::simulation::GameTimeCause::ActionRecovery
    );
    assert_eq!(result.report.input_time.totals[0].duration_us, 6_000_000);
    crate::simulation::tests::assert_native_replay(&result);
}

#[test]
fn source_clock_world_buff_cooldown_and_proc_icd_retain_independent_remainders() {
    use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
    use ntesim_core::kernel::timing::{ModePauseKind, PauseOwner};
    let mut request = crate::simulation::tests::research_request();
    request.actions.clear();
    let (mut execution, _) = PreparedExecution::new(&request, 42, &[], &[]).unwrap();
    let declaration = || {
        crate::simulation::state::TimedBuffDeclaration::reset(
            "test.world",
            "timer.test",
            BuffRecipient::SelfActor,
            2_000_000,
            BuffStackingScope::PerSource,
            Default::default(),
        )
    };
    execution.state.apply_buff(0, declaration()).unwrap();
    execution.state.actors[0]
        .ability_cooldowns
        .insert("before".into(), 3_000_000);
    execution.state.actors[0].equipment.moon.ready_at_us = 3_000_000;
    let pause = PauseOwner {
        owner: 1,
        generation: 123,
    };
    execution
        .state
        .mode_clock
        .start_pause(pause, ModePauseKind::ActiveStop, 0)
        .unwrap();
    execution.queue.sync(&execution.state.mode_clock).unwrap();
    execution.advance_clock(&request, 5_000_000).unwrap();
    execution.state.actors[0]
        .ability_cooldowns
        .insert("inside".into(), 7_000_000);
    execution.state.apply_buff(1, declaration()).unwrap();
    execution.advance_clock(&request, 10_000_000).unwrap();
    execution.state.expire_buffs(&mut execution.trace);
    assert_eq!(execution.state.mode_clock.relative_us(), 0);
    assert_eq!(
        execution.state.actors[0]
            .ability_cooldowns
            .remaining_us("before"),
        Some(3_000_000)
    );
    assert_eq!(
        execution.state.actors[0]
            .ability_cooldowns
            .remaining_us("inside"),
        Some(2_000_000)
    );
    assert_eq!(
        execution.state.actors[0].equipment.moon.ready_at_us,
        13_000_000
    );
    let timers = execution.timeline.encode_timers(&execution.state).unwrap();
    for (key, expected) in [
        ("cooldown:before", 3_000_000),
        ("cooldown:inside", 2_000_000),
    ] {
        let timer = timers
            .iter()
            .find(|timer| execution.timeline.timer_catalog[timer.timer_def_id].key == key)
            .unwrap();
        assert_eq!(timer.expires_at_us, expected);
        assert_eq!(
            execution.timeline.timer_catalog[timer.timer_def_id].clock_basis,
            ntesim_core::kernel::timing::ClockDomain::World
        );
    }
    assert_eq!(
        execution
            .state
            .buffs
            .instances()
            .iter()
            .filter(|buff| buff.id == "test.world")
            .count(),
        2
    );
    execution
        .state
        .buffs
        .remove_owned_buff("test.world", "timer.test", 1);
    assert_eq!(
        execution
            .state
            .buffs
            .instances()
            .iter()
            .filter(|buff| buff.id == "test.world")
            .count(),
        1
    );
    execution
        .state
        .mode_clock
        .end_pause(pause, 10_000_000)
        .unwrap();
    execution.queue.sync(&execution.state.mode_clock).unwrap();
    execution.advance_clock(&request, 11_999_999).unwrap();
    execution.state.expire_buffs(&mut execution.trace);
    assert_eq!(
        execution
            .state
            .buffs
            .instances()
            .iter()
            .filter(|buff| buff.id == "test.world")
            .count(),
        1
    );
    execution.advance_clock(&request, 12_000_000).unwrap();
    execution.state.expire_buffs(&mut execution.trace);
    assert!(
        !execution
            .state
            .buffs
            .instances()
            .iter()
            .any(|buff| buff.id == "test.world")
    );
    assert_eq!(
        execution.state.actors[0]
            .ability_cooldowns
            .remaining_us("before"),
        Some(1_000_000)
    );
    assert_eq!(
        execution.state.actors[0]
            .ability_cooldowns
            .remaining_us("inside"),
        Some(0)
    );
    execution.advance_clock(&request, 13_000_000).unwrap();
    assert_eq!(
        execution.state.actors[0]
            .ability_cooldowns
            .remaining_us("before"),
        Some(0)
    );
}

#[test]
fn source_clock_object_expires_during_active_stop_without_a_healing_backlog() {
    use crate::simulation::events::{GameEventData, ScheduledGameEvent};
    use ntesim_core::kernel::timing::{ClockDomain, ModePauseKind, PauseOwner};
    let mut request = crate::simulation::tests::research_request();
    request.actions.clear();
    request.actors[0].character_id = "character:1021".into();
    let (mut execution, _) = PreparedExecution::new(&request, 42, &[], &[]).unwrap();
    execution.state.actors[0]
        .character
        .edgar_mut()
        .unwrap()
        .field = Some(crate::characters::edgar::effects::Field {
        generation: 123,
        coordinate: 1,
        until: 2_000_000,
    });
    let event = |time_us, seq, event| ScheduledGameEvent {
        time_us,
        seq,
        priority: crate::execution::CausalPriority::Observation,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Edgar(event)),
    };
    execution.queue.push_on(
        ClockDomain::World,
        event(
            1_000_000,
            1,
            crate::characters::edgar::clock::Event::FieldTick {
                owner: 0,
                generation: 123,
            },
        ),
    );
    execution.queue.push_on(
        ClockDomain::ObjectLifetime,
        event(
            2_000_000,
            2,
            crate::characters::edgar::clock::Event::FieldExpiry {
                owner: 0,
                generation: 123,
            },
        ),
    );
    let pause = PauseOwner {
        owner: 1,
        generation: 100,
    };
    execution
        .state
        .mode_clock
        .start_pause(pause, ModePauseKind::ActiveStop, 0)
        .unwrap();
    execution.queue.sync(&execution.state.mode_clock).unwrap();
    let timers = execution.timeline.encode_timers(&execution.state).unwrap();
    let field = timers
        .iter()
        .find(|timer| execution.timeline.timer_catalog[timer.timer_def_id].key == "field")
        .unwrap();
    assert_eq!(field.expires_at_us, 2_000_000);
    assert_eq!(
        execution.timeline.timer_catalog[field.timer_def_id].clock_basis,
        ClockDomain::ObjectLifetime
    );
    execution
        .advance_to(&request, &Default::default(), &[], 4_000_000)
        .unwrap();
    assert_eq!(execution.state.current_time_us, 4_000_000);
    assert_eq!(execution.state.mode_clock.relative_us(), 0);
    assert!(
        execution.state.actors[0]
            .character
            .edgar_mut()
            .unwrap()
            .field
            .is_none()
    );
    assert!(execution.queue.is_empty());
    assert!(execution.state.healing_receipts.is_empty());
    assert_eq!(execution.events_processed, 1); // Actual expiry only; no polling or emissions.
    execution
        .state
        .mode_clock
        .end_pause(pause, 4_000_000)
        .unwrap();
    execution.queue.sync(&execution.state.mode_clock).unwrap();
    execution
        .advance_to(&request, &Default::default(), &[], 6_000_000)
        .unwrap();
    assert!(execution.state.healing_receipts.is_empty());
}

#[test]
fn source_clock_hotori_defeat_clips_setup_and_retires_stale_entry_and_deadline() {
    let request = crate::dsl::compiler::compile_scenario(
        "options duration=40 crit=fixed; hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120; target hp=1000000000 def=0 resist=0; active hotori; hotori burst;",
        Default::default()).unwrap().request;
    let packets =
        crate::simulation::validation::validate_and_resolve(&request, &Default::default()).unwrap();
    let (mut execution, _) = PreparedExecution::new(&request, 42, &[], &[]).unwrap();
    execution
        .advance_to(&request, &Default::default(), &packets, 1_000_000)
        .unwrap();
    execution.state.actors[0].hp = 0;
    let mut callback = crate::runtime_api::CallbackContext {
        state: &mut execution.state,
        request: &request,
        queue: &mut execution.queue,
        sequence: &mut execution.seq,
        trace: &mut execution.trace,
    };
    crate::registry::on_actor_defeated(&mut callback, 0).unwrap();
    // Duplicate cleanup and the retired mode's callbacks cannot reopen it.
    crate::registry::on_actor_defeated(&mut callback, 0).unwrap();
    crate::characters::hotori::timepiece::event(
        &mut callback,
        crate::characters::hotori::timepiece::Event::ModeEntry {
            owner: 0,
            generation: 1,
        },
    )
    .unwrap();
    crate::characters::hotori::timepiece::event(
        &mut callback,
        crate::characters::hotori::timepiece::Event::ModeDeadline {
            owner: 0,
            generation: 1,
        },
    )
    .unwrap();
    assert!(!execution.state.mode_clock.is_paused());
    let pauses = execution
        .state
        .mode_clock
        .observed_pauses(1_000_000)
        .unwrap();
    assert_eq!(pauses.len(), 1);
    assert_eq!(
        (pauses[0].start_absolute_us, pauses[0].end_absolute_us),
        (0, 1_000_000)
    );
    let timepiece = &execution.state.actors[0]
        .character
        .hotori_mut()
        .unwrap()
        .timepiece;
    assert!(timepiece.mode.is_none() && !timepiece.passive);
    assert_eq!(
        timepiece.pool.current(),
        crate::simulation::ExactFraction::default()
    );
    execution.advance_clock(&request, 2_000_000).unwrap();
    assert_eq!(execution.state.mode_clock.relative_us(), 1_000_000);
}

#[test]
fn source_clock_ordinary_defeat_clips_notify_and_completion_cinematics() {
    for owner in ["mint", "zero", "blackbird"] {
        let request = crate::dsl::compiler::compile_scenario(&format!(
            "options duration=20 crit=fixed; {owner} char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=1000000000 def=0 resist=0; active {owner}; {owner} burst;"),
            Default::default()).unwrap().request;
        let packets =
            crate::simulation::validation::validate_and_resolve(&request, &Default::default())
                .unwrap();
        let (mut execution, _) = PreparedExecution::new(&request, 42, &[], &[]).unwrap();
        execution
            .advance_to(&request, &Default::default(), &packets, 1_000_000)
            .unwrap();
        assert!(execution.state.mode_clock.is_paused(), "{owner}");
        execution.state.actors[0].hp = 0;
        let mut callback = crate::runtime_api::CallbackContext {
            state: &mut execution.state,
            request: &request,
            queue: &mut execution.queue,
            sequence: &mut execution.seq,
            trace: &mut execution.trace,
        };
        crate::registry::on_actor_defeated(&mut callback, 0).unwrap();
        crate::registry::on_actor_defeated(&mut callback, 0).unwrap();
        assert!(!execution.state.mode_clock.is_paused(), "{owner}");
        execution.queue.sync(&execution.state.mode_clock).unwrap();
        execution
            .advance_to(&request, &Default::default(), &packets, 8_000_000)
            .unwrap();
        let pauses = execution
            .state
            .mode_clock
            .observed_pauses(8_000_000)
            .unwrap();
        assert_eq!(pauses.len(), 1, "{owner}");
        assert_eq!(
            (pauses[0].start_absolute_us, pauses[0].end_absolute_us),
            (0, 1_000_000),
            "{owner}"
        );
        assert_eq!(
            execution.state.mode_clock.relative_us(),
            7_000_000,
            "{owner}"
        );
    }
}
