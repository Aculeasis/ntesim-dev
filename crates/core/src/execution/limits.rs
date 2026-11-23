use serde::{Deserialize, Serialize};

fn default_max_events() -> u64 {
    100_000
}
fn default_max_horizon_us() -> u64 {
    3_600_000_000
}
fn default_max_proc_depth() -> u32 {
    4
}
fn default_max_pool_capacity() -> u64 {
    1_000_000_000
}
fn default_max_active_shields() -> usize {
    32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionLimits {
    #[serde(default = "default_max_events")]
    pub max_events: u64,
    #[serde(default = "default_max_horizon_us")]
    pub max_horizon_us: u64,
    #[serde(default = "default_max_proc_depth")]
    pub max_proc_depth: u32,
    #[serde(default = "default_max_pool_capacity")]
    pub max_pool_capacity: u64,
    #[serde(default = "default_max_active_shields")]
    pub max_active_shields: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_events: default_max_events(),
            max_horizon_us: default_max_horizon_us(),
            max_proc_depth: default_max_proc_depth(),
            max_pool_capacity: default_max_pool_capacity(),
            max_active_shields: default_max_active_shields(),
        }
    }
}

/// Strict causal priority for deterministic tie-breaking of simultaneous events.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum CausalPriority {
    ModePause = 0,
    ActionAdmission = 1,
    ResourceCost = 2,
    BuffTickOrExpiry = 3,
    HitDelivery = 4,
    ReactionOrBreak = 5,
    ProcCallback = 6,
    SettlementAndDeath = 7,
    Observation = 8,
}
