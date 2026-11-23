use super::ExactFraction;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameAtkTransferCause {
    NovaEnd,
    CombatExit,
}

/// One source's flat grant per equipped actor and one target's exact ATK change.
/// These structural changes never emit damage, healing or resource awards.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameAtkTransferReceipt {
    pub time_us: u64,
    pub source_actor_index: usize,
    pub target_index: usize,
    pub cause: GameAtkTransferCause,
    pub nova_generation: Option<u64>,
    pub captured_base_atk: ExactFraction,
    pub party_flat_atk_before: ExactFraction,
    pub party_flat_atk_after: ExactFraction,
    pub target_atk_before: ExactFraction,
    pub target_atk_after: ExactFraction,
}
