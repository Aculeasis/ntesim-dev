use crate::{formula::Ratio, state::Pool};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod runtime;
pub use runtime::{DEFAULT_MAX_DISPATCH_DEPTH, DispatchEngine};
mod icd;
pub use icd::{
    IcdEntry, IcdRule, IcdScope, IcdTracker, STANDARD_ICD_HIT_INTERVAL, STANDARD_ICD_WINDOW_US,
};
mod modifiers;
pub use modifiers::{
    ActiveBuffInstance, BuffDefinition, BuffDuration, BuffManager, BuffRecipient,
    BuffSourceLifetime, BuffStackingScope, DecayPolicy, RefreshPolicy, TargetRecipient,
};
mod callbacks;
pub use callbacks::{
    CallbackRegistration, CombatEvent, DispatchError, DispatchReceipt, ProcCondition, ProcPayload,
    ProcReceipt,
};
mod scenario;
pub use scenario::{DispatchScenarioRequest, DispatchStep, PoolInput, evaluate_dispatch_scenario};

#[cfg(test)]
mod tests;
