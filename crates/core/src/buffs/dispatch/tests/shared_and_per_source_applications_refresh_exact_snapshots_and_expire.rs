use super::*;

#[test]
fn shared_and_per_source_applications_refresh_exact_snapshots_and_expire() {
    for scope in [BuffStackingScope::Shared, BuffStackingScope::PerSource] {
        let mut manager = BuffManager::default();
        let mut def = synthetic_buff(scope, RefreshPolicy::Reset);
        manager.apply_buff(def.clone(), 0, 0).unwrap();
        def.source = "new-unlisted-carrier".into();
        def.stat_modifiers.get_mut("atk").unwrap().numerator = 11;
        manager.apply_buff(def, 1, 5).unwrap();
        // Independent arithmetic: Shared is latest 11/2; PerSource is 7/2+11/2=9.
        assert_eq!(
            manager.get_stat_modifiers_for_actor(2, 2, 5).unwrap()["atk"],
            if scope == BuffStackingScope::Shared {
                Ratio {
                    numerator: 11,
                    denominator: 2,
                }
            } else {
                Ratio {
                    numerator: 9,
                    denominator: 1,
                }
            }
        );
        assert_eq!(
            manager.get_stat_modifiers_for_actor(2, 2, 10).unwrap()["atk"],
            Ratio {
                numerator: 11,
                denominator: 2
            }
        );
        assert!(
            manager
                .get_stat_modifiers_for_actor(2, 2, 15)
                .unwrap()
                .is_empty()
        );
    }
}
