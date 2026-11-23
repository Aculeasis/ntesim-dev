use crate::build::{
    WeaponParameters, compile_weapon_values, finish_weapon_projection, ratio, weapon_seconds_to_us,
    weapon_wearer_item,
};
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) const PARAMETERS: &[(&str, bool)] = &[
    ("buff_Nakipeda_Hp", true),
    ("buff_Nakipeda_effect1", true),
    ("buff_Nakipeda_effect2", true),
    ("buff_Nakipeda_effect2CD", false),
    ("buff_Nakipeda_effect3", true),
    ("buff_Nakipeda_CD", false),
];

pub(crate) fn project(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    let item = weapon_wearer_item(tables, &actor.character_id, "fork_Nakupeda", "Nacupeda")?;
    if item["ElementData"]["ApplyGroupType"].as_str() != Some(super::GROUP_ID) {
        return Err("Nacupeda frozen weapon group changed".into());
    }
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Nacupeda exclusive character list missing")?;
    if !exclusive.is_empty()
        || item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"].is_string()
    {
        return Err("Nacupeda frozen item has unsupported exclusive-character data".into());
    }

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;

    let parameters = WeaponParameters::new(&native, PARAMETERS, "Nacupeda")?;
    let parameter = |name: &str| parameters.fraction(name);
    let proc = GearProcConfig::Nakupeda {
        hp_bonus: ratio(parameter("buff_Nakipeda_Hp")?)?,
        lowest_hp_heal_ratio: ratio(parameter("buff_Nakipeda_effect1")?)?,
        shield_ratio: ratio(parameter("buff_Nakipeda_effect2")?)?,
        shield_duration_us: weapon_seconds_to_us(
            parameter("buff_Nakipeda_effect2CD")?,
            "Nacupeda shield duration",
        )?,
        team_heal_ratio: ratio(parameter("buff_Nakipeda_effect3")?)?,
        cooldown_us: weapon_seconds_to_us(parameter("buff_Nakipeda_CD")?, "Nacupeda cooldown")?,
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
            "scope": crate::weapons::nakupeda::HYPOTHESIS_ID.description()
        }),
    )
}
