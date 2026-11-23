//! Source-bound purchase prerequisites; combat bonuses belong to content.
use crate::native::text;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub mod frozen;

pub fn progression(
    character_id: &str,
    level: u32,
    max_level: Option<u32>,
) -> Result<Value, String> {
    if !(1..=80).contains(&level) || max_level.is_some_and(|cap| level > cap) {
        return Err("character level exceeds declared talent progression limit".into());
    }
    let character = crate::character::definition(character_id)?;
    for (stage, cap) in crate::character::progression::LEVEL_CAPS
        .into_iter()
        .enumerate()
    {
        if max_level.map_or(level <= cap, |declared| declared == cap) {
            return Ok(
                json!({"breakthrough":stage,"max_level":cap,"evidence":[character.evidence()]}),
            );
        }
    }
    Err("unsupported character maximum level for talent progression".into())
}

pub fn compile(
    tables: &BTreeMap<String, Value>,
    character_id: &str,
    ability_id: &str,
    purchased_level: u32,
    breakthrough: u32,
    awakening: u32,
) -> Result<Value, String> {
    let table = crate::character_actions::table(tables)?;
    let native = character_id
        .strip_prefix("character:")
        .ok_or("canonical skill character ID required")?;
    let (id, row) = table.lookup(native)?;
    let actions = row["CharacterAbilityList"]
        .as_array()
        .ok_or("skill purchase actions missing")?;
    let matching = actions
        .iter()
        .enumerate()
        .filter(|(_, action)| {
            action["Key"]
                .as_str()
                .is_some_and(|s| s.eq_ignore_ascii_case(ability_id))
        })
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err("purchased skill must identify exactly one configured combat ability".into());
    }
    let (action_index, action) = matching[0];
    let value = &action["Value"];
    if value["AbilityType"] != "ECharacterAbilityType::Proactive" {
        return Err("unsupported purchased skill ability type".into());
    }
    let costs = value["LevelsCostItems"]
        .as_array()
        .ok_or("skill purchase levels missing")?;
    let mut levels = BTreeSet::new();
    for cost in costs {
        let level = cost["Level"]
            .as_u64()
            .ok_or("skill purchase coordinate missing")?;
        if !levels.insert(level) || level == 0 {
            return Err("unsupported or duplicate skill purchase coordinate".into());
        }
        for field in ["RequireTupoLevel", "RequireAwakenLevel"] {
            cost[field]
                .as_u64()
                .ok_or("skill purchase prerequisite missing")?;
        }
    }
    if levels.iter().copied().ne(1..=costs.len() as u64) {
        return Err("skill purchase coordinates must be contiguous from one".into());
    }
    // Model: initial level is one; a cost coordinate L buys level L + 1.
    if purchased_level == 0 || u64::from(purchased_level) > costs.len() as u64 + 1 {
        return Err("purchased skill level out of supported range".into());
    }
    let mut reached = Vec::new();
    for (index, cost) in costs.iter().enumerate() {
        if cost["Level"].as_u64().unwrap() >= u64::from(purchased_level) {
            continue;
        }
        if cost["RequireTupoLevel"].as_u64().unwrap() > u64::from(breakthrough)
            || cost["RequireAwakenLevel"].as_u64().unwrap() > u64::from(awakening)
        {
            return Err("purchased skill level exceeds native progression prerequisites".into());
        }
        reached.push(json!({"pointer": format!("/rows/{id}/CharacterAbilityList/{action_index}/Value/LevelsCostItems/{index}"), "native_prerequisites": cost}));
    }
    Ok(
        json!({"ability_id":text(action,"Key")?, "purchased_level":purchased_level,
        "breakthrough":breakthrough, "awakening":awakening, "purchase_steps":reached,
        "evidence":table.evidence(id),
        "model_assumption":"Initial purchased level 1; native cost coordinate L buys level L+1."}),
    )
}
