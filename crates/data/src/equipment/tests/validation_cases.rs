use super::*;

#[test]
fn every_static_pack_producer_requires_explicit_condition_metadata() {
    let tables = BTreeMap::new();
    let request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/build-stats.json"
    )))
    .unwrap();
    let baseline = compile(&tables, &request).unwrap();
    let mut reached = Vec::new();
    for source in baseline["evidence"].as_array().unwrap() {
        let Some(table) = source["table"].as_str() else {
            continue;
        };
        let row = source["row"].as_str().unwrap();
        if [
            crate::weapon_modifiers::TABLE_NAME,
            crate::set_modifiers::TABLE_NAME,
        ]
        .contains(&table)
        {
            reached.push((table.to_owned(), row.to_owned()));
        }
    }

    assert!(
        reached
            .iter()
            .any(|(table, _)| table == crate::set_modifiers::TABLE_NAME)
    );
    for (table, row) in reached {
        for malformed in [Value::Null, json!(false), json!(["requires combat state"])] {
            let mut changed = tables.clone();
            let table = if table == crate::set_modifiers::TABLE_NAME {
                let mut overlay = crate::set_modifiers::transport().clone();
                overlay["source_id"] = json!("table:DT_EquipmentModifyData");
                changed.insert("DT_EquipmentModifyData".into(), overlay);
                "DT_EquipmentModifyData"
            } else {
                changed.insert(
                    "DT_ForkModifyData".into(),
                    crate::weapon_modifiers::overlay(),
                );
                "DT_ForkModifyData"
            };
            changed.get_mut(table).unwrap()["rows"][&row]["ConditionArray"] = malformed;
            assert!(compile(&changed, &request).is_err(), "{table}:{row}");
        }
    }
}
