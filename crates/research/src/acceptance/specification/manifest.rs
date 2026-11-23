use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcceptanceRunState {
    pub status: String,
    pub composition_enumeration_complete: bool,
    pub compositions: usize,
    pub published_archetypes: usize,
    pub published_variants: usize,
    pub game_simulations: usize,
    pub rotation_search_performed: bool,
    pub top50_refinements: usize,
    pub finalist_monte_carlo_runs: usize,
    pub unexpected_team_found: Option<String>,
    pub unknowns_as_zero: bool,
}

impl Default for AcceptanceRunState {
    fn default() -> Self {
        Self {
            status: "INCOMPLETE".into(),
            composition_enumeration_complete: false,
            compositions: 0,
            published_archetypes: 0,
            published_variants: 0,
            game_simulations: 0,
            rotation_search_performed: false,
            top50_refinements: 0,
            finalist_monte_carlo_runs: 0,
            unexpected_team_found: None,
            unknowns_as_zero: false,
        }
    }
}

pub fn generate_run_manifest(conditions: &Value, state: &AcceptanceRunState) -> Value {
    json!({
        "status": state.status,
        "composition_enumeration_complete": state.composition_enumeration_complete,
        "compositions": state.compositions,
        "published_archetypes": state.published_archetypes,
        "published_variants": state.published_variants,
        "game_simulations": state.game_simulations,
        "rotation_search_performed": state.rotation_search_performed,
        "top50_refinements": state.top50_refinements,
        "finalist_monte_carlo_runs": state.finalist_monte_carlo_runs,
        "unexpected_team_found": state.unexpected_team_found,
        "unknowns_as_zero": state.unknowns_as_zero,
        "conditions": conditions
    })
}
