use super::*;

#[test]
fn explicit_owner_packet_keeps_configured_parent_and_checks_native_child_identity() {
    let (mut tables, request) = fixture();
    tables.get_mut("DT_SkillDamageData").unwrap()["rows"]["Damage_A"]["GAName"] = json!("GA_Child");
    assert!(
        compile(&tables, &request)
            .unwrap_err()
            .contains("belongs to another ability")
    );
    assert!(compile_owner_packet(&tables, &request, "GA_Other").is_err());
    let report = compile_owner_packet(&tables, &request, "GA_Child").unwrap();
    assert_eq!(
        report["effects"][0]["arrays"]["AtkRateBaseArray"]["selected_value"],
        json!({"numerator":"771","denominator":"500"})
    );
    let mut unconfigured = request.clone();
    unconfigured.ability_id = "GA_Child".into();
    assert!(
        compile_owner_packet(&tables, &unconfigured, "GA_Child")
            .unwrap_err()
            .contains("configured character action")
    );
}
fn fixture() -> (BTreeMap<String, Value>, Request) {
    let rows = [
        json!({"Damage_A":{"GAName":"GA_Test","AtkRateBaseArray":[0.9,1.542],"DefRateBaseArray":[],"HPRateBaseArray":[],
            "ChargeAdd":2.45,"HeterochromeAdd":4.1,"UnbalValue":0.5,"FixedCritRate":0,"StroyBlanceGERate":0.9,
            "DamageTypeEX":"native:NATURE","DamageSourceCategory":"native:E"}}),
    ];
    let mut tables: BTreeMap<String, Value> = ["DT_SkillDamageData"]
        .into_iter()
        .zip(rows)
        .map(|(name, rows)| {
            (
                name.to_owned(),
                json!({"schema_version":1,"source_id":format!("table:{name}"),"rows":rows}),
            )
        })
        .collect();
    tables.insert(
        "DT_CharacterAbilityConfig".into(),
        json!({
            "schema_version":1,"source_id":"table:DT_CharacterAbilityConfig",
            "rows":{"1019":{"CharacterAbilityList":[{"Key":"GA_Test"}]}}
        }),
    );
    (
        tables,
        Request {
            schema_version: 1,
            content_id: "ntesim-current".into(),
            character_id: "character:1019".into(),
            ability_id: "ga_test".into(),
            effect_ids: vec!["damage_a".into()],
            array_index: 1,
        },
    )
}
#[test]
fn exact_values_and_absence_are_distinct_from_runtime_zero() {
    let (tables, request) = fixture();
    let report = compile(&tables, &request).unwrap();
    assert_eq!(
        report["effects"][0]["arrays"]["AtkRateBaseArray"]["selected_value"],
        json!({"numerator":"771","denominator":"500"})
    );
    assert_eq!(
        report["effects"][0]["scalars"]["ChargeAdd"],
        json!({"numerator":"49","denominator":"20"})
    );
    assert_eq!(
        report["effects"][0]["arrays"]["DefRateBaseArray"]["state"],
        "declared_empty"
    );
    assert!(report["effects"][0]["arrays"]["DefRateBaseArray"]["selected_value"].is_null());
    assert!(report["effects"][0]["hit_count"].is_null());
}
#[test]
fn unused_native_cost_payload_does_not_enter_damage_values() {
    let (mut tables, request) = fixture();
    let before = compile(&tables, &request).unwrap();
    tables.insert(
        "DT_GameplayAbilityCostData".into(),
        json!({"rows":{
            "GA_Test":{"GACost":100,"GAStrengthCost":0}
        }}),
    );
    let report = compile(&tables, &request).unwrap();
    assert_eq!(report, before);
    assert!(report.get("ability_cost").is_none());
}
#[test]
fn malformed_or_unselected_bad_source_values_fail() {
    let (tables, request) = fixture();
    for (pointer, value) in [
        ("/rows/Damage_A/ChargeAdd", Value::Null),
        ("/rows/Damage_A/AtkRateBaseArray/0", json!(-1)),
        ("/rows/Damage_A/AtkRateBaseArray/0", json!(1e100)),
        ("/rows/Damage_A/DefRateBaseArray", Value::Null),
        ("/rows/Damage_A/GAName", json!("GA_Other")),
    ] {
        let mut bad = tables.clone();
        *bad.get_mut("DT_SkillDamageData")
            .unwrap()
            .pointer_mut(pointer)
            .unwrap() = value;
        assert!(compile(&bad, &request).is_err(), "{pointer}");
    }
}
#[test]
fn ownership_identity_and_coordinate_requests_are_checked() {
    let (tables, request) = fixture();
    for case in 0..6 {
        let mut bad = request.clone();
        match case {
            0 => bad.array_index = 2,
            1 => bad.effect_ids.push("Damage_A".into()),
            2 => bad.effect_ids.clear(),
            3 => bad.ability_id = "GA_Other".into(),
            4 => bad.content_id = "synthetic-v1".into(),
            _ => bad.effect_ids = vec!["damage_a".into(); 65],
        }
        assert!(compile(&tables, &bad).is_err());
    }
}
#[test]
fn unused_effect_class_mapping_does_not_enter_damage_values() {
    let (mut tables, request) = fixture();
    let expected = compile(&tables, &request).unwrap();
    tables.insert(
        "DT_GameplayEffectMappingData".into(),
        json!({"rows":{
            "damage_a":{"GameplayEffectClass":null,"UniqueIndex":269}, "Buffï¼ˆ1ï¼‰":{}
        }}),
    );
    let actual = compile(&tables, &request).unwrap();
    assert_eq!(actual, expected);
    assert!(actual["effects"][0].get("effect_class").is_none());
    assert!(actual["effects"][0].get("mapping_unique_index").is_none());
}
