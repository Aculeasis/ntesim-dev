use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombatError {
    NonFiniteParameter(String),
    NegativeDefense(String),
    DefenseShredOutOfRange(String),
    DefenseIgnoreOutOfRange(String),
    InvalidCritRate(String),
    InvalidCritDamage(String),
    ZeroAttackerLevel,
    NegativeDamageBonus(String),
    MissingStat(String),
    InvalidTargetIndex(usize),
    EmptyTargetProfile,
    NegativeBreakParameter(String),
    ZeroHealthCapacity,
    DeadRecipient,
    InvalidHealthState,
    UnknownModifier(String),
    QuantizationOverflow,
}

impl std::fmt::Display for CombatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CombatError {}
