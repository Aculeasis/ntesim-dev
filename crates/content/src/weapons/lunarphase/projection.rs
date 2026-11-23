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
        .ok_or("Blushing Mirage parameters missing")?;
    let expected = [
        ("buff_LunarPhase_Atk", true),
        ("buff_LunarPhase_Up", true),
        ("buff_LunarPhase_DefIgnore", true),
        ("buff_LunarPhase_CD", false),
    ];
    if parameters.len() != expected.len()
        || parameters.iter().zip(expected).any(|(p, (name, percent))| {
            !p["name_id"]
                .as_str()
                .is_some_and(|id| id.eq_ignore_ascii_case(name))
                || p["b_is_percent"] != percent
        })
    {
        return Err("unsupported Blushing Mirage parameters or unit hint".into());
    }
    let duration_us = weapon_seconds_to_us(
        receipt(&parameters[3]["selected"]["value"])?,
        "Blushing Mirage duration",
    )?;
    let proc = GearProcConfig::LunarPhase {
        atk_bonus: ratio(receipt(&parameters[0]["selected"]["value"])?)?,
        cosmos_bonus: ratio(receipt(&parameters[1]["selected"]["value"])?)?,
        def_ignore: ratio(receipt(&parameters[2]["selected"]["value"])?)?,
        duration_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":true,
        "runtime_evidence":"T1-T5 legal Shinku scripts fund two initial Bursts from zero, check native packet damage with independent base-ATK/Cosmos/DEF arithmetic and preserve off-field window expiry, aggregation and replay through native/WASM/Worker consumers.",
        "known_gaps":"Public action cost/timing, Surging Crimson and weapon callback/refresh policies remain disclosed product models. This does not establish native Rage follow-up dispatch or Global release availability.",
        "scope":crate::weapons::lunarphase::HYPOTHESIS_ID.description()}),
    )
}
