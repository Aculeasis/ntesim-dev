use crate::build::{
    WeaponParameters, compile_weapon_values, finish_weapon_projection, ratio, weapon_wearer_item,
};
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) const PARAMETERS: &[(&str, bool)] = &[
    ("buff_bowenchanggui_HP", true),
    ("buff_bowenchanggui_shield", true),
];

pub(crate) fn project(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    let item = weapon_wearer_item(tables, &actor.character_id, "fork_yuren", "Rainman")?;
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Rainman exclusive character list missing")?;
    if !exclusive.is_empty()
        || item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"].is_string()
    {
        return Err("Rainman frozen item has unsupported exclusive-character data".into());
    }

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;

    let parameters = WeaponParameters::new(&native, PARAMETERS, "Rainman")?;
    let parameter = |name: &str| parameters.fraction(name);
    let proc = GearProcConfig::YurenHp {
        hp_bonus: ratio(parameter("buff_bowenchanggui_HP")?)?,
        shield_bonus: ratio(parameter("buff_bowenchanggui_shield")?)?,
    };
    let description = native["description"].clone();
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "native_description": {
                "key": description["Key"],
                "localized": description["LocalizedString"],
            },
            "scope": crate::weapons::yuren::HYPOTHESIS_ID.description()
        }),
    )
}
