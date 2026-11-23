use crate::formula::Ratio;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldStatus {
    OnField,
    OffField,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorVitality {
    Alive,
    Dead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPhaseKind {
    Idle,
    Startup,
    Active,
    Recovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelType {
    Dash,
    Jump,
    Switch,
    Skill,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptionCause {
    Stagger,
    Knockdown,
    Death,
    ForcedAbort,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "snake_case")]
pub enum RefundPolicy {
    FullRefund,
    PartialRefund { ratio: Ratio },
    NoRefund,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CancelWindow {
    pub cancel_type: CancelType,
    pub earliest_offset_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPhaseTimings {
    pub startup_us: u64,
    pub active_us: u64,
    pub recovery_us: u64,
    pub recovery_cancels: Vec<CancelWindow>,
    pub startup_cancels: Vec<CancelWindow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "phase", rename_all = "snake_case")]
pub enum ActionPhase {
    Idle,
    Startup {
        action_id: String,
        started_at_us: u64,
        ends_at_us: u64,
    },
    Active {
        action_id: String,
        started_at_us: u64,
        ends_at_us: u64,
    },
    Recovery {
        action_id: String,
        started_at_us: u64,
        ends_at_us: u64,
    },
}

impl ActionPhase {
    pub fn kind(&self) -> ActionPhaseKind {
        match self {
            Self::Idle => ActionPhaseKind::Idle,
            Self::Startup { .. } => ActionPhaseKind::Startup,
            Self::Active { .. } => ActionPhaseKind::Active,
            Self::Recovery { .. } => ActionPhaseKind::Recovery,
        }
    }

    pub fn action_id(&self) -> Option<&str> {
        match self {
            Self::Idle => None,
            Self::Startup { action_id, .. }
            | Self::Active { action_id, .. }
            | Self::Recovery { action_id, .. } => Some(action_id.as_str()),
        }
    }

    pub fn ends_at_us(&self) -> Option<u64> {
        match self {
            Self::Idle => None,
            Self::Startup { ends_at_us, .. }
            | Self::Active { ends_at_us, .. }
            | Self::Recovery { ends_at_us, .. } => Some(*ends_at_us),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SwitchRecord {
    pub from_index: usize,
    pub to_index: usize,
    pub timestamp_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PhaseTransition {
    pub actor_index: usize,
    pub action_id: String,
    pub from_kind: ActionPhaseKind,
    pub to_kind: ActionPhaseKind,
    pub timestamp_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CancelReceipt {
    pub actor_index: usize,
    pub action_id: String,
    pub phase: ActionPhaseKind,
    pub cancel_type: CancelType,
    pub timestamp_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct InterruptionReceipt {
    pub actor_index: usize,
    pub action_id: String,
    pub from_phase: ActionPhaseKind,
    pub cause: InterruptionCause,
    pub timestamp_us: u64,
    pub refunded_costs: std::collections::BTreeMap<String, u64>,
    /// Capacity-clipped part of the requested refund, settled by TeamLifecycle.
    #[serde(default)]
    pub refund_excess: std::collections::BTreeMap<String, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormInstance {
    pub form_id: String,
    pub entered_at_us: u64,
    pub expires_at_us: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecastInstance {
    pub base_action: String,
    pub recast_action: String,
    pub opened_at_us: u64,
    pub expires_at_us: u64,
    pub uses_remaining: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum LifecycleError {
    InvalidRefundRatio,
    DeadlineOverflow,
    InvalidTeamSize(usize),
    ActorIndexOutOfBounds {
        index: usize,
        max: usize,
    },
    CharacterCountMismatch {
        expected: usize,
        actual: usize,
    },
    CannotSwitchToSelf(usize),
    ActorDead(usize),
    SwitchOnCooldown {
        until_us: u64,
        now_us: u64,
    },
    ActiveActorLockedInPhase {
        phase: ActionPhaseKind,
        until_us: u64,
    },
    RequiresOnField {
        actor_index: usize,
        active_actor: usize,
    },
    ActionBusy {
        current_phase: ActionPhaseKind,
    },
    InvalidActionTimings,
    InvalidActionState,
    NoActionToInterrupt,
    InvalidFormState,
    InvalidRecastState,
    InvalidRecastDuration,
    InvalidRecastUses,
    CancelNotAllowed {
        phase: ActionPhaseKind,
        cancel_type: CancelType,
    },
    CancelTooEarly {
        earliest_us: u64,
        now_us: u64,
    },
    NoActiveRecast,
    RecastExpired {
        expired_at_us: u64,
        now_us: u64,
    },
    RecastActionMismatch {
        expected: String,
        actual: String,
    },
    RecastNoUsesRemaining,
    InsufficientResource {
        pool: String,
        required: u64,
        available: u64,
    },
    UnknownPool(String),
    TimeReversal {
        last_us: u64,
        now_us: u64,
    },
    AllActorsDead,
}
