//! Consumed level-to-modifier bindings for the 51 supported weapons.
//! Ten referenced upgrade groups each expose levels 1..=80. NeedExp and the
//! ten unreferenced source groups are cache-only. Exact stat values remain
//! owned by the modifier compiler; these bindings do not approximate growth.
use crate::native::Table;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub const TABLE_NAME: &str = "typed_weapon_upgrade_levels";
pub const SOURCE_ID: &str = "table:typed_weapon_upgrade_levels";
const OVERLAY_TABLE: &str = "DT_ForkUpgradeData";

pub const PACKS: &[&str] = &[
    "upgrade_pack_Fork_rank3_1",
    "upgrade_pack_Fork_rank3_3",
    "upgrade_pack_Fork_rank4_1",
    "upgrade_pack_Fork_rank4_3",
    "upgrade_pack_Fork_rank4_5",
    "upgrade_pack_Fork_rank52_1",
    "upgrade_pack_Fork_rank52_3",
    "upgrade_pack_Fork_rank5_1",
    "upgrade_pack_Fork_rank5_3",
    "upgrade_pack_Fork_rank5_5",
];

pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let rows = PACKS
            .iter()
            .flat_map(|pack| {
                let prefix = pack
                    .strip_prefix("upgrade_pack_")
                    .expect("typed upgrade group prefix");
                (1..=80).map(move |level| {
                    (
                        format!("{pack}_{level}"),
                        json!({"ModifyPack":format!("{prefix}_lv{level}")}),
                    )
                })
            })
            .collect::<BTreeMap<_, _>>();
        json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
    })
}

pub(crate) fn table(tables: &BTreeMap<String, Value>) -> Result<Table<'_>, String> {
    match tables.get(OVERLAY_TABLE) {
        Some(value) => Table::new(OVERLAY_TABLE, value),
        None => Table::new(TABLE_NAME, transport()),
    }
}

/// Explicit test/research transport; no missing-row/field merge with defaults.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:DT_ForkUpgradeData");
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn only_referenced_groups_and_supported_level_coordinates_are_retained() {
        let tables = crate::stat_projection::frozen_tables();
        let required = crate::weapon_definitions::rows(tables)
            .unwrap()
            .1
            .values()
            .map(|row| row["ElementData"]["UpgradePackId"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(required, PACKS.iter().copied().collect::<BTreeSet<_>>());
        assert_eq!(PACKS.len(), 10);
        let rows = transport()["rows"].as_object().unwrap();
        assert_eq!(rows.len(), 800);
        for row in rows.values() {
            assert_eq!(row.as_object().unwrap().len(), 1);
            assert!(row["ModifyPack"].is_string());
        }
        for pack in PACKS {
            assert!(rows.get(&format!("{pack}_0")).is_none());
            assert!(rows.get(&format!("{pack}_81")).is_none());
        }
        assert!(rows.get("upgrade_pack_Fork_rank3_2_1").is_none());
    }

    #[test]
    fn explicit_upgrade_overlay_keeps_missing_rows_and_source_validation() {
        let mut value = overlay();
        value["rows"]
            .as_object_mut()
            .unwrap()
            .remove("upgrade_pack_Fork_rank3_1_80");
        let mut tables = BTreeMap::from([("DT_ForkUpgradeData".into(), value)]);
        assert!(
            table(&tables)
                .unwrap()
                .lookup("upgrade_pack_Fork_rank3_1_80")
                .is_err()
        );
        tables.get_mut("DT_ForkUpgradeData").unwrap()["rows"]["upgrade_pack_Fork_rank3_1_1"] =
            json!({});
        let (_, row) = table(&tables)
            .unwrap()
            .lookup("upgrade_pack_Fork_rank3_1_1")
            .unwrap();
        assert!(crate::native::text(row, "ModifyPack").is_err());
        tables.get_mut("DT_ForkUpgradeData").unwrap()["source_id"] = json!(SOURCE_ID);
        assert!(table(&tables).is_err());
        assert!(
            table(&BTreeMap::new())
                .unwrap()
                .lookup("upgrade_pack_Fork_rank3_1_80")
                .is_ok()
        );
    }
}
