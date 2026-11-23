use crate::build::{
    WeaponParameters, compile_weapon_values, finish_weapon_projection, ratio, weapon_wearer_item,
};
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn project(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    let item = weapon_wearer_item(
        tables,
        &actor.character_id,
        "fork_yaodao",
        "Crimson Hexblade",
    )?;
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Crimson Hexblade exclusive character list missing")?;
    if !exclusive.is_empty()
        || item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"].is_string()
    {
        return Err("Crimson Hexblade frozen item has unsupported exclusive-character data".into());
    }

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let expected = [("GE_Fork_yaodao_Skill_Damage", true)];
    let parameters = WeaponParameters::new(&native, &expected, "Crimson Hexblade")?;
    let parameter = |name: &str| parameters.fraction(name);
    let proc = GearProcConfig::YaodaoParry {
        parry_damage_ratio: ratio(parameter("GE_Fork_yaodao_Skill_Damage")?)?,
    };
    let description = native["description"].clone();
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": false,
            "native_description": {
                "key": description["Key"],
                "localized": description["LocalizedString"],
            },
            "scope": crate::weapons::yaodao::HYPOTHESIS_ID.description()
        }),
    )
}
