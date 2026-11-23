use super::*;

#[test]
fn owned_removal_keeps_other_sources_owners_and_effect_ids() {
    let mut manager = BuffManager::default();
    for (id, source, owner) in [
        ("field", "weapon", 0),
        ("field", "weapon", 1),
        ("field", "ally", 0),
        ("other", "weapon", 0),
    ] {
        manager
            .apply_buff(
                BuffDefinition {
                    stacking_scope: BuffStackingScope::PerSource,
                    semantic_tags: Default::default(),
                    source_lifetime: Default::default(),
                    id: id.into(),
                    source: source.into(),
                    recipient: BuffRecipient::SelfActor,
                    duration_us: None,
                    refresh_policy: RefreshPolicy::Reset,
                    max_stacks: 1,
                    decay_policy: DecayPolicy::AllAtOnce,
                    stat_modifiers: BTreeMap::from([(
                        "atk_percent".into(),
                        Ratio {
                            numerator: 1,
                            denominator: 10,
                        },
                    )]),
                },
                owner,
                0,
            )
            .unwrap();
    }
    assert_eq!(manager.remove_owned_buff("field", "weapon", 0), 1);
    assert_eq!(manager.instances().len(), 3);
    assert_eq!(manager.remove_owned_buff("field", "weapon", 0), 0);
    assert_eq!(
        manager.get_stat_modifiers_for_actor(0, 0, 0).unwrap()["atk_percent"],
        Ratio {
            numerator: 1,
            denominator: 5
        }
    );
    assert_eq!(
        manager.get_stat_modifiers_for_actor(1, 0, 0).unwrap()["atk_percent"],
        Ratio {
            numerator: 1,
            denominator: 10
        }
    );
}
