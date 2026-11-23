use super::*;
use crate::scheduler::{RATE_SCALE, Scheduler};
fn action(recovery: u64, cooldown: u64, ready: u64, costs: BTreeMap<String, u64>) -> ActionState {
    ActionState {
        rule: Rule {
            recovery_us: recovery,
            cooldown_us: cooldown,
            costs,
        },
        ready_at_us: ready,
    }
}
fn resources(energy: u64, points: u64) -> BTreeMap<String, Pool> {
    BTreeMap::from([
        ("energy".into(), Pool::new(energy, 10).unwrap()),
        ("points".into(), Pool::new(points, 10).unwrap()),
    ])
}
#[test]
fn shared_timing_admission_matches_independent_boundary_arithmetic() {
    for now in 0..12 {
        let state = TimingState {
            last_start_us: 2,
            locked_until_us: 4,
            ready_at_us: 7,
        };
        let result = state.prepare("new-carrier", now, 3, 5);
        match now {
            0..=1 => assert_eq!(result, Err(ActionError::TimeReversal { last: 2, now })),
            2..=3 => assert_eq!(result, Err(ActionError::RecoveryLock { until: 4 })),
            4..=6 => assert_eq!(
                result,
                Err(ActionError::Cooldown {
                    action: "new-carrier".into(),
                    until: 7
                })
            ),
            _ => assert_eq!(
                result,
                Ok(TimingAdmission {
                    locked_until_us: now + 3,
                    ready_at_us: now + 5
                })
            ),
        }
    }
    let state = TimingState {
        last_start_us: 0,
        locked_until_us: 0,
        ready_at_us: 0,
    };
    assert_eq!(
        state.prepare("new-carrier", u64::MAX, 1, 0),
        Err(ActionError::Overflow)
    );
    assert_eq!(
        state.prepare("new-carrier", u64::MAX, 0, 1),
        Err(ActionError::Overflow)
    );
    assert_eq!(
        state.prepare("new-carrier", u64::MAX, 0, 0),
        Ok(TimingAdmission {
            locked_until_us: u64::MAX,
            ready_at_us: u64::MAX,
        })
    );
}
#[test]
fn exact_recovery_and_cooldown_boundaries_apply_to_independent_actions() {
    let mut gate = Gate::new(
        7,
        0,
        5,
        BTreeMap::from([
            ("basic".into(), action(3, 0, 0, BTreeMap::new())),
            (
                "skill".into(),
                action(4, 10, 10, BTreeMap::from([("energy".into(), 2)])),
            ),
        ]),
    )
    .unwrap();
    let mut pools = resources(10, 10);
    assert_eq!(
        gate.start("skill", 7, 4, &mut pools),
        Err(ActionError::RecoveryLock { until: 5 })
    );
    assert_eq!(
        gate.start("skill", 7, 5, &mut pools),
        Err(ActionError::Cooldown {
            action: "skill".into(),
            until: 10
        })
    );
    assert_eq!(
        gate.start("basic", 7, 5, &mut pools)
            .unwrap()
            .locked_until_us,
        8
    );
    assert_eq!(gate.ready_at_us("skill").unwrap(), 10);
    assert_eq!(
        gate.start("skill", 7, 8, &mut pools),
        Err(ActionError::Cooldown {
            action: "skill".into(),
            until: 10
        })
    );
    let started = gate.start("skill", 7, 10, &mut pools).unwrap();
    assert_eq!(started.locked_until_us, 14);
    assert_eq!(started.ready_at_us, 20);
    assert_eq!(pools["energy"].current(), 8);
    assert_eq!(
        gate.start("basic", 7, 11, &mut pools),
        Err(ActionError::RecoveryLock { until: 14 })
    );
}
#[test]
fn multi_pool_costs_fail_atomically_and_can_retry_after_explicit_gain() {
    let mut gate = Gate::new(
        0,
        0,
        0,
        BTreeMap::from([(
            "skill".into(),
            action(
                5,
                10,
                0,
                BTreeMap::from([("energy".into(), 2), ("points".into(), 4)]),
            ),
        )]),
    )
    .unwrap();
    let mut pools = resources(10, 3);
    let before = (gate.clone(), pools.clone());
    assert_eq!(
        gate.start("skill", 0, 10, &mut pools),
        Err(ActionError::InsufficientResource {
            pool: "points".into(),
            required: 4,
            available: 3
        })
    );
    assert_eq!((gate.clone(), pools.clone()), before);
    pools.get_mut("points").unwrap().grant(1);
    let start = gate.start("skill", 0, 10, &mut pools).unwrap();
    assert_eq!(
        start.spent,
        BTreeMap::from([("energy".into(), 2), ("points".into(), 4)])
    );
    assert_eq!(pools["energy"].current(), 8);
    assert_eq!(pools["points"].current(), 0);
}
#[test]
fn clock_mismatch_time_reversal_and_overflow_preserve_all_state() {
    let mut gate = Gate::new(
        1,
        10,
        0,
        BTreeMap::from([(
            "skill".into(),
            action(2, 3, 0, BTreeMap::from([("energy".into(), 1)])),
        )]),
    )
    .unwrap();
    let mut pools = resources(10, 10);
    let before = (gate.clone(), pools.clone());
    for (clock, time) in [(0, 10), (1, 9), (1, u64::MAX)] {
        assert!(gate.start("skill", clock, time, &mut pools).is_err());
        assert_eq!((gate.clone(), pools.clone()), before);
    }
    gate.start("skill", 1, 10, &mut pools).unwrap();
    let after = (gate.clone(), pools.clone());
    assert_eq!(
        gate.start("skill", 1, 9, &mut pools),
        Err(ActionError::TimeReversal { last: 10, now: 9 })
    );
    assert_eq!((gate, pools), after);
}
#[test]
fn explicit_zero_durations_allow_same_tick_without_implicit_input_gap() {
    let mut gate = Gate::new(
        0,
        0,
        0,
        BTreeMap::from([(
            "instant".into(),
            action(0, 0, 0, BTreeMap::from([("energy".into(), 1)])),
        )]),
    )
    .unwrap();
    let mut pools = resources(3, 0);
    for _ in 0..3 {
        gate.start("instant", 0, 0, &mut pools).unwrap();
    }
    assert_eq!(pools["energy"].current(), 0);
    assert!(matches!(
        gate.start("instant", 0, 0, &mut pools),
        Err(ActionError::InsufficientResource { .. })
    ));
}
#[test]
fn missing_definitions_and_pool_dependencies_never_become_ready_or_free() {
    assert_eq!(
        Gate::new(0, 0, 0, BTreeMap::new()),
        Err(ActionError::EmptyActions)
    );
    let mut states = BTreeMap::from([("".into(), action(0, 0, 0, BTreeMap::new()))]);
    assert_eq!(
        Gate::new(0, 0, 0, states.clone()),
        Err(ActionError::EmptyAction)
    );
    states = BTreeMap::from([(
        "skill".into(),
        action(0, 0, 0, BTreeMap::from([("".into(), 0)])),
    )]);
    assert_eq!(Gate::new(0, 0, 0, states), Err(ActionError::EmptyPool));
    let mut gate = Gate::new(
        0,
        0,
        0,
        BTreeMap::from([(
            "skill".into(),
            action(0, 0, 0, BTreeMap::from([("missing".into(), 0)])),
        )]),
    )
    .unwrap();
    let mut pools = resources(10, 10);
    assert_eq!(
        gate.ready_at_us("unknown"),
        Err(ActionError::UnknownAction("unknown".into()))
    );
    assert_eq!(
        gate.start("unknown", 0, 0, &mut pools),
        Err(ActionError::UnknownAction("unknown".into()))
    );
    assert_eq!(
        gate.start("skill", 0, 0, &mut pools),
        Err(ActionError::UnknownPool("missing".into()))
    );
}
#[test]
fn paused_local_clock_does_not_expire_cooldown_on_real_time_alone() {
    let mut queue: Scheduler<()> = Scheduler::default();
    queue.add_clock(1, 0).unwrap();
    let mut gate = Gate::new(
        1,
        0,
        0,
        BTreeMap::from([("skill".into(), action(5, 10, 10, BTreeMap::new()))]),
    )
    .unwrap();
    let mut pools = BTreeMap::new();
    queue.pop_before(100).unwrap();
    assert!(matches!(
        gate.start("skill", 1, queue.local_time_us(1).unwrap(), &mut pools),
        Err(ActionError::Cooldown { .. })
    ));
    queue.set_rate(1, RATE_SCALE).unwrap();
    queue.pop_before(110).unwrap();
    let start = gate
        .start("skill", 1, queue.local_time_us(1).unwrap(), &mut pools)
        .unwrap();
    assert_eq!(start.started_at_us, 10);
    assert_eq!(start.ready_at_us, 20);
}
#[test]
fn generated_admissions_match_independent_boolean_oracle_and_conserve_costs() {
    for now in 0..16 {
        for recovery in 0..8 {
            for ready in 0..8 {
                for available in 0..4 {
                    for cost in 0..4 {
                        let mut gate = Gate::new(
                            0,
                            0,
                            recovery,
                            BTreeMap::from([(
                                "skill".into(),
                                action(5, 9, ready, BTreeMap::from([("energy".into(), cost)])),
                            )]),
                        )
                        .unwrap();
                        let mut pools = resources(available, 0);
                        let before = (gate.clone(), pools.clone());
                        let result = gate.start("skill", 0, now, &mut pools);
                        let allowed = now >= recovery && now >= ready && available >= cost;
                        assert_eq!(result.is_ok(), allowed);
                        if allowed {
                            assert_eq!(pools["energy"].current() + cost, available);
                            assert_eq!(gate.locked_until_us(), now + 5);
                            assert_eq!(gate.ready_at_us("skill").unwrap(), now + 9);
                        } else {
                            assert_eq!((gate, pools), before);
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn json_requires_explicit_ready_deadline_rule_and_costs() {
    let good = r#"{"rule":{"recovery_us":0,"cooldown_us":0,"costs":{}},"ready_at_us":0}"#;
    assert!(serde_json::from_str::<ActionState>(good).is_ok());
    for bad in [
        good.replace(",\"ready_at_us\":0", ""),
        good.replace("\"costs\":{}", "\"costs_typo\":{}"),
        good.replace("\"cooldown_us\":0,", ""),
        good.replace("\"recovery_us\":0", "\"recovery_us\":-1"),
    ] {
        assert!(serde_json::from_str::<ActionState>(&bad).is_err());
    }
}
