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
        .ok_or("Knight Candy parameters missing")?;
    if parameters.len() != 2
        || !parameters[0]["name_id"]
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case("buff_KnightCandy_CritDamageUp"))
        || parameters[0]["b_is_percent"] != true
        || !parameters[1]["name_id"]
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case("buff_KnightCandy_CD"))
        || parameters[1]["b_is_percent"] != false
    {
        return Err("unsupported Knight Candy parameters or unit hint".into());
    }
    let duration_us = weapon_seconds_to_us(
        receipt(&parameters[1]["selected"]["value"])?,
        "Knight Candy duration",
    )?;
    let proc = GearProcConfig::KnightCandyCritDamage {
        crit_damage_per_stack: ratio(receipt(&parameters[0]["selected"]["value"])?)?,
        duration_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":true,
        "scope":crate::weapons::knight_candy::HYPOTHESIS_ID.description()}),
    )
}
