use crate::native::text;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn weapon_group_binding(
    tables: &BTreeMap<String, Value>,
    character_id: &str,
    weapon_id: &str,
) -> Result<Value, String> {
    let character = crate::character::definition(character_id)?;
    let weapon_table = crate::weapon_definitions::table(tables)?;
    let native_weapon = weapon_id
        .strip_prefix("weapon:")
        .filter(|s| !s.is_empty())
        .ok_or("expected weapon:native_id")?;
    let char_id = character.native_id;
    let (weapon_id, weapon) = weapon_table.lookup(native_weapon)?;
    let character_group = character.weapon_group.native_key();
    let weapon_group = text(&weapon["ElementData"], "ApplyGroupType")?;
    let group = crate::character::WeaponGroup::from_native(weapon_group)?;
    if character.weapon_group != group {
        return Err(format!(
            "native weapon group incompatible: {character_group} vs {weapon_group}"
        ));
    }
    let character_evidence = character.evidence();
    let mut weapon_evidence = weapon_table.evidence(weapon_id);
    weapon_evidence["pointer"] = json!(format!(
        "{}/ElementData/ApplyGroupType",
        weapon_evidence["pointer"].as_str().unwrap()
    ));
    Ok(json!({
        "policy":"native-character-group-match-v1", "character_id":format!("character:{char_id}"),
        "weapon_id":format!("weapon:{weapon_id}"), "character_group":character_group, "weapon_group":weapon_group,
        "groups_match":true, "evidence":[character_evidence, weapon_evidence],
        "exclusive_character_ids":weapon["ElementData"]["ExclusiveCharacterIDArray"],
        "exclusive_character_semantics_validated":false, "full_compatibility_validated":false,
    }))
}
