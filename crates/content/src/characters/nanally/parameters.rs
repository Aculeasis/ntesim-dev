use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) a4_underboss_damage_bonus: ExactFraction,
    pub(super) resonance_damage_bonus: ExactFraction,
    pub(super) authority_crit_damage: ExactFraction,
    pub(super) a5_atk_per_stack: ExactFraction,
    pub(super) a1_charge_basis: ExactFraction,
    pub(super) a1_interval_us: u64,
    pub(super) a3_interval_us: u64,
    pub(super) fair_duel_interval_us: u64,
    pub(super) a5_duration_us: u64,
    pub(super) authority_duration_us: u64,
    pub(super) a6_authority_duration_us: u64,
    pub(super) a6_out_of_combat_authority_duration_us: u64,
    pub(super) underboss_duration_us: u64,
    pub(super) a2_underboss_duration_us: u64,
}

// Global 2026-10-04 DT_NanallyEffectFigure; only consumed owner parameters.
// Constant CatDur/Lv2_CatDur, Lv4_DamUp, Lv5_AtkUp, LvExtraFarTarget,
// Pas2Charge/Pas2CD, WalkDur/Lv6_BattleWalkDur/Lv6_NoBattleWalkDur and CritDamageUp. A3 ICD
// and A5 duration retain their accepted models; generation/receipt ownership stays separate.
pub(super) const PARAMETERS: Parameters = Parameters {
    a4_underboss_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 1,
    },
    resonance_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    authority_crit_damage: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    a5_atk_per_stack: ExactFraction {
        numerator: 1,
        denominator: 50,
    },
    a1_charge_basis: ExactFraction {
        numerator: 5,
        denominator: 2,
    },
    a1_interval_us: 1000000,
    a3_interval_us: 1000000,
    fair_duel_interval_us: 2000000,
    a5_duration_us: 15000000,
    authority_duration_us: 12000000,
    a6_authority_duration_us: 15000000,
    a6_out_of_combat_authority_duration_us: 20000000,
    underboss_duration_us: 6000000,
    a2_underboss_duration_us: 9000000,
};
