use crate::{equipment, native::Scalar};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub use equipment::TABLES;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActorLoadout {
    pub slot: usize,
    pub build: equipment::Request,
    #[serde(default)]
    pub awakening: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_energy: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_hp_fraction: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetProfile {
    pub id: String,
    pub level: u32,
    pub base_hp: u64,
    pub base_def: u64,
    #[serde(default)]
    pub resistances: BTreeMap<String, f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weakness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_gauge: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub content_id: String,
    pub actors: Vec<ActorLoadout>,
    pub targets: Vec<TargetProfile>,
    pub duration_us: u64,
    #[serde(default)]
    pub active_actor_slot: usize,
}

pub fn compile(tables: &BTreeMap<String, Value>, request: &Request) -> Result<Value, String> {
    if request.schema_version != 1 {
        return Err("schema_version must be 1".into());
    }
    if request.content_id != "ntesim-current" {
        return Err("unsupported content_id: expected ntesim-current".into());
    }
    if request.duration_us == 0 {
        return Err("duration_us must be positive".into());
    }
    if request.actors.is_empty() {
        return Err("team must contain at least one actor".into());
    }
    if request.actors.len() > 4 {
        return Err("team cannot exceed four actors".into());
    }
    if request.active_actor_slot >= request.actors.len() {
        return Err(format!(
            "active_actor_slot {} out of bounds for team size {}",
            request.active_actor_slot,
            request.actors.len()
        ));
    }
    if request.targets.is_empty() {
        return Err("scenario must specify at least one target".into());
    }

    let mut seen_slots = BTreeSet::new();
    let mut seen_characters = BTreeSet::new();
    for a in &request.actors {
        if a.slot >= request.actors.len() {
            return Err(format!("actor slot {} out of bounds", a.slot));
        }
        if !seen_slots.insert(a.slot) {
            return Err(format!("duplicate actor slot: {}", a.slot));
        }
        let cid = &a.build.character_id;
        if !seen_characters.insert(cid.clone()) {
            return Err(format!("duplicate character in team: {cid}"));
        }
        if a.awakening > 6 {
            return Err(format!(
                "awakening level {} exceeds maximum 6 for {cid}",
                a.awakening
            ));
        }
        if let Some(frac) = a.initial_hp_fraction
            && !(0.0..=1.0).contains(&frac)
        {
            return Err(format!(
                "initial_hp_fraction {frac} out of range [0.0, 1.0] for {cid}"
            ));
        }
        if a.initial_energy
            .is_some_and(|energy| !energy.is_finite() || energy < 0.0)
        {
            return Err(format!(
                "initial_energy must be finite and nonnegative for {cid}"
            ));
        }
    }

    let mut seen_targets = BTreeSet::new();
    let mut total_target_hp: u128 = 0;
    for t in &request.targets {
        if t.id.trim().is_empty() {
            return Err("target id cannot be empty".into());
        }
        if !seen_targets.insert(t.id.clone()) {
            return Err(format!("duplicate target id: {}", t.id));
        }
        if t.level == 0 || t.level > 120 {
            return Err(format!("target level {} out of range [1, 120]", t.level));
        }
        if t.base_hp == 0 {
            return Err(format!("target base_hp must be positive for {}", t.id));
        }
        total_target_hp = total_target_hp
            .checked_add(t.base_hp as u128)
            .ok_or("total target HP overflow")?;
        for (elem, &res) in &t.resistances {
            if !res.is_finite() || !(-1.0..=2.0).contains(&res) {
                return Err(format!(
                    "resistance {res} for element {elem} out of reasonable range [-1.0, 2.0]"
                ));
            }
        }
    }

    let mut sorted_actors = request.actors.clone();
    sorted_actors.sort_by_key(|a| a.slot);

    let mut compiled_actors = Vec::new();
    let mut team_elements = BTreeSet::new();
    let mut team_base_atk = Scalar::zero();

    for actor in &sorted_actors {
        let report = equipment::compile(tables, &actor.build)?;

        let elem_name = crate::character::definition(&actor.build.character_id)?
            .element
            .key();
        team_elements.insert(elem_name.to_string());

        let base_atk_obj = &report["base_atk"];
        let base_atk_scalar = Scalar::new(
            base_atk_obj["numerator"].as_str().unwrap().parse().unwrap(),
            base_atk_obj["denominator"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
        );
        team_base_atk = team_base_atk.add(base_atk_scalar)?;

        compiled_actors.push(json!({
            "slot": actor.slot,
            "character_id": report["character_id"],
            "character_level": report["character_level"],
            "character_breakthrough": report["character_breakthrough"],
            "element": elem_name,
            "awakening": actor.awakening,
            "weapon_id": report["weapon_id"],
            "weapon_level": report["weapon_level"],
            "weapon_tier": report["weapon_tier"],
            "weapon_group_binding": report["weapon_group_binding"],
            "weapon_refinement_binding": report["weapon_refinement_binding"],
            "set_id": report["set_id"],
            "initial_energy": actor.initial_energy,
            "initial_hp_fraction": actor.initial_hp_fraction.unwrap_or(1.0),
            "base_atk": report["base_atk"],
            "known_derived_stats": report["known_derived_stats"],
            "known_attributes": report["known_attributes"],
            "unresolved_set_buffs": report["unresolved_set_buffs"],
            "unresolved": report["unresolved"]
        }));
    }

    Ok(json!({
        "schema_version": 1,
        "kind": "static-team-scenario-v1",
        "content_id": request.content_id,
        "duration_us": request.duration_us,
        "active_actor_slot": request.active_actor_slot,
        "active_character_id": compiled_actors[request.active_actor_slot]["character_id"],
        "team_size": sorted_actors.len(),
        "team_elements": team_elements.into_iter().collect::<Vec<_>>(),
        "team_base_atk": team_base_atk.receipt(),
        "total_target_hp": total_target_hp.to_string(),
        "actors": compiled_actors,
        "targets": request.targets,
        "quality": "incomplete",
        "status": "INCOMPLETE",
        "combat_adapter_complete": false,
        "game_dps_computed": false,
        "scope": "Static team loadouts and scenario target definition; combat execution and dynamic buff simulation pending"
    }))
}

#[cfg(test)]
mod tests;
