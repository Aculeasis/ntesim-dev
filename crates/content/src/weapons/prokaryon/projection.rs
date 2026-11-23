use crate::build::{compile_weapon_values, finish_weapon_projection, ratio, receipt};
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
        .ok_or("Prokaryon parameters missing")?;
    if parameters.len() != 1
        || !parameters[0]["name_id"]
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case("buff_Prokaryon_Up"))
        || parameters[0]["b_is_percent"] != true
    {
        return Err("unsupported Prokaryon parameters or unit hint".into());
    }
    let proc = GearProcConfig::ProkaryonDamage {
        damage_bonus: ratio(receipt(&parameters[0]["selected"]["value"])?)?,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":false,
        "scope":crate::weapons::prokaryon::HYPOTHESIS_ID.description()}),
    )
}
