use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) capacity_multiplier: ExactFraction,
    pub(super) normal_acquisition_multiplier: ExactFraction,
    pub(super) homecoming_resistance_reduction: ExactFraction,
    pub(super) homecoming_duration_us: u64,
    pub(super) coin_damage_bonus: ExactFraction,
    pub(super) debt_acquisition_bonus: ExactFraction,
    pub(super) market_duration_us: u64,
    pub(super) hoarding_efficiency_add: ExactFraction,
    pub(super) hanahaki_charge_add: u64,
    pub(super) surplus_interest: ExactFraction,
    pub(super) interest_interval_us: u64,
}

// Global 2026-10-04 DT_ChiichanEffectFigure Chiichan_Level1/4/Extra2 and
// UltraSkill_Dur_Show. Additive .10/.20 to the identity becomes 1.10/1.20;
// the target-reduction API takes the magnitude of source -.10. Market duration
// is consumed by metadata and the admitted finite financial market clock.
pub(super) const PARAMETERS: Parameters = Parameters {
    capacity_multiplier: ExactFraction {
        numerator: 11,
        denominator: 10,
    },
    normal_acquisition_multiplier: ExactFraction {
        numerator: 6,
        denominator: 5,
    },
    homecoming_resistance_reduction: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
    homecoming_duration_us: 50_000,
    debt_acquisition_bonus: ExactFraction {
        numerator: 1,
        denominator: 2,
    },
    coin_damage_bonus: ExactFraction {
        numerator: 15,
        denominator: 100,
    },
    market_duration_us: 7_000_000,
    // Chiichan_Pas2 additive ChargeGetEfficiencyBase; selected Hanahaki
    // description's extra raw Charge, consumed only by a real Charge event.
    hoarding_efficiency_add: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    hanahaki_charge_add: 4,
    // Selected A2 description:10% per real second. The source.5 update is
    // bookkeeping, not two grants per second or per-half compounding.
    surplus_interest: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
    interest_interval_us: 1_000_000,
};
