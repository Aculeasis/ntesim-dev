use super::*;

pub const VERSION: &str = "acceptance-large-scale-runner-v1";

pub const DEFAULT_ACCEPTANCE_HYPOTHESIS: &str = "Stage G automated large-scale calculation and research pipeline covering composition screening, finalist validation, multi-cycle sustainability, and acceptance deliverables compilation.";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcceptanceWorkerConfig {
    pub max_workers: usize,
    pub chunk_size: usize,
}

impl Default for AcceptanceWorkerConfig {
    fn default() -> Self {
        let detected = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Self {
            max_workers: detected.min(16),
            chunk_size: 16,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptancePhase {
    Screening,
    FinalistRefinement,
    Deliverables,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompositionScreeningSummary {
    pub scenario: AcceptanceScenarioKind,
    pub seeds: Vec<u64>,
    pub composition_index: usize,
    pub composition: String,
    pub is_published_variant: bool,
    pub matched_archetype: Option<String>,
    pub nominal_dps: f64,
    pub nominal_damage_exact: ExactFraction,
    pub events_processed: usize,
    pub invariants_passed: bool,
    pub energy_conserved: bool,
    pub result_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompositionScreeningFailure {
    pub scenario: AcceptanceScenarioKind,
    pub seeds: Vec<u64>,
    pub composition_index: usize,
    pub composition: String,
    pub is_published_variant: bool,
    pub matched_archetype: Option<String>,
    pub reason: String,
    pub failure_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TopCandidateEntry {
    pub scenario: AcceptanceScenarioKind,
    pub rank: usize,
    pub composition: String,
    pub nominal_dps: f64,
    pub is_published_variant: bool,
    pub matched_archetype: Option<String>,
    pub superiority_vs_baseline_pct: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcceptanceCheckpoint {
    pub checkpoint_id: String,
    pub schema_version: u32,
    pub request_sha256: String,
    pub current_phase: AcceptancePhase,
    pub completed_composition_indices: Vec<usize>,
    #[serde(default)]
    pub processed_composition_indices: Vec<usize>,
    pub screening_results: BTreeMap<String, CompositionScreeningSummary>,
    #[serde(default)]
    pub screening_failures: BTreeMap<String, CompositionScreeningFailure>,
    pub finalist_results: BTreeMap<String, FinalistEvaluationReport>,
    pub cumulative_sha256: String,
    pub is_complete: bool,
    pub resume_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcceptanceRunRequest {
    pub schema_version: u32,
    pub content_id: String,
    pub hypothesis: String,
    pub scenarios: Vec<AcceptanceScenarioKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_compositions_to_evaluate: Option<usize>,
    pub screening_seeds: Vec<u64>,
    pub finalist_seeds: Vec<u64>,
    pub timing_multipliers: Vec<Ratio>,
    pub top_candidates_per_scenario: usize,
    #[serde(default)]
    pub worker_config: AcceptanceWorkerConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_checkpoint: Option<AcceptanceCheckpoint>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcceptanceRunReport {
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
    pub current_phase: AcceptancePhase,
    pub compositions_enumerated: usize,
    pub compositions_processed: usize,
    pub compositions_screened: usize,
    pub finalists_evaluated: usize,
    pub published_variants_evaluated: usize,
    pub unexpected_finalists_found: usize,
    pub top_candidates: Vec<TopCandidateEntry>,
    pub finalists: Vec<FinalistEvaluationReport>,
    pub checkpoint: AcceptanceCheckpoint,
    pub deliverables: Option<AcceptanceDeliverables>,
    pub blockers: Vec<String>,
    pub input_hashes: BTreeMap<String, String>,
}
