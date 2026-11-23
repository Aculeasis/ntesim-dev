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
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_prism")?
        .1
        .as_object()
        .ok_or("Far Side of the Moon native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .filter(|group| !group.is_empty())
        .ok_or("Far Side of the Moon item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Far Side of the Moon group is incompatible with the wearer".into());
    }
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Far Side of the Moon parameters missing")?;
    let expected = [
        ("buff_Prism_CritUp", true),
        ("buff_Prism_DamageUp1", true),
        ("buff_Prism_DamageUp2", true),
        ("buff_Prism_LakshanaUp", true),
        ("buff_Prism_DamageUp2_CD", false),
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
        return Err("unsupported Far Side of the Moon parameters or unit hint".into());
    }
    let parameter = |name: &str| -> Result<ExactFraction, String> {
        let selected = parameters
            .iter()
            .find(|parameter| {
                parameter["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
            })
            .map(|parameter| &parameter["selected"]["value"])
            .ok_or_else(|| format!("Far Side of the Moon required parameter {name} missing"))?;
        receipt(selected)
    };
    let duration_us = weapon_seconds_to_us(
        parameter("buff_Prism_DamageUp2_CD")?,
        "Far Side of the Moon duration",
    )?;
    let proc = GearProcConfig::FarSideMoon {
        crit_rate_bonus: ratio(parameter("buff_Prism_CritUp")?)?,
        skill_damage_per_stack: ratio(
            parameter("buff_Prism_DamageUp1")?
                .div(ExactFraction::new(u128::from(super::MAX_SKILL_STACKS), 1)?)?,
        )?,
        basic_attack_damage_bonus: ratio(parameter("buff_Prism_DamageUp2")?)?,
        lakshana_damage_bonus: ratio(parameter("buff_Prism_LakshanaUp")?)?,
        duration_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({"runtime_execution_validated":true,
        "runtime_evidence":"Continuing Daffodil sources exercise Basic Attack stack generation and Redirect Skill delivery at all five refinements. A controlled Lakshana packet test checks the overlap bonus and its expiry.",
        "known_gaps":"Shared stack refresh at cap, world-time duration and equal-time grant order are explicit model assumptions. The supported roster has no native Lakshana wearer in this weapon group.",
        "scope":crate::weapons::far_side_moon::HYPOTHESIS_ID.description()}),
    )
}
