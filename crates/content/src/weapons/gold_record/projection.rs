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
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_GoldRecord")?
        .1
        .as_object()
        .ok_or("Voice of the Voyager native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .ok_or("Voice of the Voyager item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Voice of the Voyager group is incompatible with the wearer".into());
    }

    let native = compile_native(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Voice of the Voyager parameters missing")?;
    let parameter = |name: &str| -> Result<ExactFraction, String> {
        let value = parameters
            .iter()
            .find(|p| p["name_id"].as_str() == Some(name))
            .ok_or_else(|| format!("Voice of the Voyager required parameter {name} missing"))?;
        receipt(&value["selected"]["value"])
    };

    let duration_us = weapon_seconds_to_us(
        parameter("buff_GoldRecord_CD")?,
        "Voice of the Voyager duration",
    )?;

    let proc = GearProcConfig::GoldRecord {
        atk_bonus: ratio(parameter("buff_GoldRecord_AtkUp")?)?,
        qte_crit_damage_bonus: ratio(parameter("buff_GoldRecord_QteCritDmaUp")?)?,
        ult_crit_damage_per_stack: ratio(parameter("buff_GoldRecord_QCritDmaUp")?)?,
        duration_us,
    };

    let upgrade_pack = item["ElementData"]["UpgradeStarPackID"]
        .as_str()
        .ok_or("Voice of the Voyager upgrade pack missing")?;
    let upgrade_row = format!("{upgrade_pack}_{}", actor.weapon_tier);
    let description = ntesim_data::weapon_refinements::lookup(tables, &upgrade_row)?
        .1
        .as_object()
        .ok_or("Voice of the Voyager native upgrade row missing")?;
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
            "scope": crate::weapons::gold_record::HYPOTHESIS_ID.description(),
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
        .ok_or("Voice of the Voyager parameters missing")?;
    let expected = [
        ("buff_GoldRecord_AtkUp", true),
        ("buff_GoldRecord_QteCritDmaUp", true),
        ("buff_GoldRecord_QCritDmaUp", true),
        ("buff_GoldRecord_CD", false),
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
        return Err("unsupported Voice of the Voyager parameters or unit hint".into());
    }
    Ok(native)
}
