use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum CombatEvent {
    ActionStarted {
        actor_index: usize,
        action_id: String,
    },
    HitDealt {
        source_actor: usize,
        target_id: String,
        action_id: String,
        element: String,
        is_crit: bool,
        damage: u64,
    },
    DamageReceived {
        target_id: String,
        source_actor: usize,
        damage: u64,
    },
    HealDealt {
        source_actor: usize,
        target_actor: usize,
        amount: u64,
    },
    ShieldApplied {
        source_actor: usize,
        target_actor: usize,
        amount: u64,
    },
    ActorSwitched {
        from_index: usize,
        to_index: usize,
    },
    TargetDied {
        target_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "snake_case")]
pub enum ProcCondition {
    AnyEvent,
    RequiresCrit,
    ElementMatches { element: String },
    ActionMatches { action_id: String },
    RequiresOnField { on_field: bool },
    TargetMatches { target_id: String },
}

impl ProcCondition {
    pub fn matches(&self, event: &CombatEvent, owner_actor: usize, active_actor: usize) -> bool {
        let is_owner_on_field = owner_actor == active_actor;
        match self {
            Self::AnyEvent => true,
            Self::RequiresCrit => match event {
                CombatEvent::HitDealt { is_crit, .. } => *is_crit,
                _ => false,
            },
            Self::ElementMatches { element } => match event {
                CombatEvent::HitDealt {
                    element: hit_elem, ..
                } => hit_elem == element,
                _ => false,
            },
            Self::ActionMatches { action_id } => match event {
                CombatEvent::ActionStarted {
                    action_id: act_id, ..
                }
                | CombatEvent::HitDealt {
                    action_id: act_id, ..
                } => act_id == action_id,
                _ => false,
            },
            Self::RequiresOnField { on_field } => is_owner_on_field == *on_field,
            Self::TargetMatches { target_id } => match event {
                CombatEvent::HitDealt {
                    target_id: t_id, ..
                }
                | CombatEvent::DamageReceived {
                    target_id: t_id, ..
                }
                | CombatEvent::TargetDied {
                    target_id: t_id, ..
                } => t_id == target_id,
                _ => false,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum ProcPayload {
    GrantPoolResource {
        pool: String,
        amount: u64,
    },
    ApplyBuff {
        buff: BuffDefinition,
    },
    TriggerBonusDamage {
        element: String,
        multiplier_percent: u64,
        target_id: String,
    },
    TriggerHeal {
        amount: u64,
        target_actor: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CallbackRegistration {
    pub id: String,
    pub owner_actor: usize,
    pub conditions: Vec<ProcCondition>,
    pub icd_rule: IcdRule,
    pub icd_scope: IcdScope,
    pub payload: ProcPayload,
    pub max_triggers: Option<u32>,
    pub triggers_so_far: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcReceipt {
    pub callback_id: String,
    pub owner_actor: usize,
    pub triggered_at_us: u64,
    pub payload_kind: String,
    pub details: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DispatchReceipt {
    pub event: CombatEvent,
    pub timestamp_us: u64,
    pub depth: usize,
    pub procs_triggered: Vec<ProcReceipt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum DispatchError {
    RecursionDepthExceeded { max_depth: usize },
    UnknownPool(String),
    UnknownActor(usize),
    InvalidTeamSize(usize),
    InvalidIcd(String),
    InvalidSwitchFrom { expected: usize, actual: usize },
    TimeReversal { last_us: u64, now_us: u64 },
    InvalidBuff(String),
    MissingSourceLivenessContext,
    DuplicateCallback(String),
    ArithmeticOverflow,
    InvalidBonusDamageBasis,
}
