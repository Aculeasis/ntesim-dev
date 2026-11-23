use super::*;

#[test]
fn extend_refreshes_snapshot_but_disallow_retains_original_snapshot() {
    for policy in [
        RefreshPolicy::Extend {
            max_duration_us: Some(12),
        },
        RefreshPolicy::Disallow,
    ] {
        let mut manager = BuffManager::default();
        let mut def = synthetic_buff(BuffStackingScope::Shared, policy.clone());
        manager.apply_buff(def.clone(), 0, 0).unwrap();
        def.source = "second".into();
        def.stat_modifiers.get_mut("atk").unwrap().numerator = 13;
        manager.apply_buff(def, 1, 5).unwrap();
        let expected = if policy == RefreshPolicy::Disallow {
            7
        } else {
            13
        };
        assert_eq!(
            manager.get_stat_modifiers_for_actor(0, 0, 5).unwrap()["atk"],
            Ratio {
                numerator: expected,
                denominator: 2
            }
        );
        let expiry = if policy == RefreshPolicy::Disallow {
            10
        } else {
            17
        };
        assert!(
            manager
                .get_stat_modifiers_for_actor(0, 0, expiry)
                .unwrap()
                .is_empty()
        );
    }
}
