use crate::build::{compile_weapon_values, finish_weapon_projection, ratio, receipt};
use crate::formula::Ratio;
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
        .ok_or("The Forgotten native parameters missing")?;
    if parameters.len() != 2 {
        return Err("unsupported additional The Forgotten parameter dependencies".into());
    }
    let parameter = |name: &str| -> Result<Ratio, String> {
        let value = parameters
            .iter()
            .find(|parameter| parameter["name_id"].as_str() == Some(name))
            .ok_or_else(|| format!("The Forgotten required parameter {name} missing"))?;
        if value["b_is_percent"] != true {
            return Err(format!("The Forgotten {name} must be a percentage ratio"));
        }
        ratio(receipt(&value["selected"]["value"])?)
    };
    let proc = GearProcConfig::ForgottenHpStats {
        atk_bonus: parameter("buff_kudangao_Atk")?,
        def_bonus: parameter("buff_kudangao_Def")?,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "scope": crate::weapons::forgotten::HYPOTHESIS_ID.description()
        }),
    )
}
