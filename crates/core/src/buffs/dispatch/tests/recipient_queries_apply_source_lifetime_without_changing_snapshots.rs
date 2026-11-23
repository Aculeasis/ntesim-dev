use super::*;

#[test]
fn recipient_queries_apply_source_lifetime_without_changing_snapshots() {
    let mut manager = BuffManager::default();
    let make = |id: &str, source_lifetime| BuffDefinition {
        stacking_scope: BuffStackingScope::PerSource,
        semantic_tags: Default::default(),
        source_lifetime,
        id: id.into(),
        source: id.into(),
        recipient: BuffRecipient::TeamAll,
        duration_us: None,
        refresh_policy: RefreshPolicy::Disallow,
        max_stacks: 1,
        decay_policy: DecayPolicy::AllAtOnce,
        stat_modifiers: BTreeMap::from([(
            "crit_damage_bonus_dot".into(),
            Ratio {
                numerator: 1,
                denominator: 2,
            },
        )]),
    };
    manager
        .apply_buff(make("continuous", BuffSourceLifetime::WhileAlive), 0, 0)
        .unwrap();
    manager
        .apply_buff(make("snapshot", BuffSourceLifetime::Snapshot), 1, 0)
        .unwrap();

    let living = std::collections::BTreeSet::from([0, 1]);
    assert_eq!(
        manager
            .get_stat_modifiers_for_actor_with_living_sources(2, 2, 0, &living)
            .unwrap()["crit_damage_bonus_dot"],
        Ratio {
            numerator: 1,
            denominator: 1
        }
    );
    let living = std::collections::BTreeSet::from([1]);
    assert_eq!(
        manager
            .get_stat_modifiers_for_actor_with_living_sources(2, 2, 0, &living)
            .unwrap()["crit_damage_bonus_dot"],
        Ratio {
            numerator: 1,
            denominator: 2
        }
    );
    assert_eq!(
        manager.get_stat_modifiers_for_actor(2, 2, 0),
        Err(DispatchError::MissingSourceLivenessContext)
    );
}
