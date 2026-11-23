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
    let native = compile_native(tables, schema_version, content_id, actor)?;
    let tier = actor.weapon_tier;
    let damage_reduction = reduction_ratio(tables, tier)?;
    let duration_us = duration(tables, tier)?;
    let description = native_description(&native, tier, damage_reduction)?;
    let proc = GearProcConfig::NestBirdGaze {
        damage_reduction: ratio(damage_reduction)?,
        duration_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": true,
            "native_description": description,
            "curves": {
                "buff_NestBird_down": selected_curve(tables, "buff_NestBird_down", tier, true)?,
                "buff_NestBird_CD1": selected_curve(tables, "buff_NestBird_CD1", tier, false)?,
            },
            "scope": crate::weapons::nest_bird::HYPOTHESIS_ID.description(),
        }),
    )
}

pub(crate) fn compile_native(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &GameActorConfig,
) -> Result<Value, String> {
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_NestBird")?
        .1
        .as_object()
        .ok_or("Tears Beneath the Mask native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    let item_group = item["ElementData"]["ApplyGroupType"]
        .as_str()
        .ok_or("Tears Beneath the Mask item group missing")?;
    let character_group = character.weapon_group.native_key();
    if item_group != character_group {
        return Err("Tears Beneath the Mask group is incompatible with the wearer".into());
    }
    let exclusive = item["ElementData"]["ExclusiveCharacterIDArray"]
        .as_array()
        .ok_or("Tears Beneath the Mask exclusive character list missing")?;
    if !exclusive.is_empty()
        || item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"].is_string()
    {
        return Err(
            "Tears Beneath the Mask frozen item has unsupported exclusive-character data".into(),
        );
    }
    compile_weapon_values(tables, schema_version, content_id, actor)
}

fn fork_buff_curve(tables: &BTreeMap<String, Value>, curve_id: &str) -> Result<Value, String> {
    Ok(ntesim_data::weapon_curves::lookup(tables, curve_id)?
        .1
        .clone())
}

fn positive_curve(curve: &Value, curve_id: &str) -> Result<Value, String> {
    let mut curve = curve.clone();
    let keys = curve["Keys"]
        .as_array_mut()
        .ok_or_else(|| format!("Tears Beneath the Mask buff curve {curve_id} keys missing"))?;
    for key in keys.iter_mut() {
        let text = key["Value"]
            .as_number()
            .ok_or_else(|| {
                format!("Tears Beneath the Mask buff curve {curve_id} value not numeric")
            })?
            .to_string();
        let magnitude = text.strip_prefix('-').ok_or_else(|| {
            format!("Tears Beneath the Mask buff curve {curve_id} must hold negative ratios")
        })?;
        key["Value"] = Value::Number(magnitude.parse().map_err(|_| {
            format!("Tears Beneath the Mask buff curve {curve_id} value unparseable")
        })?);
    }
    Ok(curve)
}

fn selected_curve(
    tables: &BTreeMap<String, Value>,
    curve_id: &str,
    tier: u32,
    negate: bool,
) -> Result<Value, String> {
    let curve = fork_buff_curve(tables, curve_id)?;
    let curve = if negate {
        positive_curve(&curve, curve_id)?
    } else {
        curve
    };
    ntesim_data::native::curve_selection_receipt(&curve, tier)
        .map_err(|error| format!("Tears Beneath the Mask buff curve {curve_id}: {error}"))
}

fn reduction_ratio(tables: &BTreeMap<String, Value>, tier: u32) -> Result<ExactFraction, String> {
    let selected = selected_curve(tables, "buff_NestBird_down", tier, true)?;
    receipt(&selected["value"])
}

fn duration(tables: &BTreeMap<String, Value>, tier: u32) -> Result<u64, String> {
    let selected = selected_curve(tables, "buff_NestBird_CD1", tier, false)?;
    weapon_seconds_to_us(
        receipt(&selected["value"])?,
        "Tears Beneath the Mask duration",
    )
}

fn native_description(
    native: &Value,
    tier: u32,
    damage_reduction: ExactFraction,
) -> Result<Value, String> {
    let description = &native["description"];
    let key = description["Key"]
        .as_str()
        .ok_or("Tears Beneath the Mask description key missing")?;
    if key != format!("buff_fork_NestBird_effect{tier}") {
        return Err("Tears Beneath the Mask description row mismatch".into());
    }
    let percent = damage_reduction
        .numerator
        .checked_mul(100)
        .ok_or("Tears Beneath the Mask percent overflow")?;
    if percent % damage_reduction.denominator != 0 {
        return Err("Tears Beneath the Mask reduction has no integral display percent".into());
    }
    let percent = percent / damage_reduction.denominator;
    let localized = description["LocalizedString"]
        .as_str()
        .ok_or("Tears Beneath the Mask description text missing")?;
    if !localized.contains(&format!("{percent}%")) {
        return Err(format!(
            "Tears Beneath the Mask tier {tier} description percent does not match the frozen reduction curve"
        ));
    }
    Ok(json!({
        "key": description["Key"],
        "localized": description["LocalizedString"],
    }))
}
