use crate::build::{compile_weapon_values, finish_weapon_projection, ratio, receipt};
use crate::formula::Ratio;
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
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_koinobori")?
        .1
        .as_object()
        .ok_or("A Time Will Come native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .ok_or("A Time Will Come item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("A Time Will Come group is incompatible with the wearer".into());
    }
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("A Time Will Come exclusive character list missing")?;
    if !exclusive.is_empty()
        || item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"].is_string()
    {
        return Err("A Time Will Come frozen item has unsupported exclusive-character data".into());
    }

    let native = compile_native(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("A Time Will Come parameters missing")?;
    let parameter = |name: &str| -> Result<Ratio, String> {
        let value = parameters
            .iter()
            .find(|p| p["name_id"].as_str() == Some(name))
            .ok_or_else(|| format!("A Time Will Come required parameter {name} missing"))?;
        ratio(receipt(&value["selected"]["value"])?)
    };

    let proc = GearProcConfig::KoinoboriStats {
        atk_bonus: parameter("buff_koinobori_AtkUp")?,
        def_bonus: parameter("buff_koinobori_DefUp")?,
        hp_bonus: parameter("buff_koinobori_HpMaxUp")?,
    };

    let upgrade_pack = item["ElementData"]["UpgradeStarPackID"]
        .as_str()
        .ok_or("A Time Will Come upgrade pack missing")?;
    let upgrade_row = format!("{upgrade_pack}_{}", actor.weapon_tier);
    let description = ntesim_data::weapon_refinements::lookup(tables, &upgrade_row)?
        .1
        .as_object()
        .ok_or("A Time Will Come native upgrade row missing")?;
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
            "scope": crate::weapons::koinobori::HYPOTHESIS_ID.description(),
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
        .ok_or("A Time Will Come parameters missing")?;
    let expected = [
        ("buff_koinobori_AtkUp", true),
        ("buff_koinobori_DefUp", true),
        ("buff_koinobori_HpMaxUp", true),
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
        return Err("unsupported A Time Will Come parameters or unit hint".into());
    }
    Ok(native)
}

pub(crate) fn add_base_hp_ratio(
    derived_hp: ExactFraction,
    native_base_hp: ExactFraction,
    weapon_hp_ratio: ExactFraction,
) -> Result<ExactFraction, String> {
    derived_hp.add(native_base_hp.mul(weapon_hp_ratio)?)
}
