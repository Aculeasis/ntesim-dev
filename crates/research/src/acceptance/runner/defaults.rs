use super::*;

pub fn default_acceptance_request() -> AcceptanceRunRequest {
    AcceptanceRunRequest {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        hypothesis: DEFAULT_ACCEPTANCE_HYPOTHESIS.into(),
        scenarios: vec![
            AcceptanceScenarioKind::SingleTargetReal120,
            AcceptanceScenarioKind::FourTargetReal120,
        ],
        max_compositions_to_evaluate: None,
        screening_seeds: vec![42],
        finalist_seeds: vec![101, 102, 103],
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
        top_candidates_per_scenario: 50,
        worker_config: AcceptanceWorkerConfig::default(),
        resume_checkpoint: None,
    }
}
