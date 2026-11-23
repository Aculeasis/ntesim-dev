use super::*;

#[test]
fn dispatch_scenario_rejects_source_lifetime_without_actor_liveness_input() {
    let request = DispatchScenarioRequest {
        content_id: "ntesim-current".into(),
        team_size: 1,
        initial_active: 0,
        initial_resources: BTreeMap::new(),
        steps: vec![DispatchStep::ApplyBuff {
            buff: BuffDefinition {
                stacking_scope: BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                source_lifetime: BuffSourceLifetime::WhileAlive,
                id: "continuous-owner".into(),
                source: "actor".into(),
                recipient: BuffRecipient::TeamAll,
                duration_us: None,
                refresh_policy: RefreshPolicy::Disallow,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: BTreeMap::from([(
                    "atk_add".into(),
                    Ratio {
                        numerator: 1,
                        denominator: 1,
                    },
                )]),
            },
            owner_actor: 0,
            timestamp_us: 0,
        }],
    };
    assert!(matches!(
        evaluate_dispatch_scenario(&request),
        Err(error) if error.contains("MissingSourceLivenessContext")
    ));
}
