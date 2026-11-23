use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn bonus(
    tables: &BTreeMap<String, Value>,
    ability: &str,
    awakening: u32,
) -> Result<(u32, Value), String> {
    if awakening > 6 {
        return Err("Mint skill-level projection supports awakening 0..=6".into());
    }
    let (table_name, table) = ntesim_data::resonance_levels::table(tables, "1019");
    if table["schema_version"] != 1 || table["source_id"] != format!("table:{table_name}") {
        return Err("Mint resonance table schema/source mismatch".into());
    }
    let effects = table["rows"]["1019"]["AwakenEffectStructList"]
        .as_array()
        .ok_or("Mint awakening effects missing")?;
    let matching = effects
        .iter()
        .enumerate()
        .filter(|(_, effect)| effect["EffectID"] == "resonance_3")
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err("Mint resonance_3 must identify exactly one effect".into());
    }
    let (index, effect) = matching[0];
    if effect["AwakenType"] != "EAwakenEffect::Awaken_Resonance" {
        return Err("unsupported Mint skill-level effect type".into());
    }
    let modifications = effect["ModifyDataList"]
        .as_array()
        .ok_or("Mint resonance modifications missing")?;
    let mut found = BTreeSet::new();
    for modification in modifications {
        let name = modification["SkillName"]
            .as_str()
            .ok_or("Mint resonance skill missing")?;
        if ![
            "GA_Mint019_Melee",
            "GA_Mint019_Skill",
            "GA_Mint019_UltraSkill",
        ]
        .contains(&name)
            || !found.insert(name)
            || modification["SkillLevel"] != 1
        {
            return Err("unsupported Mint resonance skill-level modification".into());
        }
    }
    if found.len() != 3 {
        return Err("Mint resonance skill dependencies missing".into());
    }
    let bonus = u32::from(awakening >= 3 && found.contains(ability));
    Ok((
        bonus,
        json!({"effect_id":"resonance_3", "active":awakening>=3, "applied_bonus":bonus, "modifications":modifications,
        "evidence":{"source_id":format!("table:{table_name}"), "pointer":format!("/rows/1019/AwakenEffectStructList/{index}")},
        "model_assumption":"Mint resonance_3 activates at awakening 3 and remains active at higher awakenings."}),
    ))
}
