use crate::HypothesisId;
use crate::runtime_api::{CastContext, DirectPacketEnergyPolicy};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiPlungeV1;
pub(crate) const PACKET: &str = "GE_Player_Oneiroi_AirAttack_Damage";

#[allow(dead_code)]
pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn is_plunge_packet(packet: &str) -> bool {
    packet == PACKET
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    ctx.scope_primary_aoe(PACKET)
}

pub(crate) fn validate_plunge(
    ctx: &crate::runtime_api::PlungeInputContext<'_>,
) -> Result<(), String> {
    if !accepted(ctx.request)
        || ctx.metadata.packet.damage_effect_id != PACKET
        || ctx.input.height_bonus.numerator > ctx.input.height_bonus.denominator
    {
        return Err("Iroi plunge requires iroi.plunge_v1 and normalized height in 0..1".into());
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .ok_or("Iroi plunge landing time overflow")?;
    Ok(())
}

pub(crate) fn direct_packet_energy(
    ability: &str,
    packet: &str,
) -> Option<DirectPacketEnergyPolicy> {
    (ability == "GA_Oneiroi_Melee" && packet == PACKET).then_some(DirectPacketEnergyPolicy {
        trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
        hypothesis_id: HYPOTHESIS_ID,
        event_kind: "iroi_plunge_energy_grant",
    })
}
