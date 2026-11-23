use super::*;

#[test]
fn arbitrary_icd_ids_do_not_alias_and_unreachable_timer_deadlines_never_fire() {
    let mut tracker = IcdTracker::default();
    let rule = IcdRule::TimerOnly { window_us: 10 };
    assert!(
        tracker
            .record_event(
                "carrier:target:x",
                &IcdScope::PerTarget("y".into()),
                &rule,
                0
            )
            .unwrap()
    );
    assert!(
        tracker
            .record_event(
                "carrier",
                &IcdScope::PerTarget("x:target:y".into()),
                &rule,
                1
            )
            .unwrap()
    );
    let mut entry = IcdEntry::default();
    assert!(entry.record_event(&rule, u64::MAX - 5).unwrap());
    assert!(!entry.can_trigger(&rule, u64::MAX).unwrap());
    assert!(!entry.record_event(&rule, u64::MAX).unwrap());
}
