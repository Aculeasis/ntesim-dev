//! Replay checkpoints bind exact input and a verified completed prefix. They do
//! not serialize report observations as if those contained the event queue.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct SourceBoundary {
    pub completed_inputs: usize,
    pub time_us: u64,
    pub events_processed: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceCheckpoint {
    pub schema_version: u32,
    pub input_sha256: String,
    pub completed_inputs: usize,
    pub prefix_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceControlReport {
    pub execution: SourceScenarioReport,
    /// None for complete or failed executions. A cancelled prefix never ranks.
    pub checkpoint: Option<SourceCheckpoint>,
}

impl SourceCheckpoint {
    fn identity(program: &PreparedSourceScenario, seed: u64) -> Result<String, String> {
        let bytes = serde_json::to_vec(&(
            "source-replay-checkpoint-v1",
            program.export(),
            program.limits,
            seed,
        ))
        .map_err(|e| e.to_string())?;
        Ok(ntesim_contracts::sha256(&bytes))
    }

    pub(super) fn capture(trial: &Trial<'_>, seed: u64) -> Result<Self, String> {
        Ok(Self {
            schema_version: 1,
            input_sha256: Self::identity(trial.program, seed)?,
            completed_inputs: trial.occurrences.len(),
            prefix_sha256: trial.boundary_digest()?,
        })
    }

    pub(super) fn validate(
        &self,
        program: &PreparedSourceScenario,
        seed: u64,
    ) -> Result<(), String> {
        if self.schema_version != 1
            || self.input_sha256 != Self::identity(program, seed)?
            || self.completed_inputs > program.source_limits.max_actions
        {
            return Err("Source checkpoint identity mismatch.".into());
        }
        Ok(())
    }
}

impl Trial<'_> {
    pub(super) fn boundary(&self) -> SourceBoundary {
        SourceBoundary {
            completed_inputs: self.occurrences.len(),
            time_us: self.execution.state.current_time_us,
            events_processed: self.execution.events_processed,
        }
    }

    pub(super) fn boundary_digest(&self) -> Result<String, String> {
        // This verifies the reconstructed prefix; the digest is not combat
        // state. Replay rebuilds lexical scopes, pending packets, owner state,
        // all RNG streams and equal-time sequence numbers before continuation.
        let bytes = serde_json::to_vec(&(
            &self.request,
            &self.replay_actions,
            &self.occurrences,
            &self.execution.trace,
            &self.execution.hit_receipts,
            &self.execution.rng,
            self.execution.events_processed,
            self.execution.state.current_time_us,
            self.execution.state.mode_clock.replay_state(),
            format!("{:?}", self.execution.state.replay_clocks),
            self.execution.state.actor_summaries(&self.request)?,
            self.execution.state.target_summaries()?,
            self.execution.state.buffs.instances(),
        ))
        .map_err(|e| e.to_string())?;
        Ok(ntesim_contracts::sha256(&bytes))
    }
}
