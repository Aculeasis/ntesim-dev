use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) a1_damage_per_recorded_skill: ExactFraction,
    pub(super) a2_time_stop_damage_bonus: ExactFraction,
    pub(super) a6_time_stop_defense_ignore: ExactFraction,
    pub(super) a5_single_target_damage_bonus: ExactFraction,
    pub(super) resonance_team_atk_percent: ExactFraction,
}

// Global 2026-10-04 DT_JinEffectFigure; only consumed owner parameters.
// Constant Jin_lv1_Damup/lv2_TimeDamUp/lv6_TimeDef/lv5_SingleDamUp.
// The 120/10 resource model and its exact transaction clocks are typed in
// timepiece.rs. R6 uses the bound Jin_lvExtra_AtkUp 0.30 row, corroborated by
// the current public description.
pub(super) const PARAMETERS: Parameters = Parameters {
    a1_damage_per_recorded_skill: ExactFraction {
        numerator: 3,
        denominator: 25,
    },
    a2_time_stop_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    a6_time_stop_defense_ignore: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    a5_single_target_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    resonance_team_atk_percent: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
};
