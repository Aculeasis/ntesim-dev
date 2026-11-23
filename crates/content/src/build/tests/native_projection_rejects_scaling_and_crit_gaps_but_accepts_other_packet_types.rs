use super::*;

#[test]
fn native_projection_rejects_scaling_and_crit_gaps_but_accepts_other_packet_types() {
    let (tables, scenario, projection) = action_fixture();
    let row = "GE_Player_Mint_Melee1_Damage_New";
    for (field, value, expected) in [
        (
            "AtkRateBaseArray",
            json!([]),
            "no selected ATK or DEF scaling",
        ),
        (
            "HPRateBaseArray",
            json!([1, 1, 1, 1, 1, 1, 1, 1]),
            "native action HP scaling",
        ),
    ] {
        let mut changed = tables.clone();
        changed.get_mut("DT_SkillDamageData").unwrap()["rows"][row][field] = value;
        let error = project_actions(&changed, &mut scenario.clone(), &projection).unwrap_err();
        assert!(error.contains(expected), "{field}: {error}");
    }
    let mut changed = tables.clone();
    changed.get_mut("DT_SkillDamageData").unwrap()["rows"][row]["FixedCritRate"] = json!(1);
    assert!(
        project_actions(&changed, &mut scenario.clone(), &projection)
            .unwrap_err()
            .contains(crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID.as_str())
    );
    let mut accepted_scenario = scenario.clone();
    accepted_scenario.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    let receipts = project_actions(&changed, &mut accepted_scenario, &projection).unwrap();
    assert_eq!(receipts.len(), 3);
    for damage_type in [
        "EExecutionDamageTypeEX::DAMAGE_TYPE_CHAOS",
        "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHICALLY",
        "EExecutionDamageTypeEX::DAMAGE_TYPE_TRUE",
    ] {
        let mut changed = tables.clone();
        changed.get_mut("DT_SkillDamageData").unwrap()["rows"][row]["DamageTypeEX"] =
            json!(damage_type);
        let mut changed_scenario = scenario.clone();
        let receipts = project_actions(&changed, &mut changed_scenario, &projection).unwrap();
        assert_eq!(receipts.len(), 3, "{damage_type}");
        assert_eq!(
            receipts[0]["native_damage_report"]["effects"][0]["damage_type"],
            json!(damage_type)
        );
    }
}
