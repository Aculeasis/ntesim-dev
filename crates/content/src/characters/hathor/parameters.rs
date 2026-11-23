use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) emergency_atk_percent: ExactFraction,
    pub(super) a3_damage_bonus: ExactFraction,
    pub(super) a5_damage_per_power: ExactFraction,
    pub(super) a6_emergency_crit_damage: ExactFraction,
    pub(super) resonance_defense_ignore: ExactFraction,
    pub(super) emergency_duration_us: u64,
}

// Global 2026-10-04 DT_HathorEffectFigure; only consumed owner parameters.
// AngerActive_AtkUp/CritDmgUp/DefIgnore, Level3_DmgUp and DmgUpLevel5
// remain in their separate zones. Existing Emergency Delivery duration, Power
// cap and action applicability are preserved; unused source branches are omitted.
pub(super) const PARAMETERS: Parameters = Parameters {
    emergency_atk_percent: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    a3_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    a5_damage_per_power: ExactFraction {
        numerator: 1,
        denominator: 100,
    },
    a6_emergency_crit_damage: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    resonance_defense_ignore: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
    emergency_duration_us: 5000000,
};
