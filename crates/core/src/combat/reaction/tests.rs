use super::*;

#[test]
fn spatial_geometry_splash_does_not_multiply_linearly() {
    let targets = vec![
        TargetProfile {
            id: "target_0".to_string(),
            position: Point3D {
                x_mm: 0,
                y_mm: 0,
                z_mm: 0,
            },
            defense: 1000,
            resistances: BTreeMap::from([("Physical".to_string(), Ratio::zero())]),
            break_cap: 100,
            current_break_gauge: 100,
            break_state: BreakState::Unbroken,
            auras: BTreeMap::new(),
        },
        TargetProfile {
            id: "target_1".to_string(),
            position: Point3D {
                x_mm: 500,
                y_mm: 0,
                z_mm: 0,
            },
            defense: 1000,
            resistances: BTreeMap::from([("Physical".to_string(), Ratio::zero())]),
            break_cap: 100,
            current_break_gauge: 100,
            break_state: BreakState::Unbroken,
            auras: BTreeMap::new(),
        },
        TargetProfile {
            id: "target_2".to_string(),
            position: Point3D {
                x_mm: 1500,
                y_mm: 0,
                z_mm: 0,
            },
            defense: 1000,
            resistances: BTreeMap::from([("Physical".to_string(), Ratio::zero())]),
            break_cap: 100,
            current_break_gauge: 100,
            break_state: BreakState::Unbroken,
            auras: BTreeMap::new(),
        },
        TargetProfile {
            id: "target_3".to_string(),
            position: Point3D {
                x_mm: 5000,
                y_mm: 0,
                z_mm: 0,
            },
            defense: 1000,
            resistances: BTreeMap::from([("Physical".to_string(), Ratio::zero())]),
            break_cap: 100,
            current_break_gauge: 100,
            break_state: BreakState::Unbroken,
            auras: BTreeMap::new(),
        },
    ];

    let actors = vec![ActorCombatProfile {
        actor_index: 0,
        character_id: "sakiri".to_string(),
        level: 80,
        base_break: 100,
        break_intensity: Ratio::zero(),
        energy_pool: Pool::new(0, 100).unwrap(),
    }];

    let mut engine = ReactionEngine::new(targets, actors, true).unwrap();

    let rule = TargetingRule::SplashRadius {
        center_target: "target_0".to_string(),
        radius_mm: 1000,
    };
    let (hits, _, _) = engine.process_attack(0, &rule, 500, 10, 10, 0).unwrap();

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].target_id, "target_0");
    assert_eq!(hits[1].target_id, "target_1");
    assert_eq!(engine.actors()[0].energy_pool.current(), 10);
}

#[test]
fn break_state_triggers_all_four_characters_break_damage() {
    let targets = vec![TargetProfile {
        id: "boss".to_string(),
        position: Point3D::default(),
        defense: 654,
        resistances: BTreeMap::from([(
            "Physical".to_string(),
            Ratio {
                numerator: 20,
                denominator: 100,
            },
        )]),
        break_cap: 45,
        current_break_gauge: 45,
        break_state: BreakState::Unbroken,
        auras: BTreeMap::new(),
    }];

    let actors = vec![
        ActorCombatProfile {
            actor_index: 0,
            character_id: "zero".to_string(),
            level: 1,
            base_break: 91,
            break_intensity: Ratio::zero(),
            energy_pool: Pool::new(100, 100).unwrap(),
        },
        ActorCombatProfile {
            actor_index: 1,
            character_id: "mint".to_string(),
            level: 1,
            base_break: 91,
            break_intensity: Ratio::zero(),
            energy_pool: Pool::new(100, 100).unwrap(),
        },
        ActorCombatProfile {
            actor_index: 2,
            character_id: "sakiri".to_string(),
            level: 1,
            base_break: 91,
            break_intensity: Ratio::zero(),
            energy_pool: Pool::new(100, 100).unwrap(),
        },
        ActorCombatProfile {
            actor_index: 3,
            character_id: "lacrimosa".to_string(),
            level: 1,
            base_break: 91,
            break_intensity: Ratio::zero(),
            energy_pool: Pool::new(100, 100).unwrap(),
        },
    ];

    let mut engine = ReactionEngine::new(targets, actors, true).unwrap();

    let rule = TargetingRule::SingleTarget {
        target_id: "boss".to_string(),
    };
    let (_, breaks, _) = engine.process_attack(0, &rule, 100, 45, 0, 0).unwrap();

    assert_eq!(breaks.len(), 1);
    let b = &breaks[0];
    assert_eq!(b.target_id, "boss");
    assert_eq!(b.contributions.len(), 4);
    // Each actor at Lv1 with base_break 91, cap 45, def 654, res 20% deals 525 break damage!
    assert_eq!(b.contributions[0].damage, 525);
    assert_eq!(b.contributions[1].damage, 525);
    assert_eq!(b.contributions[2].damage, 525);
    assert_eq!(b.contributions[3].damage, 525);
    assert_eq!(b.total_break_damage, 2100);

    assert!(matches!(
        engine.targets().get("boss").unwrap().break_state,
        BreakState::Broken { .. }
    ));

    engine.advance_time(6_000_000).unwrap();
    assert_eq!(
        engine.targets().get("boss").unwrap().break_state,
        BreakState::Unbroken
    );
    assert_eq!(
        engine.targets().get("boss").unwrap().current_break_gauge,
        45
    );
}

#[test]
fn no_break_scenario_suppresses_break_events() {
    let targets = vec![TargetProfile {
        id: "boss".to_string(),
        position: Point3D::default(),
        defense: 654,
        resistances: BTreeMap::new(),
        break_cap: 45,
        current_break_gauge: 45,
        break_state: BreakState::Unbroken,
        auras: BTreeMap::new(),
    }];

    let actors = vec![ActorCombatProfile {
        actor_index: 0,
        character_id: "zero".to_string(),
        level: 1,
        base_break: 91,
        break_intensity: Ratio::zero(),
        energy_pool: Pool::new(100, 100).unwrap(),
    }];

    let mut engine = ReactionEngine::new(targets, actors, false).unwrap();

    let rule = TargetingRule::SingleTarget {
        target_id: "boss".to_string(),
    };
    let (_, breaks, _) = engine.process_attack(0, &rule, 100, 100, 0, 0).unwrap();

    assert!(breaks.is_empty());
    assert_eq!(
        engine.targets().get("boss").unwrap().current_break_gauge,
        45
    );
    assert_eq!(
        engine.targets().get("boss").unwrap().break_state,
        BreakState::Unbroken
    );
}

#[test]
fn reaction_scorch_periodic_ticks_and_cap() {
    let targets = vec![TargetProfile {
        id: "enemy".to_string(),
        position: Point3D::default(),
        defense: 1000,
        resistances: BTreeMap::new(),
        break_cap: 100,
        current_break_gauge: 100,
        break_state: BreakState::Unbroken,
        auras: BTreeMap::new(),
    }];

    let actors = vec![ActorCombatProfile {
        actor_index: 0,
        character_id: "test-carrier".into(),
        level: 80,
        base_break: 1,
        break_intensity: Ratio::zero(),
        energy_pool: Pool::new(0, 100).unwrap(),
    }];
    let mut engine = ReactionEngine::new(targets, actors, false).unwrap();

    engine
        .trigger_reaction(0, "enemy", ReactionKind::Scorch, 0)
        .unwrap();
    assert_eq!(engine.active_scorch().get("enemy").unwrap().stacks, 1);

    let ticks = engine.advance_time(2_100_000).unwrap();
    assert_eq!(ticks.len(), 2);
    assert_eq!(ticks[0].damage, SCORCH_BASE_DAMAGE);
    assert_eq!(ticks[1].damage, SCORCH_BASE_DAMAGE);
}

#[test]
fn scorch_delivery_is_independent_of_polling_and_refresh_retains_tick_phase() {
    let target = TargetProfile {
        id: "fresh-target".into(),
        position: Point3D::default(),
        defense: 0,
        resistances: BTreeMap::new(),
        break_cap: 1,
        current_break_gauge: 1,
        break_state: BreakState::Unbroken,
        auras: BTreeMap::new(),
    };
    let actors = (0..2)
        .map(|actor_index| ActorCombatProfile {
            actor_index,
            character_id: format!("fresh-carrier-{actor_index}"),
            level: 80,
            base_break: 1,
            break_intensity: Ratio::zero(),
            energy_pool: Pool::new(0, 100).unwrap(),
        })
        .collect();
    let mut engine = ReactionEngine::new(vec![target], actors, false).unwrap();
    engine
        .trigger_reaction(0, "fresh-target", ReactionKind::Scorch, 0)
        .unwrap();
    let mut sparse = engine.clone();
    let mut frequent = Vec::new();
    for time in [1_000_000, 4_000_000, 15_000_000, 20_000_000] {
        frequent.extend(engine.advance_time(time).unwrap());
    }
    let jumped = sparse.advance_time(20_000_000).unwrap();
    assert_eq!(jumped, frequent);
    assert_eq!(jumped.len(), 15); // floor((15s-epsilon)/0.99s)
    assert!(
        jumped
            .iter()
            .all(|receipt| receipt.source_actor == 0 && receipt.damage == 2700)
    );
    assert!(sparse.active_scorch().is_empty());
    sparse
        .trigger_reaction(0, "fresh-target", ReactionKind::Scorch, 21_000_000)
        .unwrap();
    let advanced_ticks = sparse
        .trigger_reaction(1, "fresh-target", ReactionKind::Scorch, 22_000_000)
        .unwrap();
    assert_eq!(advanced_ticks[0].timestamp_us, 21_990_000);
    assert_eq!(advanced_ticks[0].source_actor, 0);
    let next = sparse.advance_time(22_980_000).unwrap();
    assert_eq!(
        (next[0].timestamp_us, next[0].source_actor, next[0].damage),
        (22_980_000, 1, 5400)
    );
    assert!(sparse.advance_time(22_000_000).is_err());
    let before = sparse.clone();
    assert!(
        sparse
            .trigger_reaction(1, "fresh-target", ReactionKind::Scorch, u64::MAX)
            .is_err()
    );
    assert_eq!(sparse, before);
}

#[test]
fn geometry_handles_all_i64_coordinates_without_fake_coverage_or_signed_overflow() {
    let low = Point3D {
        x_mm: i64::MIN,
        y_mm: i64::MIN,
        z_mm: i64::MIN,
    };
    let high = Point3D {
        x_mm: i64::MAX,
        y_mm: i64::MAX,
        z_mm: i64::MAX,
    };
    assert!(!low.is_within_radius(&high, u64::MAX));
    let midpoint = low.interpolate(&high, 1, 2);
    assert_eq!(
        midpoint,
        Point3D {
            x_mm: -1,
            y_mm: -1,
            z_mm: -1
        }
    );
}

#[test]
fn direct_reaction_construction_validates_profiles_before_break_delivery() {
    let actor = ActorCombatProfile {
        actor_index: 0,
        character_id: "new-carrier".into(),
        level: 80,
        base_break: 100,
        break_intensity: Ratio::zero(),
        energy_pool: Pool::new(0, 100).unwrap(),
    };
    let target = TargetProfile {
        id: "a".into(),
        position: Point3D::default(),
        defense: 0,
        resistances: BTreeMap::from([("Physical".into(), Ratio::zero())]),
        break_cap: 10,
        current_break_gauge: 10,
        break_state: BreakState::Unbroken,
        auras: BTreeMap::new(),
    };
    assert!(
        ReactionEngine::new(
            vec![target.clone(), target.clone()],
            vec![actor.clone()],
            true
        )
        .is_err()
    );
    let mut bad_actor = actor.clone();
    bad_actor.actor_index = 1;
    assert!(ReactionEngine::new(vec![target.clone()], vec![bad_actor], true).is_err());
    let mut second = target.clone();
    second.id = "b".into();
    second.resistances.clear();
    assert!(
        ReactionEngine::new(vec![target.clone(), second], vec![actor.clone()], true)
            .unwrap_err()
            .contains("Physical")
    );
    let mut engine = ReactionEngine::new(vec![target], vec![actor], true).unwrap();
    let before = engine.clone();
    assert!(
        engine
            .process_attack(0, &TargetingRule::AllTargets, 1, 10, 20, u64::MAX)
            .is_err()
    );
    assert_eq!(engine, before);
}
