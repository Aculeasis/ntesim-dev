use super::*;

pub fn default_pilot_request() -> PilotRequest {
    PilotRequest {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        hypothesis: DEFAULT_PILOT_HYPOTHESIS.into(),
        teams: vec![
            "mint+sakiri+zero+iroi".into(),
            "blackbird+daffodil+haniel+sakiri".into(),
            "aurelia+baicang+daffodil+sakiri".into(),
        ],
        scenarios: vec![
            AcceptanceScenarioKind::SingleTargetReal120,
            AcceptanceScenarioKind::FourTargetReal120,
            AcceptanceScenarioKind::NoBreakReal120,
        ],
        timing_multipliers: vec![
            Ratio {
                numerator: 1,
                denominator: 1,
            },
            Ratio {
                numerator: 11,
                denominator: 10,
            },
            Ratio {
                numerator: 6,
                denominator: 5,
            },
        ],
        seeds: vec![42, 43],
        budget: PilotBudget::default(),
        resume_checkpoint: None,
        actor_overrides: None,
    }
}
