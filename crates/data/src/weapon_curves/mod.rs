//! Exact sparse Constant curves reached by supported weapon refinements.
//! 154 listed parameter curves plus two Nest Bird mechanic curves are retained.
//! Default/extrapolation sentinels and 51 unconsumed curves stay cache-only.
use crate::native::Table;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

mod curves;

pub const TABLE_NAME: &str = "typed_weapon_curves";
pub const SOURCE_ID: &str = "table:typed_weapon_curves";
const OVERLAY_TABLE: &str = "CT_ForkBuff";

struct Key {
    coordinate: u8,
    /// Exact native property units, multiplied by 10,000. No percent conversion.
    ten_thousandths: i64,
}

const fn key(coordinate: u8, ten_thousandths: i64) -> Key {
    Key {
        coordinate,
        ten_thousandths,
    }
}

struct Curve {
    native_id: &'static str,
    keys: &'static [Key],
}

fn decimal(amount: i64) -> Value {
    let magnitude = amount.unsigned_abs();
    let sign = if amount < 0 { "-" } else { "" };
    let text = format!("{sign}{}.{:04}", magnitude / 10_000, magnitude % 10_000);
    serde_json::from_str(text.trim_end_matches('0').trim_end_matches('.'))
        .expect("typed weapon curve decimal")
}

pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let rows = curves::CURVES
            .iter()
            .map(|curve| {
                let keys = curve
                    .keys
                    .iter()
                    .map(|key| json!({"Time":key.coordinate,"Value":decimal(key.ten_thousandths)}))
                    .collect::<Vec<_>>();
                (
                    curve.native_id,
                    json!({"InterpMode":"ERichCurveInterpMode::RCIM_Constant","Keys":keys}),
                )
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

pub fn lookup<'a>(
    tables: &'a BTreeMap<String, Value>,
    id: &str,
) -> Result<(&'a str, &'a Value), String> {
    table(tables)?.lookup(id)
}

/// Explicit test/research overlay; missing entries never use the baseline.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:CT_ForkBuff");
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn retained_sparse_keys_are_exactly_the_listed_and_owner_consumed_curves() {
        let tables = crate::stat_projection::frozen_tables();
        let mut required = crate::weapon_refinements::rows(tables)
            .unwrap()
            .values()
            .flat_map(|row| row["DataList"].as_array().unwrap())
            .map(|entry| entry["NameID"].as_str().unwrap().to_ascii_lowercase())
            .collect::<BTreeSet<_>>();
        assert_eq!(required.len(), 154);
        required.extend(["buff_nestbird_down".into(), "buff_nestbird_cd1".into()]);
        let rows = transport()["rows"].as_object().unwrap();
        assert_eq!(rows.len(), 156);
        assert_eq!(
            required,
            rows.keys().map(|name| name.to_ascii_lowercase()).collect()
        );
        assert_eq!(
            rows.values()
                .map(|row| row["Keys"].as_array().unwrap().len())
                .sum::<usize>(),
            618
        );
        for row in rows.values() {
            assert_eq!(row.as_object().unwrap().len(), 2);
            for key in row["Keys"].as_array().unwrap() {
                assert_eq!(key.as_object().unwrap().len(), 2);
            }
        }
        assert!(!rows.contains_key("buff_NestBird_CD2"));
    }

    #[test]
    fn sparse_selection_preserves_left_key_method_index_and_no_extrapolation() {
        let tables = BTreeMap::new();
        let (_, curve) = lookup(&tables, "buff_xiangdao_CD").unwrap();
        assert_eq!(curve["Keys"].as_array().unwrap().len(), 2);
        let selected = crate::native::curve_selection_receipt(curve, 2).unwrap();
        assert_eq!(selected["selection"], "serialized_constant_left_key");
        assert_eq!(selected["selected_key_index"], 0);
        assert_eq!(
            selected["value"],
            json!({"numerator":"20","denominator":"1"})
        );
        assert_eq!(
            crate::native::curve_selection_receipt(curve, 5).unwrap()["selected_key_index"],
            1
        );
        assert!(crate::native::curve_selection_receipt(curve, 0).is_err());
        assert!(crate::native::curve_selection_receipt(curve, 6).is_err());
        let (_, reduction) = lookup(&tables, "buff_NestBird_down").unwrap();
        assert!(
            reduction["Keys"][0]["Value"]
                .as_number()
                .unwrap()
                .to_string()
                .starts_with('-')
        );
        assert!(crate::native::curve_selection_receipt(reduction, 1).is_err());
    }

    #[test]
    fn explicit_overlay_keeps_missing_entries_and_source_checks() {
        let mut value = overlay();
        value["rows"]
            .as_object_mut()
            .unwrap()
            .remove("buff_xiangdao_CD");
        let mut tables = BTreeMap::from([(OVERLAY_TABLE.into(), value)]);
        assert!(lookup(&tables, "buff_xiangdao_CD").is_err());
        tables.get_mut(OVERLAY_TABLE).unwrap()["source_id"] = json!(SOURCE_ID);
        assert!(lookup(&tables, "buff_xiangdao_energy").is_err());
    }
}
