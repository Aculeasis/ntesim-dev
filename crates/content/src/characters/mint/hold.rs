use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::MintHoldAttackV1;
pub(crate) const EVADE1: &str = "GE_Player_Mint_Evade1_Damage";
pub(crate) const EVADE4: &str = "GE_Player_Mint_Evade4_Damage";
pub(crate) const EVADE5: &str = "GE_Player_Mint_Evade5_Damage";

pub(crate) fn is_hold_packet(packet: &str) -> bool {
    matches!(packet, EVADE1 | EVADE4 | EVADE5)
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let hit_time = ctx.hit_time_us;
    let packet_id = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str());
    if packet_id == Some(EVADE1) {
        // Opener delivers 3x Evade1 + 1x Evade4 (first Evade1 is primary).
        ctx.schedule_native_child_packet(EVADE1, ctx.target_index, hit_time)?;
        ctx.schedule_native_child_packet(EVADE1, ctx.target_index, hit_time)?;
        ctx.schedule_native_child_packet(EVADE4, ctx.target_index, hit_time)?;
    }
    Ok(())
}
