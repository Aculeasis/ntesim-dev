use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) a1_blood_ratio_add: ExactFraction,
    pub(super) resonance_defense_ignore: ExactFraction,
    pub(super) a2_damage_bonus: ExactFraction,
    pub(super) a2_duration_us: u64,
}

// Global 2026-10-04 DT_LacrimosaEffectFigure; only consumed owner parameters.
// Constant lvExtra_DefIgnore and lv2_Damup/DamupDur plus the accepted A1
// coefficient model. Blood durations/stacks/period already have typed consumed
// constants in dot.rs; its stack lifecycle and remaining conflicts are preserved.
pub(super) const PARAMETERS: Parameters = Parameters {
    a1_blood_ratio_add: ExactFraction {
        numerator: 1,
        denominator: 2,
    },
    resonance_defense_ignore: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    a2_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    a2_duration_us: 15000000,
};
