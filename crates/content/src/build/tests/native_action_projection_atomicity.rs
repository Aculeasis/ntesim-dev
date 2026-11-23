use super::*;

#[test]
fn late_packet_failure_preserves_all_original_actions_and_allows_a_clean_retry() {
    let (tables, mut scenario, projection) = action_fixture();
    if let GameActionCommand::InvokeAbility {
        atk_rate,
        ability_id,
        ..
    } = &mut scenario.actions[0]
    {
        *atk_rate = Ratio::one();
        *ability_id = ability_id.to_ascii_lowercase();
    }
    let mut broken = tables.clone();
    broken.get_mut("DT_SkillDamageData").unwrap()["rows"]
        .as_object_mut()
        .unwrap()
        .remove("GE_Player_Mint_Skill1_Damage_New");
    let before = serde_json::to_value(&scenario).unwrap();
    let error = project_actions(&broken, &mut scenario, &projection).unwrap_err();
    assert!(
        error.contains("GE_Player_Mint_Skill1_Damage_New"),
        "{error}"
    );
    assert_eq!(serde_json::to_value(&scenario).unwrap(), before);

    let mut fresh: GameScenarioRequest = serde_json::from_value(before).unwrap();
    let retried = project_actions(&tables, &mut scenario, &projection).unwrap();
    let first_try = project_actions(&tables, &mut fresh, &projection).unwrap();
    assert_eq!(retried, first_try);
    assert_eq!(
        serde_json::to_value(&scenario).unwrap(),
        serde_json::to_value(&fresh).unwrap()
    );
    assert_eq!(
        run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap(),
        run_game_scenario(&fresh, &ExecutionLimits::default()).unwrap()
    );
}

#[test]
fn failed_level_projection_preserves_coefficients_and_coordinates() {
    let (tables, scenario, mut projection) = action_fixture();
    projection.policy = ActionProjectionPolicy::NativeSkillLevelV1;
    // The first action can select its new level; the second selection is absent.
    let levels = BTreeMap::from([(
        0,
        BTreeMap::from([("GA_Mint019_Melee".into(), json!({"array_index":0}))]),
    )]);
    let before = serde_json::to_value(&scenario).unwrap();
    let mut failed = scenario.clone();
    assert!(
        project_actions_with_levels(&tables, &mut failed, &projection, &levels)
            .unwrap_err()
            .contains("exactly one purchased skill level")
    );
    assert_eq!(serde_json::to_value(&failed).unwrap(), before);
}
