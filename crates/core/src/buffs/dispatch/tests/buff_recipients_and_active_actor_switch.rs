use super::*;

#[test]
fn buff_recipients_and_active_actor_switch() {
    let mut manager = BuffManager::default();

    let mut mods = BTreeMap::new();
    mods.insert(
        "atk_percent".to_string(),
        Ratio {
            numerator: 20,
            denominator: 100,
        },
    );

    let buff = BuffDefinition {
        stacking_scope: BuffStackingScope::PerSource,
        semantic_tags: Default::default(),
        source_lifetime: Default::default(),
        id: "bennett_burst".to_string(),
        source: "bennett".to_string(),
        recipient: BuffRecipient::ActiveActor,
        duration_us: Some(10_000_000),
        refresh_policy: RefreshPolicy::Reset,
        max_stacks: 1,
        decay_policy: DecayPolicy::AllAtOnce,
        stat_modifiers: mods,
    };

    manager.apply_buff(buff, 0, 0).unwrap();

    let mods_0 = manager
        .get_stat_modifiers_for_actor(0, 0, 1_000_000)
        .unwrap();
    assert_eq!(
        *mods_0.get("atk_percent").unwrap(),
        Ratio {
            numerator: 1,
            denominator: 5
        }
    );

    let mods_1 = manager
        .get_stat_modifiers_for_actor(1, 0, 1_000_000)
        .unwrap();
    assert!(!mods_1.contains_key("atk_percent"));

    let mods_0_after = manager
        .get_stat_modifiers_for_actor(0, 1, 1_000_000)
        .unwrap();
    assert!(!mods_0_after.contains_key("atk_percent"));

    let mods_1_after = manager
        .get_stat_modifiers_for_actor(1, 1, 1_000_000)
        .unwrap();
    assert_eq!(
        *mods_1_after.get("atk_percent").unwrap(),
        Ratio {
            numerator: 1,
            denominator: 5
        }
    );
}
