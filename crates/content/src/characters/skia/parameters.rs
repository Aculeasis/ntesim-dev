use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) a3_single_mark_damage_bonus: ExactFraction,
    pub(super) a3_multiple_mark_damage_bonus: ExactFraction,
    pub(super) a1_shadow_skill_ratio_add: ExactFraction,
    pub(super) a2_crit_rate: ExactFraction,
    pub(super) a2_duration_us: u64,
    pub(super) resonance_resistance_reduction: ExactFraction,
    pub(super) mark_duration_us: u64,
    pub(super) a4_mark_duration_us: u64,
}

// Global 2026-10-04 DT_SkiaEffectFigure; only consumed owner parameters.
// Leve1CoefModify, Leve2CritBase/Duaration, Leve3DamageUpGeneralBase1/2
// and LevelExtraDamageResistLakshanaBase retain the existing conditional model:
// sole mark sums two .04 contributions, multiple marks use one; lifetimes stay owner-bound.
pub(super) const PARAMETERS: Parameters = Parameters {
    a3_single_mark_damage_bonus: ExactFraction {
        numerator: 2,
        denominator: 25,
    },
    a3_multiple_mark_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 25,
    },
    a1_shadow_skill_ratio_add: ExactFraction {
        numerator: 3,
        denominator: 5,
    },
    a2_crit_rate: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
    a2_duration_us: 15000000,
    resonance_resistance_reduction: ExactFraction {
        numerator: 1,
        denominator: 50,
    },
    mark_duration_us: 10000000,
    a4_mark_duration_us: 12000000,
};
