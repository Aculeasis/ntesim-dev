use crate::report_context::constant_report;
use ntesim_contracts::sha256;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

pub(crate) fn composition_enumeration_report(csv_path: Option<&Path>) -> Result<Value, String> {
    let coverage = ntesim_data::character::released_inventory();
    let csv_content = crate::report_context::variants(csv_path)?;
    let report = ntesim_research::composition::enumerate_compositions(&coverage, &csv_content)?;
    let mut value = serde_json::to_value(&report).map_err(|e| e.to_string())?;

    value["input_hashes"] = json!({"variants.csv":sha256(csv_content.as_bytes())});

    Ok(value)
}

pub(crate) fn published_acceptance_report(out_dir: Option<&Path>) -> Result<Value, String> {
    let coverage = ntesim_data::character::released_inventory();
    let conditions_bytes = ntesim_data::research_inputs::CONDITIONS_JSON.as_bytes();
    let conditions: Value = serde_json::from_slice(conditions_bytes).map_err(|e| e.to_string())?;
    let variants_csv = ntesim_data::research_inputs::VARIANTS_CSV;
    let (deliverables, all_comp_csv, published_csv, arch_csv, manifest_json) =
        ntesim_research::acceptance_spec::compile_acceptance_deliverables(
            &coverage,
            &conditions,
            variants_csv,
        )?;

    if let Some(dir) = out_dir {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        fs::write(dir.join("all_compositions.csv"), all_comp_csv.as_bytes())
            .map_err(|e| format!("{}: {e}", dir.join("all_compositions.csv").display()))?;
        fs::write(dir.join("published_variants.csv"), published_csv.as_bytes())
            .map_err(|e| format!("{}: {e}", dir.join("published_variants.csv").display()))?;
        fs::write(dir.join("archetypes.csv"), arch_csv.as_bytes())
            .map_err(|e| format!("{}: {e}", dir.join("archetypes.csv").display()))?;
        fs::write(dir.join("run_manifest.json"), manifest_json.as_bytes())
            .map_err(|e| format!("{}: {e}", dir.join("run_manifest.json").display()))?;
    }

    let mut report_value = serde_json::to_value(&deliverables).map_err(|e| e.to_string())?;
    let mut input_hashes = BTreeMap::new();
    input_hashes.insert("conditions.json".to_string(), sha256(conditions_bytes));
    input_hashes.insert(
        "published_variants.csv".to_string(),
        sha256(variants_csv.as_bytes()),
    );
    report_value["input_hashes"] = json!(input_hashes);

    Ok(report_value)
}

pub(crate) fn published_finalist_report(
    input: &[u8],
    variants_csv_path: Option<&Path>,
) -> Result<Value, String> {
    if input.len() > 1024 * 1024 {
        return Err("finalist evaluation request exceeds 1 MiB".into());
    }
    let request: ntesim_research::finalists::FinalistValidationRequest =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;

    let variants_csv = crate::report_context::variants(variants_csv_path)?;

    constant_report(input, &ntesim_data::team::TABLES, |_tables, coverage| {
        let limits = ntesim_core::execution::ExecutionLimits::default();
        let report = ntesim_research::finalists::evaluate_finalist(
            coverage,
            &variants_csv,
            &request,
            &limits,
        )?;
        serde_json::to_value(&report).map_err(|e| e.to_string())
    })
}

pub(crate) fn verify_tabular_report(fixture_path: Option<&Path>) -> Result<Value, String> {
    let (fixture_bytes, custom_cases) = match fixture_path {
        Some(p) => {
            let bytes = fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
            if bytes.len() > 1024 * 1024 {
                return Err("tabular fixture exceeds 1 MiB".into());
            }
            let cases: Vec<ntesim_research::tabular_verification::TabularTestCase> =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            (bytes, Some(cases))
        }
        None => (b"default-tabular-battery".to_vec(), None),
    };

    constant_report(
        &fixture_bytes,
        &ntesim_data::team::TABLES,
        |_tables, _coverage| {
            let report = ntesim_research::tabular_verification::run_tabular_verification(
                custom_cases.as_deref(),
            )?;
            serde_json::to_value(&report).map_err(|e| e.to_string())
        },
    )
}

pub(crate) fn pilot_report(
    request_path: Option<&Path>,
    checkpoint_path: Option<&Path>,
) -> Result<Value, String> {
    let (request_bytes, mut request) = match request_path {
        Some(p) => {
            let bytes = fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
            if bytes.len() > 1024 * 1024 {
                return Err("pilot request exceeds 1 MiB".into());
            }
            let req: ntesim_research::pilot::PilotRequest =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            (bytes, req)
        }
        None => {
            let req = ntesim_research::pilot::default_pilot_request();
            let bytes = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
            (bytes, req)
        }
    };

    if let Some(cp_path) = checkpoint_path {
        let cp_bytes = fs::read(cp_path).map_err(|e| format!("{}: {e}", cp_path.display()))?;
        if cp_bytes.len() > 16 * 1024 * 1024 {
            return Err("checkpoint file exceeds 16 MiB".into());
        }
        let checkpoint: ntesim_research::pilot::PilotCheckpoint =
            serde_json::from_slice(&cp_bytes).map_err(|e| e.to_string())?;
        request.resume_checkpoint = Some(checkpoint);
    }

    constant_report(
        &request_bytes,
        &ntesim_data::team::TABLES,
        |_tables, coverage| {
            let limits = ntesim_core::execution::ExecutionLimits::default();
            let report = ntesim_research::pilot::run_pilot(coverage, &request, &limits)?;
            serde_json::to_value(&report).map_err(|e| e.to_string())
        },
    )
}

pub(crate) fn acceptance_run_report(
    request_path: Option<&Path>,
    checkpoint_path: Option<&Path>,
    variants_csv_path: Option<&Path>,
    conditions_path: Option<&Path>,
) -> Result<Value, String> {
    let (request_bytes, mut request) = match request_path {
        Some(p) => {
            let bytes = fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
            if bytes.len() > 1024 * 1024 {
                return Err("acceptance run request exceeds 1 MiB".into());
            }
            let req: ntesim_research::acceptance_runner::AcceptanceRunRequest =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            (bytes, req)
        }
        None => {
            let req = ntesim_research::acceptance_runner::default_acceptance_request();
            let bytes = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
            (bytes, req)
        }
    };

    if let Some(cp_path) = checkpoint_path {
        let cp_bytes = fs::read(cp_path).map_err(|e| format!("{}: {e}", cp_path.display()))?;
        if cp_bytes.len() > 32 * 1024 * 1024 {
            return Err("checkpoint file exceeds 32 MiB".into());
        }
        let checkpoint: ntesim_research::acceptance_runner::AcceptanceCheckpoint =
            serde_json::from_slice(&cp_bytes).map_err(|e| e.to_string())?;
        request.resume_checkpoint = Some(checkpoint);
    }

    let variants_csv = crate::report_context::variants(variants_csv_path)?;

    let conditions: Value =
        serde_json::from_str(&crate::report_context::conditions(conditions_path)?)
            .map_err(|e| e.to_string())?;

    constant_report(
        &request_bytes,
        &ntesim_data::team::TABLES,
        |_tables, coverage| {
            let limits = ntesim_core::execution::ExecutionLimits::default();
            let report = ntesim_research::acceptance_runner::run_acceptance_pipeline(
                coverage,
                &conditions,
                &variants_csv,
                &request,
                &limits,
            )?;
            serde_json::to_value(&report).map_err(|e| e.to_string())
        },
    )
}
