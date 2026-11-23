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
        .ok_or("Blast Candy parameters missing")?;
    if parameters.len() != 2 {
        return Err("unsupported additional Blast Candy parameter dependencies".into());
    }
    let parameter = |name: &str, percent: bool| -> Result<ExactFraction, String> {
        let value = parameters
            .iter()
            .find(|p| {
                p["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
            })
            .ok_or_else(|| format!("Blast Candy required parameter {name} missing"))?;
        if value["b_is_percent"] != percent {
            return Err(format!("unsupported Blast Candy unit hint for {name}"));
        }
        receipt(&value["selected"]["value"])
    };
    let duration_us = weapon_seconds_to_us(
        parameter("buff_BlastCandy_CD", false)?,
        "Blast Candy duration",
    )?;
    relative_duration_us
        .checked_add(duration_us)
        .ok_or("Blast Candy projected deadline overflow")?;
    let proc = GearProcConfig::BlastCandyAtk {
        atk_bonus: ratio(parameter("buff_BlastCandy_AtkUp", true)?)?,
        duration_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":true,
        "scope":crate::weapons::blast_candy::HYPOTHESIS_ID.description()}),
    )
}
