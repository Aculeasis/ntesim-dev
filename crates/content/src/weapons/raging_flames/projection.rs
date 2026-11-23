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
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_wushoutieyu")?
        .1
        .as_object()
        .ok_or("Raging Flames native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .filter(|group| !group.is_empty())
        .ok_or("Raging Flames item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Raging Flames group is incompatible with the wearer".into());
    }
    let exclusives = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Raging Flames exclusive wearer list missing")?;
    if !exclusives.is_empty()
        && !exclusives
            .iter()
            .any(|id| id.as_str().is_some_and(|id| id == character.native_id))
    {
        return Err("Raging Flames is exclusive to a different wearer".into());
    }
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Raging Flames parameters missing")?;
    let expected = [
        ("buff_wushoutieyu_Up", true),
        ("buff_wushoutieyu_CD", false),
        ("buff_wushoutieyu_Up2", true),
        ("buff_wushoutieyu_UpLakshana", true),
    ];
    if parameters.len() != expected.len()
        || parameters.iter().any(|parameter| {
            let Some(name) = parameter["name_id"].as_str() else {
                return true;
            };
            expected
                .iter()
                .find(|(expected_name, _)| name.eq_ignore_ascii_case(expected_name))
                .is_none_or(|(_, percent)| parameter["b_is_percent"] != *percent)
        })
    {
        return Err("unsupported Raging Flames parameters or unit hint".into());
    }
    let parameter = |name: &str| -> Result<&Value, String> {
        parameters
            .iter()
            .find(|parameter| {
                parameter["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
            })
            .map(|parameter| &parameter["selected"]["value"])
            .ok_or_else(|| format!("Raging Flames required parameter {name} missing"))
    };
    let duration_us = weapon_seconds_to_us(
        receipt(parameter("buff_wushoutieyu_CD")?)?,
        "Raging Flames duration",
    )?;
    let proc = GearProcConfig::RagingFlames {
        redirect_ultimate_bonus: ratio(receipt(parameter("buff_wushoutieyu_Up")?)?)?,
        duration_us,
        redirect_skill_stack_bonus: ratio(receipt(parameter("buff_wushoutieyu_Up2")?)?)?,
        lakshana_bonus: ratio(receipt(parameter("buff_wushoutieyu_UpLakshana")?)?)?,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":true,
        "runtime_evidence":"T1-T5 public Hathor scripts fund two Bursts from zero and compare native packets against a passive-free control, including both Skill stacks, additive damage zones, switch persistence and expiry. Nanally and synthetic boundary checks retain cap, exact expiry, delayed delivery and Ultimate refresh reset.",
        "known_gaps":"The frozen native data does not establish live-game buff lifecycle, timing, or Global release availability. Ultimate-refresh stack reset and cast-order behavior remain explicit tested hypotheses.",
        "scope":crate::weapons::raging_flames::HYPOTHESIS_ID.description()}),
    )
}
