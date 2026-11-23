use super::*;

#[test]
fn buff_stacking_and_decay_policies() {
    let mut manager = BuffManager::default();

    let mut mods = BTreeMap::new();
    mods.insert(
        "dmg_bonus".to_string(),
        Ratio {
            numerator: 10,
            denominator: 100,
        },
    );

    let buff = BuffDefinition {
        stacking_scope: BuffStackingScope::PerSource,
        semantic_tags: Default::default(),
        source_lifetime: Default::default(),
        id: "warrior_stacks".to_string(),
        source: "passive".to_string(),
        recipient: BuffRecipient::SelfActor,
        duration_us: Some(2_000_000),
        refresh_policy: RefreshPolicy::Reset,
        max_stacks: 3,
        decay_policy: DecayPolicy::OneByOne,
        stat_modifiers: mods,
    };

    manager.apply_buff(buff.clone(), 0, 0).unwrap();
    manager.apply_buff(buff.clone(), 0, 100_000).unwrap();
    manager.apply_buff(buff.clone(), 0, 200_000).unwrap();

    assert_eq!(manager.active_buffs[0].stacks, 3);

    manager.advance_time(2_200_000);
    assert_eq!(manager.active_buffs[0].stacks, 2);

    manager.advance_time(4_200_000);
    assert_eq!(manager.active_buffs[0].stacks, 1);

    manager.advance_time(6_200_000);
    assert!(manager.active_buffs.is_empty());
}
