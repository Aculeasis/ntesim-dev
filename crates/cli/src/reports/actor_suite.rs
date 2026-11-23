use crate::report_context::constant_report;
use ntesim_content::build::studio_projection::{Request, project};
use serde_json::{Value, json};

pub(crate) fn actor_suite_report(input: &[u8]) -> Result<Value, String> {
    if input.len() > 4 * 1024 * 1024 {
        return Err("actor suite input exceeds 4 MiB".into());
    }
    let requests: Vec<Request> = serde_json::from_slice(input).map_err(|e| e.to_string())?;
    if requests.is_empty() || requests.len() > 512 {
        return Err("actor suite requires 1..=512 requests".into());
    }
    constant_report(
        input,
        &ntesim_data::stat_projection::TABLES,
        |tables, _coverage| {
            let reports = requests
                .iter()
                .enumerate()
                .map(|(index, request)| {
                    project(tables, request)
                        .and_then(|report| serde_json::to_value(report).map_err(|e| e.to_string()))
                        .map_err(|error| format!("actor {index}: {error}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!({ "reports": reports }))
        },
    )
}
