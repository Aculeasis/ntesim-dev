//! Explicit current-content context, independent of external raw packages.
use ntesim_contracts::sha256;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

pub(crate) fn constant_report(
    input: &[u8],
    names: &[&str],
    compile: impl FnOnce(&BTreeMap<String, Value>, &Value) -> Result<Value, String>,
) -> Result<Value, String> {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for name in names {
        if !tables.contains_key(*name) {
            return Err(format!("required current typed table unavailable: {name}"));
        }
    }
    let mut report = compile(tables, &ntesim_data::character::released_inventory())?;
    if !report.is_object() {
        return Err("compiled content report must be an object".into());
    }
    report["request_sha256"] = json!(sha256(input));
    report["engine_version"] = json!(ntesim_contracts::ENGINE_VERSION);
    report["engine_source_sha256"] = json!(ntesim_contracts::ENGINE_SOURCE_SHA256);
    Ok(report)
}

pub(crate) fn variants(path: Option<&Path>) -> Result<String, String> {
    match path {
        Some(path) => fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display())),
        None => Ok(ntesim_data::research_inputs::VARIANTS_CSV.to_owned()),
    }
}

pub(crate) fn conditions(path: Option<&Path>) -> Result<String, String> {
    match path {
        Some(path) => fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display())),
        None => Ok(ntesim_data::research_inputs::CONDITIONS_JSON.to_owned()),
    }
}
