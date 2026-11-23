pub(crate) mod gameplay;
pub(crate) mod research;
mod source_control;
mod source_export;
mod source_search;
pub(crate) mod stats;
pub(crate) mod synthetic;

use crate::commands::definition::USAGE;
use serde_json::Value;

pub(crate) fn execute(args: &[String]) -> Result<(Value, bool), String> {
    let command = crate::commands::parse::parse_command(args)?;
    if let Some(result) = source_control::dispatch(&command)? {
        return Ok(result);
    }
    if let Some(result) = source_export::dispatch(&command)? {
        return Ok(result);
    }
    if let Some(result) = source_search::dispatch(&command)? {
        return Ok(result);
    }
    if let Some(res) = synthetic::dispatch_synthetic(&command)? {
        return Ok(res);
    }
    if let Some(res) = stats::dispatch_package_and_stats(&command)? {
        return Ok(res);
    }
    if let Some(res) = gameplay::dispatch_gameplay(&command)? {
        return Ok(res);
    }
    if let Some(res) = research::dispatch_research(&command)? {
        return Ok(res);
    }
    Err(USAGE.into())
}
