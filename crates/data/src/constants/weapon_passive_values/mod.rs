//! Serialized refinement parameter values, without passive execution/unit claims.
use crate::native::{Table, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const TABLES: [&str; 0] = [];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub content_id: String,
    pub weapon_id: String,
    pub refinement: u32,
}

fn select_curve(table: &Table, name: &str, time: u32) -> Result<Value, String> {
    let (id, row) = table.lookup(name)?;
    let selected = crate::native::curve_selection_receipt(row, time)?;
    let index = selected["selected_key_index"]
        .as_u64()
        .ok_or("curve key index missing")?;
    let value = crate::native::from_receipt(&selected["value"])?;
    let mut evidence = table.evidence(id);
    evidence["pointer"] = json!(format!(
        "{}/Keys/{index}/Value",
        evidence["pointer"]
            .as_str()
            .ok_or("curve evidence missing")?
    ));
    Ok(
        json!({"curve_id":id, "coordinate":time, "selection":selected["selection"], "selected_key_index":index,
        "selected_key":selected["selected_key"], "curve":row, "value":value.receipt(), "display_value":value.to_f64(),
        "evidence":evidence, "native_units_converted":false, "runtime_application_verified":false}),
    )
}

pub fn compile(tables: &BTreeMap<String, Value>, request: &Request) -> Result<Value, String> {
    if request.schema_version != 1
        || request.content_id != "ntesim-current"
        || !(1..=5).contains(&request.refinement)
    {
        return Err("unsupported passive value schema/package/refinement".into());
    }
    let native = request
        .weapon_id
        .strip_prefix("weapon:")
        .filter(|id| !id.is_empty())
        .ok_or("expected weapon:native_id")?;
    let items = &crate::weapon_definitions::table(tables)?;
    let curves = &crate::weapon_curves::table(tables)?;
    let (weapon_id, item) = items.lookup(native)?;
    let meta = &item["ElementData"];
    if u64::from(request.refinement)
        > meta["MaxUpgradeStar"]
            .as_u64()
            .ok_or("maximum refinement missing")?
    {
        return Err("requested passive refinement exceeds weapon limit".into());
    }
    let stars = &crate::weapon_refinements::table(tables)?;
    let (star_id, star) = stars.lookup(&format!(
        "{}_{}",
        text(meta, "UpgradeStarPackID")?,
        request.refinement
    ))?;
    let list = star["DataList"]
        .as_array()
        .ok_or("passive parameter DataList missing")?;
    if list.len() > 32 {
        return Err("passive parameter list exceeds 32".into());
    }
    let mut names = BTreeSet::new();
    let mut parameters = Vec::new();
    for (index, parameter) in list.iter().enumerate() {
        let name = text(parameter, "NameID")?;
        if !name.is_ascii() || !names.insert(name.to_ascii_lowercase()) {
            return Err("passive parameter names must be distinct ASCII native IDs".into());
        }
        let percent_hint = parameter["bIsPercent"]
            .as_bool()
            .ok_or("passive percent hint missing")?;
        let mut reference = stars.evidence(star_id);
        reference["pointer"] = json!(format!(
            "{}/DataList/{index}",
            reference["pointer"]
                .as_str()
                .ok_or("parameter evidence missing")?
        ));
        parameters.push(json!({"index":index, "name_id":name, "b_is_percent":percent_hint,
            "reference_evidence":reference, "selected":select_curve(curves, name, request.refinement)?}));
    }
    Ok(
        json!({"kind":"weapon-passive-values-v1", "schema_version":1, "content_id":request.content_id,
        "weapon_id":format!("weapon:{weapon_id}"), "refinement":request.refinement,
        "selected_refinement_row":star_id,
        "parameter_count":parameters.len(), "parameters":parameters, "listed_parameters_resolved":true,
        "description":star["Description"], "evidence":[items.evidence(weapon_id), stars.evidence(star_id)],
        "status":"INCOMPLETE", "quality":"incomplete", "game_dps_computed":false,
        "passive_adapter_complete":false, "release_status":"unverified",
        "scope":"Exact serialized parameter keys and interior native Constant selection only; percent flags are retained hints without unit conversion. Defaults/extrapolation are not inferred. Runtime curve consumer, trigger, recipient, units, clocks and application remain unresolved."}),
    )
}

#[cfg(test)]
mod tests;
