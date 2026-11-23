use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) insight_damage_bonus: ExactFraction,
    pub(super) phantom_step_damage_bonus: ExactFraction,
    pub(super) defense_ignore: ExactFraction,
    pub(super) self_insight_duration_us: u64,
    pub(super) target_insight_duration_us: u64,
    pub(super) a3_target_insight_duration_us: u64,
    pub(super) broken_resistance_reduction: ExactFraction,
    pub(super) skill_damage_per_resonance: ExactFraction,
    pub(super) a1_skill_damage_per_resonance: ExactFraction,
    pub(super) skill_break_efficiency_per_resonance: ExactFraction,
    pub(super) a1_skill_break_efficiency_per_resonance: ExactFraction,
    pub(super) break_damage_bonus: ExactFraction,
    pub(super) a4_break_damage_per_insight: ExactFraction,
}

// Global 2026-10-04 DT_DaffodillEffectFigure; only consumed owner parameters.
// Consumed Insight lifetimes/bonuses, skill Resonance capture, Break modifiers
// and Broken-linked resistance retain the accepted owner model. Target Insight
// 45s/9999s is separate from self Eyes 30s; source negative RES becomes magnitude.
pub(super) const PARAMETERS: Parameters = Parameters {
    insight_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
    phantom_step_damage_bonus: ExactFraction {
        numerator: 4,
        denominator: 5,
    },
    defense_ignore: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    self_insight_duration_us: 30000000,
    target_insight_duration_us: 45000000,
    a3_target_insight_duration_us: 9999000000,
    broken_resistance_reduction: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
    skill_damage_per_resonance: ExactFraction {
        numerator: 1,
        denominator: 2,
    },
    a1_skill_damage_per_resonance: ExactFraction {
        numerator: 1,
        denominator: 1,
    },
    skill_break_efficiency_per_resonance: ExactFraction {
        numerator: 5,
        denominator: 4,
    },
    a1_skill_break_efficiency_per_resonance: ExactFraction {
        numerator: 5,
        denominator: 2,
    },
    break_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    a4_break_damage_per_insight: ExactFraction {
        numerator: 3,
        denominator: 20,
    },
};
