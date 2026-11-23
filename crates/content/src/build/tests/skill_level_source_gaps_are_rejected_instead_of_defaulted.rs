use super::*;

#[test]
fn skill_level_source_gaps_are_rejected_instead_of_defaulted() {
    let (mut tables, coverage, request) = skill_fixture();
    tables.insert(
        "DT_CharacterAbilityConfig".into(),
        ntesim_data::character_actions::overlay(),
    );
    tables.insert(
        "1019AwakenEffect1".into(),
        ntesim_data::resonance_levels::overlay("1019").unwrap(),
    );
    for (table, pointer, value) in [
        (
            "DT_CharacterAbilityConfig",
            "/rows/1019/CharacterAbilityList/0/Value/LevelsCostItems/1/Level",
            json!(1),
        ),
        (
            "DT_CharacterAbilityConfig",
            "/rows/1019/CharacterAbilityList/0/Value/LevelsCostItems/0/RequireTupoLevel",
            Value::Null,
        ),
        (
            "1019AwakenEffect1",
            "/rows/1019/AwakenEffectStructList/0/ModifyDataList/0/SkillLevel",
            json!(2),
        ),
    ] {
        let mut changed = tables.clone();
        *changed
            .get_mut(table)
            .unwrap()
            .pointer_mut(pointer)
            .unwrap() = value;
        assert!(compile(&changed, &coverage, &request, &ExecutionLimits::default()).is_err());
    }
}
