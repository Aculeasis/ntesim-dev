//! Exact static values reached by supported weapon level/terminal-stage packs.
//! Only consumed attack and secondary-stat channels are stored. Unreferenced
//! upgrade groups and source SortKey fields remain outside the runtime package.
use crate::native::Table;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

mod breakthrough;
mod levels;

pub const TABLE_NAME: &str = "typed_weapon_modifiers";
pub const SOURCE_ID: &str = "table:typed_weapon_modifiers";
const OVERLAY_TABLE: &str = "DT_ForkModifyData";

struct LevelGroup {
    prefix: &'static str,
    attack: [u16; 80],
}

struct BreakthroughGroup {
    prefix: &'static str,
    attack: [u16; 7],
    secondary: SecondaryStat,
    /// Native property units multiplied by 10,000; BI remains points, while
    /// the ratio channels remain fractions. This is exact decimal storage.
    ten_thousandths: [u32; 7],
}

enum SecondaryStat {
    AttackRatio,
    EnergyGainRatio,
    CritRate,
    CritDamage,
    DefenseRatio,
    HealthRatio,
    BreakIntensity,
}

impl SecondaryStat {
    fn native_key(&self) -> &'static str {
        match self {
            Self::AttackRatio => "AtkUp",
            Self::EnergyGainRatio => "ChargeGetEfficiencyBase",
            Self::CritRate => "CritBase",
            Self::CritDamage => "CritDamageBase",
            Self::DefenseRatio => "DefUp",
            Self::HealthRatio => "HPMaxUp",
            Self::BreakIntensity => "UnbalIntensityBase",
        }
    }
}

fn decimal(amount: u32) -> Value {
    let text = format!("{}.{:04}", amount / 10_000, amount % 10_000);
    serde_json::from_str(text.trim_end_matches('0').trim_end_matches('.'))
        .expect("typed weapon modifier decimal")
}

fn modifier(attribute: &str, value: Value) -> Value {
    json!({"ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE",
        "PropName":attribute,"PropValue":value})
}

fn pack(modifiers: Vec<Value>) -> Value {
    json!({"ConditionArray":[],"ModifyData":modifiers})
}

pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let mut rows = BTreeMap::new();
        for group in levels::GROUPS {
            for (index, attack) in group.attack.iter().enumerate() {
                rows.insert(
                    format!("{}_lv{}", group.prefix, index + 1),
                    pack(vec![modifier("AtkBase", json!(attack))]),
                );
            }
        }
        for group in breakthrough::GROUPS {
            for (stage, attack) in group.attack.iter().enumerate() {
                rows.insert(
                    format!("{}_stage{stage}", group.prefix),
                    pack(vec![
                        modifier("AtkBase", json!(attack)),
                        modifier(
                            group.secondary.native_key(),
                            decimal(group.ten_thousandths[stage]),
                        ),
                    ]),
                );
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

/// Explicit test/research overlay; no absent rows/fields merge with defaults.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:DT_ForkModifyData");
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn retained_values_are_exactly_the_reached_level_and_terminal_packs() {
        let required = [
            crate::weapon_upgrade_levels::transport(),
            crate::weapon_breakthrough::transport(),
        ]
        .into_iter()
        .flat_map(|value| value["rows"].as_object().unwrap().values())
        .map(|row| {
            row.get("ModifyPack")
                .or_else(|| row.get("ModifyPackID"))
                .unwrap()
                .as_str()
                .unwrap()
                .to_ascii_lowercase()
        })
        .collect::<BTreeSet<_>>();
        let rows = transport()["rows"].as_object().unwrap();
        assert_eq!(rows.len(), 1157);
        assert_eq!(
            required,
            rows.keys().map(|key| key.to_ascii_lowercase()).collect()
        );
        for row in rows.values() {
            assert_eq!(row.as_object().unwrap().len(), 2);
            assert_eq!(row["ConditionArray"], json!([]));
            for entry in row["ModifyData"].as_array().unwrap() {
                assert_eq!(entry.as_object().unwrap().len(), 3);
            }
        }
    }

    #[test]
    fn explicit_zero_attack_is_retained_separately_from_fractional_secondary() {
        let tables = BTreeMap::new();
        let bound = table(&tables).unwrap();
        let (_, row) = bound.lookup("fork_Rose_rank5_stage0").unwrap();
        let attributes = crate::native::additive_pack_receipts(row).unwrap();
        assert_eq!(
            attributes["AtkBase"],
            json!({"numerator":"0","denominator":"1"})
        );
        assert_eq!(
            attributes["CritBase"],
            json!({"numerator":"12","denominator":"125"})
        );
        assert_eq!(row["ModifyData"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn explicit_overlay_keeps_absence_zero_and_source_checks() {
        let mut value = overlay();
        value["rows"]
            .as_object_mut()
            .unwrap()
            .remove("Fork_rank5_1_lv80");
        let mut tables = BTreeMap::from([(OVERLAY_TABLE.into(), value)]);
        assert!(table(&tables).unwrap().lookup("Fork_rank5_1_lv80").is_err());
        tables.get_mut(OVERLAY_TABLE).unwrap()["rows"]["Fork_rank5_1_lv1"]["ModifyData"][0]["PropValue"] =
            json!(0);
        let bound = table(&tables).unwrap();
        let (_, row) = bound.lookup("Fork_rank5_1_lv1").unwrap();
        assert_eq!(
            crate::native::additive_pack_receipts(row).unwrap()["AtkBase"],
            json!({"numerator":"0","denominator":"1"})
        );
        tables.get_mut(OVERLAY_TABLE).unwrap()["source_id"] = json!(SOURCE_ID);
        assert!(table(&tables).is_err());
    }
}
