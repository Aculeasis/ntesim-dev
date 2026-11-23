use super::*;

pub(crate) fn default_scenario_kind() -> AcceptanceScenarioKind {
    AcceptanceScenarioKind::SingleTargetReal120
}

pub(crate) fn default_timing_multipliers() -> Vec<Ratio> {
    vec![
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
    ]
}
