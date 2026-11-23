use super::*;

#[test]
pub(crate) fn resource_spending_and_interruption_refunds() {
    let mut team = create_test_team();
    let mut costs = BTreeMap::new();
    costs.insert("energy".to_string(), 40);
    costs.insert("stamina".to_string(), 50);

    let timings = ActionPhaseTimings {
        startup_us: 300_000,
        active_us: 200_000,
        recovery_us: 200_000,
        recovery_cancels: vec![],
        startup_cancels: vec![],
    };

    team.start_action(0, "burst".to_string(), timings.clone(), costs, 0, true)
        .unwrap();
    assert_eq!(team.resources.get("energy").unwrap().current(), 60);
    assert_eq!(team.resources.get("stamina").unwrap().current(), 150);

    let receipt = team
        .interrupt_action(
            0,
            InterruptionCause::Stagger,
            &RefundPolicy::FullRefund,
            100_000,
        )
        .unwrap();
    assert_eq!(receipt.from_phase, ActionPhaseKind::Startup);
    assert_eq!(receipt.refunded_costs.get("energy").copied(), Some(40));
    assert_eq!(receipt.refunded_costs.get("stamina").copied(), Some(50));
    assert_eq!(team.resources.get("energy").unwrap().current(), 100);
    assert_eq!(team.resources.get("stamina").unwrap().current(), 200);

    let mut costs2 = BTreeMap::new();
    costs2.insert("energy".to_string(), 40);
    team.start_action(
        0,
        "burst".to_string(),
        timings.clone(),
        costs2,
        200_000,
        true,
    )
    .unwrap();
    assert_eq!(team.resources.get("energy").unwrap().current(), 60);

    let half_ratio = Ratio {
        numerator: 1,
        denominator: 2,
    };
    let receipt2 = team
        .interrupt_action(
            0,
            InterruptionCause::Knockdown,
            &RefundPolicy::PartialRefund { ratio: half_ratio },
            250_000,
        )
        .unwrap();
    assert_eq!(receipt2.refunded_costs.get("energy").copied(), Some(20));
    assert_eq!(team.resources.get("energy").unwrap().current(), 80);

    let mut costs3 = BTreeMap::new();
    costs3.insert("energy".to_string(), 40);
    team.start_action(0, "burst".to_string(), timings, costs3, 300_000, true)
        .unwrap();
    assert_eq!(team.resources.get("energy").unwrap().current(), 40);

    team.advance_time(650_000).unwrap();
    assert_eq!(
        team.actors[0].action_state.phase.kind(),
        ActionPhaseKind::Active
    );

    let receipt3 = team
        .interrupt_action(
            0,
            InterruptionCause::Stagger,
            &RefundPolicy::FullRefund,
            700_000,
        )
        .unwrap();
    assert_eq!(receipt3.from_phase, ActionPhaseKind::Active);
    assert!(receipt3.refunded_costs.is_empty());
    assert_eq!(team.resources.get("energy").unwrap().current(), 40);
    assert!(team.actors[0].action_state.is_idle());
}

#[test]
fn failed_lifecycle_commands_preserve_clock_phases_and_resources_atomically() {
    let mut team = TeamLifecycle::new(
        2,
        vec!["new-carrier".into(), "second".into()],
        0,
        0,
        BTreeMap::from([("energy".into(), Pool::new(3, 10).unwrap())]),
    )
    .unwrap();
    let before = serde_json::to_value(&team).unwrap();
    assert!(
        team.start_action(
            0,
            "new-action".into(),
            ActionPhaseTimings {
                startup_us: 1,
                active_us: 1,
                recovery_us: 1,
                recovery_cancels: vec![],
                startup_cancels: vec![],
            },
            BTreeMap::from([("energy".into(), 4)]),
            5,
            true
        )
        .is_err()
    );
    assert_eq!(serde_json::to_value(&team).unwrap(), before);
    assert!(team.switch_to(9, 5).is_err());
    assert_eq!(serde_json::to_value(&team).unwrap(), before);
    team.field.switch_cooldown_us = 10;
    let before = team.clone();
    assert_eq!(
        team.switch_to(1, u64::MAX - 5),
        Err(LifecycleError::DeadlineOverflow)
    );
    assert_eq!(team, before);
}

#[test]
fn refund_policy_is_validated_and_capacity_settlement_is_reported() {
    let mut team = create_test_team();
    team.start_action(
        0,
        "synthetic-refund".into(),
        ActionPhaseTimings {
            startup_us: 100,
            active_us: 10,
            recovery_us: 10,
            startup_cancels: vec![],
            recovery_cancels: vec![],
        },
        BTreeMap::from([("energy".into(), 40)]),
        0,
        true,
    )
    .unwrap();
    let before = team.clone();
    for ratio in [
        Ratio {
            numerator: 1,
            denominator: 0,
        },
        Ratio {
            numerator: 3,
            denominator: 2,
        },
    ] {
        assert_eq!(
            team.interrupt_action(
                0,
                InterruptionCause::Stagger,
                &RefundPolicy::PartialRefund { ratio },
                5
            ),
            Err(LifecycleError::InvalidRefundRatio)
        );
        assert_eq!(team, before);
    }
    team.resources.get_mut("energy").unwrap().grant(30);
    let receipt = team
        .interrupt_action(0, InterruptionCause::Stagger, &RefundPolicy::FullRefund, 5)
        .unwrap();
    assert_eq!(receipt.refunded_costs["energy"], 10);
    assert_eq!(receipt.refund_excess["energy"], 30);
    assert_eq!(team.resources["energy"].current(), 100);
    assert!(team.actors[0].action_state.is_idle());
}
