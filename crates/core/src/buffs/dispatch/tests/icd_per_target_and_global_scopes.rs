use super::*;

#[test]
fn icd_per_target_and_global_scopes() {
    let mut tracker = IcdTracker::default();
    let rule = IcdRule::TimerOnly {
        window_us: 1_000_000,
    };

    let target_a = IcdScope::PerTarget("enemy_a".to_string());
    let target_b = IcdScope::PerTarget("enemy_b".to_string());

    assert!(
        tracker
            .record_event("slash_proc", &target_a, &rule, 0)
            .unwrap()
    );

    assert!(
        !tracker
            .record_event("slash_proc", &target_a, &rule, 500_000)
            .unwrap()
    );

    assert!(
        tracker
            .record_event("slash_proc", &target_b, &rule, 500_000)
            .unwrap()
    );

    let global = IcdScope::Global;
    assert!(
        tracker
            .record_event("global_proc", &global, &rule, 0)
            .unwrap()
    );
    assert!(
        !tracker
            .record_event("global_proc", &global, &rule, 500_000)
            .unwrap()
    );
}
