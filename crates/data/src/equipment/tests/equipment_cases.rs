use super::*;

#[test]
pub(crate) fn native_weapon_group_binding_rejects_mismatch_without_inventing_exclusive_semantics() {
    let tables = group_tables();
    let mint =
        weapon_group_binding(&tables, "character:1019", "weapon:FORK_LINGGANZHONGJIEZHE").unwrap();
    assert_eq!(mint["weapon_id"], "weapon:fork_lingganzhongjiezhe");
    assert_eq!(mint["groups_match"], true);
    assert_eq!(mint["evidence"][0]["kind"], "typed_character_definition");
    assert!(
        weapon_group_binding(&tables, "character:1019", "weapon:fork_mofeikesi")
            .unwrap_err()
            .contains("incompatible")
    );
    let same_group = weapon_group_binding(&tables, "character:1019", "weapon:fork_Rose").unwrap();
    assert_eq!(same_group["exclusive_character_ids"], json!(["1004"]));
    assert_eq!(same_group["exclusive_character_semantics_validated"], false);
    assert_eq!(same_group["full_compatibility_validated"], false);
}

#[test]
pub(crate) fn native_weapon_groups_require_known_values_even_when_unknown_tokens_match() {
    let original = group_tables();
    for token in [
        Value::Null,
        json!(""),
        json!("ECharacterGroupType::CHARACTER_GROUP_TYPE_SIX"),
        json!(7),
    ] {
        let mut tables = original.clone();
        tables.get_mut("DT_ForkItemData").unwrap()["rows"]["fork_lingganzhongjiezhe"]["ElementData"]
            ["ApplyGroupType"] = token;
        assert!(
            weapon_group_binding(&tables, "character:1019", "weapon:fork_lingganzhongjiezhe")
                .is_err()
        );
    }
    for (suffix, id) in [
        ("ONE", "1020"),
        ("TWO", "1019"),
        ("THREE", "1010"),
        ("FOUR", "1003"),
        ("FIVE", "1033"),
    ] {
        let mut tables = original.clone();
        let token = json!(format!(
            "ECharacterGroupType::CHARACTER_GROUP_TYPE_{suffix}"
        ));
        tables.get_mut("DT_ForkItemData").unwrap()["rows"]["fork_lingganzhongjiezhe"]["ElementData"]
            ["ApplyGroupType"] = token;
        assert!(
            weapon_group_binding(
                &tables,
                &format!("character:{id}"),
                "weapon:fork_lingganzhongjiezhe"
            )
            .is_ok()
        );
    }
}

#[test]
pub(crate) fn empty_listed_parameters_do_not_establish_passive_execution() {
    let mut tables = BTreeMap::from([(
        "DT_ForkUpgradeStarDataTable".into(),
        crate::weapon_refinements::overlay(),
    )]);
    tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_lingganzhongjiezhe_5"]
        ["DataList"] = json!([]);
    let request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../reviews/s1-mint-a0/build.json"
    )))
    .unwrap();
    let report = compile(&tables, &request).unwrap();
    assert!(report.get("selected_weapon_buffs").is_none());
    assert_eq!(report["unresolved"]["weapon_passive_execution"], true);
    assert!(report["weapon_refinement_binding"].get("buffs").is_none());
    assert_eq!(
        report["weapon_refinement_binding"]["passive_execution_validated"],
        false
    );
    assert_eq!(report["quality"], "incomplete");
}
