//! Opt-in CLI measurements. Encoding into a counting sink isolates JSON cost
//! from stdout IO without allocating another report-sized buffer.
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{self, Write},
    time::Instant,
};

#[derive(Default)]
struct Counter(u64);

impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 += bytes.len() as u64;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) struct Profile {
    output: File,
    started: Instant,
}

impl Profile {
    pub(crate) fn start(args: &[String]) -> Result<Option<Self>, String> {
        if !args
            .iter()
            .any(|arg| arg == "--profile" || arg.starts_with("--profile="))
        {
            return Ok(None);
        }
        let matches = crate::clap_app::build_cli()
            .try_get_matches_from(std::iter::once("ntesim".to_string()).chain(args.iter().cloned()))
            .map_err(|e| e.to_string())?;
        matches
            .get_one::<String>("profile")
            .map(|path| {
                Ok(Self {
                    output: File::create_new(path).map_err(|e| format!("{path}: {e}"))?,
                    started: Instant::now(),
                })
            })
            .transpose()
    }

    pub(crate) fn finish(mut self, report: &Value, success: bool) -> Result<(), String> {
        let command_ms = self.started.elapsed().as_secs_f64() * 1000.0;
        let mut bytes = Counter::default();
        let start = Instant::now();
        crate::write_report(&mut bytes, report).map_err(|e| e.to_string())?;
        let serialization_ms = start.elapsed().as_secs_f64() * 1000.0;
        let checkpoint = report.get("checkpoint").filter(|value| !value.is_null());
        let mut retained = Counter::default();
        if let Some(checkpoint) = checkpoint {
            crate::write_report(&mut retained, checkpoint).map_err(|e| e.to_string())?;
        }
        let events = report
            .pointer("/execution/report/total_events_processed")
            .or_else(|| report.pointer("/checkpoint/events_charged"));
        let measurement = json!({
            "schema_version": 1,
            "engine_version": ntesim_contracts::ENGINE_VERSION,
            "engine_source_sha256": ntesim_contracts::ENGINE_SOURCE_SHA256,
            "operation_success": success,
            "command_ms": command_ms,
            "serialization_ms": serialization_ms,
            "serialized_report_bytes": bytes.0,
            "serialized_checkpoint_bytes": retained.0,
            "retained_search_receipt_bytes": report.pointer("/checkpoint/retained_receipt_bytes"),
            "events_processed": events,
        });
        crate::write_report(&mut self.output, &measurement).map_err(|e| e.to_string())
    }
}
