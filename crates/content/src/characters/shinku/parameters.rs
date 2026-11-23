use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) rage_damage_bonus: ExactFraction,
    pub(super) resonance_rage_damage_bonus: ExactFraction,
    pub(super) a6_rage_skill_ratio_add: ExactFraction,
    pub(super) resonance_crit_rate: ExactFraction,
    pub(super) a1_defense_ignore: ExactFraction,
    pub(super) resonance_defense_ignore: ExactFraction,
    pub(super) rage_duration_us: u64,
    pub(super) a2_cosmos_resistance_reduction: ExactFraction,
    pub(super) a2_resistance_duration_us: u64,
}

// Global 2026-10-04 DT_ShinkuEffectFigure; only consumed owner parameters.
// Constant Rage_DmgUp/Rage_DmgUpEx_L3, RageSkillDmgCoefL6, CritUp_L6Ex,
// DefIgnore_L1/DefIgnore_L6Extra, Rage_Duration and DmgResistDownCosmo/Dur.
// Damage, coefficient and defense zones stay separate; refresh remains the tested model.
pub(super) const PARAMETERS: Parameters = Parameters {
    rage_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    resonance_rage_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    a6_rage_skill_ratio_add: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    resonance_crit_rate: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
    a1_defense_ignore: ExactFraction {
        numerator: 3,
        denominator: 25,
    },
    resonance_defense_ignore: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    rage_duration_us: 13000000,
    a2_cosmos_resistance_reduction: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
    a2_resistance_duration_us: 20000000,
};
