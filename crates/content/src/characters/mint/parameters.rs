use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) skill_damage_bonus: ExactFraction,
    pub(super) single_target_crit_rate: ExactFraction,
    pub(super) skill_crit_damage: ExactFraction,
    pub(super) skill_crit_duration_us: u64,
    pub(super) low_hp_threshold: ExactFraction,
    pub(super) low_hp_damage_bonus: ExactFraction,
    pub(super) resonance_energy: u64,
    pub(super) entry_def_bonus: ExactFraction,
    pub(super) a1_guard_duration_us: u64,
}

// Global 2026-10-04 DT_MintEffectFigure: Mint_Adcance_2_1,
// 4_1, 5_1, 5_2, 6_1, 6_2 and 6_AddEnergy. These rows are constant;
// only parameters consumed by the existing bounded owner model are retained.
// The strict-below HP predicate and energy channel remain that model's choices.
pub(super) const PARAMETERS: Parameters = Parameters {
    skill_damage_bonus: ExactFraction {
        numerator: 2,
        denominator: 5,
    },
    single_target_crit_rate: ExactFraction {
        numerator: 3,
        denominator: 25,
    },
    skill_crit_damage: ExactFraction {
        numerator: 1,
        denominator: 4,
    },
    skill_crit_duration_us: 6_000_000,
    low_hp_threshold: ExactFraction {
        numerator: 2,
        denominator: 5,
    },
    low_hp_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    resonance_energy: 2,
    entry_def_bonus: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    a1_guard_duration_us: 3_000_000,
};
