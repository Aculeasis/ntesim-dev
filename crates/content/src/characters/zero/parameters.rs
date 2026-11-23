use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) execution_defense_ignore: ExactFraction,
    pub(super) skill_energy: u64,
    pub(super) burst_crit_rate: ExactFraction,
    pub(super) burst_damage_per_base_atk: ExactFraction,
    pub(super) burst_damage_cap: ExactFraction,
    pub(super) support_atk_percent: ExactFraction,
    pub(super) support_atk_duration_us: u64,
    pub(super) cosmos_cycle_atk_percent: ExactFraction,
    pub(super) cosmos_cycle_atk_duration_us: u64,
}

// Global 2026-10-04 DT_Female051EffectFigure Female051_LV1/2/3/4_2/4_3/5.
// Constant rows, consumed by the existing A1-A5 owner callbacks. The A5 20s
// duration remains the accepted runtime model, not an inferred R3 curve binding.
pub(super) const PARAMETERS: Parameters = Parameters {
    execution_defense_ignore: ExactFraction {
        numerator: 3,
        denominator: 4,
    },
    skill_energy: 8,
    burst_crit_rate: ExactFraction {
        numerator: 1,
        denominator: 2,
    },
    burst_damage_per_base_atk: ExactFraction {
        numerator: 1,
        denominator: 1000,
    },
    burst_damage_cap: ExactFraction {
        numerator: 1,
        denominator: 4,
    },
    support_atk_percent: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
    support_atk_duration_us: 20_000_000,
    // Female051_Extra1_1 and Female051_Extra1_2.
    cosmos_cycle_atk_percent: ExactFraction {
        numerator: 3,
        denominator: 25,
    },
    cosmos_cycle_atk_duration_us: 20_000_000,
};
