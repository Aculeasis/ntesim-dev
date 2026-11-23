use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) suppress_us: u64,
    pub(super) a1_suppress_us: u64,
    pub(super) control_def_down: crate::formula::Ratio,
    pub(super) control_def_down_us: u64,
    pub(super) defeat_damage_per_stack: ExactFraction,
    pub(super) defeat_duration_us: u64,
    pub(super) team_base_atk_ratio: ExactFraction,
    pub(super) a4_base_atk_ratio: ExactFraction,
    pub(super) team_atk_duration_us: u64,
    pub(super) damage_per_debuff: ExactFraction,
    pub(super) debuff_limit: usize,
}

// Global 2026-10-04 DT_SagiriEffectFigure: Sagiri_Lv3_DamUp/DamUpDur,
// Sagiri_QDamUp_AtkAdd/AtkAddDur and Sagiri_lvExtra_Damup/DamupMax.
// Suppress/Sagiri_lv1_Suppress and Sagiri_Pas1_DefUpAdd/DefUpAddDur
// additionally supply the successful-control path.
// Constant rows, with seconds converted exactly to microseconds. The .12 cap
// divided by .03 per debuff permits four contributions. Recipients and the
// separate ten-stack defeat policy remain owned by the existing runtime model.
pub(super) const PARAMETERS: Parameters = Parameters {
    suppress_us: 3_000_000,
    a1_suppress_us: 6_000_000,
    control_def_down: crate::formula::Ratio {
        numerator: 1,
        denominator: 10,
    },
    control_def_down_us: 20_000_000,
    defeat_damage_per_stack: ExactFraction {
        numerator: 3,
        denominator: 50,
    },
    defeat_duration_us: 15_000_000,
    team_base_atk_ratio: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    a4_base_atk_ratio: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    team_atk_duration_us: 20_000_000,
    damage_per_debuff: ExactFraction {
        numerator: 3,
        denominator: 100,
    },
    debuff_limit: 4,
};
