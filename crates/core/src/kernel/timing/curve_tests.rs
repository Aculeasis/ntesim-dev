use super::*;

fn curve(actor: usize, start: u64, duration: u64, points: &[(u64, u32)]) -> HitlagCurveWindow {
    HitlagCurveWindow {
        actor_index: actor,
        start_real_us: start,
        duration_real_us: duration,
        points: points
            .iter()
            .map(|&(offset_us, rate_ppm)| HitlagRatePoint {
                offset_us,
                rate_ppm,
            })
            .collect(),
    }
}
#[test]
fn linear_curve_clipping_retains_fractional_area_and_inverts_inside_a_ramp() {
    let mut tracker = HitlagTracker::default();
    tracker
        .add_curve_window(curve(
            0,
            0,
            130000,
            &[(0, 0), (60000, 0), (120000, 10000), (160000, 1000000)],
        ))
        .unwrap();
    // 60000*(0+.01)/2 + 10000*(.01+.2575)/2 =1637.5 actor microseconds.
    assert_eq!(tracker.elapsed_actor_time_us(0, 0, 125000).unwrap(), 659);
    assert_eq!(tracker.elapsed_actor_time_us(0, 0, 130000).unwrap(), 1637);
    assert_eq!(tracker.elapsed_actor_time_us(0, 0, 130001).unwrap(), 1638);
    assert_eq!(tracker.resolve_real_deadline(0, 0, 2000).unwrap(), 130363);
    // In the first ramp, x²/12000000 reaches2 at sqrt(24000000), rounded upward.
    assert_eq!(tracker.resolve_real_deadline(0, 0, 2).unwrap(), 64899);
    assert_eq!(tracker.elapsed_actor_time_us(0, 0, 64898).unwrap(), 1);
    let serialized = serde_json::to_value(&tracker).unwrap();
    assert_eq!(
        serialized["curve_windows"][0]["points"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        serialized["curve_windows"][0]["points"][3]["offset_us"],
        160000
    );
}
#[test]
fn strongest_overlapping_curves_split_at_their_crossings_independent_of_insertion_order() {
    let ascending = curve(7, 0, 10, &[(0, 0), (10, 1000000)]);
    let descending = curve(7, 0, 10, &[(0, 1000000), (10, 0)]);
    let mut left = HitlagTracker::default();
    let mut right = HitlagTracker::default();
    left.add_curve_window(ascending.clone()).unwrap();
    left.add_curve_window(descending.clone()).unwrap();
    right.add_curve_window(descending).unwrap();
    right.add_curve_window(ascending).unwrap();
    // Two triangles of base5/height.5 yield2.5us, rather than either curve's5us.
    assert_eq!(left.elapsed_actor_time_us(7, 0, 10).unwrap(), 2);
    assert_eq!(left.resolve_real_deadline(7, 0, 2).unwrap(), 7);
    assert_eq!(left.resolve_real_deadline(7, 0, 3).unwrap(), 11);
    assert_eq!(left.resolve_real_deadline(7, 3, 2).unwrap(), 9);
    assert_eq!(left.elapsed_actor_time_us(8, 0, 10).unwrap(), 10);
    for offset in 0..15 {
        let deadline = left.resolve_real_deadline(7, 0, offset).unwrap();
        assert_eq!(deadline, right.resolve_real_deadline(7, 0, offset).unwrap());
        assert!(left.elapsed_actor_time_us(7, 0, deadline).unwrap() >= offset);
        if deadline > 0 {
            assert!(left.elapsed_actor_time_us(7, 0, deadline - 1).unwrap() < offset);
        }
    }
    for tracker in [&mut left, &mut right] {
        tracker
            .add_window(HitlagWindow {
                actor_index: 7,
                start_real_us: 4,
                duration_real_us: 2,
                dilated_rate_ppm: 0,
            })
            .unwrap();
    }
    // The freeze removes .9 from2.5; ordinary time resumes after10.
    assert_eq!(left.elapsed_actor_time_us(7, 0, 10).unwrap(), 1);
    assert_eq!(left.resolve_real_deadline(7, 0, 2).unwrap(), 11);
    assert_eq!(
        left.resolve_real_deadline(7, 0, 2).unwrap(),
        right.resolve_real_deadline(7, 0, 2).unwrap()
    );
}
#[test]
fn noninteger_curve_crossings_and_separate_fractional_windows_do_not_round_intermediate_progress() {
    let mut tracker = HitlagTracker::default();
    tracker
        .add_curve_window(curve(0, 0, 3, &[(0, 0), (3, 1000000)]))
        .unwrap();
    tracker
        .add_curve_window(curve(0, 0, 4, &[(0, 1000000), (4, 0)]))
        .unwrap();
    // Crossing12/7 gives total area8/7 through4; progress at3 is57/56.
    assert_eq!(tracker.elapsed_actor_time_us(0, 0, 3).unwrap(), 1);
    assert_eq!(tracker.elapsed_actor_time_us(0, 0, 4).unwrap(), 1);
    assert_eq!(tracker.resolve_real_deadline(0, 0, 1).unwrap(), 3);
    assert_eq!(tracker.resolve_real_deadline(0, 0, 2).unwrap(), 5);
    let mut halves = HitlagTracker::default();
    for start in [0, 3] {
        halves
            .add_curve_window(curve(0, start, 3, &[(0, 500000)]))
            .unwrap();
    }
    assert_eq!(halves.elapsed_actor_time_us(0, 0, 6).unwrap(), 3);
    assert_eq!(halves.resolve_real_deadline(0, 0, 2).unwrap(), 4);
}
#[test]
fn invalid_curves_do_not_mutate_tracker_and_boundary_deadlines_remain_checked() {
    let mut tracker = HitlagTracker::default();
    for invalid in [
        curve(0, 0, 1, &[]),
        curve(0, 0, 1, &[(1, 0)]),
        curve(0, 0, 1, &[(0, 0), (0, 1)]),
        curve(0, 0, 1, &[(0, 1000001)]),
        curve(0, u64::MAX, 1, &[(0, 0)]),
    ] {
        assert!(tracker.add_curve_window(invalid).is_err());
        assert!(tracker.curve_windows().is_empty());
    }
    tracker
        .add_curve_window(curve(0, u64::MAX - 2, 2, &[(0, 0), (2, 1000000)]))
        .unwrap();
    assert_eq!(
        tracker.resolve_real_deadline(0, u64::MAX - 2, 1).unwrap(),
        u64::MAX
    );
    assert_eq!(
        tracker.resolve_real_deadline(0, u64::MAX - 2, 2),
        Err(TimingError::DeadlineOverflow)
    );
    assert_eq!(
        tracker.resolve_real_deadline(0, u64::MAX, 0).unwrap(),
        u64::MAX
    );
    assert_eq!(
        tracker.elapsed_actor_time_us(0, 1, 0),
        Err(TimingError::TimeReversal)
    );
}
