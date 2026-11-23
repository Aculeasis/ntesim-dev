use super::*;

#[test]
pub(crate) fn child_scheduling_failure_rolls_back_admission_and_every_pool() {
    let input: Fixture = serde_json::from_value(actor_fixture()).unwrap();
    let actor = &input.actors["hero"];
    let skill = &actor.actions["skill"];
    let mut actors = BTreeMap::from([(
        "hero".into(),
        Gate::new(
            1,
            0,
            0,
            BTreeMap::from([(
                "skill".into(),
                ActionState {
                    rule: skill.rule.clone(),
                    ready_at_us: 0,
                },
            )]),
        )
        .unwrap(),
    )]);
    let mut resources = BTreeMap::from([
        ("energy".into(), Pool::new(6, 10).unwrap()),
        ("points".into(), Pool::new(3, 3).unwrap()),
    ]);
    let before = (actors.clone(), resources.clone());
    let mut queue = Scheduler::default();
    queue.add_clock(1, 1_000_000).unwrap();
    assert!(
        invoke(
            Invocation {
                index: 0,
                clock: 1,
                local_time_us: 0,
                cause: 42
            },
            "hero",
            "skill",
            &skill.effects,
            &mut actors,
            &mut resources,
            &mut queue
        )
        .is_err()
    );
    assert_eq!((actors, resources), before);
    assert_eq!(queue.pending_count(), 0);
}
