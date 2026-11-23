use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) cadenza_duration_us: u64,
    pub(super) a2_extension_us: u64,
    pub(super) a3_burst_damage_bonus: ExactFraction,
    pub(super) awakening_crit_rate_bonus: ExactFraction,
    pub(super) resonance_psyche_damage_bonus: ExactFraction,
    pub(super) resonance_duration_us: u64,
}

// Global 2026-10-04 DT_MitsukiEffectFigure; only consumed owner parameters.
// SkillBuffDur, Star2_Dur, Star3_DmgUp, Star1/5_CritUp and
// Star6extra_DmgUp/Star6exra_DmgUpDur are constant rows. Existing Cadenza
// extension/cap and packet categories remain unchanged; timers use microseconds.
pub(super) const PARAMETERS: Parameters = Parameters {
    cadenza_duration_us: 12000000,
    a2_extension_us: 5000000,
    a3_burst_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    awakening_crit_rate_bonus: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    resonance_psyche_damage_bonus: ExactFraction {
        numerator: 2,
        denominator: 25,
    },
    resonance_duration_us: 15000000,
};
