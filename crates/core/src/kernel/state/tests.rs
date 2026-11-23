use super::*;
#[test]
fn resource_shortage_is_atomic_and_never_waits_implicitly() {
    let mut pool = Pool::new(3, 10).unwrap();
    let before = pool.clone();
    assert_eq!(
        pool.spend(4),
        Err(StateError::InsufficientResource {
            required: 4,
            available: 3
        })
    );
    assert_eq!(pool, before);
}
#[test]
fn grant_and_capacity_changes_report_lost_amounts_without_overflow() {
    let mut pool = Pool::new(u64::MAX - 1, u64::MAX).unwrap();
    assert_eq!(
        pool.grant(u64::MAX),
        Grant {
            applied: 1,
            excess: u64::MAX - 1
        }
    );
    assert_eq!(pool.set_capacity(10), u64::MAX - 10);
    assert_eq!(pool.current(), 10);
}
#[test]
fn generated_pool_operations_preserve_bounds_and_conservation() {
    for capacity in 0..64 {
        for initial in 0..=capacity {
            for amount in [0, 1, capacity, capacity + 1, u64::MAX] {
                let mut pool = Pool::new(initial, capacity).unwrap();
                let result = pool.grant(amount);
                assert_eq!(
                    u128::from(result.applied) + u128::from(result.excess),
                    u128::from(amount)
                );
                assert_eq!(pool.current(), initial + result.applied);
                assert!(pool.current() <= pool.capacity());
                let before = pool.clone();
                let result = pool.spend(amount);
                if result.is_err() {
                    assert_eq!(pool, before);
                } else {
                    assert_eq!(pool.current() + amount, before.current());
                }
            }
        }
    }
}
#[test]
fn shield_absorption_order_is_an_explicit_policy() {
    for (order, first, remaining) in [
        (ShieldOrder::OldestFirst, "a", "b"),
        (ShieldOrder::NewestFirst, "b", "a"),
    ] {
        let mut state = VitalState::new(100, 100, order).unwrap();
        state.add_shield("a".into(), "ally".into(), 10).unwrap();
        state.add_shield("b".into(), "self".into(), 10).unwrap();
        let result = state.damage(15).unwrap();
        assert_eq!(result.absorbed[0].shield, first);
        assert_eq!(state.shields()[0].id, remaining);
        assert_eq!(state.shields()[0].remaining(), 5);
        assert_eq!(state.hp(), 100);
    }
}

#[test]
fn shield_and_residual_damage_phases_settle_independently() {
    let mut state = VitalState::new(40, 100, ShieldOrder::OldestFirst).unwrap();
    state.add_shield("s".into(), "ally".into(), 20).unwrap();
    let absorbed = state.absorb_shields(30).unwrap();
    assert_eq!(absorbed[0].amount, 20);
    assert_eq!(state.hp(), 40);
    assert!(state.shields().is_empty());
    let (hp_lost, overkill, died) = state.settle_hp_damage(10).unwrap();
    assert_eq!((hp_lost, overkill, died), (10, 0, false));
    assert_eq!(state.hp(), 30);

    let mut lethal = VitalState::new(5, 5, ShieldOrder::OldestFirst).unwrap();
    let (hp_lost, overkill, died) = lethal.settle_hp_damage(9).unwrap();
    assert_eq!((hp_lost, overkill, died), (5, 4, true));
}

#[test]
fn damage_healing_overkill_and_death_are_distinct() {
    let mut state = VitalState::new(50, 100, ShieldOrder::OldestFirst).unwrap();
    state.add_shield("s".into(), "ally".into(), 20).unwrap();
    assert_eq!(state.damage(30).unwrap().hp_lost, 10);
    assert_eq!(
        state.heal(100).unwrap(),
        Grant {
            applied: 60,
            excess: 40
        }
    );
    let result = state.damage(150).unwrap();
    assert_eq!(result.hp_lost, 100);
    assert_eq!(result.overkill, 50);
    assert!(result.died);
    assert_eq!(state.heal(1), Err(StateError::DeadRecipient));
}
#[test]
fn invalid_shield_changes_leave_state_unchanged() {
    let mut state = VitalState::new(10, 10, ShieldOrder::OldestFirst).unwrap();
    state.add_shield("s".into(), "ally".into(), 5).unwrap();
    let before = state.clone();
    assert!(state.add_shield("s".into(), "ally".into(), 9).is_err());
    assert_eq!(state, before);
    assert!(state.remove_shield("absent").is_err());
    assert_eq!(state, before);
}
#[test]
fn generated_damage_preserves_per_target_amounts() {
    for hp in 1..32 {
        for shield in 0..32 {
            for amount in 0..96 {
                let mut state = VitalState::new(hp, hp, ShieldOrder::OldestFirst).unwrap();
                state.add_shield("s".into(), "ally".into(), shield).unwrap();
                let result = state.damage(amount).unwrap();
                assert_eq!(
                    result.absorbed.iter().map(|a| a.amount).sum::<u64>()
                        + result.hp_lost
                        + result.overkill,
                    amount
                );
                assert_eq!(state.hp() + result.hp_lost, hp);
                assert_eq!(result.died, state.hp() == 0);
            }
        }
    }
}
#[test]
fn each_target_settles_independently_and_cycles_keep_state() {
    let mut first = VitalState::new(100, 100, ShieldOrder::OldestFirst).unwrap();
    let mut second = VitalState::new(100, 100, ShieldOrder::OldestFirst).unwrap();
    second.add_shield("s".into(), "ally".into(), 50).unwrap();
    for _ in 0..3 {
        first.damage(20).unwrap();
        second.damage(20).unwrap();
    }
    assert_eq!(first.hp(), 40);
    assert_eq!(second.hp(), 90);
}

#[test]
fn deserialized_pools_obey_the_same_capacity_contract_as_new() {
    assert!(serde_json::from_str::<Pool>(r#"{"current":2,"capacity":1}"#).is_err());
    let mut pool = serde_json::from_str::<Pool>(r#"{"current":1,"capacity":2}"#).unwrap();
    let grant = pool.grant(5);
    assert_eq!((grant.applied, grant.excess, pool.current()), (1, 4, 2));
}
