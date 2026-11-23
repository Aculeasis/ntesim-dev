use super::*;

#[test]
fn fireflies_hypotheses_require_exact_strict_acceptance() {
    let mut request = sample_s1_request();
    for (id, description) in [
        (
            crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.as_str(),
            crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.description(),
        ),
        (
            crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.as_str(),
            crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.description(),
        ),
    ] {
        let mut resolved = BTreeMap::new();
        crate::runtime_api::record_hypothesis(&request, &mut resolved, id, description).unwrap();
        assert_eq!(resolved[id], description);

        request
            .model_assumptions
            .insert(id.into(), description.into());
        let mut resolved = BTreeMap::new();
        crate::runtime_api::record_hypothesis(&request, &mut resolved, id, description).unwrap();
        crate::simulation::validate_model_assumptions(&request).unwrap();
        assert_eq!(resolved[id], description);

        request
            .model_assumptions
            .insert(id.into(), "mismatched hypothesis".into());
        let mismatch =
            crate::runtime_api::record_hypothesis(&request, &mut BTreeMap::new(), id, description)
                .unwrap_err();
        assert!(mismatch.contains("conflicting reached model hypothesis"));
        request.model_assumptions.remove(id);
    }
}
