use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AcceptanceConditionsSummary {
    pub duration_s: u64,
    pub opening_duration_s: u64,
    pub target_counts: Vec<usize>,
    pub enemy_level: u32,
    pub neutral_resistance: f64,
    pub enemy_defense: u64,
    pub input_margin_s: f64,
    pub substats_budget: usize,
    pub random_seeds_count: usize,
    pub timing_sensitivities: Vec<f64>,
    pub top_n_refinements_per_scenario: usize,
    pub superiority_threshold: f64,
}

impl Default for AcceptanceConditionsSummary {
    fn default() -> Self {
        Self {
            duration_s: 120,
            opening_duration_s: 20,
            target_counts: vec![1, 4],
            enemy_level: 80,
            neutral_resistance: 0.20,
            enemy_defense: 1080,
            input_margin_s: 0.25,
            substats_budget: 15,
            random_seeds_count: 1000,
            timing_sensitivities: vec![1.0, 1.1, 1.2],
            top_n_refinements_per_scenario: 50,
            superiority_threshold: 0.05,
        }
    }
}
