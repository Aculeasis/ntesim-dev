use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceSourceKind {
    PublishedCalculator,
    CommunitySpreadsheet,
    PublishedBaseline,
    IndependentDeterministicOracle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationCaseStatus {
    ExactMatch,
    WithinTolerance,
    DiscrepancyDocumented,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TabularTestCase {
    pub case_id: String,
    pub source_kind: ReferenceSourceKind,
    pub description: String,
    pub reference: String,
    pub applicability: String,
    pub tolerance_rationale: String,
    pub attacker_level: u32,
    pub base_atk: f64,
    pub atk_percent: f64,
    pub atk_flat: f64,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub damage_bonus: f64,
    pub hit_ratio: f64,
    pub hit_flat: f64,
    pub target_defense: f64,
    pub defense_shred: f64,
    pub defense_ignore: f64,
    pub target_resistance: f64,
    pub negative_res_model: NegativeResistanceModel,
    pub expected_non_crit: f64,
    pub expected_crit: Option<f64>,
    pub tolerance_pct: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discrepancy_note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TabularCaseResult {
    pub case: TabularTestCase,
    pub engine_non_crit: f64,
    pub non_crit_diff_pct: f64,
    pub engine_crit: Option<f64>,
    pub crit_diff_pct: Option<f64>,
    pub status: VerificationCaseStatus,
    pub passed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentedDiscrepancy {
    pub discrepancy_id: String,
    pub source: String,
    pub standard_formula: String,
    pub community_formula: String,
    pub numerical_impact: String,
    pub engine_policy: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TabularVerificationReport {
    pub schema_version: u32,
    pub report_kind: String,
    pub content_id: String,
    pub status: String,
    pub quality: Quality,
    pub game_dps_computed: bool,
    pub acceptance_ready: bool,
    pub total_cases: usize,
    pub passed_cases: usize,
    pub exact_matches: usize,
    pub within_tolerance_cases: usize,
    pub documented_discrepancy_cases: usize,
    pub failed_cases: usize,
    pub case_results: Vec<TabularCaseResult>,
    pub documented_discrepancies: Vec<DocumentedDiscrepancy>,
    pub blockers: Vec<String>,
    pub engine_version: String,
    pub engine_source_sha256: String,
}
