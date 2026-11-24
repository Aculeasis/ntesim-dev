//! Consumed refinement rows: ordered parameter names/unit hints and English
//! descriptions. Buff-class paths, costs, unused titles and localization
//! metadata are source-only observations, never weapon execution inputs.
use crate::native::Table;
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

mod definitions;

pub const ECHOES_ON_HOLD_PARAMETERS: [(&str, bool); 5] = [
    ("ntesim.echoes_on_hold.atk", true),
    ("ntesim.echoes_on_hold.team_def_ignore", true),
    ("ntesim.echoes_on_hold.wearer_def_ignore", true),
    ("ntesim.echoes_on_hold.cosmos_penetration", true),
    ("ntesim.echoes_on_hold.duration", false),
];

pub const TABLE_NAME: &str = "typed_weapon_refinements";
pub const SOURCE_ID: &str = "table:typed_weapon_refinements";
const OVERLAY_TABLE: &str = "DT_ForkUpgradeStarDataTable";

struct Parameter {
    native_id: &'static str,
    percent_hint: bool,
}

enum Description {
    Shared {
        key: &'static str,
        text: &'static str,
    },
    Ranked {
        key_prefix: &'static str,
        texts: [&'static str; 5],
    },
}

struct Definition {
    pack: &'static str,
    parameters: &'static [Parameter],
    description: Description,
}

pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let mut rows = BTreeMap::new();
        for definition in definitions::DEFINITIONS {
            let parameters = definition.parameters.iter().map(|parameter| {
                json!({"NameID":parameter.native_id,"bIsPercent":parameter.percent_hint})
            }).collect::<Vec<_>>();
            for tier in 1..=5 {
                let description = match &definition.description {
                    Description::Shared { key, text } => json!({"Key":key,"LocalizedString":text}),
                    Description::Ranked { key_prefix, texts } => json!({"Key":format!("{key_prefix}{tier}"),"LocalizedString":texts[tier - 1]}),
                };
                rows.insert(format!("{}_{tier}", definition.pack), json!({"DataList":parameters,"Description":description}));
            }
        }
        json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
    })
}

pub(crate) fn table(tables: &BTreeMap<String, Value>) -> Result<Table<'_>, String> {
    match tables.get(OVERLAY_TABLE) {
        Some(value) => Table::new(OVERLAY_TABLE, value),
        None => Table::new(TABLE_NAME, transport()),
    }
}

pub fn rows(tables: &BTreeMap<String, Value>) -> Result<&Map<String, Value>, String> {
    Ok(table(tables)?.rows())
}

pub fn lookup<'a>(
    tables: &'a BTreeMap<String, Value>,
    id: &str,
) -> Result<(&'a str, &'a Value), String> {
    table(tables)?.lookup(id)
}

/// Explicit test/research transport; no absent row/field merges with defaults.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:DT_ForkUpgradeStarDataTable");
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn only_consumed_fields_and_supported_refinement_coordinates_remain() {
        let required = crate::weapon_definitions::definitions()
            .iter()
            .flat_map(|definition| {
                (1..=5).map(move |tier| {
                    format!("{}_{tier}", definition.refinement_pack).to_ascii_lowercase()
                })
            })
            .collect::<BTreeSet<_>>();
        let rows = transport()["rows"].as_object().unwrap();
        assert_eq!(rows.len(), required.len());
        assert_eq!(
            required,
            rows.keys().map(|name| name.to_ascii_lowercase()).collect()
        );
        assert_eq!(
            rows.values()
                .map(|row| row["DataList"].as_array().unwrap().len())
                .sum::<usize>(),
            795
        );
        for row in rows.values() {
            assert_eq!(row.as_object().unwrap().len(), 2);
            assert_eq!(row["Description"].as_object().unwrap().len(), 2);
            for parameter in row["DataList"].as_array().unwrap() {
                assert_eq!(parameter.as_object().unwrap().len(), 2);
            }
        }
    }

    #[test]
    fn explicit_overlay_keeps_missing_rows_fields_and_source_validation() {
        let key = "upgradestar_pack_fork_lingganzhongjiezhe_5";
        let mut value = overlay();
        value["rows"].as_object_mut().unwrap().remove(key);
        let mut tables = BTreeMap::from([(OVERLAY_TABLE.into(), value)]);
        assert!(lookup(&tables, key).is_err());
        tables.get_mut(OVERLAY_TABLE).unwrap()["rows"][key] = json!({"Description":{}});
        assert!(
            lookup(&tables, key).unwrap().1["DataList"]
                .as_array()
                .is_none()
        );
        tables.get_mut(OVERLAY_TABLE).unwrap()["source_id"] = json!(SOURCE_ID);
        assert!(lookup(&tables, key).is_err());
    }
}
