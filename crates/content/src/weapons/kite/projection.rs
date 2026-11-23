use crate::build::{
    compile_weapon_values, finish_weapon_projection, ratio, receipt, weapon_seconds_to_us,
};
use crate::simulation::{ExactFraction, GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn project(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_Kite")?
        .1
        .as_object()
        .ok_or("Watch Your Heads! native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .ok_or("Watch Your Heads! item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Watch Your Heads! group is incompatible with the wearer".into());
    }

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Watch Your Heads! parameters missing")?;
    let expected = [
        ("buff_Kite_AtkUp", true),
        ("buff_Kite_CD", false),
        ("buff_Kite_Up", true),
    ];
    if parameters.len() != expected.len()
        || parameters.iter().any(|parameter| {
            let Some(name) = parameter["name_id"].as_str() else {
                return true;
            };
            expected
                .iter()
                .find(|(expected_name, _)| name.eq_ignore_ascii_case(expected_name))
                .is_none_or(|(_, percent)| parameter["b_is_percent"] != *percent)
        })
    {
        return Err("unsupported Watch Your Heads! parameters or unit hint".into());
    }
    let parameter = |name: &str| -> Result<ExactFraction, String> {
        let selected = parameters
            .iter()
            .find(|parameter| {
                parameter["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
            })
            .map(|parameter| &parameter["selected"]["value"])
            .ok_or_else(|| format!("Watch Your Heads! required parameter {name} missing"))?;
        receipt(selected)
    };
    let duration_us =
        weapon_seconds_to_us(parameter("buff_Kite_CD")?, "Watch Your Heads! duration")?;
    let proc = GearProcConfig::KiteAtk {
        atk_bonus: ratio(parameter("buff_Kite_AtkUp")?)?,
        lakshana_bonus: ratio(parameter("buff_Kite_Up")?)?,
        duration_us,
    };
    let upgrade_pack = item["ElementData"]["UpgradeStarPackID"]
        .as_str()
        .ok_or("Watch Your Heads! upgrade pack missing")?;
    let upgrade_row = format!("{upgrade_pack}_{}", actor.weapon_tier);
    let description = ntesim_data::weapon_refinements::lookup(tables, &upgrade_row)?
        .1
        .as_object()
        .ok_or("Watch Your Heads! native upgrade row missing")?;
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "native_description": {
                "key": description["Description"]["Key"],
                "localized": description["Description"]["LocalizedString"],
            },
            "scope": crate::weapons::kite::HYPOTHESIS_ID.description()
        }),
    )
}
