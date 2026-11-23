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
    relative_duration_us: u64,
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Bittercake parameters missing")?;
    if parameters.len() != 3 {
        return Err("unsupported additional Bittercake parameter dependencies".into());
    }
    let parameter = |name: &str, percent: bool| -> Result<ExactFraction, String> {
        let value = parameters
            .iter()
            .find(|parameter| {
                parameter["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
            })
            .ok_or_else(|| format!("Bittercake required parameter {name} missing"))?;
        if value["b_is_percent"] != percent {
            return Err(format!("unsupported Bittercake unit hint for {name}"));
        }
        receipt(&value["selected"]["value"])
    };
    let duration_us = weapon_seconds_to_us(
        parameter("buff_BitterCake_CD", false)?,
        "Bittercake duration",
    )?;
    let cooldown_us = weapon_seconds_to_us(
        parameter("buff_BitterCake_CD2", false)?,
        "Bittercake cooldown",
    )?;
    relative_duration_us
        .checked_add(duration_us.max(cooldown_us))
        .ok_or("Bittercake projected deadline overflow")?;
    let proc = GearProcConfig::BitterCake {
        def_bonus: ratio(parameter("buff_BitterCake_DefUp", true)?)?,
        duration_us,
        cooldown_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "scope": crate::weapons::bittercake::HYPOTHESIS_ID.description()
        }),
    )
}
