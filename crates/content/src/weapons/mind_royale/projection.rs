use crate::build::{
    compile_weapon_values, finish_weapon_projection, integer, receipt, weapon_seconds_to_us,
};
use crate::simulation::{GameActorConfig, GearProcConfig};
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
        .ok_or("Mind Royale parameters missing")?;
    if parameters.len() != 2 {
        return Err("unsupported additional Mind Royale parameter dependencies".into());
    }
    let parameter = |name: &str| -> Result<&Value, String> {
        let value = parameters
            .iter()
            .find(|p| {
                p["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
            })
            .ok_or("Mind Royale required parameter missing")?;
        if value["b_is_percent"] != false {
            return Err("unsupported Mind Royale percent semantics".into());
        }
        Ok(&value["selected"]["value"])
    };
    let energy = integer(parameter("buff_xiangdao_energy")?, "Mind Royale energy")?;
    let cooldown_us = weapon_seconds_to_us(
        receipt(parameter("buff_xiangdao_CD")?)?,
        "Mind Royale cooldown",
    )?;
    relative_duration_us
        .checked_add(cooldown_us)
        .ok_or("Mind Royale projected deadline overflow")?;
    let proc = GearProcConfig::MindRoyaleEnergy {
        energy_grant: energy,
        cooldown_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "scope": "Explicit coordinate T -> integer energy and seconds -> microseconds hypothesis, using the existing Mint Redirect Skill/self/once-per-cooldown research policy."
        }),
    )
}
