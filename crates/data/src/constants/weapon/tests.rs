use super::*;

// An explicit legacy overlay fixture keeps arbitrary synthetic pack bindings.
const FIXTURE_TABLES: [&str; 5] = [
    "DT_ForkItemData",
    "DT_ForkUpgradeData",
    "DT_ForkBreakthroughData",
    "DT_ForkModifyData",
    "DT_ForkUpgradeStarDataTable",
];

fn fixture() -> (BTreeMap<String, Value>, Request) {
    let rows = [
        json!({"Fork_Test":{"ItemName":{"LocalizedString":"Test"},"ItemQuality":"test","ElementData":{"MaxBreakthrough":6,"MaxUpgradeStar":5,"UpgradePackId":"UP","BreakthroughPackId":"BR","UpgradeStarPackID":"ST","ApplyGroupType":"test"}}}),
        json!({"up_80":{"ModifyPack":"LV"}}),
        json!({"br_6":{"MaxForkLevel":80,"ModifyPackID":"END"},"br_5":{"MaxForkLevel":70,"ModifyPackID":"OLD"}}),
        json!({"lv":{"ConditionArray":[],"ModifyData":[{"ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE","PropName":"AtkBase","PropValue":416}]},"end":{"ConditionArray":[],"ModifyData":[{"ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE","PropName":"AtkBase","PropValue":154},{"ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE","PropName":"CritBase","PropValue":0.24}]},"old":{"ConditionArray":[],"ModifyData":[{"ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE","PropName":"AtkBase","PropValue":9999}]}}),
        json!({"st_1":{},"st_5":{}}),
    ];
    let tables = FIXTURE_TABLES
        .into_iter()
        .zip(rows)
        .map(|(name, rows)| {
            (
                name.to_owned(),
                json!({"schema_version":1,"source_id":format!("table:{name}"),"rows":rows}),
            )
        })
        .collect();
    (
        tables,
        Request {
            schema_version: 1,
            content_id: "ntesim-current".into(),
            weapon_id: "weapon:fork_test".into(),
            level: 80,
            breakthrough: 6,
            refinement: 1,
        },
    )
}

#[test]
fn terminal_pack_and_case_lookup_do_not_sum_historical_breakthroughs() {
    let (tables, request) = fixture();
    let report = compile(&tables, &request).unwrap();
    assert_eq!(
        report["attributes"]["AtkBase"],
        json!({"numerator":"570","denominator":"1"})
    );
    assert_eq!(
        report["attributes"]["CritBase"],
        json!({"numerator":"6","denominator":"25"})
    );
    assert_eq!(report["weapon_id"], "weapon:Fork_Test");
    assert_eq!(report["steps"].as_array().unwrap().len(), 2);
    assert_eq!(report["evidence"].as_array().unwrap().len(), 6);
}

#[test]
fn refinement_changes_selected_row_not_constants() {
    let (tables, mut request) = fixture();
    let first = compile(&tables, &request).unwrap();
    request.refinement = 5;
    let fifth = compile(&tables, &request).unwrap();
    assert_eq!(first["attributes"], fifth["attributes"]);
    assert_eq!(first["selected_refinement_row"], "st_1");
    assert_eq!(fifth["selected_refinement_row"], "st_5");
    assert!(fifth.get("selected_refinement_buffs").is_none());
    request.breakthrough = 5;
    assert!(compile(&tables, &request).is_err());
}

#[test]
fn unknowns_ambiguities_and_unsupported_operations_fail() {
    let (tables, request) = fixture();
    for (table, pointer, value) in [
        (FIXTURE_TABLES[0], "/rows/fork_test", json!({})),
        (
            FIXTURE_TABLES[3],
            "/rows/end/ConditionArray",
            json!(["unknown"]),
        ),
        (
            FIXTURE_TABLES[3],
            "/rows/end/ModifyData/0/ModifierOp",
            json!("multiply"),
        ),
        (
            FIXTURE_TABLES[3],
            "/rows/end/ModifyData/0/PropValue",
            Value::Null,
        ),
        (
            FIXTURE_TABLES[3],
            "/rows/end/ModifyData/0/PropValue",
            json!(-1),
        ),
        (FIXTURE_TABLES[4], "/rows/st_1", Value::Null),
        (FIXTURE_TABLES[1], "/source_id", json!("wrong")),
    ] {
        let mut bad = tables.clone();
        if pointer == "/rows/fork_test" {
            bad.get_mut(table).unwrap()["rows"]["fork_test"] = value;
        } else {
            *bad.get_mut(table).unwrap().pointer_mut(pointer).unwrap() = value;
        }
        assert!(compile(&bad, &request).is_err(), "{table}:{pointer}");
    }
    let mut bad = request.clone();
    bad.weapon_id = "Test".into();
    assert!(compile(&tables, &bad).is_err());
    for level in [0, 81] {
        bad = request.clone();
        bad.level = level;
        assert!(compile(&tables, &bad).is_err());
    }
    for refinement in [0, 6] {
        bad = request.clone();
        bad.refinement = refinement;
        assert!(compile(&tables, &bad).is_err());
    }
    bad = request.clone();
    bad.breakthrough = 7;
    assert!(compile(&tables, &bad).is_err());
}

#[test]
fn exact_decimals_scientific_notation_and_overflow_are_explicit() {
    assert_eq!(Scalar::parse(&json!(0.4125)).unwrap(), Scalar::new(33, 80));
    assert_eq!(
        Scalar::parse(&json!(1e-7)).unwrap(),
        Scalar::new(1, 10_000_000)
    );
    assert!(Scalar::parse(&json!(1e100)).is_err());
    assert!(Scalar::parse(&json!(true)).is_err());
    assert!(Scalar::new(u128::MAX, 1).add(Scalar::new(1, 1)).is_err());
    let a = Scalar::parse(&json!(0.1)).unwrap();
    let b = Scalar::parse(&json!(0.2)).unwrap();
    assert_eq!(a.add(b).unwrap(), Scalar::new(3, 10));
}
