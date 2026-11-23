use crate::commands::definition::CliCommand;
use ntesim_content::simulation::export::SourceReportExport;
use ntesim_contracts::sha256;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

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

fn write_bundle(output: &str, export: &SourceReportExport) -> Result<Value, String> {
    let report = serde_json::to_vec_pretty(export).map_err(|e| e.to_string())?;
    if report.len() > 64 * 1024 * 1024 {
        return Err("source report export exceeds 64 MiB replay limit".into());
    }
    let mut files = export
        .csv_tables()?
        .into_iter()
        .map(|(name, text)| (name, text.into_bytes()))
        .collect::<BTreeMap<_, _>>();
    files.insert("report.json", report);
    let hashes = files
        .iter()
        .map(|(name, bytes)| (*name, sha256(bytes)))
        .collect::<BTreeMap<_, _>>();
    let manifest = json!({"schema_version":1,"source_sha256":export.source_sha256,
        "input_sha256":export.input_sha256,
        "request_sha256":export.request_sha256,"engine_source_sha256":export.engine_source_sha256,
        "success":export.execution.report.success,"execution_completed":export.execution.report.execution_completed,
        "quality":export.execution.report.quality,
        "damage":export.damage.total,"files":hashes});
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    // Refuse existing directories/files. Generate and bound the full artifact
    // before touching the destination; writing a report never overwrites work.
    let output = Path::new(output);
    fs::create_dir(output).map_err(|e| format!("{}: {e}", output.display()))?;
    for (name, data) in files
        .into_iter()
        .chain(std::iter::once(("manifest.json", manifest_bytes)))
    {
        let path = output.join(name);
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .and_then(|mut file| file.write_all(&data))
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(manifest)
}

pub(super) fn dispatch(command: &CliCommand<'_>) -> Result<Option<(Value, bool)>, String> {
    match command {
        CliCommand::ExportScript { source, output } => {
            let source =
                String::from_utf8(read(source, 4 * 1024 * 1024)?).map_err(|e| e.to_string())?;
            let response = ntesim_content::simulation::run_source(
                &ntesim_content::dsl::compiler::SourceRequest {
                    source,
                    limits: Default::default(),
                },
                &Default::default(),
            )?;
            let Some(execution) = response.execution else {
                return Ok(Some((json!({"diagnostics":response.diagnostics}), false)));
            };
            let complete = execution.report.success
                && execution.report.execution_completed
                && execution.report.invariants_passed;
            let export =
                SourceReportExport::new(execution, Default::default(), Default::default())?;
            Ok(Some((write_bundle(output, &export)?, complete)))
        }
        CliCommand::ReplayScript { input } => {
            let export: SourceReportExport =
                serde_json::from_slice(&read(input, 64 * 1024 * 1024)?)
                    .map_err(|e| format!("invalid source report export: {e}"))?;
            Ok(Some((
                serde_json::to_value(export.replay_verification()?).map_err(|e| e.to_string())?,
                true,
            )))
        }
        _ => Ok(None),
    }
}
