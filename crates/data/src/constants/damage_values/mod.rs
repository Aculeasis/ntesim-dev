//! Source-bound native damage row coordinates, never action execution or DPS.
use crate::native::{Scalar, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const TABLES: [&str; 0] = [];
const ARRAYS: [&str; 3] = ["AtkRateBaseArray", "DefRateBaseArray", "HPRateBaseArray"];
const SCALARS: [&str; 3] = ["ChargeAdd", "UnbalValue", "FixedCritRate"];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub content_id: String,
    pub character_id: String,
    pub ability_id: String,
    pub effect_ids: Vec<String>,
    /// Explicit zero-based native array coordinate, not a purchased skill level.
    pub array_index: u32,
}

pub fn compile(tables: &BTreeMap<String, Value>, request: &Request) -> Result<Value, String> {
    compile_packet(tables, request, None)
}

/// The content owner supplies a reviewed exact child GAName. The requested
/// initiating action must still be configured on this character, and the row
/// must match the supplied native identity. This does not grant execution.
pub fn compile_owner_packet(
    tables: &BTreeMap<String, Value>,
    request: &Request,
    packet_ability: &str,
) -> Result<Value, String> {
    compile_packet(tables, request, Some(packet_ability))
}

fn compile_packet(
    tables: &BTreeMap<String, Value>,
    request: &Request,
    packet_ability: Option<&str>,
) -> Result<Value, String> {
    if request.schema_version != 1 || request.content_id != "ntesim-current" {
        return Err("unsupported damage-values request schema/package".into());
    }
    if request.effect_ids.is_empty() || request.effect_ids.len() > 64 {
        return Err("select 1..64 explicit damage effect IDs".into());
    }
    let damage = crate::native_damage::table(tables)?;
    let character = crate::character::definition(&request.character_id)?;
    let character_id = character.native_id;
    let configs = crate::character_actions::table(tables)?;
    let (config_id, config) = configs.lookup(character_id)?;
    let actions = config["CharacterAbilityList"]
        .as_array()
        .ok_or("character actions missing")?;
    let matching = actions
        .iter()
        .enumerate()
        .filter(|(_, action)| {
            action["Key"]
                .as_str()
                .is_some_and(|key| key.eq_ignore_ascii_case(&request.ability_id))
        })
        .collect::<Vec<_>>();
    let (action_index, ability_id) = if matching.len() == 1 {
        let (index, action) = matching[0];
        let id = text(action, "Key")?;
        (index, id)
    } else if let Some(parent) =
        crate::character_actions::resolve_evolved_parent(character_id, &request.ability_id)
    {
        let parent_matching = actions
            .iter()
            .enumerate()
            .filter(|(_, action)| {
                action["Key"]
                    .as_str()
                    .is_some_and(|key| key.eq_ignore_ascii_case(parent))
            })
            .collect::<Vec<_>>();
        if parent_matching.len() != 1 {
            return Err("ability must identify exactly one configured character action".into());
        }
        let (index, _action) = parent_matching[0];
        (index, request.ability_id.as_str())
    } else {
        return Err("ability must identify exactly one configured character action".into());
    };
    let mut evidence = vec![character.evidence(), configs.evidence(config_id)];
    let mut selected = BTreeSet::new();
    let mut effects = Vec::new();
    for requested in &request.effect_ids {
        let (effect_id, row) = damage.lookup(requested)?;
        if !selected.insert(effect_id) {
            return Err("duplicate canonical damage effect selection".into());
        }
        if !text(row, "GAName")?.eq_ignore_ascii_case(packet_ability.unwrap_or(ability_id)) {
            return Err(format!(
                "damage effect belongs to another ability: {effect_id}"
            ));
        }
        let mut arrays = BTreeMap::new();
        for field in ARRAYS {
            let values = row[field]
                .as_array()
                .ok_or_else(|| format!("missing damage array: {field}"))?;
            let exact = values
                .iter()
                .map(Scalar::parse)
                .collect::<Result<Vec<_>, _>>()?;
            let value = if exact.is_empty() {
                Value::Null
            } else {
                exact
                    .get(request.array_index as usize)
                    .ok_or_else(|| format!("array_index out of range for {effect_id}:{field}"))?
                    .receipt()
            };
            arrays.insert(
                field,
                json!({"length":exact.len(),"selected_value":value,
                "state":if exact.is_empty(){"declared_empty"}else{"selected_native_coordinate"}}),
            );
        }
        let mut scalars = BTreeMap::new();
        for field in SCALARS {
            scalars.insert(field, Scalar::parse(&row[field])?.receipt());
        }
        effects.push(json!({
            "effect_id":effect_id,"array_index":request.array_index,"arrays":arrays,"scalars":scalars,
            "damage_type":text(row,"DamageTypeEX")?,"source_category":text(row,"DamageSourceCategory")?,
            "reached_by_action":null,"hit_count":null,"runtime_resource_recipient":null,
            "evidence":[damage.evidence(effect_id)]
        }));
        evidence.push(damage.evidence(effect_id));
    }
    Ok(json!({
        "schema_version":1,"kind":"native-damage-values-v1","content_id":request.content_id,
        "character_id":format!("character:{character_id}"),"ability_id":ability_id,
        "array_index":request.array_index,"array_index_semantics":"explicit zero-based native coordinate; purchased/effective skill-level mapping unbound",
        "configured_action_pointer":format!("/rows/{config_id}/CharacterAbilityList/{action_index}"),
        "effects":effects,"evidence":evidence,
        "numeric_status":"source_bound_native_values","arithmetic":"exact normalized JSON decimals",
        "units":"native table fields; formula/percentage/resource conversion and application remain unbound",
        "quality":"incomplete","status":"INCOMPLETE","game_dps_computed":false,
        "combat_adapter_complete":false,"release_status":"unverified",
        "gaps":["selected IDs are explicit candidates, not proof of dispatch or reachability",
            "retained GAName membership does not prove every owner branch is executable",
            "native array coordinate to purchased/effective skill level remains unbound",
            "delivery/hit count, geometry, capture/crit, rounding, resource recipients/caps and clocks remain unbound",
            "empty scaling arrays do not imply zero damage",
            "release, hotfix and decoder/runtime applicability remain separate evidence requirements"]
    }))
}

#[cfg(test)]
mod tests;
