//! Thin transport for the same exact source reports and comparisons as the CLI.
use ntesim_content::{
    dsl::source::CompiledSourceProgram,
    simulation::{PreparedSourceScenario, export::SourceReportExport},
};
use ntesim_research::trials::source;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExportRequest {
    compiled: CompiledSourceProgram,
    seed: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayRequest {
    report_json: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompareRequest {
    baseline: String,
    candidate: String,
    request: source::Request,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchRequest {
    request: source::search::SearchRequest,
    checkpoint_json: Option<String>,
    max_new_trials: usize,
}

pub(super) fn dispatch(action: &str, payload: &[u8]) -> Result<Value, String> {
    match action {
        "search_source_reports" => {
            let input: SearchRequest =
                serde_json::from_slice(payload).map_err(|e| e.to_string())?;
            if input
                .request
                .experiment
                .seeds
                .iter()
                .any(|seed| *seed > 9_007_199_254_740_991)
            {
                return Err("search seeds must be JavaScript safe integers".into());
            }
            let checkpoint = input
                .checkpoint_json
                .as_deref()
                .map(|text| {
                    if text.len() > 64 * 1024 * 1024 {
                        return Err(
                            "source search checkpoint exceeds 64 MiB transport limit".into()
                        );
                    }
                    serde_json::from_str(text).map_err(|e| e.to_string())
                })
                .transpose()?;
            let search = source::search::run(input.request, checkpoint, input.max_new_trials)?;
            let report_json = serde_json::to_string_pretty(&search).map_err(|e| e.to_string())?;
            if report_json.len() > 64 * 1024 * 1024 {
                return Err("source search export exceeds 64 MiB transport limit".into());
            }
            let checkpoint_json =
                serde_json::to_string(&search.checkpoint).map_err(|e| e.to_string())?;
            Ok(json!({"search":search,"report_json":report_json,"checkpoint_json":checkpoint_json}))
        }
        "export_source_report" => {
            let input: ExportRequest =
                serde_json::from_slice(payload).map_err(|e| e.to_string())?;
            if input.seed > 9_007_199_254_740_991 {
                return Err("report seed must be a JavaScript safe integer".into());
            }
            let program =
                PreparedSourceScenario::from_export(&input.compiled, &Default::default())?;
            let export = SourceReportExport::new(
                program.run(input.seed)?,
                input.compiled.input.limits,
                Default::default(),
            )?;
            export.replay()?;
            let report_json = serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?;
            if report_json.len() > 64 * 1024 * 1024 {
                return Err("source report export exceeds 64 MiB replay limit".into());
            }
            Ok(json!({"report_json":report_json,
                "report":export,"csv_tables":export.csv_tables()?}))
        }
        "replay_source_report" => {
            let input: ReplayRequest =
                serde_json::from_slice(payload).map_err(|e| e.to_string())?;
            if input.report_json.len() > 64 * 1024 * 1024 {
                return Err("source report export exceeds 64 MiB replay limit".into());
            }
            let export: SourceReportExport =
                serde_json::from_str(&input.report_json).map_err(|e| e.to_string())?;
            serde_json::to_value(export.replay_verification()?).map_err(|e| e.to_string())
        }
        "compare_source_reports" => {
            let input: CompareRequest =
                serde_json::from_slice(payload).map_err(|e| e.to_string())?;
            if input
                .request
                .seeds
                .iter()
                .any(|seed| *seed > 9_007_199_254_740_991)
            {
                return Err("comparison seeds must be JavaScript safe integers".into());
            }
            let comparison = source::compare(&input.baseline, &input.candidate, &input.request)?;
            Ok(
                json!({"report_json":serde_json::to_string_pretty(&comparison).map_err(|e| e.to_string())?,
                "comparison":comparison}),
            )
        }
        _ => Err(format!("unsupported source report action: {action}")),
    }
}
