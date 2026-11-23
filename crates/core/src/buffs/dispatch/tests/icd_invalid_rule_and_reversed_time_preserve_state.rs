use super::*;

#[test]
fn icd_invalid_rule_and_reversed_time_preserve_state() {
    let mut tracker = IcdTracker::default();
    let scope = IcdScope::Global;
    let before = tracker.clone();
    assert!(matches!(
        tracker.record_event("new", &scope, &IcdRule::HitCountOnly { hit_interval: 0 }, 1),
        Err(DispatchError::InvalidIcd(_))
    ));
    assert_eq!(tracker, before);
    assert!(
        tracker
            .record_event("new", &scope, &IcdRule::None, 10)
            .unwrap()
    );
    let before = tracker.clone();
    assert_eq!(
        tracker.record_event("new", &scope, &IcdRule::None, 9),
        Err(DispatchError::TimeReversal {
            last_us: 10,
            now_us: 9
        })
    );
    assert_eq!(tracker, before);
}
