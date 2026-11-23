use super::actor::ActorLifecycle;
use super::model::{
    ActionPhaseTimings, ActorVitality, CancelReceipt, CancelType, FieldStatus, InterruptionCause,
    InterruptionReceipt, LifecycleError, PhaseTransition, RefundPolicy, SwitchRecord,
};
use crate::state::Pool;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod field;
pub use field::TeamFieldState;
mod admission;
pub use admission::TeamLifecycle;
