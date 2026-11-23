use crate::report_context::constant_report;
use serde_json::{Value, json};

pub(crate) fn studio_build_catalog() -> Result<Value, String> {
    constant_report(
        b"studio-build-catalog-v1",
        &ntesim_content::build::studio::TABLES,
        ntesim_content::build::studio::catalog,
    )
}

pub(crate) fn character_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_data::character::Request =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(
        input,
        &ntesim_data::character::TABLES,
        |_tables, coverage| {
            let mut report = ntesim_data::character::compile(&request)?;
            let entities = coverage["entities"]
                .as_array()
                .ok_or("coverage entities missing")?;
            let entity = entities
                .iter()
                .find(|entity| entity["id"] == report["character_id"])
                .ok_or("character coverage identity missing")?;
            report["release_status"] = entity["availability"].clone();
            if entity["availability"] == "released" {
                report["release_evidence"] =
                    ntesim_data::character::definition(&request.character_id)?.evidence();
                report["gaps"].as_array_mut().unwrap().remove(0);
            }
            Ok(report)
        },
    )
}

pub(crate) fn weapon_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_data::weapon::Request =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::weapon::TABLES, |tables, _| {
        ntesim_data::weapon::compile(tables, &request)
    })
}

pub(crate) fn damage_values_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_data::damage_values::Request =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::damage_values::TABLES, |tables, _| {
        ntesim_data::damage_values::compile(tables, &request)
    })
}

pub(crate) fn build_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_data::equipment::Request =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(
        input,
        &ntesim_data::equipment::TABLES,
        |tables, coverage| {
            let mut report = ntesim_data::equipment::compile(tables, &request)?;
            let entities = coverage["entities"]
                .as_array()
                .ok_or("coverage entities missing")?;
            let entity = entities
                .iter()
                .find(|entity| entity["id"] == report["character_id"])
                .ok_or("character coverage identity missing")?;
            report["release_status"] = entity["availability"].clone();
            if entity["availability"] == "released" {
                report["release_evidence"] =
                    ntesim_data::character::definition(&request.character_id)?.evidence();
            }
            Ok(report)
        },
    )
}

pub(crate) fn team_report(input: &[u8]) -> Result<Value, String> {
    let request: ntesim_data::team::Request =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(input, &ntesim_data::team::TABLES, |tables, coverage| {
        let mut report = ntesim_data::team::compile(tables, &request)?;
        let entities = coverage["entities"]
            .as_array()
            .ok_or("coverage entities missing")?;
        if let Some(actors) = report["actors"].as_array_mut() {
            for actor in actors {
                let cid = actor["character_id"].as_str().unwrap_or("");
                if let Some(entity) = entities.iter().find(|entity| entity["id"] == cid) {
                    let evidence = ntesim_data::character::definition(cid)?.evidence();
                    actor["release_status"] = entity["availability"].clone();
                    if entity["availability"] == "released" {
                        actor["release_evidence"] = evidence;
                    }
                } else {
                    actor["release_status"] = json!("unverified");
                }
            }
        }
        Ok(report)
    })
}

pub(crate) fn weapon_passive_values_report(input: &[u8]) -> Result<Value, String> {
    if input.len() > 1024 * 1024 {
        return Err("weapon passive request exceeds 1 MiB".into());
    }
    let request: ntesim_data::weapon_passive_values::Request =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    constant_report(
        input,
        &ntesim_data::weapon_passive_values::TABLES,
        |tables, _coverage| ntesim_data::weapon_passive_values::compile(tables, &request),
    )
}
