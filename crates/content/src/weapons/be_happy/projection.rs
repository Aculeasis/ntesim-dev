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
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_vine")?
        .1
        .as_object()
        .ok_or("Be Happy native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .ok_or("Be Happy item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Be Happy group is incompatible with the wearer".into());
    }
    let native = compile_native(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Be Happy parameters missing")?;
    let parameter = |name: &str| -> Result<ExactFraction, String> {
        let value = parameters
            .iter()
            .find(|p| {
                p["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
            })
            .ok_or_else(|| format!("Be Happy parameter {name} missing"))?;
        receipt(&value["selected"]["value"])
    };
    let cooldown_us = weapon_seconds_to_us(parameter("buff_vine_CD")?, "Be Happy cooldown")?;
    let proc = GearProcConfig::BeHappy {
        heal_ratio: ratio(parameter("buff_vine_Hp")?)?,
        cooldown_us,
    };
    let pack = item["ElementData"]["UpgradeStarPackID"]
        .as_str()
        .ok_or("Be Happy upgrade pack missing")?;
    let upgrade =
        ntesim_data::weapon_refinements::lookup(tables, &format!("{pack}_{}", actor.weapon_tier))?
            .1
            .as_object()
            .ok_or("Be Happy native upgrade row missing")?;
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "native_description": {
                "key": upgrade["Description"]["Key"],
                "localized": upgrade["Description"]["LocalizedString"],

            },
            "scope": crate::weapons::be_happy::HYPOTHESIS_ID.description(),
        }),
    )
}

pub(crate) fn compile_native(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &GameActorConfig,
) -> Result<Value, String> {
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Be Happy parameters missing")?;
    let expected = [("buff_vine_Hp", true), ("buff_vine_CD", false)];
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
        return Err("unsupported Be Happy parameters or unit hint".into());
    }
    Ok(native)
}
