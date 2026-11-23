//! Synthetic fixtures only; no game models or inferred NTE mechanics.
use crate::{
    action::{ActionState, Gate, Rule},
    formula::{Formula, Ratio},
    rng::{self, Rng},
    scheduler::{ClockId, ClockReading, Scheduler},
    state::{Pool, ShieldOrder, VitalState},
};
use ntesim_contracts::{Diagnostic, ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

mod model;
pub use model::{
    ActorActionInput, ActorInput, ClockInput, Command, Effect, Fixture, PoolInput, TargetInput,
    WeightedDamage,
};
mod tasks;
pub(crate) use tasks::Task;
mod receipts;
pub(crate) use receipts::Observation;
pub use receipts::{ActionOrigin, Result, Trace};
mod runtime;
pub(crate) use runtime::execute;
pub use runtime::run;
mod validation;
pub(crate) use validation::validate_leaf;
mod actions;
pub(crate) use actions::{Invocation, invoke};
mod random;
pub(crate) use random::{RandomHit, random_hit, total_weight};

pub mod rotation;
#[cfg(test)]
mod tests;
