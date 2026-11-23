use crate::report_context::constant_report;
use ntesim_contracts::sha256;
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn published_variants_report(csv_content: &str) -> Result<Value, String> {
    constant_report(csv_content.as_bytes(), &[], |_tables, _coverage| {
        let limits = ntesim_core::execution::ExecutionLimits::default();
        let batch_report =
            ntesim_research::published_teams::run_published_batch(csv_content, &limits)?;
        serde_json::to_value(batch_report).map_err(|e| e.to_string())
    })
}

pub(crate) fn published_built_variants_report(
    csv_path: Option<&Path>,
    input: &[u8],
) -> Result<Value, String> {
    if input.len() > 4 * 1024 * 1024 {
        return Err("built variants request exceeds 4 MiB".into());
    }
    let request: ntesim_research::built_variants::Request =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    let default_csv = crate::report_context::variants(csv_path)?;

    constant_report(
        input,
        &ntesim_research::built_variants::TABLES,
        |tables, coverage| {
            let limits = ntesim_core::execution::ExecutionLimits::default();
            ntesim_research::built_variants::run(tables, coverage, &default_csv, &request, &limits)
        },
    )
}

pub(crate) fn published_profile_audit_report(input: &[u8]) -> Result<Value, String> {
    let csv = std::str::from_utf8(input).map_err(|e| e.to_string())?;
    constant_report(
        input,
        &ntesim_research::profile_audit::TABLES,
        |tables, coverage| ntesim_research::profile_audit::audit(tables, coverage, csv),
    )
}

pub(crate) fn published_gear_report(input: &[u8]) -> Result<Value, String> {
    if input.len() > 1024 * 1024 {
        return Err("recommended gear CSV exceeds 1 MiB".into());
    }
    let csv = std::str::from_utf8(input).map_err(|e| e.to_string())?;
    let mut report = constant_report(
        input,
        &ntesim_data::recommended_gear::TABLES,
        |tables, inventory| ntesim_data::recommended_gear::bind(tables, inventory, csv),
    )?;
    report["recommendation_source"] =
        json!({"path":"presets/research/recommended-gear.csv", "sha256":sha256(input)});
    Ok(report)
}

pub(crate) fn published_search_report(csv_content: &str) -> Result<Value, String> {
    constant_report(csv_content.as_bytes(), &[], |_tables, _coverage| {
        let limits = ntesim_core::execution::ExecutionLimits::default();
        let search_report =
            ntesim_research::build_search::run_published_search(csv_content, &limits)?;
        serde_json::to_value(search_report).map_err(|e| e.to_string())
    })
}
