use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub schema_version: u32,
    pub input_sha256: String,
    pub engine_source_sha256: String,
    pub runs: Vec<Run>,
    pub events_charged: u64,
    /// Account for consumed work whose receipt could not fit the retained budget.
    pub unretained_trial_events: Option<u64>,
    pub retained_receipt_bytes: usize,
    pub receipt_sha256: String,
}

impl Checkpoint {
    pub(super) fn new(input_sha256: &str) -> Self {
        Self {
            schema_version: 1,
            input_sha256: input_sha256.into(),
            engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
            runs: Vec::new(),
            events_charged: 0,
            unretained_trial_events: None,
            retained_receipt_bytes: 0,
            receipt_sha256: String::new(),
        }
    }

    fn digest(&self) -> Result<String, String> {
        Ok(identity(&[
            self.input_sha256.as_bytes(),
            self.engine_source_sha256.as_bytes(),
            &encode(&self.runs)?,
            &encode(&(
                self.events_charged,
                self.unretained_trial_events,
                self.retained_receipt_bytes,
            ))?,
        ]))
    }

    pub(super) fn seal(&mut self) -> Result<(), String> {
        self.receipt_sha256 = self.digest()?;
        Ok(())
    }

    pub(super) fn validate(
        &self,
        request: &SearchRequest,
        input_sha256: &str,
    ) -> Result<(), String> {
        if self.schema_version != 1
            || self.input_sha256 != input_sha256
            || self.engine_source_sha256 != ENGINE_SOURCE_SHA256
        {
            return Err("source search checkpoint engine/input identity mismatch".into());
        }
        if self.runs.len() > request.candidates.len() * request.experiment.seeds.len()
            || self.receipt_sha256 != self.digest()?
            || self.events_charged > request.experiment.max_total_events
            || self.retained_receipt_bytes > request.experiment.max_receipt_bytes
        {
            return Err("corrupted source search checkpoint".into());
        }
        if self
            .unretained_trial_events
            .is_some_and(|events| events > request.experiment.max_events_per_run)
        {
            return Err("checkpoint unretained trial exceeds its event allowance".into());
        }
        let mut events = self.unretained_trial_events.unwrap_or(0);
        let mut bytes = 0_usize;
        for (index, row) in self.runs.iter().enumerate() {
            let candidate = &request.candidates[index / request.experiment.seeds.len()];
            let seed = request.experiment.seeds[index % request.experiment.seeds.len()];
            if row.seed != seed {
                return Err("checkpoint seed prefix differs".into());
            }
            let charged = if let Some(execution) = &row.execution {
                if execution.source != candidate.source
                    || execution.report.seed != seed
                    || execution.request.seed != seed
                    || execution.report.engine_source_sha256 != ENGINE_SOURCE_SHA256
                    || row.error.is_some()
                    || row.eligible != eligible(execution)
                {
                    return Err(
                        "checkpoint trial receipt differs from source/seed/engine identity".into(),
                    );
                }
                execution.report.total_events_processed as u64
            } else {
                if row.error.is_none() || row.eligible {
                    return Err("checkpoint failed trial is inconsistent".into());
                }
                request.experiment.max_events_per_run
            };
            if charged > request.experiment.max_events_per_run {
                return Err("checkpoint trial exceeds event budget".into());
            }
            events = events
                .checked_add(charged)
                .ok_or("checkpoint event count overflow")?;
            bytes = bytes
                .checked_add(encode(row)?.len())
                .ok_or("checkpoint receipt size overflow")?;
        }
        if events != self.events_charged || bytes != self.retained_receipt_bytes {
            return Err("checkpoint cumulative work/receipt accounting differs".into());
        }
        Ok(())
    }
}
