//! Only weapon identity, compatibility and progression fields consumed by Rust.
//! Acquisition, artwork, model, inventory and untranslated source-only fields
//! are not runtime definitions. The original export remains cache-only.
use crate::{character::WeaponGroup, native::Table};
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

mod definitions;

pub const TABLE_NAME: &str = "typed_weapon_definitions";
pub const SOURCE_ID: &str = "table:typed_weapon_definitions";
const OVERLAY_TABLE: &str = "DT_ForkItemData";
pub const MAX_BREAKTHROUGH: u32 = 6;
pub const MAX_REFINEMENT: u32 = 5;

pub struct Definition {
    pub native_id: &'static str,
    pub title: &'static str,
    pub rarity: Rarity,
    pub group: WeaponGroup,
    pub upgrade_pack: &'static str,
    pub breakthrough_pack: &'static str,
    pub refinement_pack: &'static str,
    pub exclusive_character_ids: &'static [&'static str],
    pub exclusive_description: Option<ExclusiveDescription>,
}

/// Owner guards consume twelve explicit empty descriptions and Rose's
/// localized cosmetic-skin description. Other source text is not retained.
pub enum ExclusiveDescription {
    Empty,
    Localized(&'static str),
}

pub enum Rarity {
    Blue,
    Purple,
    Orange,
}

impl Rarity {
    pub fn native_key(&self) -> &'static str {
        match self {
            Self::Blue => "EItemQuality::ITEM_QUALITY_BLUE",
            Self::Purple => "EItemQuality::ITEM_QUALITY_PURPLE",
            Self::Orange => "EItemQuality::ITEM_QUALITY_ORANGE",
        }
    }
}

pub fn definitions() -> &'static [Definition] {
    definitions::DEFINITIONS
}

pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let rows = definitions()
            .iter()
            .map(|definition| {
                let mut row = json!({"ItemName":{"LocalizedString":definition.title},
                "ItemQuality":definition.rarity.native_key(),
                "ElementData":{"ApplyGroupType":definition.group.native_key(),
                    "MaxBreakthrough":MAX_BREAKTHROUGH,"MaxUpgradeStar":MAX_REFINEMENT,
                    "UpgradePackId":definition.upgrade_pack,
                    "BreakthroughPackId":definition.breakthrough_pack,
                    "UpgradeStarPackID":definition.refinement_pack,
                    "ExclusiveCharacterIDArray":definition.exclusive_character_ids}});
                if let Some(description) = &definition.exclusive_description {
                    row["ElementData"]["ExclusiveDesc"] = match description {
                        ExclusiveDescription::Empty => json!({"CultureInvariantString":null}),
                        ExclusiveDescription::Localized(text) => json!({"LocalizedString":text}),
                    };
                }
                (definition.native_id, row)
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

pub fn rows(
    tables: &BTreeMap<String, Value>,
) -> Result<(&'static str, &Map<String, Value>), String> {
    let name = if tables.contains_key(OVERLAY_TABLE) {
        OVERLAY_TABLE
    } else {
        TABLE_NAME
    };
    Ok((name, table(tables)?.rows()))
}

pub fn lookup<'a>(
    tables: &'a BTreeMap<String, Value>,
    id: &str,
) -> Result<(&'a str, &'a Value), String> {
    table(tables)?.lookup(id)
}

/// Explicit test/research overlay; missing rows/fields never use the baseline.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:DT_ForkItemData");
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_consumed_fields_and_explicit_exclusive_owner_lists_remain() {
        assert_eq!(definitions().len(), 51);
        assert_eq!(
            definitions()
                .iter()
                .filter(|value| !value.exclusive_character_ids.is_empty())
                .count(),
            11
        );
        let rows = transport()["rows"].as_object().unwrap();
        assert_eq!(rows.len(), 51);
        for row in rows.values() {
            assert_eq!(row.as_object().unwrap().len(), 3);
            assert_eq!(row["ItemName"].as_object().unwrap().len(), 1);
            let meta = row["ElementData"].as_object().unwrap();
            assert_eq!(
                meta.len(),
                7 + usize::from(meta.contains_key("ExclusiveDesc"))
            );
        }
        assert_eq!(
            rows.values()
                .filter(|row| row["ElementData"].get("ExclusiveDesc").is_some())
                .count(),
            13
        );
        assert_eq!(
            rows["fork_Butterfly"]["ElementData"]["ExclusiveDesc"],
            json!({"CultureInvariantString":null})
        );
        assert_eq!(
            rows["fork_Rose"]["ElementData"]["ExclusiveCharacterIDArray"],
            json!(["1004"])
        );
        assert_eq!(
            rows["fork_Butterfly"]["ElementData"]["ExclusiveCharacterIDArray"],
            json!([])
        );
    }

    #[test]
    fn explicit_overlay_keeps_missing_rows_fields_and_source_validation() {
        let mut value = overlay();
        value["rows"].as_object_mut().unwrap().remove("fork_Rose");
        let mut tables = BTreeMap::from([(OVERLAY_TABLE.into(), value)]);
        assert!(lookup(&tables, "fork_Rose").is_err());
        tables.get_mut(OVERLAY_TABLE).unwrap()["rows"]["fork_Rose"] = json!({});
        assert!(
            crate::native::text(
                &lookup(&tables, "fork_Rose").unwrap().1["ElementData"],
                "ApplyGroupType"
            )
            .is_err()
        );
        tables.get_mut(OVERLAY_TABLE).unwrap()["source_id"] = json!(SOURCE_ID);
        assert!(rows(&tables).is_err());
        assert_eq!(
            lookup(&BTreeMap::new(), "FORK_ROSE").unwrap().0,
            "fork_Rose"
        );
    }
}
