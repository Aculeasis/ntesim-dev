use super::*;

#[test]
fn icd_read_only_eligibility_predicts_the_same_next_event_for_every_rule() {
    for rule in [
        IcdRule::None,
        IcdRule::TimerOnly { window_us: 7 },
        IcdRule::Standard3Hit2_5s,
        IcdRule::HitCountOnly { hit_interval: 3 },
    ] {
        let mut entry = IcdEntry::default();
        for time in [0, 1, 2, 3, 7, 8, 2_500_001, u64::MAX] {
            let before = entry.clone();
            let eligible = entry.can_trigger(&rule, time).unwrap();
            assert_eq!(entry, before);
            assert_eq!(eligible, entry.record_event(&rule, time).unwrap());
        }
    }
    let mut entry = IcdEntry::default();
    let rule = IcdRule::HitCountOnly { hit_interval: 3 };
    assert!(!entry.can_trigger(&rule, 0).unwrap());
    assert!(!entry.record_event(&rule, 0).unwrap());
    assert!(!entry.record_event(&rule, 1).unwrap());
    assert!(entry.can_trigger(&rule, 2).unwrap());
    assert!(entry.record_event(&rule, 2).unwrap());
}
