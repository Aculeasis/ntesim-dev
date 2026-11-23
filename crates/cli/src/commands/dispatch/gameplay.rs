use crate::commands::definition::{CliCommand, completion};
use crate::reports::*;
use serde_json::Value;
use std::fs;

pub(crate) fn dispatch_gameplay(cmd: &CliCommand<'_>) -> Result<Option<(Value, bool)>, String> {
    match cmd {
        CliCommand::BatchScript { source, request } => {
            let request = read_experiment(request)?;
            let report = ntesim_research::trials::source::batch(
                &fs::read_to_string(source).map_err(|e| e.to_string())?,
                &request,
            )?;
            let complete = report.complete;
            Ok(Some((
                serde_json::to_value(report).map_err(|e| e.to_string())?,
                complete,
            )))
        }
        CliCommand::CompareScripts {
            baseline,
            candidate,
            request,
        } => {
            let request = read_experiment(request)?;
            let report = ntesim_research::trials::source::compare(
                &fs::read_to_string(baseline).map_err(|e| e.to_string())?,
                &fs::read_to_string(candidate).map_err(|e| e.to_string())?,
                &request,
            )?;
            let complete = report.complete;
            Ok(Some((
                serde_json::to_value(report).map_err(|e| e.to_string())?,
                complete,
            )))
        }
        CliCommand::RunScript {
            source,
            stop_after_inputs: None,
            resume: None,
            cancel_file: None,
        } => {
            let response = ntesim_content::simulation::run_source(
                &ntesim_content::dsl::compiler::SourceRequest {
                    source: fs::read_to_string(source).map_err(|e| e.to_string())?,
                    limits: Default::default(),
                },
                &Default::default(),
            )?;
            completion(
                serde_json::to_value(response).map_err(|e| e.to_string())?,
                &[
                    "/execution/report/success",
                    "/execution/report/execution_completed",
                    "/execution/report/invariants_passed",
                ],
            )
            .map(Some)
        }
        CliCommand::RunCompiledScript { input } => {
            let artifact = serde_json::from_slice(&fs::read(input).map_err(|e| e.to_string())?)
                .map_err(|e| format!("invalid source artifact: {e}"))?;
            let response =
                ntesim_content::simulation::run_compiled_source(&artifact, &Default::default())?;
            completion(
                serde_json::to_value(response).map_err(|e| e.to_string())?,
                &[
                    "/execution/report/success",
                    "/execution/report/execution_completed",
                    "/execution/report/invariants_passed",
                ],
            )
            .map(Some)
        }
        CliCommand::InspectScript { source } => {
            let response = ntesim_content::dsl::inspection::inspect_response(
                &ntesim_content::dsl::inspection::InspectionRequest {
                    source: fs::read_to_string(source).map_err(|e| e.to_string())?,
                },
            );
            let valid = response.diagnostics.is_empty();
            Ok(Some((
                serde_json::to_value(response).map_err(|e| e.to_string())?,
                valid,
            )))
        }
        CliCommand::CompileScript {
            source,
            validate_only,
        } => {
            let source = fs::read_to_string(source).map_err(|e| e.to_string())?;
            let response = ntesim_content::dsl::source::compile_response(
                &ntesim_content::dsl::compiler::SourceRequest {
                    source,
                    limits: Default::default(),
                },
            );
            let valid = response.diagnostics.is_empty();
            let value = if *validate_only {
                serde_json::to_value(response.diagnostics)
            } else {
                serde_json::to_value(response)
            }
            .map_err(|e| e.to_string())?;
            Ok(Some((value, valid)))
        }
        CliCommand::RunGameSuite { input } => completion(
            game_suite_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            &["/execution_completed"],
        )
        .map(Some),
        CliCommand::RunGame { input } => {
            let report = game_runner_report(&fs::read(input).map_err(|e| e.to_string())?)?;
            let complete = report["success"] == true
                && report["execution_completed"] == true
                && report["invariants_passed"] == true;
            Ok(Some((report, complete)))
        }
        CliCommand::RunGameProgram { input } => {
            let report = game_program_report(&fs::read(input).map_err(|e| e.to_string())?)?;
            let complete = report["success"] == true
                && report["execution_completed"] == true
                && report["invariants_passed"] == true;
            Ok(Some((report, complete)))
        }
        CliCommand::RunGameBuilt { input } => {
            let report = game_build_report(&fs::read(input).map_err(|e| e.to_string())?)?;
            let complete = report["execution"]["success"] == true
                && report["execution"]["execution_completed"] == true
                && report["execution"]["invariants_passed"] == true;
            Ok(Some((report, complete)))
        }
        CliCommand::BatchGame { input, batch } => {
            let report = game_batch_report(
                &fs::read(input).map_err(|e| e.to_string())?,
                &fs::read(batch).map_err(|e| e.to_string())?,
            )?;
            let complete = report["complete"] == true;
            Ok(Some((report, complete)))
        }
        CliCommand::BatchGameBuilt { input, batch } => {
            let report = game_built_batch_report(
                &fs::read(input).map_err(|e| e.to_string())?,
                &fs::read(batch).map_err(|e| e.to_string())?,
            )?;
            let complete = report["complete"] == true;
            Ok(Some((report, complete)))
        }
        _ => Ok(None),
    }
}

fn read_experiment(path: &str) -> Result<ntesim_research::trials::source::Request, String> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > 1024 * 1024 {
        return Err("source experiment request exceeds 1 MiB".into());
    }
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
