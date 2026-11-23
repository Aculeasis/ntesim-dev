//! Consumed single-key parameters for implemented four-piece set effects.
//! Global 2026-10-04: owner projectors interpret ratios, seconds and counts.
//! Source default/infinity metadata and unconsumed rows remain cache-only.
use crate::native::Table;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub const TABLE_NAME: &str = "typed_set_parameters";
pub const SOURCE_ID: &str = "table:typed_set_parameters";
const OVERLAY_TABLE: &str = "CT_Equipmentbuff";

#[derive(Clone, Copy, Debug)]
pub struct ConstantSetParameter {
    pub key: &'static str,
    /// Value = hundredths / 100; units belong to the effect owner.
    pub hundredths: u32,
}

pub const PARAMETERS: &[ConstantSetParameter] = &[
    ConstantSetParameter {
        key: "Attack2_4_AtkUp",
        hundredths: 25,
    },
    ConstantSetParameter {
        key: "Attack2_4_CD",
        hundredths: 2000,
    },
    ConstantSetParameter {
        key: "Chaos2_4_CD",
        hundredths: 2000,
    },
    ConstantSetParameter {
        key: "Chaos2_4_IgnoreChaos",
        hundredths: 12,
    },
    ConstantSetParameter {
        key: "Chaos2_4_IgnoreChaos2",
        hundredths: 24,
    },
    ConstantSetParameter {
        key: "Cosmos2_4_CD",
        hundredths: 2000,
    },
    ConstantSetParameter {
        key: "Cosmos2_4_DefIgnore",
        hundredths: 25,
    },
    ConstantSetParameter {
        key: "GetEfficiency2_4_Atk",
        hundredths: 15,
    },
    ConstantSetParameter {
        key: "GetEfficiency2_4_CD",
        hundredths: 2000,
    },
    ConstantSetParameter {
        key: "Hpsuit_4_CD",
        hundredths: 1000,
    },
    ConstantSetParameter {
        key: "Hpsuit_4_HpMaxUp",
        hundredths: 4,
    },
    ConstantSetParameter {
        key: "Incantation_4_1",
        hundredths: 6,
    },
    ConstantSetParameter {
        key: "Incantation_4_2",
        hundredths: 600,
    },
    ConstantSetParameter {
        key: "Incantation_4_3",
        hundredths: 1000,
    },
    ConstantSetParameter {
        key: "Lakshana_4_1",
        hundredths: 14,
    },
    ConstantSetParameter {
        key: "Lakshana_4_2",
        hundredths: 14,
    },
    ConstantSetParameter {
        key: "Lakshana_4_CD",
        hundredths: 2000,
    },
    ConstantSetParameter {
        key: "Nature2_4_CD",
        hundredths: 1000,
    },
    ConstantSetParameter {
        key: "Nature2_4_CritDamage",
        hundredths: 8,
    },
    ConstantSetParameter {
        key: "Nature2_4_Stack",
        hundredths: 700,
    },
    ConstantSetParameter {
        key: "Psyche_4_1",
        hundredths: 18,
    },
    ConstantSetParameter {
        key: "Psyche_4_2",
        hundredths: 18,
    },
    ConstantSetParameter {
        key: "Psychically_4",
        hundredths: 12,
    },
    ConstantSetParameter {
        key: "Psychically_4_CD",
        hundredths: 600,
    },
    ConstantSetParameter {
        key: "Psychically_4_stack",
        hundredths: 300,
    },
];

/// JSON transport declares the consumed coordinate-one single-key shape.
pub fn transport() -> &'static Value {
    static TABLE: OnceLock<Value> = OnceLock::new();
    TABLE.get_or_init(|| {
        let rows = PARAMETERS.iter().map(|parameter| {
            let decimal = format!("{}.{:02}", parameter.hundredths / 100, parameter.hundredths % 100);
            let value: Value = serde_json::from_str(decimal.trim_end_matches('0').trim_end_matches('.'))
                .expect("typed set parameter decimal");
            (parameter.key, json!({"InterpMode":"ERichCurveInterpMode::RCIM_Linear","Keys":[{"Time":1.0,"Value":value}]}))
        }).collect::<BTreeMap<_,_>>();
        json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
    })
}

/// Explicit transport overlay; missing or changed rows never merge with defaults.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:CT_Equipmentbuff");
    value
}

pub(crate) fn table(tables: &BTreeMap<String, Value>) -> Result<Table<'_>, String> {
    if let Some(value) = tables.get(OVERLAY_TABLE) {
        Table::new(OVERLAY_TABLE, value)
    } else {
        Table::new(TABLE_NAME, transport())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumed_parameters_keep_exact_values_without_unused_source_metadata() {
        let rows = transport()["rows"].as_object().unwrap();
        assert_eq!(rows.len(), 25);
        for row in rows.values() {
            assert!(row.get("DefaultValue").is_none());
            assert!(row.get("PreInfinityExtrap").is_none());
            assert!(row.get("PostInfinityExtrap").is_none());
        }
        assert!(rows.get("Attack_4").is_none());
        let mut tables = BTreeMap::from([(OVERLAY_TABLE.into(), overlay())]);
        tables.get_mut(OVERLAY_TABLE).unwrap()["rows"]
            .as_object_mut()
            .unwrap()
            .remove("Nature2_4_CD");
        assert!(table(&tables).unwrap().lookup("Nature2_4_CD").is_err());
        tables.get_mut(OVERLAY_TABLE).unwrap()["rows"]["Nature2_4_CD"] = json!({"InterpMode":"ERichCurveInterpMode::RCIM_Linear","Keys":[{"Time":1.0,"Value":0}]});
        assert_eq!(
            table(&tables).unwrap().lookup("nature2_4_cd").unwrap().1["Keys"][0]["Value"],
            0
        );
    }
}
