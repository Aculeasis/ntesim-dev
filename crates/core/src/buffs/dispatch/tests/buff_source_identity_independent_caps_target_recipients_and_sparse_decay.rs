use super::*;

#[test]
fn buff_source_identity_independent_caps_target_recipients_and_sparse_decay() {
    let buff = BuffDefinition {
        stacking_scope: BuffStackingScope::PerSource,
        semantic_tags: Default::default(),
        source_lifetime: Default::default(),
        id: "new-buff".into(),
        source: "new-item".into(),
        recipient: BuffRecipient::SelfActor,
        duration_us: Some(10),
        refresh_policy: RefreshPolicy::Independent,
        max_stacks: 2,
        decay_policy: DecayPolicy::AllAtOnce,
        stat_modifiers: BTreeMap::from([(
            "atk_percent".into(),
            Ratio {
                numerator: 1,
                denominator: 10,
            },
        )]),
    };
    let mut manager = BuffManager::default();
    manager.apply_buff(buff.clone(), 0, 0).unwrap();
    manager.apply_buff(buff.clone(), 0, 2).unwrap();
    manager.apply_buff(buff.clone(), 0, 3).unwrap(); // cap does not refresh
    manager.apply_buff(buff.clone(), 1, 3).unwrap(); // another carrier is independent
    assert_eq!(manager.active_buffs.len(), 3);
    assert_eq!(
        manager.get_stat_modifiers_for_actor(0, 0, 10).unwrap()["atk_percent"],
        Ratio {
            numerator: 1,
            denominator: 10
        }
    );
    assert!(
        manager
            .get_stat_modifiers_for_actor(0, 0, 12)
            .unwrap()
            .is_empty()
    );
    let mut gradual = buff.clone();
    gradual.refresh_policy = RefreshPolicy::Reset;
    gradual.max_stacks = 3;
    gradual.decay_policy = DecayPolicy::OneByOne;
    gradual.recipient = BuffRecipient::SpecificTarget("target-a".into());
    let mut manager = BuffManager::default();
    for _ in 0..3 {
        manager.apply_buff(gradual.clone(), 0, 0).unwrap();
    }
    assert!(
        manager
            .get_stat_modifiers_for_actor(0, 0, 0)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        manager
            .get_stat_modifiers_for_target("target-a", 21)
            .unwrap()["atk_percent"],
        Ratio {
            numerator: 1,
            denominator: 10
        }
    );
    let mut advanced = manager.clone();
    advanced.advance_time(21);
    assert_eq!(
        advanced
            .get_stat_modifiers_for_target("target-a", 21)
            .unwrap(),
        manager
            .get_stat_modifiers_for_target("target-a", 21)
            .unwrap()
    );
    assert!(
        manager
            .get_stat_modifiers_for_target("target-a", 30)
            .unwrap()
            .is_empty()
    );
}
