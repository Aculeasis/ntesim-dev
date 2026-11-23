use super::*;

/// Proposed substat data shape; no optimizer currently emits validated lines.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SubstatLine {
    pub item_id: String,
    pub item_type: String,
    pub stat_name: String,
    pub numerator: u128,
    pub denominator: u128,
}

/// Proposed profile shape retained for compatibility. No current optimizer
/// produces a legal/source-bound character build profile.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CharacterBuildProfile {
    pub slug: String,
    pub character_id: String,
    pub role: CharacterRole,
    pub cartridge_main: String,
    pub base_atk: u64,
    pub baseline_atk: Ratio,
    pub optimized_atk: Ratio,
    pub baseline_crit_rate: Ratio,
    pub optimized_crit_rate: Ratio,
    pub baseline_crit_damage_bonus: Ratio,
    pub optimized_crit_damage_bonus: Ratio,
    pub baseline_elemental_bonus: Ratio,
    pub optimized_elemental_bonus: Ratio,
    pub substats: Vec<SubstatLine>,
    pub useful_substat_count: usize,
    pub baseline_score: f64,
    pub optimized_score: f64,
    pub score_improvement_pct: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VariantComparisonResult {
    #[serde(flatten)]
    pub readiness: ScenarioReadiness,
    pub variant_index: usize,
    pub archetype: String,
    pub tier: String,
    pub slots: [String; 4],
    pub status: String,
    pub failure_reason: Option<String>,
    pub baseline_damage: Option<f64>,
    pub optimized_damage: Option<f64>,
    pub damage_delta: Option<f64>,
    pub percentage_gain: Option<f64>,
    pub settled_hits: Option<usize>,
    pub invariants_verified: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArchetypeRefinement {
    #[serde(flatten)]
    pub readiness: ScenarioReadiness,
    pub archetype: String,
    pub tier: String,
    pub best_variant_index: usize,
    pub best_slots: [String; 4],
    pub baseline_damage: f64,
    pub optimized_damage: f64,
    pub damage_delta: f64,
    pub percentage_gain: f64,
    pub total_variants_in_archetype: usize,
}

/// Result report for build search. Damage totals are absent when no supported
/// legality-backed search model is available.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildSearchResult {
    #[serde(flatten)]
    pub readiness: ScenarioReadiness,
    pub schema_version: u32,
    pub content_id: String,
    pub total_variants: usize,
    pub total_archetypes: usize,
    pub total_characters: usize,
    pub character_builds: BTreeMap<String, CharacterBuildProfile>,
    pub variant_comparisons: Vec<VariantComparisonResult>,
    pub archetype_rankings: Vec<ArchetypeRefinement>,
    pub total_baseline_damage: Option<f64>,
    pub total_optimized_damage: Option<f64>,
    pub total_percentage_gain: Option<f64>,
    pub execution_status: String,
}
