use crate::{
    combat::{BreakDamageRequest, NegativeResistanceModel, break_damage},
    formula::Ratio,
    state::Pool,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod constants;
pub use constants::{
    BLOSSOM_BASE_DAMAGE, DEFAULT_BREAK_DURATION_US, NOVA_BASE_DAMAGE, SCORCH_BASE_DAMAGE,
    SCORCH_DURATION_US, SCORCH_MAX_STACKS, SCORCH_TICK_INTERVAL_US,
};
mod targeting;
pub use targeting::{Point3D, TargetingRule};
mod model;
pub use model::{ActorCombatProfile, BreakState, ReactionKind, ScorchInstance, TargetProfile};
mod receipts;
pub use receipts::{BreakEventReceipt, BreakHitContribution, HitDeliveryReceipt, ReactionReceipt};
mod runtime;
pub use runtime::ReactionEngine;
mod scenario;
pub use scenario::{
    ActorProfileInput, ReactionScenarioRequest, ReactionScenarioStep, TargetProfileInput,
    evaluate_reaction_scenario,
};

#[cfg(test)]
mod tests;
