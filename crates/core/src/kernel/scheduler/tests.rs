use super::*;
#[test]
fn clock_readings_preserve_fractional_phase_rates_and_pauses_without_mutation() {
    let mut queue = Scheduler::default();
    queue.add_clock(0, 500_001).unwrap();
    queue.add_clock(1, 0).unwrap();
    queue.schedule_after(0, 1, None, "event").unwrap();
    queue.pop_before(2).unwrap().unwrap();
    let reading = queue.clock_readings().unwrap();
    assert_eq!(
        reading[&0],
        ClockReading {
            local_time_us: 1,
            phase_millionths: 2,
            rate_ppm: 500_001
        }
    );
    assert_eq!(
        reading[&1],
        ClockReading {
            local_time_us: 0,
            phase_millionths: 0,
            rate_ppm: 0
        }
    );
    assert_eq!(reading, queue.clock_readings().unwrap());
    assert_eq!(queue.pending_count(), 0);
    queue.set_rate(0, 333_333).unwrap();
    queue.pop_before(5).unwrap();
    assert_eq!(
        queue.clock_readings().unwrap()[&0],
        ClockReading {
            local_time_us: 2,
            phase_millionths: 1,
            rate_ppm: 333_333
        }
    );
}
#[test]
fn invalid_clock_readings_fail_without_truncating_or_mutating_phase() {
    let mut queue: Scheduler<()> = Scheduler::default();
    queue.add_clock(0, 1_000_000).unwrap();
    queue.clocks.get_mut(&0).unwrap().phase = u128::MAX;
    assert_eq!(queue.clock_readings(), Err(ScheduleError::Overflow));
    assert_eq!(queue.clocks[&0].phase, u128::MAX);
    assert_eq!(queue.real_time_us(), 0);
}
#[test]
fn causal_batch_matches_individual_scheduling_with_fractional_clock_phase() {
    let mut individual = queue();
    let mut batch = queue();
    for q in [&mut individual, &mut batch] {
        q.set_rate(1, RATE_SCALE / 2).unwrap();
        q.pop_before(1).unwrap();
        q.schedule_after(0, 0, None, "parent").unwrap();
        q.pop_before(1).unwrap().unwrap();
    }
    assert_eq!(
        batch
            .schedule_batch_after(1, 3, Some(0), vec!["a", "b", "c"])
            .unwrap(),
        [1, 2, 3]
    );
    for value in ["a", "b", "c"] {
        individual.schedule_after(1, 3, Some(0), value).unwrap();
    }
    for _ in 0..3 {
        let expected = individual.pop_before(10).unwrap().unwrap();
        let actual = batch.pop_before(10).unwrap().unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.real_time_us, 7);
    }
}
#[test]
fn batch_errors_leave_ids_pending_tasks_and_clocks_unchanged() {
    let mut q = queue();
    let pending = q.schedule_after(0, 3, None, "existing").unwrap();
    q.next_id = u64::MAX - 1;
    assert_eq!(
        q.schedule_batch_after(0, 0, None, vec!["a", "b"]),
        Err(ScheduleError::Overflow)
    );
    assert_eq!(q.next_id, u64::MAX - 1);
    assert_eq!(q.pending_count(), 1);
    assert_eq!(q.real_time_us(), 0);
    assert_eq!(
        q.schedule_batch_after(9, 0, None, vec![]),
        Err(ScheduleError::UnknownClock(9))
    );
    assert_eq!(
        q.schedule_batch_after(0, 0, Some(pending), vec![]),
        Err(ScheduleError::UnknownCause(pending))
    );
    assert!(
        q.schedule_batch_after(0, 0, None, vec![])
            .unwrap()
            .is_empty()
    );
    assert_eq!(q.pop_before(3).unwrap().unwrap().payload, "existing");
}
fn queue() -> Scheduler<&'static str> {
    let mut q = Scheduler::default();
    q.add_clock(0, RATE_SCALE).unwrap();
    q.add_clock(1, RATE_SCALE).unwrap();
    q
}
#[test]
fn simultaneous_clocks_and_immediate_children_follow_global_order() {
    let mut q = queue();
    q.schedule_after(1, 10, None, "first").unwrap();
    q.schedule_after(0, 10, None, "second").unwrap();
    let first = q.pop_before(10).unwrap().unwrap();
    q.schedule_after(1, 0, Some(first.event_id), "child")
        .unwrap();
    assert_eq!(q.pop_before(10).unwrap().unwrap().payload, "second");
    let child = q.pop_before(10).unwrap().unwrap();
    assert_eq!(child.payload, "child");
    assert_eq!(child.cause, Some(first.event_id));
}
#[test]
fn paused_mode_clock_retains_deadlines_across_real_time() {
    let mut q = queue();
    q.set_rate(1, 0).unwrap();
    q.schedule_after(1, 20, None, "mode").unwrap();
    assert!(q.pop_before(100).unwrap().is_none());
    assert_eq!(q.local_time_us(1).unwrap(), 0);
    q.set_rate(1, RATE_SCALE).unwrap();
    assert_eq!(q.pop_before(120).unwrap().unwrap().real_time_us, 120);
}
#[test]
fn rate_changes_keep_fractional_phase_and_reorder_tasks() {
    let mut q = queue();
    q.set_rate(1, RATE_SCALE / 2).unwrap();
    q.schedule_after(1, 2, None, "slow").unwrap();
    q.schedule_after(0, 3, None, "normal").unwrap();
    assert!(q.pop_before(1).unwrap().is_none());
    q.set_rate(1, RATE_SCALE).unwrap();
    let event = q.pop_before(3).unwrap().unwrap();
    assert_eq!(event.payload, "slow");
    assert_eq!(event.real_time_us, 3);
    assert_eq!(q.pop_before(3).unwrap().unwrap().payload, "normal");
}
#[test]
fn canceled_event_has_no_delivery_or_time_effect() {
    let mut q = queue();
    let canceled = q.schedule_after(0, 3, None, "gone").unwrap();
    q.cancel(canceled).unwrap();
    assert_eq!(q.pending_count(), 0);
    assert!(q.pop_before(1).unwrap().is_none());
    assert_eq!(q.real_time_us(), 1);
    assert!(q.cancel(canceled).is_err());
    assert!(q.schedule_after(0, 0, Some(canceled), "bad").is_err());
}
#[test]
fn illegal_operations_do_not_mutate_queue() {
    let mut q = queue();
    assert!(q.add_clock(0, 0).is_err());
    assert!(q.schedule_after(2, 1, None, "unknown").is_err());
    assert_eq!(q.pending_count(), 0);
    q.pop_before(10).unwrap();
    assert!(q.pop_before(9).is_err());
    assert_eq!(q.real_time_us(), 10);
    q.next_id = u64::MAX;
    assert!(q.schedule_after(0, 1, None, "overflow").is_err());
    assert_eq!(q.pending_count(), 0);
}
#[test]
fn generated_schedules_obey_monotonicity_and_exactly_once_delivery() {
    for seed in 0_u64..64 {
        let mut q = queue();
        let mut state = seed + 1;
        let mut expected = Vec::new();
        for id in 0..128 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let time = state % 97;
            let clock = (state % 2) as u32;
            q.schedule_after(clock, time, None, "event").unwrap();
            expected.push((time, id));
        }
        expected.sort();
        let mut actual = Vec::new();
        while let Some(e) = q.pop_before(100).unwrap() {
            actual.push((e.real_time_us, e.event_id));
        }
        assert_eq!(actual, expected);
        assert_eq!(q.pending_count(), 0);
        assert_eq!(q.real_time_us(), 100);
    }
}

#[test]
fn faster_clock_deadlines_sharing_a_real_tick_use_global_ids() {
    let mut q = queue();
    q.set_rate(1, 2 * RATE_SCALE).unwrap();
    q.schedule_after(1, 2, None, "first").unwrap();
    q.schedule_after(1, 1, None, "second").unwrap();
    q.schedule_after(0, 1, None, "third").unwrap();
    for expected in ["first", "second", "third"] {
        let event = q.pop_before(1).unwrap().unwrap();
        assert_eq!(event.payload, expected);
        assert_eq!(event.real_time_us, 1);
    }
}
#[test]
fn clock_overflow_rejects_advance_without_partial_mutation() {
    let mut q = queue();
    q.set_rate(1, u32::MAX).unwrap();
    assert_eq!(q.pop_before(u64::MAX), Err(ScheduleError::Overflow));
    assert_eq!(q.real_time_us(), 0);
    assert_eq!(q.local_time_us(0).unwrap(), 0);
}
#[test]
#[allow(clippy::manual_div_ceil)] // Independent integer formula used as the test oracle.
fn generated_rate_combinations_match_independent_deadline_sorting() {
    for slow in [
        RATE_SCALE / 4,
        RATE_SCALE / 2,
        RATE_SCALE,
        3 * RATE_SCALE / 2,
        2 * RATE_SCALE,
    ] {
        for fast in [RATE_SCALE / 2, RATE_SCALE, 2 * RATE_SCALE] {
            let mut q = queue();
            q.set_rate(0, slow).unwrap();
            q.set_rate(1, fast).unwrap();
            let mut expected = Vec::new();
            for id in 0..96_u64 {
                let clock = (id % 2) as u32;
                let delay = (id * 31 + 7) % 89;
                let rate = u64::from(if clock == 0 { slow } else { fast });
                q.schedule_after(clock, delay, None, "event").unwrap();
                expected.push(((delay * u64::from(RATE_SCALE) + rate - 1) / rate, id));
            }
            expected.sort();
            let mut actual = Vec::new();
            while let Some(e) = q.pop_before(400).unwrap() {
                actual.push((e.real_time_us, e.event_id));
            }
            assert_eq!(actual, expected);
        }
    }
}
