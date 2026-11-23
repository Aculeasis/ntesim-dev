use crate::native::{Scalar, text};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn speedy_hedgehog(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    timed_set(
        tables,
        "Suit11",
        "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_015/Buff_Equipment_GetEfficiency2_4.0",
        ["GetEfficiency2_4_Atk", "GetEfficiency2_4_CD"],
    )
}

pub fn shadow_creed(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    timed_set(
        tables,
        "Suit8",
        "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_023/Buff_Equipment_Atk_4.0",
        ["Attack2_4_AtkUp", "Attack2_4_CD"],
    )
}

pub fn lost_radiance(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    timed_set(
        tables,
        "Suit6",
        "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_021/Buff_Equipment_Cosmos2_4.0",
        ["Cosmos2_4_DefIgnore", "Cosmos2_4_CD"],
    )
}

pub fn quiet_manor(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    timed_set(
        tables,
        "Suit12",
        "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_011/Buff_Equipment_Psychically_4.0",
        ["Psychically_4", "Psychically_4_CD", "Psychically_4_stack"],
    )
}

pub fn hp_suit(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    timed_set(
        tables,
        "Suit10",
        "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_024/Buff_Equipment_HPsuit_4.0",
        ["Hpsuit_4_HpMaxUp", "Hpsuit_4_CD"],
    )
}

pub fn crimson(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    timed_set(
        tables,
        "Suit4",
        "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_006/Buff_Equipment_Incantation_4.0",
        ["Incantation_4_1", "Incantation_4_2", "Incantation_4_3"],
    )
}

pub fn devils_blood(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    timed_set(
        tables,
        "Suit3",
        "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_005/Buff_Equipment_Psyche_4.0",
        ["Psyche_4_1", "Psyche_4_2"],
    )
}

pub fn diabolos(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    unbound_set(
        tables,
        "Suit1",
        [
            "Chaos2_4_IgnoreChaos",
            "Chaos2_4_IgnoreChaos2",
            "Chaos2_4_CD",
        ],
    )
}

pub fn street_boxer(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    unbound_set(
        tables,
        "Suit5",
        ["Lakshana_4_1", "Lakshana_4_2", "Lakshana_4_CD"],
    )
}

fn unbound_set<const N: usize>(
    tables: &BTreeMap<String, Value>,
    set: &str,
    names: [&str; N],
) -> Result<Value, String> {
    let (set_table, _) = crate::set_definitions::rows(tables)?;
    let suits = crate::set_definitions::table(tables)?;
    let (id, suit) = suits.lookup(set)?;
    let effects = suit["SuitStructList"]
        .as_array()
        .ok_or("unbound set effects missing")?;
    let matches: Vec<_> = effects
        .iter()
        .enumerate()
        .filter(|(_, e)| e["SuitCondition"] == 4)
        .collect();
    if matches.len() != 1 {
        return Err("unbound set requires one four-condition effect".into());
    }
    let (index, effect) = matches[0];
    if effect["SuitModifyPackID"] != "None" || effect["SuitBuff"]["binding"] != "unbound" {
        return Err("unsupported unbound set dependencies".into());
    }
    let parameters = read_constant_parameters(tables, names)?;
    Ok(
        json!({"set_id":format!("set:{set}"),"parameters":parameters,"effect":effect,"evidence":{"source_id":format!("table:{set_table}"),"pointer":format!("/rows/{id}/SuitStructList/{index}")}}),
    )
}

fn read_constant_parameters<const N: usize>(
    tables: &BTreeMap<String, Value>,
    names: [&str; N],
) -> Result<BTreeMap<String, Value>, String> {
    let curves = crate::set_parameters::table(tables)?;
    let mut parameters = BTreeMap::new();
    for name in names {
        let (row, curve) = curves.lookup(name)?;
        let keys = curve["Keys"].as_array().ok_or("set keys missing")?;
        if keys.len() != 1
            || Scalar::parse(&keys[0]["Time"])? != Scalar::one()
            || ![
                "ERichCurveInterpMode::RCIM_Linear",
                "ERichCurveInterpMode::RCIM_Constant",
            ]
            .contains(&text(curve, "InterpMode")?)
        {
            return Err("unsupported set parameter curve".into());
        }
        parameters.insert(name.to_string(), json!({"selected":Scalar::parse(&keys[0]["Value"])?.receipt(),"evidence":curves.evidence(row)}));
    }
    Ok(parameters)
}

fn timed_set<const N: usize>(
    tables: &BTreeMap<String, Value>,
    set: &str,
    path: &str,
    names: [&str; N],
) -> Result<Value, String> {
    let (set_table, _) = crate::set_definitions::rows(tables)?;
    let suits = crate::set_definitions::table(tables)?;
    let curves = crate::set_parameters::table(tables)?;
    let (id, suit) = suits.lookup(set)?;
    let effects = suit["SuitStructList"]
        .as_array()
        .ok_or("timed set effects missing")?;
    let matches: Vec<_> = effects
        .iter()
        .enumerate()
        .filter(|(_, e)| e["SuitCondition"] == 4)
        .collect();
    if matches.len() != 1 {
        return Err("timed set requires one four-condition effect".into());
    }
    let (index, effect) = matches[0];
    if effect["SuitModifyPackID"] != "None" || effect["SuitBuff"]["ObjectPath"] != path {
        return Err("unsupported timed set dependencies".into());
    }
    let mut parameters = BTreeMap::new();
    for name in names {
        let (row, curve) = curves.lookup(name)?;
        let keys = curve["Keys"].as_array().ok_or("timed set keys missing")?;
        if keys.len() != 1
            || Scalar::parse(&keys[0]["Time"])? != Scalar::one()
            || ![
                "ERichCurveInterpMode::RCIM_Linear",
                "ERichCurveInterpMode::RCIM_Constant",
            ]
            .contains(&text(curve, "InterpMode")?)
        {
            return Err("unsupported timed set curve".into());
        }
        parameters.insert(name, json!({"selected":Scalar::parse(&keys[0]["Value"])?.receipt(),"evidence":curves.evidence(row)}));
    }
    Ok(
        json!({"set_id":format!("set:{set}"),"parameters":parameters,"effect":effect,"evidence":{"source_id":format!("table:{set_table}"),"pointer":format!("/rows/{id}/SuitStructList/{index}")}}),
    )
}

pub fn fireflies(tables: &BTreeMap<String, Value>) -> Result<Value, String> {
    let (set_table, _) = crate::set_definitions::rows(tables)?;
    let suits = crate::set_definitions::table(tables)?;
    let curves = crate::set_parameters::table(tables)?;
    let (id, suit) = suits.lookup("Suit2")?;
    let effects = suit["SuitStructList"]
        .as_array()
        .ok_or("Fireflies set effects missing")?;
    let matches = effects
        .iter()
        .enumerate()
        .filter(|(_, e)| e["SuitCondition"] == 4)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err("Fireflies requires exactly one four-condition effect".into());
    }
    let (effect_index, effect) = matches[0];
    if effect["SuitModifyPackID"] != "None"
        || effect["SuitBuff"]["ObjectPath"]
            != "/Game/Blueprints/Abilities/Buff/Equipment/Equipment_013/Buff_Equipment_Nature2_4.0"
    {
        return Err("unsupported Fireflies four-condition dependencies".into());
    }
    let mut parameters = BTreeMap::new();
    for name in ["Nature2_4_CritDamage", "Nature2_4_Stack", "Nature2_4_CD"] {
        let (row, curve) = curves.lookup(name)?;
        let keys = curve["Keys"]
            .as_array()
            .ok_or("Fireflies curve keys missing")?;
        if keys.len() != 1 || Scalar::parse(&keys[0]["Time"])? != Scalar::one() {
            return Err("Fireflies requires a single exact coordinate-one key".into());
        }
        if ![
            "ERichCurveInterpMode::RCIM_Linear",
            "ERichCurveInterpMode::RCIM_Constant",
        ]
        .contains(&text(curve, "InterpMode")?)
        {
            return Err("unsupported Fireflies curve interpolation".into());
        }
        parameters.insert(name,json!({"selected":Scalar::parse(&keys[0]["Value"])?.receipt(),"curve":curve,"evidence":curves.evidence(row)}));
    }
    Ok(json!({"set_id":"set:Suit2", "activation_threshold":4,
        "effect":effect, "parameters":parameters, "evidence":{"source_id":format!("table:{set_table}"),"pointer":format!("/rows/{id}/SuitStructList/{effect_index}")},
        "units_and_trigger_binding":"native scalars; content supplies explicit unit, declared set count and trigger hypotheses"}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hp_suit_binds_exact_curves_and_rejects_dependency_drift() {
        let mut tables = BTreeMap::from([
            (
                "DT_EquipmentSuitData".into(),
                crate::set_definitions::overlay(),
            ),
            ("CT_Equipmentbuff".into(), crate::set_parameters::overlay()),
        ]);
        let report = hp_suit(&tables).unwrap();
        // Independent oracle: Hpsuit_4_HpMaxUp = 0.04, Hpsuit_4_CD = 10.
        for (key, n, d) in [("Hpsuit_4_HpMaxUp", "1", "25"), ("Hpsuit_4_CD", "10", "1")] {
            assert_eq!(
                report["parameters"][key]["selected"],
                json!({"numerator":n,"denominator":d})
            );
            let mut drifted = tables.clone();
            drifted.get_mut("CT_Equipmentbuff").unwrap()["rows"][key]["Keys"][0]["Time"] = json!(2);
            assert!(hp_suit(&drifted).unwrap_err().contains("curve"));
        }
        tables.get_mut("DT_EquipmentSuitData").unwrap()["rows"]["Suit10"]["SuitStructList"][1]["SuitBuff"]
            ["ObjectPath"] = json!("unsupported");
        assert!(hp_suit(&tables).unwrap_err().contains("dependencies"));
    }

    #[test]
    fn quiet_manor_binds_exact_curves_and_rejects_dependency_drift() {
        let mut tables = BTreeMap::from([
            (
                "DT_EquipmentSuitData".into(),
                crate::set_definitions::overlay(),
            ),
            ("CT_Equipmentbuff".into(), crate::set_parameters::overlay()),
        ]);
        let report = quiet_manor(&tables).unwrap();
        for (key, n, d) in [
            ("Psychically_4", "3", "25"),
            ("Psychically_4_CD", "6", "1"),
            ("Psychically_4_stack", "3", "1"),
        ] {
            assert_eq!(
                report["parameters"][key]["selected"],
                json!({"numerator":n,"denominator":d})
            );
            let mut drifted = tables.clone();
            drifted.get_mut("CT_Equipmentbuff").unwrap()["rows"][key]["Keys"][0]["Time"] = json!(2);
            assert!(quiet_manor(&drifted).unwrap_err().contains("curve"));
        }
        tables.get_mut("DT_EquipmentSuitData").unwrap()["rows"]["Suit12"]["SuitStructList"][1]["SuitBuff"]
            ["ObjectPath"] = json!("unsupported");
        assert!(quiet_manor(&tables).unwrap_err().contains("dependencies"));
    }

    #[test]
    fn shadow_creed_matches_current_description_and_rejects_dependency_drift() {
        let mut tables: BTreeMap<String, Value> = ["DT_EquipmentSuitData", "CT_Equipmentbuff"]
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    if name == "CT_Equipmentbuff" {
                        crate::set_parameters::overlay()
                    } else {
                        crate::set_definitions::overlay()
                    },
                )
            })
            .collect();
        let report = shadow_creed(&tables).unwrap();
        assert_eq!(
            report["parameters"]["Attack2_4_AtkUp"]["selected"],
            json!({"numerator":"1","denominator":"4"})
        );
        assert_eq!(
            report["parameters"]["Attack2_4_CD"]["selected"],
            json!({"numerator":"20","denominator":"1"})
        );
        tables.get_mut("CT_Equipmentbuff").unwrap()["rows"]["Attack_4"] = json!({"InterpMode":"ERichCurveInterpMode::RCIM_Linear","Keys":[{"Time":1.0,"Value":9}]});
        assert_eq!(report, shadow_creed(&tables).unwrap());
        tables.get_mut("CT_Equipmentbuff").unwrap()["rows"]["Attack2_4_AtkUp"]["Keys"][0]["Time"] =
            json!(2);
        assert!(shadow_creed(&tables).unwrap_err().contains("curve"));
        tables.get_mut("DT_EquipmentSuitData").unwrap()["rows"]["Suit8"]["SuitStructList"][1]["SuitBuff"]
            ["ObjectPath"] = json!("unsupported");
        assert!(shadow_creed(&tables).unwrap_err().contains("dependencies"));
    }

    #[test]
    fn speedy_hedgehog_binds_current_curves_and_rejects_dependency_drift() {
        let mut tables: BTreeMap<String, Value> = ["DT_EquipmentSuitData", "CT_Equipmentbuff"]
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    if name == "CT_Equipmentbuff" {
                        crate::set_parameters::overlay()
                    } else {
                        crate::set_definitions::overlay()
                    },
                )
            })
            .collect();
        let report = speedy_hedgehog(&tables).unwrap();
        assert_eq!(
            report["parameters"]["GetEfficiency2_4_Atk"]["selected"],
            json!({"numerator":"3","denominator":"20"})
        );
        assert_eq!(
            report["parameters"]["GetEfficiency2_4_CD"]["selected"],
            json!({"numerator":"20","denominator":"1"})
        );
        tables.get_mut("CT_Equipmentbuff").unwrap()["rows"]["GetEfficiency2_4_Atk"]["Keys"][0]["Time"] =
            json!(2);
        assert!(speedy_hedgehog(&tables).unwrap_err().contains("curve"));
        tables.get_mut("DT_EquipmentSuitData").unwrap()["rows"]["Suit11"]["SuitStructList"][1]["SuitBuff"]
            ["ObjectPath"] = json!("different");
        assert!(
            speedy_hedgehog(&tables)
                .unwrap_err()
                .contains("dependencies")
        );
    }

    #[test]
    fn lost_radiance_binds_current_curves_and_rejects_dependency_drift() {
        let mut tables: BTreeMap<String, Value> = ["DT_EquipmentSuitData", "CT_Equipmentbuff"]
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    if name == "CT_Equipmentbuff" {
                        crate::set_parameters::overlay()
                    } else {
                        crate::set_definitions::overlay()
                    },
                )
            })
            .collect();
        let report = lost_radiance(&tables).unwrap();
        assert_eq!(
            report["parameters"]["Cosmos2_4_DefIgnore"]["selected"],
            json!({"numerator":"1","denominator":"4"})
        );
        assert_eq!(
            report["parameters"]["Cosmos2_4_CD"]["selected"],
            json!({"numerator":"20","denominator":"1"})
        );
        tables.get_mut("CT_Equipmentbuff").unwrap()["rows"]["Cosmos2_4_DefIgnore"]["Keys"][0]["Time"] =
            json!(2);
        assert!(lost_radiance(&tables).unwrap_err().contains("curve"));
        tables.get_mut("DT_EquipmentSuitData").unwrap()["rows"]["Suit6"]["SuitStructList"][1]["SuitBuff"]
            ["ObjectPath"] = json!("different");
        assert!(lost_radiance(&tables).unwrap_err().contains("dependencies"));
    }
}
