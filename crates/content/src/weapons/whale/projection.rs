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
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_Whale")?
        .1
        .as_object()
        .ok_or("Song of the Whale native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .ok_or("Song of the Whale item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Song of the Whale group is incompatible with the wearer".into());
    }
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Song of the Whale exclusive character list missing")?;
    if !exclusive.is_empty()
        || item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"].is_string()
    {
        return Err(
            "Song of the Whale frozen item has unsupported exclusive-character data".into(),
        );
    }

    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("Song of the Whale parameters missing")?;
    let expected = [
        ("buff_Whale_AtkUp", true),
        ("buff_Whale_Up", true),
        ("buff_Whale_Hp", true),
        ("buff_Whale_CD", false),
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
        return Err("unsupported Song of the Whale parameters or unit hint".into());
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
            .ok_or_else(|| format!("Song of the Whale required parameter {name} missing"))?;
        receipt(selected)
    };
    let proc = GearProcConfig::WhaleAtk {
        atk_bonus: ratio(parameter("buff_Whale_AtkUp")?)?,
        broken_damage_bonus: ratio(parameter("buff_Whale_Up")?)?,
        kill_heal: ratio(parameter("buff_Whale_Hp")?)?,
        kill_heal_cooldown_us: weapon_seconds_to_us(
            parameter("buff_Whale_CD")?,
            "Song of the Whale kill-heal cooldown",
        )?,
    };
    let upgrade_pack = item["ElementData"]["UpgradeStarPackID"]
        .as_str()
        .ok_or("Song of the Whale upgrade pack missing")?;
    let upgrade_row = format!("{upgrade_pack}_{}", actor.weapon_tier);
    let description = ntesim_data::weapon_refinements::lookup(tables, &upgrade_row)?
        .1
        .as_object()
        .ok_or("Song of the Whale native upgrade row missing")?;
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
            "scope": crate::weapons::whale::HYPOTHESIS_ID.description()
        }),
    )
}
