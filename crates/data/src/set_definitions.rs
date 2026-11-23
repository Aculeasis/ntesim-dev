//! Equipped set thresholds, static packs and owner-checked buff dependencies.
//! English titles/descriptions supply Studio labels and tooltips. Original
//! localization metadata, icons, geometry and relive flags are cache-only.
use crate::native::Table;
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub const TABLE_NAME: &str = "typed_set_definitions";
pub const SOURCE_ID: &str = "table:typed_set_definitions";
const OVERLAY_TABLE: &str = "DT_EquipmentSuitData";

#[derive(Clone, Copy, Debug)]
pub enum SetBuff {
    CheckedPath(&'static str),
    Unbound,
}

#[derive(Clone, Copy, Debug)]
pub struct SetEffect {
    pub threshold: u8,
    pub modifier: Option<&'static str>,
    pub buff: Option<SetBuff>,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct SetDefinition {
    pub native_id: &'static str,
    pub title: &'static str,
    pub effects: [SetEffect; 2],
}

pub const SETS: &[SetDefinition] = &[
    SetDefinition {
        native_id: "Suit1",
        title: "Diabolos",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Chaos_2"),
                buff: None,
                description: "Epic (2): Chaos DMG +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::Unbound),
                description: "Legendary (4): Ignores 12% of enemies' Chaos RES. Ignores 24% of enemies' Chaos RES for 20s after the wearer participates in Nova or Scorch reactions.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit10",
        title: "Tiny Big Adventure",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Mag_2"),
                buff: None,
                description: "Epic (2): HP +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::CheckedPath(
                    "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_024/Buff_Equipment_HPsuit_4.0",
                )),
                description: "Legendary (4): Increases wearer's Max HP by 4% when taking damage, up to 10 stacks. Each stack lasts 10s. Casting Ultimate instantly grants 10 stacks.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit11",
        title: "Speedy Hedgehog",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("GetEfficiency_2"),
                buff: None,
                description: "Epic (2): Charge Efficiency +12%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::CheckedPath(
                    "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_015/Buff_Equipment_GetEfficiency2_4.0",
                )),
                description: "Legendary (4): Increases all allies' ATK by 15% for 20s after the wearer casts Ultimate. Effect does not stack.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit12",
        title: "Quiet Manor",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Psychically_2"),
                buff: None,
                description: "Epic (2): Mental DMG +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::CheckedPath(
                    "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_011/Buff_Equipment_Psychically_4.0",
                )),
                description: "Legendary (4): Grants wearer a 12% Mental DMG Bonus per Basic Attack, up to 3 stacks. Each stack lasts 6s.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit2",
        title: "Fireflies and the Forest",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Nature_2"),
                buff: None,
                description: "Epic (2): Anima DMG +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::CheckedPath(
                    "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_013/Buff_Equipment_Nature2_4.0",
                )),
                description: "Legendary (4): Increases wearer's CRIT DMG by 8% when a nearby enemy takes Anima DMG from the Team, up to 7 stacks. Each stack lasts 10s. Effect remains active when the character is off-field.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit3",
        title: "Devil's Blood: Curse",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Psyche_2"),
                buff: None,
                description: "Epic (2): Psyche DMG +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::CheckedPath(
                    "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_005/Buff_Equipment_Psyche_4.0",
                )),
                description: "Legendary (4): Increases wearer's DMG by 18%. The bonus becomes 36% against units affected by Nova or Stain.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit4",
        title: "Crimson: Twin Butterflies",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Incantation_2"),
                buff: None,
                description: "Epic (2): Incantation DMG +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::CheckedPath(
                    "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_006/Buff_Equipment_Incantation_4.0",
                )),
                description: "Legendary (4): Increases wearer's ATK by 6% when a nearby enemy takes Incantation DMG from the Team, up to 6 stacks. Each stack lasts 10s. Effect remains active when the character is off-field.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit5",
        title: "Street Boxer",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Lakshana_2"),
                buff: None,
                description: "Epic (2): Lakshana DMG +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::Unbound),
                description: "Legendary (4): Increases Crit Chance by 14%. Team triggering Remora or Stain increases the bonus to 14% for 20s.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit6",
        title: "Lost Radiance",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Cosmos_2"),
                buff: None,
                description: "Epic (2): Cosmos DMG +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::CheckedPath(
                    "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_021/Buff_Equipment_Cosmos2_4.0",
                )),
                description: "Legendary (4): Ignores 25% of enemies' DEF for 20s after the wearer casts Ultimate. Effect does not stack.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit7",
        title: "Kingdom's Guard",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Shield_2"),
                buff: None,
                description: "Epic (2): DEF +15%.",
            },
            SetEffect {
                threshold: 4,
                modifier: Some("Shield_4"),
                buff: None,
                description: "Legendary (4): Increases wearer's shields by 20%.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit8",
        title: "Shadow Creed",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Attack_2"),
                buff: None,
                description: "Epic (2): ATK +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: None,
                buff: Some(SetBuff::CheckedPath(
                    "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_023/Buff_Equipment_Atk_4.0",
                )),
                description: "Legendary (4): Increases wearer's ATK by 25% for 20s after casting a Skill.",
            },
        ],
    },
    SetDefinition {
        native_id: "Suit9",
        title: "Thea's Night Tavern",
        effects: [
            SetEffect {
                threshold: 2,
                modifier: Some("Heal_2"),
                buff: None,
                description: "Epic (2): HP +10%.",
            },
            SetEffect {
                threshold: 4,
                modifier: Some("Heal_4"),
                buff: None,
                description: "Legendary (4): Increases wearer's Healing Bonus by 20%.",
            },
        ],
    },
];

/// JSON remains the existing compiler/catalog transport, built from typed data.
pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let rows = SETS
            .iter()
            .map(|set| {
                let effects = set
                    .effects
                    .iter()
                    .map(|effect| {
                        json!({
                            "SuitCondition":effect.threshold,
                            "SuitModifyPackID":effect.modifier.unwrap_or("None"),
                            "SuitBuff":effect.buff.map(|buff| match buff {
                                SetBuff::CheckedPath(path) => json!({"ObjectPath":path}),
                                SetBuff::Unbound => json!({"binding":"unbound"}),
                            }),
                            "SuitBuffDescription":{"LocalizedString":effect.description}
                        })
                    })
                    .collect::<Vec<_>>();
                (
                    set.native_id,
                    json!({"SuitTitle":{"LocalizedString":set.title},"SuitStructList":effects}),
                )
            })
            .collect::<BTreeMap<_, _>>();
        json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
    })
}

/// Explicit research/test overlay; no missing fields merge with the baseline.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:DT_EquipmentSuitData");
    value
}

pub(crate) fn table(tables: &BTreeMap<String, Value>) -> Result<Table<'_>, String> {
    if let Some(value) = tables.get(OVERLAY_TABLE) {
        Table::new(OVERLAY_TABLE, value)
    } else {
        Table::new(TABLE_NAME, transport())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definitions_contain_only_consumed_fields_and_preserve_null_buffs() {
        let rows = transport()["rows"].as_object().unwrap();
        assert_eq!(rows.len(), 12);
        for row in rows.values() {
            assert_eq!(row.as_object().unwrap().len(), 2);
            let effects = row["SuitStructList"].as_array().unwrap();
            for (index, effect) in effects.iter().enumerate() {
                assert_eq!(effect["SuitCondition"], (index + 1) * 2);
                assert_eq!(effect.as_object().unwrap().len(), 4);
                if let Some(buff) = effect["SuitBuff"].as_object() {
                    assert_eq!(buff.len(), 1);
                    assert!(
                        buff.get("ObjectPath").is_some_and(Value::is_string)
                            || buff.get("binding").and_then(Value::as_str) == Some("unbound")
                    );
                }
            }
        }
        assert!(rows["Suit9"]["SuitStructList"][1]["SuitBuff"].is_null());
        assert_eq!(
            rows["Suit9"]["SuitStructList"][1]["SuitModifyPackID"],
            "Heal_4"
        );
    }

    #[test]
    fn explicit_overlay_cannot_fill_missing_rows_or_invalid_thresholds() {
        let mut value = overlay();
        value["rows"].as_object_mut().unwrap().remove("Suit9");
        let mut tables = BTreeMap::from([("DT_EquipmentSuitData".into(), value)]);
        assert!(crate::equipment::set_stats(&tables, "set:Suit9", 4).is_err());
        tables.get_mut("DT_EquipmentSuitData").unwrap()["rows"]["Suit2"]["SuitStructList"][0]["SuitCondition"] =
            json!(3);
        assert!(crate::equipment::set_stats(&tables, "set:Suit2", 4).is_err());
        tables.get_mut("DT_EquipmentSuitData").unwrap()["source_id"] = json!(SOURCE_ID);
        assert!(table(&tables).is_err());
        assert!(crate::equipment::set_stats(&BTreeMap::new(), "set:Suit9", 4).is_ok());
    }
}
