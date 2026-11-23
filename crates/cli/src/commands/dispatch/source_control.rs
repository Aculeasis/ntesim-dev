use crate::commands::definition::{CliCommand, completion};
use ntesim_content::simulation::{PreparedSourceScenario, SourceCheckpoint};
use serde_json::Value;
use std::{fs::File, io::Read, path::Path};

fn read(path: &str, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| format!("{path}: {e}"))?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!("{path} exceeds its input size limit"));
    }
    Ok(bytes)
}

pub(super) fn dispatch(command: &CliCommand<'_>) -> Result<Option<(Value, bool)>, String> {
    let CliCommand::RunScript {
        source,
        stop_after_inputs,
        resume,
        cancel_file,
    } = command
    else {
        return Ok(None);
    };
    if stop_after_inputs.is_none() && resume.is_none() && cancel_file.is_none() {
        return Ok(None);
    }
    let stop_after = stop_after_inputs
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "stop-after-inputs must be an unsigned integer".to_string())
        })
        .transpose()?;
    let checkpoint: Option<SourceCheckpoint> = resume
        .map(|path| serde_json::from_slice(&read(path, 4096)?).map_err(|e| e.to_string()))
        .transpose()?;
    let source = String::from_utf8(read(source, 4 * 1024 * 1024)?).map_err(|e| e.to_string())?;
    let program = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
        .map_err(|diagnostics| serde_json::to_string(&diagnostics).unwrap_or_default())?;
    let start = checkpoint
        .as_ref()
        .map_or(0, |saved| saved.completed_inputs);
    let report = program.run_controlled(program.seed(), checkpoint.as_ref(), |boundary| {
        let cancelled = cancel_file
            .map(|path| {
                Path::new(path)
                    .try_exists()
                    .map_err(|e| format!("cancel-file: {e}"))
            })
            .transpose()?
            .unwrap_or(false);
        Ok(cancelled || stop_after.is_some_and(|count| boundary.completed_inputs - start >= count))
    })?;
    completion(
        serde_json::to_value(report).map_err(|e| e.to_string())?,
        &[
            "/execution/report/success",
            "/execution/report/execution_completed",
            "/execution/report/invariants_passed",
        ],
    )
    .map(Some)
}
