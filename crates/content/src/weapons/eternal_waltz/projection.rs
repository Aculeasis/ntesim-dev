use crate::build::{finish_weapon_projection, ratio, receipt, weapon_seconds_to_us};
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
    let parameters = native["parameters"].as_array().unwrap();
    let duration_us = weapon_seconds_to_us(
        receipt(&parameters[2]["selected"]["value"])?,
        "Eternal Waltz duration",
    )?;
    let proc = GearProcConfig::EternalWaltz {
        max_hp_bonus: ratio(receipt(&parameters[0]["selected"]["value"])?)?,
        mental_bonus: ratio(receipt(&parameters[1]["selected"]["value"])?)?,
        duration_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":false,"scope":crate::weapons::eternal_waltz::HYPOTHESIS_ID.description()}),
    )
}

pub(crate) fn compile_native(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &GameActorConfig,
) -> Result<Value, String> {
    compile_native_weapon(
        tables,
        schema_version,
        content_id,
        &actor.weapon_id,
        actor.weapon_tier,
    )
}

pub(crate) fn compile_native_weapon(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    weapon_id: &str,
    refinement: u32,
) -> Result<Value, String> {
    let native = ntesim_data::constants::weapon_passive_values::compile(
        tables,
        &ntesim_data::constants::weapon_passive_values::Request {
            schema_version,
            content_id: content_id.into(),
            weapon_id: weapon_id.into(),
            refinement,
        },
    )?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Eternal Waltz parameters missing")?;
    let expected = [
        ("buff_Arachne_Hp", true),
        ("buff_Arachne_Up", true),
        ("buff_Arachne_CD", false),
    ];
    if parameters.len() != expected.len()
        || parameters
            .iter()
            .zip(expected)
            .any(|(parameter, (name, percent))| {
                !parameter["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
                    || parameter["b_is_percent"] != percent
            })
    {
        return Err("unsupported Eternal Waltz parameters or unit hint".into());
    }
    Ok(native)
}

pub(crate) fn add_base_hp_ratio(
    derived_hp: ExactFraction,
    native_base_hp: ExactFraction,
    weapon_hp_ratio: ExactFraction,
) -> Result<ExactFraction, String> {
    derived_hp.add(native_base_hp.mul(weapon_hp_ratio)?)
}
