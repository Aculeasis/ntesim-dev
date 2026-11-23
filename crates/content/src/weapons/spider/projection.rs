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
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_spider")?
        .1
        .as_object()
        .ok_or("Failing You, Heavy in My Heart native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .ok_or("Failing You, Heavy in My Heart item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Failing You, Heavy in My Heart group is incompatible with the wearer".into());
    }

    let native = compile_native(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Failing You, Heavy in My Heart parameters missing")?;
    let parameter = |name: &str| -> Result<ExactFraction, String> {
        let value = parameters
            .iter()
            .find(|p| p["name_id"].as_str() == Some(name))
            .ok_or_else(|| {
                format!("Failing You, Heavy in My Heart required parameter {name} missing")
            })?;
        receipt(&value["selected"]["value"])
    };

    let duration_us = weapon_seconds_to_us(
        parameter("buff_spider_CD")?,
        "Failing You, Heavy in My Heart duration",
    )?;

    let proc = GearProcConfig::SpiderTeamAtk {
        atk_bonus_per_stack: ratio(parameter("buff_spider_AtkUp")?)?,
        atk_bonus_max_stack: ratio(parameter("buff_spider_AtkUp2")?)?,
        duration_us,
    };

    let upgrade_pack = item["ElementData"]["UpgradeStarPackID"]
        .as_str()
        .ok_or("Failing You, Heavy in My Heart upgrade pack missing")?;
    let upgrade_row = format!("{upgrade_pack}_{}", actor.weapon_tier);
    let description = ntesim_data::weapon_refinements::lookup(tables, &upgrade_row)?
        .1
        .as_object()
        .ok_or("Failing You, Heavy in My Heart native upgrade row missing")?;
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "native_description": {
                "key": description["Description"]["Key"],
                "localized": description["Description"]["LocalizedString"],
            },
            "scope": crate::weapons::spider::HYPOTHESIS_ID.description(),
        }),
    )
}

pub(crate) fn compile_native(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &GameActorConfig,
) -> Result<Value, String> {
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Failing You, Heavy in My Heart parameters missing")?;
    let expected = [
        ("buff_spider_CD", false),
        ("buff_spider_AtkUp", true),
        ("buff_spider_AtkUp2", true),
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
        return Err("unsupported Failing You, Heavy in My Heart parameters or unit hint".into());
    }
    Ok(native)
}
