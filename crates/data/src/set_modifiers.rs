//! Static additive bonuses reached by declared 2-/4-piece set effects.
//! Global 2026-10-04: only referenced pack identity, attribute and exact
//! hundredths are retained. No console stats, geometry or specialization.
use crate::native::{Scalar, Table, apply_additive_pack};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub const TABLE_NAME: &str = "typed_set_modifiers";
pub const SOURCE_ID: &str = "table:typed_set_modifiers";
const OVERLAY_TABLE: &str = "DT_EquipmentModifyData";

#[derive(Clone, Copy, Debug)]
pub struct StaticSetModifier {
    pub pack_id: &'static str,
    pub attribute: &'static str,
    /// Fractional bonus in hundredths: 10 means 10%, never a flat stat.
    pub hundredths: u16,
}

pub const MODIFIERS: &[StaticSetModifier] = &[
    StaticSetModifier {
        pack_id: "Attack_2",
        attribute: "AtkUp",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "Chaos_2",
        attribute: "DamageUpChaosBase",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "Cosmos_2",
        attribute: "DamageUpCosmosBase",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "GetEfficiency_2",
        attribute: "ChargeGetEfficiencyBase",
        hundredths: 12,
    },
    StaticSetModifier {
        pack_id: "Heal_2",
        attribute: "HPMaxUp",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "Heal_4",
        attribute: "HealUp",
        hundredths: 20,
    },
    StaticSetModifier {
        pack_id: "Incantation_2",
        attribute: "DamageUpIncantationBase",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "Lakshana_2",
        attribute: "DamageUpLakshanaBase",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "Mag_2",
        attribute: "HPMaxUp",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "Nature_2",
        attribute: "DamageUpNatureBase",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "Psyche_2",
        attribute: "DamageUpPsycheBase",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "Psychically_2",
        attribute: "DamageUpPsychicallyBase",
        hundredths: 10,
    },
    StaticSetModifier {
        pack_id: "Shield_2",
        attribute: "DefUp",
        hundredths: 15,
    },
    StaticSetModifier {
        pack_id: "Shield_4",
        attribute: "ShieldEfficiency",
        hundredths: 20,
    },
];

/// Compact JSON transport for shared validation and explicit overlays.
pub fn transport() -> &'static Value {
    static TABLE: OnceLock<Value> = OnceLock::new();
    TABLE.get_or_init(|| {
        let rows = MODIFIERS
            .iter()
            .map(|modifier| {
                // Decimal spelling preserves exact hundredths without binary floats.
                let decimal = format!(
                    "{}.{:02}",
                    modifier.hundredths / 100,
                    modifier.hundredths % 100
                );
                let amount: Value =
                    serde_json::from_str(decimal.trim_end_matches('0').trim_end_matches('.'))
                        .expect("typed set modifier decimal");
                (
                    modifier.pack_id,
                    json!({"ConditionArray":[],"ModifyData":[{
                        "ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE",
                        "PropName":modifier.attribute,"PropValue":amount
                    }]}),
                )
            })
            .collect::<BTreeMap<_, _>>();
        json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
    })
}

pub(crate) fn compile_scalars(
    tables: &BTreeMap<String, Value>,
    pack_id: &str,
) -> Result<(BTreeMap<String, Scalar>, Value), String> {
    let (name, value) = tables
        .get(OVERLAY_TABLE)
        .map_or((TABLE_NAME, transport()), |value| (OVERLAY_TABLE, value));
    let table = Table::new(name, value)?;
    let (id, row) = table.lookup(pack_id)?;
    let mut attributes = BTreeMap::new();
    apply_additive_pack(&mut attributes, row).map_err(|error| format!("{error}: {id}"))?;
    Ok((attributes, table.evidence(id)))
}

/// Shared exact set-pack projection for equipped builds and Studio presentation.
pub fn compile(tables: &BTreeMap<String, Value>, pack_id: &str) -> Result<Value, String> {
    let (attributes, evidence) = compile_scalars(tables, pack_id)?;
    Ok(
        json!({"attributes":attributes.into_iter().map(|(name,value)|(name,value.receipt())).collect::<BTreeMap<_,_>>(),"evidence":evidence}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_pack_aliases_do_not_fill_explicit_zero_or_missing_overlays() {
        let mut tables = BTreeMap::new();
        let baseline = compile(&tables, "getefficiency_2").unwrap();
        assert_eq!(
            baseline["attributes"]["ChargeGetEfficiencyBase"],
            json!({"numerator":"3","denominator":"25"})
        );
        assert_eq!(baseline["evidence"]["source_id"], SOURCE_ID);
        for unused in [
            "rookie_Atk_2",
            "rookie_Def_2",
            "rookie_Dmg_2",
            "rookie_HP_2",
        ] {
            assert!(compile(&tables, unused).is_err());
        }
        tables.insert(
            OVERLAY_TABLE.into(),
            json!({"schema_version":1,"source_id":"table:DT_EquipmentModifyData","rows":{}}),
        );
        assert!(compile(&tables, "Attack_2").is_err());
        tables.get_mut(OVERLAY_TABLE).unwrap()["rows"]["Attack_2"] = json!({"ConditionArray":[],"ModifyData":[{
            "ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE","PropName":"AtkUp","PropValue":0
        }]});
        assert_eq!(
            compile(&tables, "Attack_2").unwrap()["attributes"]["AtkUp"],
            json!({"numerator":"0","denominator":"1"})
        );
        tables.get_mut(OVERLAY_TABLE).unwrap()["rows"]["Attack_2"]["ConditionArray"] =
            json!(["combat-only"]);
        assert!(compile(&tables, "Attack_2").is_err());
    }
}
