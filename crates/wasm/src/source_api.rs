//! Source compilation/editing transport; all language behavior belongs to content.
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditRequest {
    source: String,
    changes: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputRequest {
    name: String,
    text: String,
}

pub(super) fn dispatch(action: &str, payload: &[u8]) -> Result<Value, String> {
    match action {
        "model_assumption" => {
            let id: String = serde_json::from_slice(payload)
                .map_err(|error| format!("invalid model assumption identity: {error}"))?;
            ntesim_contracts::get_hypothesis_description(&id)
                .map(|description| json!(description))
                .ok_or_else(|| format!("unknown model assumption: {id}"))
        }
        "inspect_script" => {
            let request: ntesim_content::dsl::inspection::InspectionRequest =
                serde_json::from_slice(payload)
                    .map_err(|e| format!("invalid inspection request: {e}"))?;
            serde_json::to_value(ntesim_content::dsl::inspection::inspect_response(&request))
                .map_err(|e| e.to_string())
        }
        "compile_script" | "validate_script" => {
            let request: ntesim_content::dsl::compiler::SourceRequest =
                serde_json::from_slice(payload)
                    .map_err(|e| format!("invalid source request: {e}"))?;
            let response = ntesim_content::dsl::compiler::compile_response(&request);
            if action == "validate_script" {
                serde_json::to_value(response.diagnostics)
            } else {
                serde_json::to_value(response)
            }
            .map_err(|e| e.to_string())
        }
        "run_script" => {
            let request: ntesim_content::dsl::compiler::SourceRequest =
                serde_json::from_slice(payload)
                    .map_err(|e| format!("invalid source request: {e}"))?;
            serde_json::to_value(ntesim_content::simulation::run_source(
                &request,
                &Default::default(),
            )?)
            .map_err(|e| e.to_string())
        }
        "compile_source" | "validate_source" => {
            let request: ntesim_content::dsl::compiler::SourceRequest =
                serde_json::from_slice(payload)
                    .map_err(|e| format!("invalid source request: {e}"))?;
            if action == "validate_source" {
                let response = ntesim_content::dsl::source::compile_response(&request);
                serde_json::to_value(response.diagnostics).map_err(|e| e.to_string())
            } else {
                super::source_trials::compile_source(&request)
            }
        }
        "run_compiled_source" => {
            let artifact = serde_json::from_slice(payload)
                .map_err(|e| format!("invalid source artifact: {e}"))?;
            serde_json::to_value(ntesim_content::simulation::run_compiled_source(
                &artifact,
                &Default::default(),
            )?)
            .map_err(|e| e.to_string())
        }
        "edit_script_options" => {
            let request: EditRequest = serde_json::from_slice(payload)
                .map_err(|e| format!("invalid source edit request: {e}"))?;
            Ok(
                match ntesim_content::dsl::editing::edit_options(&request.source, &request.changes)
                {
                    Ok(source) => json!({"source":source,"diagnostics":[]}),
                    Err(diagnostics) => json!({"source":null,"diagnostics":diagnostics}),
                },
            )
        }
        "parse_option_input" => {
            let request: InputRequest = serde_json::from_slice(payload)
                .map_err(|e| format!("invalid option input request: {e}"))?;
            ntesim_content::dsl::editing::parse_option_input(&request.name, &request.text)
        }
        _ => Err(format!("unsupported source action: {action}")),
    }
}
