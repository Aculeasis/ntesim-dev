use super::*;

/// Quality of the current fixed-timeline previews, independent of execution success.
/// There is no complete evidence-bound game adapter behind this harness yet.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScenarioReadiness {
    pub status: String,
    pub quality: Quality,
    pub game_dps_computed: bool,
    pub acceptance_ready: bool,
    pub blockers: Vec<String>,
}

pub fn preview_readiness() -> ScenarioReadiness {
    ScenarioReadiness {
        status: "INCOMPLETE".into(),
        quality: Quality::Incomplete,
        game_dps_computed: false,
        acceptance_ready: false,
        blockers: [
            "Complete evidence-bound character, awakening and equipment adapters are unavailable",
            "Fixed 20-second previews use unverified hit, recovery, cooldown and switch timings",
            "Build search checks four cartridge mains with a fixed substat allocation; console geometry and recommended gear are not compiled",
            "Damage settlement is non-critical and does not consume the request seed",
            "120-second one/four-target, break/no-break and game-mode timer scenarios are missing",
            "All-composition search, native pilot, 1000-seed finalists and subsequent continuous-state cycles remain pending",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    }
}
