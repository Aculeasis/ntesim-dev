//! Skill-level addition from the `resonance_3` milestone bonus.
//!
//! In NTE mechanics, `resonance_3` is a count-gated milestone unlocked when at least 3
//! awakening effects are selected in total (`awakening.has_resonance_3()` / `count >= 3`),
//! NOT by selecting individual node 3 (`has_effect(3)`).
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn bonus(
    tables: &BTreeMap<String, Value>,
    character_id: &str,
    ability: &str,
    awakening: &ntesim_data::AwakeningSelection,
) -> Result<(u32, Value), String> {
    let awakening_count = awakening.count();
    if character_id == "character:1019" {
        return crate::characters::mint::levels::bonus(tables, ability, awakening_count);
    }
    if awakening_count > 6 {
        return Err("skill-level projection supports awakening 0..=6".into());
    }
    let native_id = character_id
        .strip_prefix("character:")
        .ok_or("canonical character ID required")?;
    let (table_name, table) = ntesim_data::resonance_levels::table(tables, native_id);
    if table["schema_version"] != 1 || table["source_id"] != format!("table:{table_name}") {
        return Err(format!(
            "resonance table {table_name} schema/source mismatch"
        ));
    }
    let effects = table["rows"][native_id]["AwakenEffectStructList"]
        .as_array()
        .ok_or_else(|| format!("character {native_id} awakening effects missing"))?;
    let matching = effects
        .iter()
        .enumerate()
        .filter(|(_, effect)| effect["EffectID"] == "resonance_3")
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err(format!(
            "character {native_id} resonance_3 must identify exactly one effect"
        ));
    }
    let (index, effect) = matching[0];
    if effect["AwakenType"] != "EAwakenEffect::Awaken_Resonance" {
        return Err(format!(
            "unsupported character {native_id} skill-level effect type"
        ));
    }
    let modifications = effect["ModifyDataList"]
        .as_array()
        .ok_or_else(|| format!("character {native_id} resonance modifications missing"))?;
    let mut found = BTreeSet::new();
    for modification in modifications {
        if let Some(level) = modification["SkillLevel"].as_u64() {
            if level != 1 {
                return Err(format!(
                    "unsupported character {native_id} resonance skill-level modification"
                ));
            }
            let name = modification["SkillName"]
                .as_str()
                .ok_or_else(|| format!("character {native_id} resonance skill missing"))?;
            if !found.insert(name) {
                return Err(format!(
                    "duplicate character {native_id} resonance skill modification"
                ));
            }
        }
    }
    if found.len() != 3 && !found.is_empty() {
        return Err(format!(
            "character {native_id} resonance skill dependencies missing"
        ));
    }
    let bonus = u32::from(awakening.has_resonance_3() && found.contains(ability));
    Ok((
        bonus,
        json!({
            "effect_id": "resonance_3",
            "active": awakening.has_resonance_3(),
            "applied_bonus": bonus,
            "modifications": modifications,
            "evidence": {
                "source_id": format!("table:{table_name}"),
                "pointer": format!("/rows/{native_id}/AwakenEffectStructList/{index}")
            },
            "model_assumption": format!(
                "Character {native_id} resonance_3 activates when at least three awakening effects are selected."
            )
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntesim_data::AwakeningSelection;

    #[test]
    fn independent_slot_bonuses_preserve_qte_exceptions_empty_chiz_and_selected_count() {
        let tables = BTreeMap::new();
        for owner in ntesim_data::character_actions::CHARACTERS {
            for (selected, active) in [
                (vec![], false),
                (vec![1, 2], false),
                (vec![3], false),
                (vec![1, 2, 3], true),
                (vec![2, 4, 6], true),
                (vec![1, 2, 3, 4, 5, 6], true),
            ] {
                let awakening = AwakeningSelection::selected(selected).unwrap();
                for (slot, ability) in owner.abilities.iter().enumerate() {
                    // Independent source endpoints: Edgar/Linko replace normal
                    // with QTE; Chiz's buff-only resonance adds no skill levels.
                    let eligible = match owner.native_id {
                        "1073" => false,
                        "1021" | "1072" => slot != 0,
                        _ => slot != 3,
                    };
                    let (actual, receipt) = bonus(
                        &tables,
                        &format!("character:{}", owner.native_id),
                        ability,
                        &awakening,
                    )
                    .unwrap();
                    assert_eq!(
                        actual,
                        u32::from(active && eligible),
                        "{} {ability} {:?}",
                        owner.native_id,
                        awakening
                    );
                    assert_eq!(
                        receipt["evidence"]["source_id"],
                        ntesim_data::resonance_levels::SOURCE_ID
                    );
                }
            }
        }
    }

    #[test]
    fn owner_overlays_diagnose_missing_rows_and_invalid_source_or_level() {
        for native in ["1019", "1020"] {
            let name = if native == "1019" {
                "1019AwakenEffect1".into()
            } else {
                format!("{native}AwakenEffect")
            };
            let value = ntesim_data::resonance_levels::overlay(native).unwrap();
            let ability = if native == "1019" {
                "GA_Mint019_Skill"
            } else {
                "GA_Haniel_Skill"
            };
            for pointer in [
                "/source_id".to_owned(),
                format!("/rows/{native}"),
                format!("/rows/{native}/AwakenEffectStructList/0/ModifyDataList/0/SkillLevel"),
            ] {
                let mut changed = value.clone();
                *changed.pointer_mut(&pointer).unwrap() = json!(2);
                let tables = BTreeMap::from([(name.clone(), changed)]);
                assert!(
                    bonus(
                        &tables,
                        &format!("character:{native}"),
                        ability,
                        &AwakeningSelection::prefix(3)
                    )
                    .is_err(),
                    "{native} {pointer}"
                );
            }
        }
    }
}
