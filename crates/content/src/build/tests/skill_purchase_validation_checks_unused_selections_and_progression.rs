use super::*;

#[test]
fn skill_purchase_validation_checks_unused_selections_and_progression() {
    let (tables, coverage, request) = skill_fixture();
    for level in [0, 11, u32::MAX] {
        let mut changed = request.clone();
        changed.builds[0]
            .purchased_skill_levels
            .as_mut()
            .unwrap()
            .insert("GA_Mint019_QTE".into(), level);
        assert!(
            compile(&tables, &coverage, &changed, &ExecutionLimits::default())
                .unwrap_err()
                .contains("purchased skill level")
        );
    }
    for (level, breakthrough, valid) in [
        (1, 0, true),
        (2, 1, false),
        (2, 2, true),
        (8, 4, false),
        (8, 5, true),
        (10, 5, false),
        (10, 6, true),
    ] {
        assert_eq!(
            ntesim_data::skill_levels::compile(
                &tables,
                "character:1019",
                "GA_Mint019_Skill",
                level,
                breakthrough,
                0
            )
            .is_ok(),
            valid
        );
    }
    let (bonus, _) = crate::characters::mint::levels::bonus(
        &tables,
        "GA_Mint019_QTE",
        &ntesim_data::AwakeningSelection::prefix(6),
    )
    .unwrap();
    assert_eq!(bonus, 0);
    let mut duplicate = request.clone();
    duplicate.builds[0]
        .purchased_skill_levels
        .as_mut()
        .unwrap()
        .insert("ga_mint019_skill".into(), 8);
    assert!(
        compile(&tables, &coverage, &duplicate, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicate canonical")
    );
    let mut missing = request.clone();
    missing.builds[0]
        .purchased_skill_levels
        .as_mut()
        .unwrap()
        .remove("GA_Mint019_Skill");
    assert!(
        compile(&tables, &coverage, &missing, &ExecutionLimits::default())
            .unwrap_err()
            .contains("exactly one purchased")
    );
    let mut old_policy = request;
    old_policy.action_projection.as_mut().unwrap().policy =
        ActionProjectionPolicy::NativeAtkCoordinateV1;
    assert!(
        compile(&tables, &coverage, &old_policy, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires native_mint")
    );
}
