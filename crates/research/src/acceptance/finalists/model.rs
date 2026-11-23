use super::*;

pub const DEFAULT_FINALIST_HYPOTHESIS: &str = "Finalist evaluation under explicit research crit hypothesis, 1000-seed Monte Carlo trials, resource sustainability, and worsened timing checks (+10%, +20%).";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalistClassification {
    PublishedVariant,
    /// Team is outside published variants, beats the comparable best team by >= 5%,
    /// and retains advantage under timing degradation (+10%, +20%).
    UnexpectedFinalist,
    /// Team is outside published variants with advantage < 5% or sensitive to timing.
    PromisingCandidate,
    Underperforming,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalistValidationRequest {
    pub schema_version: u32,
    pub content_id: String,
    pub hypothesis: String,
    /// Plus-separated composition of 4 character slugs (e.g. "sakiri+lacrimosa+skia+nanally").
    pub composition: String,
    #[serde(default = "default_scenario_kind")]
    pub scenario_kind: AcceptanceScenarioKind,
    pub seeds: Vec<u64>,
    #[serde(default = "default_timing_multipliers")]
    pub timing_multipliers: Vec<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research_crit: Option<ResearchCritPolicy>,
    pub actor_overrides: BTreeMap<String, ActorOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_damage: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_archetype: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalistTimingSummary {
    pub timing_index: usize,
    pub multiplier: Ratio,
    pub multiplier_label: String,
    pub count: usize,
    pub mean_damage: f64,
    pub min_damage: f64,
    pub max_damage: f64,
    pub advantage_vs_baseline_pct: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalistEvaluationReport {
    pub schema_version: u32,
    pub report_kind: String,
    pub content_id: String,
    pub status: String,
    pub quality: Quality,
    pub game_dps_computed: bool,
    pub acceptance_ready: bool,
    pub composition: String,
    pub slots: [String; 4],
    pub scenario: String,
    pub scenario_description: String,
    pub is_published_variant: bool,
    pub classification: FinalistClassification,
    pub seed_count: usize,
    pub resource_sustainable: bool,
    pub all_invariants_passed: bool,
    pub all_energy_conserved: bool,
    pub nominal_mean_damage: f64,
    pub baseline_damage: Option<f64>,
    pub baseline_archetype: Option<String>,
    pub nominal_advantage_pct: Option<f64>,
    pub beats_5pct_threshold: bool,
    pub retains_advantage_under_timing_degradation: bool,
    pub superiority_confirmed: bool,
    pub timing_summaries: Vec<FinalistTimingSummary>,
    pub batch_statistics: Vec<Statistics>,
    pub hypothesis: String,
    pub blockers: Vec<String>,
    pub engine_version: String,
    pub engine_source_sha256: String,
}
