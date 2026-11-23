use crate::build::{compile_weapon_values, finish_weapon_projection, ratio, receipt};
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
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Noros parameters missing")?;
    let expected = [
        ("buff_nonos_AtkUp", true),
        ("buff_nonos_CD", false),
        ("buff_nonos_CD2", false),
    ];
    if parameters.len() != expected.len()
        || parameters.iter().zip(expected).any(|(p, (name, percent))| {
            !p["name_id"]
                .as_str()
                .is_some_and(|id| id.eq_ignore_ascii_case(name))
                || p["b_is_percent"] != percent
        })
    {
        return Err("unsupported Noros parameters or unit hint".into());
    }
    let micros = |index: usize| -> Result<u64, String> {
        let duration = receipt(&parameters[index]["selected"]["value"])?
            .mul(ExactFraction::new(1_000_000, 1)?)?;
        if duration.denominator != 1 || duration.is_zero() {
            return Err("Noros requires positive integral microseconds".into());
        }
        u64::try_from(duration.numerator).map_err(|_| "Noros duration exceeds u64".into())
    };
    let proc = GearProcConfig::NorosAtk {
        atk_bonus: ratio(receipt(&parameters[0]["selected"]["value"])?)?,
        duration_us: micros(1)?,
        cooldown_us: micros(2)?,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":false,
        "scope":crate::weapons::noros::HYPOTHESIS_ID.description()}),
    )
}
