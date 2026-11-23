use crate::HypothesisId;
use crate::runtime_api::{CastContext, DirectPacketEnergyPolicy};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerPlungeV1;
pub(crate) const PACKET: &str = "GE_Player_Adler_AirAttack_Damage";

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

pub(crate) fn direct_packet_energy(
    ability: &str,
    packet: &str,
) -> Option<DirectPacketEnergyPolicy> {
    ((ability == "GA_Adler_Melee" || ability == "GA_Adler_AirAttack") && packet == PACKET)
        .then_some(DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: HYPOTHESIS_ID,
            event_kind: "adler_plunge_energy_grant",
        })
}
