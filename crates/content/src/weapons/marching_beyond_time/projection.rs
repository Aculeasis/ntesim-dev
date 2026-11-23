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
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_Time")?
        .1
        .as_object()
        .ok_or("Marching Beyond Time native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .ok_or("Marching Beyond Time item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Marching Beyond Time group is incompatible with the wearer".into());
    }

    let native = compile_native(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Marching Beyond Time parameters missing")?;
    let parameter = |name: &str| -> Result<ExactFraction, String> {
        let value = parameters
            .iter()
            .find(|p| {
                p["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
            })
            .ok_or_else(|| format!("Marching Beyond Time parameter {name} missing"))?;
        receipt(&value["selected"]["value"])
    };
    let duration_us = weapon_seconds_to_us(
        parameter("buff_Time_DefIgnore_Dur")?,
        "Marching Beyond Time duration",
    )?;
    let proc = GearProcConfig::MarchingBeyondTime {
        atk_bonus: ratio(parameter("buff_Time_AtkUp")?)?,
        state_crit_damage_bonus: ratio(parameter("buff_Time_stateCritDamageUp")?)?,
        crit_damage_per_stack: ratio(parameter("buff_Time_CritDamageUp")?)?,
        def_ignore: ratio(parameter("buff_Time_DefIgnore")?)?,
        def_ignore_duration_us: duration_us,
    };
    let pack = item["ElementData"]["UpgradeStarPackID"]
        .as_str()
        .ok_or("Marching Beyond Time upgrade pack missing")?;
    let upgrade =
        ntesim_data::weapon_refinements::lookup(tables, &format!("{pack}_{}", actor.weapon_tier))?
            .1
            .as_object()
            .ok_or("Marching Beyond Time native upgrade row missing")?;
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
            "scope": crate::weapons::marching_beyond_time::HYPOTHESIS_ID.description(),
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
        .ok_or("Marching Beyond Time parameters missing")?;
    let expected = [
        ("buff_Time_AtkUp", true),
        ("buff_Time_stateCritDamageUp", true),
        ("buff_Time_CritDamageUp", true),
        ("buff_Time_DefIgnore", true),
        ("buff_Time_DefIgnore_Dur", false),
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
        return Err("unsupported Marching Beyond Time parameters or unit hint".into());
    }
    Ok(native)
}
