//! Shared exact static attribute projection for declared research studio builds.
//! It does not infer native module legality or execute content-owned passives.
use crate::{
    character,
    native::{Scalar, accumulate_scalar, from_receipt, text},
    skill_levels, weapon,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const TABLES: [&str; 0] = [];

pub fn frozen_tables() -> &'static BTreeMap<String, Value> {
    crate::skill_levels::frozen::tables()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub character_id: String,
    pub level: u32,
    pub max_level: Option<u32>,
    pub weapon_id: Option<String>,
    pub weapon_level: Option<u32>,
    pub weapon_max_level: Option<u32>,
    pub refinement: u32,
    pub set_id: Option<String>,
    pub set_count: u32,
    pub base_overrides: BTreeMap<String, Value>,
    pub additive_attributes: BTreeMap<String, Value>,
}

fn merge(attributes: &mut BTreeMap<String, Scalar>, report: &Value) -> Result<(), String> {
    for (name, value) in report["attributes"]
        .as_object()
        .ok_or("compiled native attributes missing")?
    {
        accumulate_scalar(attributes, name, from_receipt(value)?)?;
    }
    Ok(())
}

fn weapon_stage(
    tables: &BTreeMap<String, Value>,
    id: &str,
    level: u32,
    max_level: Option<u32>,
) -> Result<u32, String> {
    let items = crate::weapon_definitions::table(tables)?;
    let breaking = crate::weapon_breakthrough::table(tables)?;
    let (_, item) = items.lookup(
        id.strip_prefix("weapon:")
            .ok_or("canonical weapon ID required")?,
    )?;
    let pack = text(&item["ElementData"], "BreakthroughPackId")?;
    let maximum = item["ElementData"]["MaxBreakthrough"]
        .as_u64()
        .ok_or("weapon breakthrough maximum missing")?;
    for stage in 0..=maximum {
        let (_, row) = breaking.lookup(&format!("{pack}_{stage}"))?;
        let cap = row["MaxForkLevel"].as_u64().ok_or("weapon cap missing")?;
        if max_level.map_or(u64::from(level) <= cap, |declared| {
            u64::from(declared) == cap
        }) {
            return u32::try_from(stage).map_err(|_| "weapon stage overflow".into());
        }
    }
    Err("unsupported weapon level/max-level progression".into())
}

pub fn compile(tables: &BTreeMap<String, Value>, request: &Request) -> Result<Value, String> {
    if !(1..=5).contains(&request.refinement) {
        return Err("refinement must be in 1..=5".into());
    }
    let progression =
        skill_levels::progression(&request.character_id, request.level, request.max_level)?;
    let character = character::compile(&character::Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        character_id: request.character_id.clone(),
        level: request.level,
        breakthrough: progression["breakthrough"]
            .as_u64()
            .ok_or("character stage missing")? as u32,
    })?;
    let mut attributes = BTreeMap::new();
    let mut evidence = character["evidence"]
        .as_array()
        .ok_or("character evidence missing")?
        .clone();
    let mut assumptions = BTreeMap::new();
    for step in character["steps"]
        .as_array()
        .ok_or("character steps missing")?
    {
        if let Some(description) = step["model_assumption"].as_str() {
            assumptions.insert("build.unascended_v1", description);
        }
    }
    merge(&mut attributes, &character)?;
    let mut stat_sources = BTreeMap::from([("character", character["attributes"].clone())]);
    let mut weapon_id = Value::Null;
    if let Some(id) = &request.weapon_id {
        crate::equipment::weapon_group_binding(tables, &request.character_id, id)?;
        let level = request.weapon_level.unwrap_or(80);
        let weapon = weapon::compile(
            tables,
            &weapon::Request {
                schema_version: 1,
                content_id: "ntesim-current".into(),
                weapon_id: id.clone(),
                level,
                breakthrough: weapon_stage(tables, id, level, request.weapon_max_level)?,
                refinement: request.refinement,
            },
        )?;
        merge(&mut attributes, &weapon)?;
        stat_sources.insert("weapon", weapon["attributes"].clone());
        evidence.extend(
            weapon["evidence"]
                .as_array()
                .ok_or("weapon evidence missing")?
                .clone(),
        );
        weapon_id = weapon["weapon_id"].clone();
    } else if request.weapon_level.is_some() || request.weapon_max_level.is_some() {
        return Err("weapon progression requires an equipped weapon".into());
    }
    let mut set_id = Value::Null;
    if let Some(id) = &request.set_id {
        let set = crate::equipment::set_stats(tables, id, request.set_count)?;
        set_id = set["set_id"].clone();
        stat_sources.insert("set", set["attributes"].clone());
        for (name, value) in set["attributes"]
            .as_object()
            .ok_or("set attributes missing")?
        {
            accumulate_scalar(&mut attributes, name, from_receipt(value)?)?;
        }
        evidence.extend(
            set["evidence"]
                .as_array()
                .ok_or("set evidence missing")?
                .clone(),
        );
    } else if request.set_count != 0 {
        return Err("set count requires an equipped set".into());
    }
    for (name, value) in &request.base_overrides {
        if !["AtkBase", "HPMaxBase", "DefBase"].contains(&name.as_str()) {
            return Err(format!("unsupported native base override {name}"));
        }
        attributes.insert(name.clone(), from_receipt(value)?);
    }
    for (name, value) in &request.additive_attributes {
        accumulate_scalar(&mut attributes, name, from_receipt(value)?)?;
    }
    stat_sources.insert("base_override", json!(request.base_overrides));
    stat_sources.insert("additive", json!(request.additive_attributes));
    let base_atk = *attributes.get("AtkBase").ok_or("base ATK missing")?;
    let derived = crate::equipment::derive_stats(&attributes, base_atk)?;
    Ok(
        json!({"character_id":character["character_id"],"weapon_id":weapon_id,"set_id":set_id,
        "element":character["element"],"attributes":attributes.into_iter().map(|(name,value)|(name,value.receipt())).collect::<BTreeMap<_,_>>(),
        "derived":derived,"stat_sources":stat_sources,"model_assumptions":assumptions,"evidence":evidence,
        "quality":"incomplete","full_build_legality_checked":false,"combat_adapter_complete":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: &str) -> Request {
        Request {
            character_id: id.into(),
            level: 80,
            max_level: Some(80),
            weapon_id: None,
            weapon_level: None,
            weapon_max_level: None,
            refinement: 1,
            set_id: None,
            set_count: 0,
            base_overrides: BTreeMap::new(),
            additive_attributes: BTreeMap::new(),
        }
    }

    #[test]
    fn all_released_character_stats_agree_with_native_compiler_without_browser_math() {
        let released = character::released_ids();
        let mut count = 0;
        for id in released {
            let projected = compile(frozen_tables(), &request(&id)).unwrap();
            let native = character::compile(&character::Request {
                schema_version: 1,
                content_id: "ntesim-current".into(),
                character_id: id,
                level: 80,
                breakthrough: 6,
            })
            .unwrap();
            assert_eq!(projected["attributes"], native["attributes"]);
            count += 1;
        }
        assert_eq!(count, 25);
    }

    #[test]
    fn caller_overrides_and_additive_layers_use_shared_exact_projection() {
        let tables = frozen_tables();
        let mut request = request("character:1003");
        request.base_overrides = BTreeMap::from([
            (
                "AtkBase".into(),
                json!({"numerator":"100","denominator":"1"}),
            ),
            (
                "HPMaxBase".into(),
                json!({"numerator":"200","denominator":"1"}),
            ),
        ]);
        request.additive_attributes = BTreeMap::from([
            ("AtkUp".into(), json!({"numerator":"1","denominator":"3"})),
            ("AtkAdd".into(), json!({"numerator":"1","denominator":"8"})),
            ("HPMaxUp".into(), json!({"numerator":"1","denominator":"4"})),
        ]);
        let report = compile(tables, &request).unwrap();
        assert_eq!(report["character_id"], "character:1003");
        // Independent hand calculation: 100*(1+1/3)+1/8 = 3203/24.
        assert_eq!(
            report["derived"]["atk_receipt"],
            json!({"numerator":"3203","denominator":"24"})
        );
        assert_eq!(
            report["derived"]["hp_receipt"],
            json!({"numerator":"250","denominator":"1"})
        );
        request
            .additive_attributes
            .insert("AtkUp".into(), json!({"numerator":"1","denominator":"0"}));
        assert!(
            compile(tables, &request)
                .unwrap_err()
                .contains("denominator")
        );
    }

    #[test]
    fn static_projection_rejects_impossible_native_set_piece_count() {
        let mut request = request("character:1003");
        request.set_id = Some("set:Suit11".into());
        request.set_count = 5;
        assert!(
            compile(frozen_tables(), &request)
                .unwrap_err()
                .contains("set count must be in 0..=4")
        );
    }
}
