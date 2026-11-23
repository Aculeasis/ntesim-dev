use super::{GameActionCommand, GameActorConfig, GameScenarioRequest};
use crate::build::{ActionProjection, ActionProjectionPolicy, project_actions_with_levels};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const MAX_EFFECTIVE_QTE_LEVEL: u32 = 13;

pub(crate) fn qte_ability(character_id: &str) -> Result<&'static str, String> {
    let native = character_id
        .strip_prefix("character:")
        .ok_or("canonical Support character ID required")?;
    ntesim_data::character_actions::CHARACTERS
        .iter()
        .find(|owner| owner.native_id == native)
        .map(|owner| owner.abilities[3])
        .ok_or_else(|| format!("{character_id} has no native Support slot"))
}

pub(crate) fn qte_coordinate(actor: &GameActorConfig) -> Result<u32, String> {
    let purchased = actor.purchased_qte_level.ok_or_else(|| {
        format!(
            "{} requires an explicit purchased QTE level",
            actor.character_id
        )
    })?;
    let ability = qte_ability(&actor.character_id)?;
    let tables = ntesim_data::skill_levels::frozen::tables();
    let progression = ntesim_data::skill_levels::progression(
        &actor.character_id,
        actor.character_level,
        actor.character_max_level,
    )?;
    let breakthrough = progression["breakthrough"]
        .as_u64()
        .ok_or("Support breakthrough missing")? as u32;
    ntesim_data::skill_levels::compile(
        tables,
        &actor.character_id,
        ability,
        purchased,
        breakthrough,
        actor.awakening.count(),
    )
    .map_err(|error| format!("invalid purchased QTE level: {error}"))?;
    let (bonus, _) = crate::characters::resonance::bonus(
        tables,
        &actor.character_id,
        ability,
        &actor.awakening,
    )?;
    Ok((purchased + bonus).min(MAX_EFFECTIVE_QTE_LEVEL) - 1)
}

fn receipts_match_excluding_indices(
    existing: &serde_json::Map<String, Value>,
    candidate: &serde_json::Map<String, Value>,
) -> bool {
    let existing_len = existing.len() - usize::from(existing.contains_key("command_indices"));
    existing_len == candidate.len() && candidate.iter().all(|(k, v)| existing.get(k) == Some(v))
}

pub(crate) fn push_receipt(
    receipts: &mut Vec<Value>,
    mut receipt: Value,
    command_index: usize,
) -> Result<(), String> {
    let Some(map) = receipt.as_object_mut() else {
        return Err("talent projection receipt must be a JSON object".into());
    };
    map.remove("command_index");
    map.remove("command_indices");
    for existing in receipts.iter_mut() {
        let Some(existing_map) = existing.as_object_mut() else {
            continue;
        };
        if receipts_match_excluding_indices(existing_map, map) {
            let indices = existing_map
                .entry("command_indices")
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .ok_or("talent projection receipt command_indices must be an array")?;
            indices.push(json!(command_index));
            return Ok(());
        }
    }
    map.insert("command_indices".into(), json!([command_index]));
    receipts.push(receipt);
    Ok(())
}

pub(crate) fn project(
    request: &GameScenarioRequest,
) -> Result<Option<(GameScenarioRequest, Vec<Value>)>, String> {
    if request.actors.iter().all(|a| a.talent_levels.is_none()) {
        return Ok(None);
    }
    let tables = ntesim_data::skill_levels::frozen::tables();
    let mut projected = request.clone();
    let mut raw_receipts = Vec::new();
    for (actor_index, actor) in request.actors.iter().enumerate() {
        let Some(talents) = actor.talent_levels else {
            continue;
        };
        let native = actor
            .character_id
            .strip_prefix("character:")
            .ok_or("canonical talent character ID required")?;
        let progression = ntesim_data::skill_levels::progression(
            &actor.character_id,
            actor.character_level,
            actor.character_max_level,
        )?;
        let breakthrough = progression["breakthrough"]
            .as_u64()
            .ok_or("talent breakthrough missing")? as u32;
        let (_, configs) = ntesim_data::character_actions::rows(tables)?;
        let abilities = configs
            .get(native)
            .and_then(|row| row.get("CharacterAbilityList"))
            .and_then(Value::as_array)
            .ok_or("talent abilities missing")?;
        let mut levels = BTreeMap::<String, Value>::new();
        for (primary_slot, purchased) in [
            crate::simulation::metadata::PrimaryActionSlot::Normal,
            crate::simulation::metadata::PrimaryActionSlot::Skill,
            crate::simulation::metadata::PrimaryActionSlot::Ultimate,
        ]
        .into_iter()
        .zip(talents)
        {
            let mut matching = Vec::new();
            for id in abilities.iter().filter_map(|entry| entry["Key"].as_str()) {
                let native = ntesim_data::native_action_metadata::resolve_ability(
                    &request.content_id,
                    tables,
                    &actor.character_id,
                    id,
                )?;
                let (_, resolved_slot, _) =
                    crate::simulation::metadata::native_input_role(&native)?;
                if resolved_slot == Some(primary_slot) {
                    matching.push(id);
                }
            }
            if matching.len() != 1 {
                return Err(format!(
                    "talent primary slot {:?} requires exactly one configured ability",
                    primary_slot
                ));
            }
            let ability = matching[0];
            let purchase = ntesim_data::skill_levels::compile(
                tables,
                &actor.character_id,
                ability,
                purchased,
                breakthrough,
                actor.awakening.count(),
            )?;
            let (bonus, resonance) = crate::characters::resonance::bonus(
                tables,
                &actor.character_id,
                ability,
                &actor.awakening,
            )?;
            let effective = purchased
                .checked_add(bonus)
                .ok_or("effective talent level overflow")?;
            levels.insert(ability.into(), json!({"purchase":purchase,"progression":progression,"resonance":resonance,"effective_level":effective,"array_index":effective-1}));
        }
        // Evolved replacements share their ordinary parent's invested level;
        // the alias still names its own native damage rows.
        for command in &request.actions {
            let GameActionCommand::InvokeAbility {
                actor_index: owner,
                ability_id,
                ..
            } = command
            else {
                continue;
            };
            if *owner != actor_index {
                continue;
            }
            let Some(parent) =
                ntesim_data::character_actions::resolve_evolved_parent(native, ability_id)
            else {
                continue;
            };
            if let Some(level) = levels.get(parent).cloned() {
                levels.entry(ability_id.clone()).or_insert(level);
            }
        }
        let indices: Vec<_> = request
            .actions
            .iter()
            .enumerate()
            .filter_map(|(index, command)| match command {
                GameActionCommand::InvokeAbility {
                    actor_index: owner,
                    ability_id,
                    ..
                } if *owner == actor_index
                    && levels.keys().any(|id| id.eq_ignore_ascii_case(ability_id)) =>
                {
                    Some(index)
                }
                _ => None,
            })
            .collect();
        if indices.is_empty() {
            continue;
        }
        let mut selected = request.clone();
        selected.actions = indices
            .iter()
            .map(|i| request.actions[*i].clone())
            .collect();
        let action_receipts = project_actions_with_levels(tables, &mut selected, &ActionProjection {
            policy: ActionProjectionPolicy::NativeSkillLevelV1,
            hypothesis: "Native ATK-only damage coefficient uses effective talent level minus one; existing explicit hit delivery and timing remain the partial combat model.".into(),
        }, &BTreeMap::from([(actor.slot, levels)]))?;
        for ((index, command), receipt) in indices
            .into_iter()
            .zip(selected.actions)
            .zip(action_receipts)
        {
            projected.actions[index] = command;
            raw_receipts.push((index, receipt));
        }
    }
    // Projection visits owners to resolve their invested levels, but the report
    // follows authored command order, including sequential source injections.
    raw_receipts.sort_by_key(|(index, _)| *index);
    let mut receipts = Vec::new();
    for (index, receipt) in raw_receipts {
        push_receipt(&mut receipts, receipt, index)?;
    }
    for hypothesis in [
        crate::HypothesisId::SkillLevelsCoordinatesV1,
        crate::HypothesisId::SkillLevelsResonance3V1,
    ] {
        let (id, description) = (hypothesis.as_str(), hypothesis.description());
        if projected
            .model_assumptions
            .get(id)
            .is_some_and(|old| old != description)
        {
            return Err(format!("conflicting compiled model assumption: {id}"));
        }
        projected
            .model_assumptions
            .insert(id.into(), description.into());
    }
    Ok(Some((projected, receipts)))
}
