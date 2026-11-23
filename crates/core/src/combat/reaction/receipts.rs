use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BreakHitContribution {
    pub actor_index: usize,
    pub character_id: String,
    pub damage: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BreakEventReceipt {
    pub target_id: String,
    pub triggered_at_us: u64,
    pub duration_us: u64,
    pub contributions: Vec<BreakHitContribution>,
    pub total_break_damage: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReactionReceipt {
    pub target_id: String,
    pub source_actor: usize,
    pub kind: ReactionKind,
    pub damage: u64,
    pub timestamp_us: u64,
    pub details: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HitDeliveryReceipt {
    pub source_actor: usize,
    pub target_id: String,
    pub damage: u64,
    pub break_shred: u64,
    pub timestamp_us: u64,
}
