use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", content = "index", rename_all = "snake_case")]
pub enum GameHpStatRecipient {
    Actor(usize),
    Target(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameHpStatCause {
    PartyMembership,
    BaseHp,
    Transfer,
    CombatExit,
}
/// Structural MaxHP settlement never contributes damage, healing or combat loss.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameStructuralHpChange {
    pub time_us: u64,
    pub source_actor_index: usize,
    pub recipient: GameHpStatRecipient,
    pub cause: GameHpStatCause,
    pub max_hp_before: u64,
    pub max_hp_after: u64,
    pub hp_before: u64,
    pub hp_after: u64,
}
