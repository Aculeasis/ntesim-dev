use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) hunt_damage_bonus: ExactFraction,
    pub(super) a1_hunt_damage_bonus: ExactFraction,
    pub(super) a2_final_multiplier: ExactFraction,
    pub(super) a6_damage_per_dot_stack: ExactFraction,
    pub(super) a6_dot_stack_cap: u32,
    pub(super) resonance_burst_ratio_add: ExactFraction,
    pub(super) resonance_atk_percent: ExactFraction,
    pub(super) resonance_duration_us: u64,
    pub(super) metadata_duration_us: u64,
    pub(super) a1_team_dot_crit_damage: crate::formula::Ratio,
    pub(super) self_cycle_intensity: crate::formula::Ratio,
    pub(super) combat_entry_cycle: u64,
    pub(super) skill_primary_delay_us: u64,
    pub(super) skill_recovery_us: u64,
    pub(super) skill_cooldown_us: u64,
}

// Global 2026-10-04 DT_ZankouEffectFigure; only consumed owner parameters.
// Constant Hunt, final multiplier, DoT-stack bonus, resonance ATK and team DoT
// crit rows. .12 per stack capped at twenty preserves the source 2.40 maximum.
// Metadata 8s and Vile Ash typed constants/lifecycle retain their accepted models.
pub(super) const PARAMETERS: Parameters = Parameters {
    hunt_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 4,
    },
    a1_hunt_damage_bonus: ExactFraction {
        numerator: 2,
        denominator: 5,
    },
    a2_final_multiplier: ExactFraction {
        numerator: 3,
        denominator: 2,
    },
    a6_damage_per_dot_stack: ExactFraction {
        numerator: 3,
        denominator: 25,
    },
    a6_dot_stack_cap: 20,
    resonance_burst_ratio_add: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    resonance_atk_percent: ExactFraction {
        numerator: 2,
        denominator: 5,
    },
    resonance_duration_us: 20000000,
    metadata_duration_us: 8000000,
    a1_team_dot_crit_damage: crate::formula::Ratio {
        numerator: 1,
        denominator: 2,
    },
    // DT_ZankouEffectFigure.ZankouMagUpPas2 (stat points, not a percentage).
    self_cycle_intensity: crate::formula::Ratio {
        numerator: 100,
        denominator: 1,
    },
    combat_entry_cycle: 100,
    // Current rolling source: entry montage and real cooldown at admission.
    skill_primary_delay_us: 122_519,
    skill_recovery_us: 998_950,
    skill_cooldown_us: 5_000_000,
};
