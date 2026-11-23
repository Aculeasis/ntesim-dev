use super::*;

#[test]
fn tagged_target_stack_query_is_target_scoped_and_excludes_other_semantics() {
    let dot_tag = "State.Damage.Dot".to_owned();
    let mut manager = BuffManager::default();
    let definition = |id: &str, source: &str, target: &str, tags: &[&str]| BuffDefinition {
        stacking_scope: BuffStackingScope::PerSource,
        semantic_tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
        source_lifetime: Default::default(),
        id: id.into(),
        source: source.into(),
        recipient: BuffRecipient::SpecificTarget(target.into()),
        duration_us: Some(10),
        refresh_policy: RefreshPolicy::Reset,
        max_stacks: 3,
        decay_policy: DecayPolicy::AllAtOnce,
        stat_modifiers: BTreeMap::new(),
    };

    let source_a = definition("dot-a", "actor-a", "0", &[&dot_tag]);
    manager.apply_buff(source_a.clone(), 0, 0).unwrap();
    manager.apply_buff(source_a, 0, 1).unwrap();
    manager
        .apply_buff(definition("dot-b", "actor-b", "0", &[&dot_tag]), 1, 1)
        .unwrap();
    manager
        .apply_buff(
            definition("stun", "actor-c", "0", &["State.Control.Stun"]),
            2,
            1,
        )
        .unwrap();
    manager
        .apply_buff(definition("dot-other", "actor-a", "1", &[&dot_tag]), 0, 1)
        .unwrap();

    assert_eq!(
        manager
            .active_tagged_stacks_for_target("0", &dot_tag, 1)
            .unwrap(),
        3
    );
    assert_eq!(
        manager
            .active_tagged_stacks_for_target("1", &dot_tag, 1)
            .unwrap(),
        1
    );
    assert_eq!(
        manager
            .active_tagged_stacks_for_target("0", "State.Control.Stun", 1)
            .unwrap(),
        1
    );
    assert_eq!(
        manager
            .active_tagged_stacks_for_target("0", &dot_tag, 11)
            .unwrap(),
        0
    );
    manager
        .apply_buff(definition("dot-a", "actor-a", "1", &[&dot_tag]), 0, 1)
        .unwrap();
    manager
        .apply_buff(definition("dot-a", "actor-b", "0", &[&dot_tag]), 1, 1)
        .unwrap();
    assert_eq!(manager.remove_target_buff("dot-a", "actor-a", "0"), 1);
    assert_eq!(
        manager
            .active_tagged_stacks_for_target("0", &dot_tag, 1)
            .unwrap(),
        2
    );
    assert_eq!(
        manager
            .active_tagged_stacks_for_target("1", &dot_tag, 1)
            .unwrap(),
        2
    );
    assert_eq!(
        manager
            .active_tagged_stacks_for_target("0", "State.Control.Stun", 1)
            .unwrap(),
        1
    );
}
