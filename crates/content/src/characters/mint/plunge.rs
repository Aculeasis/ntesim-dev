use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::MintPlungeV1;
pub(crate) const PACKET: &str = "GE_Player_Mint_AirAttack1_Damage";

pub(crate) fn is_plunge_packet(packet: &str) -> bool {
    packet == PACKET
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    ctx.scope_primary_aoe(PACKET)
}
