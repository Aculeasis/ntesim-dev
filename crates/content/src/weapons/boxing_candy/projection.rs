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
        .ok_or("Boxing Candy parameters missing")?;
    let expected = ["buff_BoxingCandy_Up", "buff_BoxingCandy_Up2"];
    if parameters.len() != expected.len()
        || parameters.iter().zip(expected).any(|(parameter, name)| {
            !parameter["name_id"]
                .as_str()
                .is_some_and(|id| id.eq_ignore_ascii_case(name))
                || parameter["b_is_percent"] != true
        })
    {
        return Err("unsupported Boxing Candy parameters or unit hint".into());
    }
    let proc = GearProcConfig::BoxingCandyDamage {
        damage_bonus: ratio(receipt(&parameters[0]["selected"]["value"])?)?,
        low_hp_damage_bonus: ratio(receipt(&parameters[1]["selected"]["value"])?)?,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":true,
        "scope":crate::weapons::boxing_candy::HYPOTHESIS_ID.description()}),
    )
}
