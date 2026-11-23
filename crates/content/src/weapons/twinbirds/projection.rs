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
        .ok_or("Twinflight parameters missing")?;
    let expected = [
        ("Buff_TwinBirds_MagUp", false),
        ("Buff_TwinBirds_PsycheUp", true),
        ("buff_TwinBirds_CritUp1", true),
        ("buff_TwinBirds_CritUp3", true),
        ("buff_TwinBirds_CritUp1_CD", false),
    ];
    if parameters.len() != expected.len()
        || parameters.iter().zip(expected).any(|(p, (name, percent))| {
            !p["name_id"]
                .as_str()
                .is_some_and(|id| id.eq_ignore_ascii_case(name))
                || p["b_is_percent"] != percent
        })
    {
        return Err("unsupported Twinflight parameters or unit hint".into());
    }
    let duration_us = weapon_seconds_to_us(
        receipt(&parameters[4]["selected"]["value"])?,
        "Twinflight duration",
    )?;
    let proc = GearProcConfig::Twinbirds {
        cycle_intensity_bonus: ratio(receipt(&parameters[0]["selected"]["value"])?)?,
        psyche_bonus: ratio(receipt(&parameters[1]["selected"]["value"])?)?,
        team_crit_damage_bonus: ratio(receipt(&parameters[2]["selected"]["value"])?)?,
        self_crit_damage_bonus: ratio(receipt(&parameters[3]["selected"]["value"])?)?,
        duration_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":true,
        "scope":crate::weapons::twinbirds::HYPOTHESIS_ID.description()}),
    )
}
