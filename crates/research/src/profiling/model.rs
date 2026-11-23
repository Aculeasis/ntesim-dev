use super::*;

pub const VERSION: &str = "native-pilot-profiling-v1";

pub const DEFAULT_PILOT_HYPOTHESIS: &str = "Stage F representative pilot profiling across published teams, 120s real/mode scenarios, dilated timings (+10%, +20%), and deterministic stop-and-resume checkpointing.";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PilotBudget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_units_per_checkpoint: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_wall_time_ms: Option<u64>,
    /// Pruning threshold percent for screening: candidates performing below this percentage
    /// of the incumbent best are flagged with prune receipts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screening_prune_threshold_pct: Option<f64>,
}

impl Default for PilotBudget {
    fn default() -> Self {
        Self {
            max_units_per_checkpoint: None,
            max_wall_time_ms: None,
            screening_prune_threshold_pct: Some(15.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PilotExecutionUnit {
    pub unit_index: usize,
    pub team: String,
    pub scenario_kind: AcceptanceScenarioKind,
    pub timing_multiplier: Ratio,
    pub seed: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PilotUnitResult {
    pub unit_index: usize,
    pub team: String,
    pub scenario: String,
    pub timing_multiplier: Ratio,
    pub seed: u64,
    pub damage_exact: ExactFraction,
    pub damage_f64: f64,
    pub dps: f64,
    pub events_processed: usize,
    pub elapsed_us: u64,
    pub estimated_peak_memory_bytes: usize,
    pub storage_bytes: usize,
    pub unit_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PilotCheckpoint {
    pub checkpoint_id: String,
    pub schema_version: u32,
    pub request_sha256: String,
    pub cumulative_sha256: String,
    pub completed_units: Vec<PilotUnitResult>,
    /// Atomic units that could not produce an executable report. These are
    /// retained individually so one unsupported profile does not hide the
    /// rest of a pilot batch.
    #[serde(default)]
    pub failed_units: Vec<PilotUnitFailure>,
    pub next_unit_index: usize,
    pub total_planned_units: usize,
    pub is_complete: bool,
    pub resume_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PilotUnitFailure {
    pub unit_index: usize,
    pub team: String,
    pub scenario: String,
    pub timing_multiplier: Ratio,
    pub seed: u64,
    pub reason: String,
    pub failure_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PilotProfilingSummary {
    pub units_completed: usize,
    pub units_failed: usize,
    pub total_wall_time_us: u64,
    pub total_events_processed: u64,
    pub average_us_per_unit: f64,
    pub min_us_per_unit: u64,
    pub max_us_per_unit: u64,
    pub events_per_second: f64,
    pub throughput_runs_per_second: f64,
    pub peak_memory_estimate_bytes: usize,
    pub average_storage_bytes_per_result: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrunedCandidateReceipt {
    pub scenario: String,
    pub team: String,
    pub score_dps: f64,
    pub baseline_dps: f64,
    pub deficit_pct: f64,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StageGWorkloadProjection {
    pub total_four_character_combinations: usize,
    pub scenarios_per_combination: usize,
    pub screening_seeds_per_scenario: usize,
    pub projected_screening_runs: u64,
    pub finalist_pool_size: usize,
    pub finalist_timings_count: usize,
    pub finalist_seeds_per_timing: usize,
    pub projected_finalist_runs: u64,
    pub total_projected_runs: u64,
    pub projected_screening_cpu_hours: f64,
    pub projected_finalist_cpu_hours: f64,
    pub projected_total_cpu_hours: f64,
    pub projected_storage_mb: f64,
    /// Absent until repeated measurements support an uncertainty estimate.
    pub uncertainty_range_pct: Option<f64>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PilotReport {
    pub schema_version: u32,
    pub report_kind: String,
    pub content_id: String,
    pub status: String,
    pub quality: Quality,
    pub game_dps_computed: bool,
    pub acceptance_ready: bool,
    pub request_sha256: String,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub profiling: PilotProfilingSummary,
    pub projection: StageGWorkloadProjection,
    pub checkpoint: PilotCheckpoint,
    pub pruned_receipts: Vec<PrunedCandidateReceipt>,
    pub blockers: Vec<String>,
    pub input_hashes: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PilotRequest {
    pub schema_version: u32,
    pub content_id: String,
    pub hypothesis: String,
    pub teams: Vec<String>,
    pub scenarios: Vec<AcceptanceScenarioKind>,
    pub timing_multipliers: Vec<Ratio>,
    pub seeds: Vec<u64>,
    #[serde(default)]
    pub budget: PilotBudget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_checkpoint: Option<PilotCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_overrides: Option<BTreeMap<String, ActorOverride>>,
}
