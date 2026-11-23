use super::*;

#[test]
fn icd_standard_3hit_2_5s_rule() {
    let mut entry = IcdEntry::default();
    let rule = IcdRule::Standard3Hit2_5s;

    assert!(entry.record_event(&rule, 0).unwrap());
    assert_eq!(entry.hits_since_trigger, 1);

    assert!(!entry.record_event(&rule, 500_000).unwrap());
    assert_eq!(entry.hits_since_trigger, 2);

    assert!(entry.record_event(&rule, 1_000_000).unwrap());
    assert_eq!(entry.hits_since_trigger, 1);

    assert!(!entry.record_event(&rule, 1_500_000).unwrap());

    assert!(entry.record_event(&rule, 3_600_000).unwrap());
    assert_eq!(entry.hits_since_trigger, 1);
}
