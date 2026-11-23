use super::*;

#[test]
fn sequential_stack_decay_requires_representable_final_expiry() {
    let mut manager = BuffManager::default();
    let mut def = synthetic_buff(BuffStackingScope::PerSource, RefreshPolicy::Reset);
    def.decay_policy = DecayPolicy::OneByOne;
    def.duration_us = Some(10);
    def.max_stacks = 3;
    manager.apply_buff(def.clone(), 0, u64::MAX - 15).unwrap();
    let before = manager.clone();
    assert_eq!(
        manager.apply_buff(def, 0, u64::MAX - 14),
        Err(DispatchError::ArithmeticOverflow)
    );
    assert_eq!(manager, before);
}
