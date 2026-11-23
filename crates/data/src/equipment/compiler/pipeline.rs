use super::*;
use crate::native::from_receipt;

pub fn compile(tables: &BTreeMap<String, Value>, request: &Request) -> Result<Value, String> {
    if request.schema_version != 1 || request.content_id != "ntesim-current" {
        return Err("unsupported static build request schema/package".into());
    }
    let char_level = request.character_level.unwrap_or(80);
    let char_stage = request.character_breakthrough.unwrap_or(6);
    let weapon_level = request.weapon_level.unwrap_or(80);
    let weapon_stage = request.weapon_breakthrough.unwrap_or(6);
    let character = character::compile(&character::Request {
        schema_version: request.schema_version,
        content_id: request.content_id.clone(),
        character_id: request.character_id.clone(),
        level: char_level,
        breakthrough: char_stage,
    })?;
    let weapon = weapon::compile(
        tables,
        &weapon::Request {
            schema_version: request.schema_version,
            content_id: request.content_id.clone(),
            weapon_id: request.weapon_id.clone(),
            level: weapon_level,
            breakthrough: weapon_stage,
            refinement: request.weapon_tier,
        },
    )?;
    let group = weapon_group_binding(tables, &request.character_id, &request.weapon_id)?;
    let set = set_stats(tables, &request.set_id, request.set_count)?;
    let mut attributes = BTreeMap::new();
    let mut evidence = Vec::new();
    let mut contributions = Vec::new();
    for (category, source) in [
        ("character_base", &character),
        ("weapon_constants", &weapon),
        ("set", &set),
    ] {
        for (name, value) in source["attributes"]
            .as_object()
            .ok_or("source attributes missing")?
        {
            accumulate_scalar(&mut attributes, name, from_receipt(value)?)?;
        }
        evidence.extend(
            source["evidence"]
                .as_array()
                .ok_or("source evidence missing")?
                .clone(),
        );
        if category != "set" {
            contributions.push(json!({"category":category,"attributes":source["attributes"],"evidence":source["evidence"]}));
        }
    }
    contributions.extend(
        set["contributions"]
            .as_array()
            .ok_or("set contributions missing")?
            .clone(),
    );
    for (name, value) in &request.additive_attributes {
        if !attributes.contains_key(name) && !matches!(name.as_str(), "HealUp" | "ShieldEfficiency")
        {
            return Err(format!("unsupported authored stat {name}"));
        }
        if matches!(
            name.as_str(),
            "AtkBase" | "HPMaxBase" | "DefBase" | "ChargeMax"
        ) {
            return Err(format!("base stat cannot be a console addition: {name}"));
        }
        accumulate_scalar(&mut attributes, name, from_receipt(value)?)?;
    }
    contributions
        .push(json!({"category":"authored_stats","attributes":request.additive_attributes}));
    let star = crate::weapon_refinements::table(tables)?;
    let row = weapon["selected_refinement_row"]
        .as_str()
        .ok_or("refinement row missing")?;
    let refinement_evidence = star.evidence(row);
    evidence.push(refinement_evidence.clone());
    let base_atk = *attributes.get("AtkBase").ok_or("base ATK missing")?;
    let derived = derive_stats(&attributes, base_atk)?;
    Ok(
        json!({"kind":"static-build-constants-v1","schema_version":1,
        "character_id":character["character_id"],"character_level":char_level,"character_breakthrough":char_stage,
        "weapon_id":weapon["weapon_id"],"weapon_level":weapon_level,"weapon_breakthrough":weapon_stage,"weapon_tier":request.weapon_tier,
        "weapon_group_binding":group,"set_id":set["set_id"],"set_count":request.set_count,
        "base_atk":base_atk.receipt(),"character_base_atk":character["attributes"]["AtkBase"],"weapon_base_atk":weapon["attributes"]["AtkBase"],
        "known_attributes":attributes.iter().map(|(k,v)|(k.clone(),json!({"receipt":v.receipt(),"value":v.to_f64()}))).collect::<BTreeMap<_,_>>(),
        "known_derived_stats":derived,"contributions":contributions,"unresolved_set_buffs":set["buffs"],
        "weapon_refinement_binding":{"row":weapon["selected_refinement_row"],"refinement":request.weapon_tier,"evidence":[refinement_evidence],"passive_execution_validated":false},
        "status":"INCOMPLETE","quality":"incomplete","combat_adapter_complete":false,"game_dps_computed":false,
        "unresolved":{"weapon_passive_execution":true,"set_buff_execution":set["buffs"],"character_passive_and_awakening_execution":true},
        "scope":"Character and weapon constants plus authored console stats and declared 2/4-piece set effects; console geometry and allocation are not modeled",
        "evidence":evidence}),
    )
}
