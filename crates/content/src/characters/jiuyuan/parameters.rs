use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) a1_atk_per_bound_target: ExactFraction,
    pub(super) a5_pact_damage_bonus: ExactFraction,
    pub(super) resonance_resistance_reduction: ExactFraction,
    pub(super) pact_duration_us: u64,
}

// Global 2026-10-04 DT_KuharaEffectFigure; only consumed owner parameters.
// Seed_AtkUp, DmgUp_L5, DmgResistDownNature and Bud_Duration are constant
// rows. The resistance API takes a positive reduction magnitude; seconds become
// microseconds. Existing bound-target cap and Pact recipients remain unchanged.
pub(super) const PARAMETERS: Parameters = Parameters {
    a1_atk_per_bound_target: ExactFraction {
        numerator: 1,
        denominator: 20,
    },
    a5_pact_damage_bonus: ExactFraction {
        numerator: 2,
        denominator: 25,
    },
    resonance_resistance_reduction: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
    pact_duration_us: 25000000,
};
