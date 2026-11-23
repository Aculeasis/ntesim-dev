use crate::commands::definition::CliCommand;
use ntesim_research::trials::source::{
    Request,
    search::{self, Candidate, Checkpoint, Objective, SearchRequest},
};
use serde::Deserialize;
use serde_json::Value;
use std::{fs::File, io::Read, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileCandidate {
    id: String,
    source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRequest {
    schema_version: u32,
    experiment: Request,
    objective: Objective,
    candidates: Vec<FileCandidate>,
    max_new_trials: usize,
}

fn read(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(format!("{} exceeds its input size limit", path.display()));
    }
    Ok(bytes)
}

pub(super) fn dispatch(command: &CliCommand<'_>) -> Result<Option<(Value, bool)>, String> {
    let CliCommand::SearchScripts {
        request,
        checkpoint,
    } = command
    else {
        return Ok(None);
    };
    let path = Path::new(request);
    let input: FileRequest =
        serde_json::from_slice(&read(path, 1024 * 1024)?).map_err(|e| e.to_string())?;
    if input.candidates.is_empty() || input.candidates.len() > 64 {
        return Err("search requires 1..=64 source files".into());
    }
    let parent = path
        .parent()
        .ok_or("search request has no parent directory")?;
    let mut candidates = Vec::new();
    let mut bytes = 0_usize;
    for candidate in input.candidates {
        let source = read(&parent.join(candidate.source), 4 * 1024 * 1024)?;
        bytes += source.len();
        if bytes > 16 * 1024 * 1024 {
            return Err("search source domain exceeds 16 MiB".into());
        }
        candidates.push(Candidate {
            id: candidate.id,
            source: String::from_utf8(source).map_err(|e| e.to_string())?,
        });
    }
    let checkpoint: Option<Checkpoint> = checkpoint
        .map(|path| {
            serde_json::from_slice(&read(Path::new(path), 65 * 1024 * 1024)?)
                .map_err(|e| e.to_string())
        })
        .transpose()?;
    let report = search::run(
        SearchRequest {
            schema_version: input.schema_version,
            experiment: input.experiment,
            objective: input.objective,
            candidates,
        },
        checkpoint,
        input.max_new_trials,
    )?;
    let success = report.success;
    Ok(Some((
        serde_json::to_value(report).map_err(|e| e.to_string())?,
        success,
    )))
}
