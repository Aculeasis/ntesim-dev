use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionError {
    EmptyTeam,
    TeamSizeExceeded(usize),
    InvalidActiveActor(usize),
    InvalidActorIndex(usize),
    TargetNotFound(String),
    PoolNotFound(String),
    ZeroHorizon,
    TimeReversal {
        previous_us: u64,
        current_us: u64,
    },
    ZeroHealthCapacity(usize),
    CurrentHealthExceedsMax {
        actor_index: usize,
        current: u64,
        max: u64,
    },
    InvalidPackage(String),
    LimitExceeded {
        limit_name: String,
        current: u64,
        max: u64,
    },
    ResourceShortage {
        pool: String,
        available: u64,
        required: u64,
    },
    ActorLocked {
        actor_index: usize,
        locked_until_us: u64,
        requested_us: u64,
    },
    ActorOnCooldown {
        actor_index: usize,
        cooldown_until_us: u64,
        requested_us: u64,
    },
    ActorNotActive {
        actor_index: usize,
        active_actor: usize,
    },
    ActorDead(usize),
    TargetDead(String),
    ProcDepthExceeded {
        depth: u32,
        max_depth: u32,
    },
    DuplicatePoolId(String),
    InvariantViolation {
        invariant_name: String,
        details: String,
    },
    MissingField(String),
    ExtraField(String),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTeam => write!(f, "team size cannot be zero"),
            Self::TeamSizeExceeded(sz) => write!(f, "team size {sz} exceeds maximum 4"),
            Self::InvalidActiveActor(act) => write!(f, "active actor {act} out of bounds"),
            Self::InvalidActorIndex(idx) => write!(f, "actor index {idx} out of bounds"),
            Self::TargetNotFound(id) => write!(f, "target not found: {id}"),
            Self::PoolNotFound(id) => write!(f, "resource pool not found: {id}"),
            Self::ZeroHorizon => write!(f, "horizon_us must be greater than zero"),
            Self::TimeReversal {
                previous_us,
                current_us,
            } => {
                write!(
                    f,
                    "time reversal: current {current_us} earlier than previous {previous_us}"
                )
            }
            Self::ZeroHealthCapacity(idx) => write!(f, "actor {idx} max_hp cannot be zero"),
            Self::CurrentHealthExceedsMax {
                actor_index,
                current,
                max,
            } => {
                write!(
                    f,
                    "actor {actor_index} current_hp {current} exceeds max_hp {max}"
                )
            }
            Self::InvalidPackage(pkg) => write!(f, "unsupported package: {pkg}"),
            Self::LimitExceeded {
                limit_name,
                current,
                max,
            } => {
                write!(
                    f,
                    "execution limit {limit_name} exceeded: {current} > {max}"
                )
            }
            Self::ResourceShortage {
                pool,
                available,
                required,
            } => {
                write!(
                    f,
                    "insufficient resource in {pool}: available {available} < required {required}"
                )
            }
            Self::ActorLocked {
                actor_index,
                locked_until_us,
                requested_us,
            } => {
                write!(
                    f,
                    "actor {actor_index} locked until {locked_until_us} (requested at {requested_us})"
                )
            }
            Self::ActorOnCooldown {
                actor_index,
                cooldown_until_us,
                requested_us,
            } => {
                write!(
                    f,
                    "actor {actor_index} on cooldown until {cooldown_until_us} (requested at {requested_us})"
                )
            }
            Self::ActorNotActive {
                actor_index,
                active_actor,
            } => {
                write!(
                    f,
                    "actor {actor_index} is not on field (current active actor {active_actor})"
                )
            }
            Self::ActorDead(idx) => write!(f, "actor {idx} is dead"),
            Self::TargetDead(id) => write!(f, "target {id} is dead"),
            Self::ProcDepthExceeded { depth, max_depth } => {
                write!(f, "proc recursion depth {depth} exceeds max {max_depth}")
            }
            Self::DuplicatePoolId(id) => write!(f, "duplicate resource pool: {id}"),
            Self::InvariantViolation {
                invariant_name,
                details,
            } => {
                write!(f, "invariant violation [{invariant_name}]: {details}")
            }
            Self::MissingField(fld) => write!(f, "missing field: {fld}"),
            Self::ExtraField(fld) => write!(f, "extra field: {fld}"),
        }
    }
}
