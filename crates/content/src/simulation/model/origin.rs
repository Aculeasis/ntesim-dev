use super::ExactFraction;
use serde::{Deserialize, Serialize};

/// Actual causal trigger; an autonomous packet is never an authored input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameCausalTrigger {
    InitialState,
    FieldEntry {
        command_index: usize,
    },
    Action {
        command_index: usize,
    },
    IncomingDamage {
        event_index: usize,
    },
    HpCost {
        event_index: usize,
    },
    VitalChange {
        event_sequence: u64,
    },
    OwnerDeadline {
        autonomous_event_index: usize,
    },
    CoordinatedHit {
        cast_command_index: usize,
        hit_index: usize,
    },
    SettledHit {
        hit_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GameAutonomousHealing {
    pub recipient_actor_index: usize,
    pub base: ExactFraction,
    pub gross: u64,
    pub applied: u64,
    pub excess: u64,
    pub hp_before: u64,
    pub hp_after: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GameAutonomousEventReceipt {
    pub actor_index: usize,
    pub active_actor_index: usize,
    pub ability_id: String,
    pub time_us: u64,
    pub trigger: GameCausalTrigger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_healing: Option<GameAutonomousHealing>,
}
