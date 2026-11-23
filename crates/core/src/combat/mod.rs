use crate::formula::Rounding;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

mod policies;
pub use policies::{CritRule, DamageKind, NegativeResistanceModel, SnapshotPolicy, StackingPolicy};
mod errors;
pub use errors::CombatError;
mod arithmetic;
pub mod exact;
pub use arithmetic::{
    crit_factor_expected, crit_factor_stochastic, cycle_effect_zone, cycle_intensity_zone,
    defense_base, defense_factor, hexed_fraction, mitigate_integer_damage, quantize,
    resistance_factor, stain_factor,
};
mod break_damage;
pub use break_damage::{BreakDamageRequest, break_damage};
pub use break_damage::{ExactBreakDamageRequest, exact_break_damage};
mod stats;
pub use stats::DynamicStats;
mod modifiers;
pub use modifiers::{Buff, DynamicBuffAmountRule};
mod damage;
pub use damage::{
    DirectDamageBreakdown, DirectDamageEvaluationRequest, DirectHitSpec, EvaluationCritMode,
    evaluate_direct_damage,
};
mod defaults;
pub(crate) use defaults::{
    default_crit_rule, default_damage_kind, default_element, default_one, default_rounding,
    default_stat,
};
mod healing;
pub use healing::{HealingBreakdown, HealingSpec, evaluate_healing};
mod shields;
pub use shields::{ShieldBreakdown, ShieldSpec, evaluate_shield};
mod request;
pub use request::{Request, evaluate_request};

#[cfg(test)]
mod formula_contract_tests;
pub mod reaction;
#[cfg(test)]
mod tests;
