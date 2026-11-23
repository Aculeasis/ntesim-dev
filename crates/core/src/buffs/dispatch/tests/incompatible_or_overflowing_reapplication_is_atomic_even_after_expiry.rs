use super::*;

#[test]
fn incompatible_or_overflowing_reapplication_is_atomic_even_after_expiry() {
    let mut manager = BuffManager::default();
    let mut def = synthetic_buff(
        BuffStackingScope::Shared,
        RefreshPolicy::Extend {
            max_duration_us: None,
        },
    );
    def.duration_us = Some(u64::MAX - 5);
    def.max_stacks = 2;
    manager.apply_buff(def.clone(), 0, 0).unwrap();
    let before = manager.clone();
    assert_eq!(
        manager.apply_buff(def.clone(), 0, 1),
        Err(DispatchError::ArithmeticOverflow)
    );
    assert_eq!(manager, before);
    def.refresh_policy = RefreshPolicy::Reset;
    assert!(matches!(
        manager.apply_buff(def, 0, 1),
        Err(DispatchError::InvalidBuff(_))
    ));
    assert_eq!(manager, before);
    let mut invalid = synthetic_buff(BuffStackingScope::Shared, RefreshPolicy::Reset);
    invalid.duration_us = Some(0);
    assert!(manager.apply_buff(invalid, 0, u64::MAX).is_err());
    assert_eq!(manager, before);
}
