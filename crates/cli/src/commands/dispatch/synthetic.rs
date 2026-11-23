use crate::commands::definition::CliCommand;
use serde_json::Value;
use std::fs;

pub(crate) fn dispatch_synthetic(cmd: &CliCommand<'_>) -> Result<Option<(Value, bool)>, String> {
    match cmd {
        CliCommand::CheckCyclesSynthetic {
            input,
            source,
            checks,
        } => {
            let report = ntesim_research::cycle::run(
                &fs::read(input).map_err(|e| e.to_string())?,
                &fs::read_to_string(source).map_err(|e| e.to_string())?,
                &fs::read(checks).map_err(|e| e.to_string())?,
            )?;
            let valid = report.passed;
            Ok(Some((
                serde_json::to_value(report).map_err(|e| e.to_string())?,
                valid,
            )))
        }
        CliCommand::SearchRotationSynthetic { input, request } => {
            let report = ntesim_research::rotation_search::run(
                &fs::read(input).map_err(|e| e.to_string())?,
                &fs::read(request).map_err(|e| e.to_string())?,
            )?;
            let valid = report.complete;
            Ok(Some((
                serde_json::to_value(report).map_err(|e| e.to_string())?,
                valid,
            )))
        }
        CliCommand::CompareRotationSynthetic {
            left_input,
            left_source,
            right_input,
            right_source,
            request,
        } => {
            let report = ntesim_research::compare::run(
                &fs::read(left_input).map_err(|e| e.to_string())?,
                &fs::read_to_string(left_source).map_err(|e| e.to_string())?,
                &fs::read(right_input).map_err(|e| e.to_string())?,
                &fs::read_to_string(right_source).map_err(|e| e.to_string())?,
                &fs::read(request).map_err(|e| e.to_string())?,
            )?;
            let valid = report.complete;
            Ok(Some((
                serde_json::to_value(report).map_err(|e| e.to_string())?,
                valid,
            )))
        }
        CliCommand::BatchRotationSynthetic {
            input,
            rotation,
            request,
        } => {
            let report = ntesim_research::batch::run(
                &fs::read(input).map_err(|e| e.to_string())?,
                &fs::read_to_string(rotation).map_err(|e| e.to_string())?,
                &fs::read(request).map_err(|e| e.to_string())?,
            )?;
            let valid = report.complete;
            Ok(Some((
                serde_json::to_value(report).map_err(|e| e.to_string())?,
                valid,
            )))
        }
        CliCommand::RunRotationSynthetic {
            input,
            rotation,
            seed,
        } => {
            let bytes = fs::read(input).map_err(|e| e.to_string())?;
            let source = fs::read_to_string(rotation).map_err(|e| e.to_string())?;
            let report = match seed {
                Some(s) => ntesim_research::rotation::run_seeded(&bytes, &source, *s)?,
                None => ntesim_research::rotation::run(&bytes, &source)?,
            };
            let valid = report.simulation.success;
            Ok(Some((
                serde_json::to_value(report).map_err(|e| e.to_string())?,
                valid,
            )))
        }
        CliCommand::RunSynthetic { input } => {
            let report =
                ntesim_research::synthetic::run(&fs::read(input).map_err(|e| e.to_string())?)?;
            let valid = report.success;
            Ok(Some((
                serde_json::to_value(report).map_err(|e| e.to_string())?,
                valid,
            )))
        }
        _ => Ok(None),
    }
}
