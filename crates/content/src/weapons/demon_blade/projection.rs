use crate::build::{
    compile_weapon_values, finish_weapon_projection, ratio, receipt, weapon_seconds_to_us,
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
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Ravenous Blade parameters missing")?;
    let expected = [
        ("buff_DemonBlade_Crit", true),
        ("buff_DemonBlade_CritDamageUp", true),
        ("buff_DemonBlade_CD", false),
    ];
    if parameters.len() != expected.len()
        || parameters.iter().zip(expected).any(|(p, (name, percent))| {
            !p["name_id"]
                .as_str()
                .is_some_and(|id| id.eq_ignore_ascii_case(name))
                || p["b_is_percent"] != percent
        })
    {
        return Err("unsupported Ravenous Blade parameters or unit hint".into());
    }
    let duration_us = weapon_seconds_to_us(
        receipt(&parameters[2]["selected"]["value"])?,
        "Ravenous Blade duration",
    )?;
    let proc = GearProcConfig::DemonBlade {
        crit_rate_bonus: ratio(receipt(&parameters[0]["selected"]["value"])?)?,
        crit_damage_per_stack: ratio(receipt(&parameters[1]["selected"]["value"])?)?,
        duration_us,
        icd_us: 300_000,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":false,
        "scope":crate::weapons::demon_blade::HYPOTHESIS_ID.description()}),
    )
}
