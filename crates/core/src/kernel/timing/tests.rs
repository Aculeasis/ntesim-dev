use super::*;

#[test]
fn projected_world_deadlines_retain_remainders_for_new_and_existing_timers() {
    let mut deadlines = ProjectedDeadlines::<&str>::default();
    deadlines.insert("cooldown", 3_000_000);
    deadlines.advance(10_000_000, 0).unwrap();
    assert_eq!(deadlines.remaining_us("cooldown"), Some(3_000_000));
    assert_eq!(deadlines.get("cooldown"), Some(&13_000_000));
    deadlines.insert("new_buff", 12_000_000);
    deadlines.advance(20_000_000, 0).unwrap();
    assert_eq!(deadlines.remaining_us("new_buff"), Some(2_000_000));
    assert_eq!(deadlines.get("new_buff"), Some(&22_000_000));
    deadlines.advance(22_000_000, 2_000_000).unwrap();
    assert_eq!(deadlines.remaining_us("new_buff"), Some(0));
    assert_eq!(deadlines.remaining_us("cooldown"), Some(1_000_000));
    deadlines.advance(32_000_000, 2_000_000).unwrap();
    assert_eq!(deadlines.get("new_buff"), Some(&22_000_000));
    assert_eq!(deadlines.get("cooldown"), Some(&33_000_000));
    let before = deadlines.clone();
    assert!(deadlines.advance(u64::MAX, 2_000_000).is_err());
    assert_eq!(deadlines, before);
    assert!(deadlines.advance(33_000_000, 2_500_000).is_err());
    assert_eq!(deadlines, before);
}

#[test]
fn live_mode_clock_unions_owners_clips_open_pauses_and_rejects_stale_generations() {
    let first = PauseOwner {
        owner: 1,
        generation: 1,
    };
    let second = PauseOwner {
        owner: 2,
        generation: 1,
    };
    let mut clock = RuntimeModeClock::default();
    assert!(
        clock
            .start_pause(first, ModePauseKind::Cinematic, 1_000_000)
            .unwrap()
    );
    assert!(
        !clock
            .start_pause(first, ModePauseKind::Cinematic, 1_000_000)
            .unwrap()
    );
    clock
        .start_pause(second, ModePauseKind::Cinematic, 2_000_000)
        .unwrap();
    clock.end_pause(first, 4_000_000).unwrap();
    assert!(clock.is_paused());
    assert_eq!(clock.relative_deadline(3_000_000).unwrap(), None);
    let clipped = clock.observed_pauses(3_000_000).unwrap();
    assert_eq!(
        clipped
            .iter()
            .map(|p| [p.start_absolute_us, p.end_absolute_us])
            .collect::<Vec<_>>(),
        vec![[1_000_000, 3_000_000], [2_000_000, 3_000_000]]
    );
    clock.end_pause(second, 5_000_000).unwrap();
    assert!(!clock.end_pause(second, 5_000_000).unwrap());
    assert!(
        !clock
            .start_pause(first, ModePauseKind::Cinematic, 5_000_000)
            .unwrap()
    );
    assert_eq!(clock.relative_deadline(3_000_000).unwrap(), Some(7_000_000));
    clock.advance_to(7_000_000).unwrap();
    // Independent union: [1,4) U [2,5) = [1,5), four paused seconds.
    assert_eq!(clock.relative_us(), 3_000_000);
    let next = PauseOwner {
        generation: 2,
        ..first
    };
    clock
        .start_pause(next, ModePauseKind::Cinematic, 7_000_000)
        .unwrap();
    assert!(!clock.end_pause(first, 8_000_000).unwrap());
    assert!(clock.is_paused());
    assert_eq!(clock.relative_us(), 3_000_000);
    assert_eq!(
        clock
            .observed_pauses(8_000_000)
            .unwrap()
            .last()
            .unwrap()
            .end_absolute_us,
        8_000_000
    );
}

#[test]
fn live_domains_compose_cinematic_ownership_hotori_exceptions_and_local_hitlag() {
    let hotori = PauseOwner {
        owner: 0,
        generation: 5,
    };
    let other = PauseOwner {
        owner: 1,
        generation: 8,
    };
    let action = ClockDomain::Action {
        context: hotori,
        advance_during_active_stop: true,
    };
    let unrelated = ClockDomain::Action {
        context: other,
        advance_during_active_stop: false,
    };
    let mut clock = RuntimeModeClock::default();
    clock.register_action(hotori, true);
    clock.register_action(other, false);
    clock
        .start_pause(hotori, ModePauseKind::Cinematic, 1)
        .unwrap();
    let mut overlap = clock.clone();
    overlap
        .start_pause(other, ModePauseKind::Cinematic, 1)
        .unwrap();
    let restrictions = overlap.progression_policy();
    assert!(restrictions.progresses(ClockDomain::Absolute));
    for domain in [
        action,
        unrelated,
        ClockDomain::World,
        ClockDomain::ObjectLifetime,
    ] {
        assert!(!restrictions.progresses(domain));
    }
    overlap.end_pause(other, 1).unwrap();
    assert!(overlap.progression_policy().progresses(action));
    clock
        .transition_pause(hotori, ModePauseKind::ActiveStop, 3)
        .unwrap();
    clock.advance_to(5).unwrap();
    assert!(clock.progression_policy().progresses(action));
    assert!(!clock.progression_policy().progresses(unrelated));
    // Object and B2 activity advance for the two active seconds, not setup.
    assert_eq!(clock.elapsed_us(ClockDomain::ObjectLifetime), Some(3));
    assert_eq!(
        clock.elapsed_us(ClockDomain::AutonomousActivity {
            advance_during_active_stop: true
        }),
        Some(3)
    );
    assert_eq!(
        clock.elapsed_us(ClockDomain::AutonomousActivity {
            advance_during_active_stop: false
        }),
        Some(1)
    );
    assert_eq!(clock.elapsed_us(action), Some(5));
    assert_eq!(clock.elapsed_us(unrelated), Some(1));
    clock
        .start_pause(other, ModePauseKind::Cinematic, 5)
        .unwrap();
    clock.advance_to(8).unwrap();
    assert_eq!(clock.elapsed_us(action), Some(5));
    assert_eq!(clock.elapsed_us(unrelated), Some(1)); // Hotori still dominates.
    assert_eq!(clock.elapsed_us(ClockDomain::ObjectLifetime), Some(3));
    assert_eq!(clock.relative_us(), 1);
    clock.end_pause(other, 8).unwrap();
    clock.end_pause(hotori, 9).unwrap();
    clock.advance_to(11).unwrap();
    assert_eq!(clock.relative_us(), 3);
    assert_eq!(clock.elapsed_us(action), Some(8));
    assert_eq!(clock.elapsed_us(unrelated), Some(3));
    // Local contact dilation composes on the eligible action coordinate alone.
    let mut hitlag = HitlagTracker::default();
    hitlag
        .add_window(HitlagWindow {
            actor_index: 0,
            start_real_us: 2,
            duration_real_us: 2,
            dilated_rate_ppm: 500_000,
        })
        .unwrap();
    assert_eq!(
        hitlag
            .elapsed_actor_time_us(0, 0, clock.elapsed_us(action).unwrap())
            .unwrap(),
        7
    );
    assert_eq!(clock.relative_us(), 3);
    let mut global = HitlagTracker::default();
    global
        .add_window(HitlagWindow {
            actor_index: 0,
            start_real_us: 1,
            duration_real_us: 4,
            dilated_rate_ppm: 500_000,
        })
        .unwrap();
    global.add_freeze(0, 2, 2).unwrap();
    global.add_freeze(0, 4, 2).unwrap();
    assert_eq!(global.windows().len(), 2); // One local window and one composed global interval.
    assert_eq!(global.elapsed_actor_time_us(0, 0, 8).unwrap(), 3);
    clock.remove_action(hotori);
    assert_eq!(clock.elapsed_us(action), None);
}

#[test]
fn live_clock_zero_length_and_invalid_advancement_are_exact_and_atomic() {
    let owner = PauseOwner {
        owner: 0,
        generation: 0,
    };
    let mut clock = RuntimeModeClock::default();
    clock
        .start_pause(owner, ModePauseKind::Cinematic, 0)
        .unwrap();
    clock.end_pause(owner, 0).unwrap();
    assert!(clock.observed_pauses(0).unwrap().is_empty());
    clock.advance_to(u64::MAX).unwrap();
    assert_eq!(clock.relative_us(), u64::MAX);
    let before = clock.clone();
    assert_eq!(clock.advance_to(0), Err(TimingError::TimeReversal));
    assert_eq!(clock, before);
    let mut paused = RuntimeModeClock::default();
    paused
        .start_pause(owner, ModePauseKind::Cinematic, 0)
        .unwrap();
    paused.end_pause(owner, 1).unwrap();
    assert_eq!(
        paused.relative_deadline(u64::MAX),
        Err(TimingError::DeadlineOverflow)
    );
    assert!(paused.observed_pauses(2).is_err());
}
#[test]
fn projection_and_inverse_use_exact_fractional_overlap_independent_of_carrier_order() {
    let mut tracker = HitlagTracker::default();
    for (start, duration, rate) in [(0, 10, 500_000), (3, 4, 0), (10, 4, 250_000)] {
        tracker
            .add_window(HitlagWindow {
                actor_index: 9,
                start_real_us: start,
                duration_real_us: duration,
                dilated_rate_ppm: rate,
            })
            .unwrap();
    }
    // Independent integral: [0,3]*1/2 + [3,7]*0 + [7,10]*1/2 + [10,14]*1/4 = 4.
    assert_eq!(tracker.elapsed_actor_time_us(9, 0, 14).unwrap(), 4);
    let mut reversed = HitlagTracker::default();
    for window in tracker.windows().iter().rev() {
        reversed.add_window(window.clone()).unwrap();
    }
    for offset in 0..20 {
        let deadline = tracker.resolve_real_deadline(9, 0, offset).unwrap();
        assert_eq!(
            reversed.resolve_real_deadline(9, 0, offset).unwrap(),
            deadline
        );
        assert!(tracker.elapsed_actor_time_us(9, 0, deadline).unwrap() >= offset);
        if deadline > 0 {
            assert!(tracker.elapsed_actor_time_us(9, 0, deadline - 1).unwrap() < offset);
        }
    }
    assert_eq!(tracker.resolve_real_deadline(9, 0, 1).unwrap(), 2);
    assert_eq!(tracker.resolve_real_deadline(9, 0, 4).unwrap(), 14);
    assert_eq!(tracker.resolve_real_deadline(10, 0, 4).unwrap(), 4);
}
#[test]
fn invalid_clock_inputs_and_unrepresentable_deadlines_reject_atomically() {
    let mut tracker = HitlagTracker::default();
    assert_eq!(
        tracker.add_window(HitlagWindow {
            actor_index: 0,
            start_real_us: u64::MAX,
            duration_real_us: 1,
            dilated_rate_ppm: 0
        }),
        Err(TimingError::DeadlineOverflow)
    );
    assert_eq!(
        tracker.add_window(HitlagWindow {
            actor_index: 0,
            start_real_us: 0,
            duration_real_us: 1,
            dilated_rate_ppm: 1_000_001
        }),
        Err(TimingError::InvalidRate(1_000_001))
    );
    assert!(tracker.windows().is_empty());
    assert_eq!(
        tracker.resolve_real_deadline(0, u64::MAX, 1),
        Err(TimingError::DeadlineOverflow)
    );
    let mut mode = ModeClockManager::default();
    assert!(
        mode.add_pause(ModePauseInterval {
            start_real_us: u64::MAX,
            duration_us: 1,
            reason: "invalid".into()
        })
        .is_err()
    );
    assert!(mode.pauses().is_empty());
    mode.add_pause(ModePauseInterval {
        start_real_us: 0,
        duration_us: 1,
        reason: "pause".into(),
    })
    .unwrap();
    assert_eq!(
        mode.mode_to_real_time(u64::MAX),
        Err(TimingError::DeadlineOverflow)
    );
}
