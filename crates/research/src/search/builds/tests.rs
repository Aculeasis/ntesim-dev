use super::*;

#[test]
fn unsupported_optimizer_does_not_emit_fabricated_legal_allocations() {
    assert!(
        optimize_character_build("mint")
            .unwrap_err()
            .contains("unsupported")
    );
    assert!(
        search_and_optimize_all_characters()
            .unwrap_err()
            .contains("unsupported")
    );
    assert!(
        builds_to_static_stats(&BTreeMap::new())
            .unwrap_err()
            .contains("source-bound equipment legality receipt")
    );
}

#[test]
fn test_search_and_refine_archetypes() {
    let content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/mint-variant.csv"
    ));
    let limits = ExecutionLimits::default();
    let result = run_published_search(content, &limits).unwrap();

    assert_eq!(result.total_variants, 1);
    assert_eq!(result.total_archetypes, 1);
    assert_eq!(result.total_characters, 0);
    assert_eq!(result.variant_comparisons.len(), 1);
    assert!(result.archetype_rankings.is_empty());
    let encoded = serde_json::to_value(&result).unwrap();
    assert_eq!(encoded["execution_status"], "BASELINE_ONLY");
    assert_eq!(encoded["status"], "INCOMPLETE");
    assert_eq!(encoded["quality"], "incomplete");
    assert_eq!(encoded["game_dps_computed"], false);
    assert_eq!(encoded["acceptance_ready"], false);
    assert!(encoded["total_baseline_damage"].is_number());
    assert_eq!(encoded["total_optimized_damage"], serde_json::Value::Null);
    assert_eq!(encoded["total_percentage_gain"], serde_json::Value::Null);
    assert!(
        encoded["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| {
                blocker
                    .as_str()
                    .unwrap()
                    .contains("shared equipment compiler")
            })
    );

    for comp in &result.variant_comparisons {
        assert!(!comp.readiness.acceptance_ready);
        if comp.status == "completed" {
            assert!(comp.baseline_damage.is_some_and(|damage| damage > 0.0));
            assert!(comp.settled_hits.is_some_and(|hits| hits > 0));
            assert_eq!(comp.invariants_verified, Some(true));
            assert!(comp.failure_reason.is_none());
        } else {
            assert_eq!(comp.status, "failed");
            assert!(
                comp.failure_reason
                    .as_deref()
                    .is_some_and(|reason| !reason.is_empty())
            );
            assert!(comp.baseline_damage.is_none());
            assert!(comp.settled_hits.is_none());
        }
        assert!(comp.optimized_damage.is_none());
        assert!(comp.damage_delta.is_none());
        assert!(comp.percentage_gain.is_none());
    }
}

#[test]
fn character_registry_drives_elements_and_unknown_elements_fail() {
    let registered = ntesim_content::registry::character_descriptors().unwrap();
    assert_eq!(registered.len(), 25);
    for descriptor in registered {
        assert_eq!(
            get_character_element(descriptor.slug)
                .unwrap()
                .to_ascii_uppercase(),
            descriptor.element
        );
    }
    assert!(get_character_element("not-a-carrier").is_err());
}
