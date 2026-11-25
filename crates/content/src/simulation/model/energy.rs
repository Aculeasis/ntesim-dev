use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GameEnergySource {
    MindRoyale,
    MintA6,
    MintHit,
    SakiriHit,
    ZeroHit,
    IroiHit,
    NanallyHit,
    ChaosHit,
    EdgarHit,
    HotoriHit,
    DaffodilHit,
    BlackbirdHit,
    AureliaHit,
    HanielHit,
    LinkoHit,
    SkiaHit,
    HathorHit,
    HathorPower,
    JiuyuanHit,
    JiuyuanBudBoom,
    BaicangHit,
    FadiaHit,
    FadiaThreshold,
    ZankouHit,
    ChizHit,
    AdlerHit,
    LacrimosaHit,
    ShinkuHit,
    AkaneHit,
    ZeroA2,
    NanallyA1,
    SakiriA5,
    AdlerA5,
    RishiEclipse,
    ReactionCharge,
}

/// Whole pool units after efficiency/carry, excluding starting energy.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameEnergyGrantReceipt {
    pub source: GameEnergySource,
    pub source_actor_index: usize,
    pub recipient_actor_index: usize,
    pub energy_before: u64,
    pub gross: u64,
    pub retained: u64,
    pub overflow: u64,
    pub energy_after: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_remainder_before: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_remainder_after: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_basis: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_basis: Option<OrdinaryHitEnergyBasis>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrdinaryHitEnergyBasis {
    pub base: ExactFraction,
    pub distribution: ExactFraction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameEnergySpendSource {
    AbilityCost,
}

/// An admitted cost, including zero-cost inputs, bound to the original command.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameEnergySpendReceipt {
    pub source: GameEnergySpendSource,
    pub source_actor_index: usize,
    pub recipient_actor_index: usize,
    pub command_index: usize,
    pub ability_id: String,
    pub energy_before: u64,
    pub spent: u64,
    pub energy_after: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_remainder_before: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_remainder_after: Option<ExactFraction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameEnergyFanoutRecipient {
    pub recipient_actor_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution: Option<ExactFraction>,
    pub energy_before: u64,
    pub gross: u64,
    pub retained: u64,
    pub overflow: u64,
    pub energy_after: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_remainder_before: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_remainder_after: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_basis: Option<ExactFraction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameEnergyFanoutReceipt {
    pub source: GameEnergySource,
    pub source_actor_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<ExactFraction>,
    pub recipients: Vec<GameEnergyFanoutRecipient>,
}

impl GameEnergyFanoutReceipt {
    pub fn grants(&self) -> impl Iterator<Item = GameEnergyGrantReceipt> + '_ {
        self.recipients
            .iter()
            .map(|recipient| GameEnergyGrantReceipt {
                source: self.source,
                source_actor_index: self.source_actor_index,
                recipient_actor_index: recipient.recipient_actor_index,
                energy_before: recipient.energy_before,
                gross: recipient.gross,
                retained: recipient.retained,
                overflow: recipient.overflow,
                energy_after: recipient.energy_after,
                charge_remainder_before: recipient.charge_remainder_before,
                charge_remainder_after: recipient.charge_remainder_after,
                charge_basis: recipient.charge_basis,
                hit_basis: self
                    .base
                    .zip(recipient.distribution)
                    .map(|(base, distribution)| OrdinaryHitEnergyBasis { base, distribution }),
            })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GameEnergyReceipt {
    Grant(GameEnergyGrantReceipt),
    Spend(GameEnergySpendReceipt),
}
