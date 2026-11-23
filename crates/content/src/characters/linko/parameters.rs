use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) a2_burst_ratio_add: ExactFraction,
    pub(super) a5_skill_damage_bonus: ExactFraction,
    pub(super) resonance_atk_percent: ExactFraction,
    pub(super) a6_skill_crit_rate: ExactFraction,
    pub(super) resonance_field_damage_bonus: ExactFraction,
    pub(super) field_duration_us: u64,
    pub(super) resonance_field_duration_us: u64,
}

// Global 2026-10-04 DT_LinkoEffectFigure; only consumed owner parameters.
// Constant Level3_DamageUpGeneralBase, Level6_CritBase, Extra1_AtkUp and
// Level_Extra2_Ling/Zhou/Duration plus the accepted base 8s field and A2 model.
// Categories, coefficient additions, recipients and microsecond timers are preserved.
pub(super) const PARAMETERS: Parameters = Parameters {
    a2_burst_ratio_add: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    a5_skill_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 2,
    },
    resonance_atk_percent: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    a6_skill_crit_rate: ExactFraction {
        numerator: 1,
        denominator: 4,
    },
    resonance_field_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    field_duration_us: 8000000,
    resonance_field_duration_us: 13000000,
};
