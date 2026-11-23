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
    let native = compile_native(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Call of the Twisted City parameters missing")?;
    let healing = parameter(parameters, "buff_Castle_HealUp", true)?;
    let duration_us = weapon_seconds_to_us(
        parameter(parameters, "buff_Castle_CD", false)?,
        "Call of the Twisted City duration",
    )?;
    let proc = GearProcConfig::CastleHealingBonus {
        healing_bonus: ratio(healing)?,
        duration_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "scope": crate::weapons::castle::HYPOTHESIS_ID.description(),
        }),
    )
}

fn parameter(parameters: &[Value], name: &str, is_percent: bool) -> Result<ExactFraction, String> {
    let parameter = parameters
        .iter()
        .find(|parameter| {
            parameter["name_id"]
                .as_str()
                .is_some_and(|id| id.eq_ignore_ascii_case(name))
        })
        .ok_or_else(|| format!("Call of the Twisted City parameter {name} missing"))?;
    if parameter["b_is_percent"] != is_percent {
        return Err(format!(
            "Call of the Twisted City has unsupported unit hint for {name}"
        ));
    }
    receipt(&parameter["selected"]["value"])
}

pub(crate) fn compile_native(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &GameActorConfig,
) -> Result<Value, String> {
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_Castle")?
        .1
        .as_object()
        .ok_or("Call of the Twisted City native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .ok_or("Call of the Twisted City item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Call of the Twisted City group is incompatible with the wearer".into());
    }
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Call of the Twisted City parameters missing")?;
    if parameters.len() != 2
        || !parameters
            .iter()
            .any(|parameter| parameter["name_id"] == "buff_Castle_HealUp")
        || !parameters
            .iter()
            .any(|parameter| parameter["name_id"] == "buff_Castle_CD")
    {
        return Err(
            "unsupported additional Call of the Twisted City parameter dependencies".into(),
        );
    }
    Ok(native)
}
