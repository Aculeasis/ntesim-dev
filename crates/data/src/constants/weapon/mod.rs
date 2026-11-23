//! In-memory native weapon constants; no passive execution or release inference.
use crate::native::{Scalar, apply_additive_pack, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const TABLES: [&str; 0] = [];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub content_id: String,
    pub weapon_id: String,
    pub level: u32,
    pub breakthrough: u32,
    pub refinement: u32,
}

/// Caller supplies normalized tables whose evidence integrity was checked by
/// its loader. This function checks interpretation/schema, never filesystem IO.
pub fn compile(tables: &BTreeMap<String, Value>, request: &Request) -> Result<Value, String> {
    if request.schema_version != 1 || request.content_id != "ntesim-current" {
        return Err("unsupported weapon request schema/package".into());
    }
    if !(1..=80).contains(&request.level)
        || !(1..=5).contains(&request.refinement)
        || request.breakthrough > 6
    {
        return Err("weapon level/refinement/breakthrough out of supported range".into());
    }
    let item_table = &crate::weapon_definitions::table(tables)?;
    let native = request
        .weapon_id
        .strip_prefix("weapon:")
        .filter(|s| !s.is_empty())
        .ok_or("expected weapon:native_id")?;
    let (item_id, item) = item_table.lookup(native)?;
    let meta = &item["ElementData"];
    let max_break = meta["MaxBreakthrough"]
        .as_u64()
        .ok_or("weapon maximum breakthrough missing")?;
    let max_star = meta["MaxUpgradeStar"]
        .as_u64()
        .ok_or("weapon maximum refinement missing")?;
    if u64::from(request.breakthrough) > max_break || u64::from(request.refinement) > max_star {
        return Err("weapon request exceeds item progression limits".into());
    }
    let upgrade_table = &crate::weapon_upgrade_levels::table(tables)?;
    let break_table = &crate::weapon_breakthrough::table(tables)?;
    let modify_table = &crate::weapon_modifiers::table(tables)?;
    let star_table = &crate::weapon_refinements::table(tables)?;
    let (level_id, level) = upgrade_table.lookup(&format!(
        "{}_{}",
        text(meta, "UpgradePackId")?,
        request.level
    ))?;
    let (break_id, breaking) = break_table.lookup(&format!(
        "{}_{}",
        text(meta, "BreakthroughPackId")?,
        request.breakthrough
    ))?;
    let max_level = breaking["MaxForkLevel"]
        .as_u64()
        .ok_or("weapon maximum level missing")?;
    if u64::from(request.level) > max_level {
        return Err("weapon level exceeds selected breakthrough limit".into());
    }
    let mut attributes: BTreeMap<String, Scalar> = BTreeMap::new();
    let mut steps = Vec::new();
    let mut evidence = vec![item_table.evidence(item_id)];
    for (kind, parent_table, parent_id, pack) in [
        ("level", upgrade_table, level_id, text(level, "ModifyPack")?),
        (
            "terminal_breakthrough",
            break_table,
            break_id,
            text(breaking, "ModifyPackID")?,
        ),
    ] {
        let (pack_id, modifier) = modify_table.lookup(pack)?;
        apply_additive_pack(&mut attributes, modifier)?;
        let modifiers = &modifier["ModifyData"];
        steps.push(json!({"kind":kind,"parent_row":parent_id,"modifier_row":pack_id,"modifiers":modifiers}));
        evidence.push(parent_table.evidence(parent_id));
        evidence.push(modify_table.evidence(pack_id));
    }
    let (star_id, star) = star_table.lookup(&format!(
        "{}_{}",
        text(meta, "UpgradeStarPackID")?,
        request.refinement
    ))?;
    star.as_object().ok_or("weapon refinement row malformed")?;
    evidence.push(star_table.evidence(star_id));
    let attributes: BTreeMap<_, _> = attributes
        .into_iter()
        .map(|(key, value)| (key, value.receipt()))
        .collect();
    Ok(json!({
        "schema_version":1,"kind":"weapon-constants-v1","content_id":request.content_id,
        "weapon_id":format!("weapon:{item_id}"),"display_name":item["ItemName"]["LocalizedString"],
        "rarity":text(item,"ItemQuality")?,"weapon_type":text(meta,"ApplyGroupType")?,
        "level":request.level,"breakthrough":request.breakthrough,"refinement":request.refinement,
        "attributes":attributes,"attribute_units":"native property units; no percentage conversion or rounding",
        "arithmetic":"exact normalized JSON decimals","steps":steps,
        "selected_refinement_row":star_id,"evidence":evidence,
        "quality":"incomplete","numeric_status":"source_bound_constant_stats",
        "release_status":"unverified","full_build_legality_checked":false,"passive_adapter_complete":false,
        "gaps":["release availability is not established by native tables","passive execution, recipients, triggers and clocks remain unbound","complete character/weapon compatibility and progression legality require the build contract"]
    }))
}

#[cfg(test)]
mod tests;
