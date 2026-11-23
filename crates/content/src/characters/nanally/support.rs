use crate::runtime_api::CastContext;
use crate::runtime_api::SupportEntry;

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::NanallySupportPacketsV1;
pub(crate) const CAT_PACKET: &str = "GE_Player_Nanally_Cat_QTE_Damage";

pub(crate) const ENTRY: SupportEntry = SupportEntry {
    ability_id: "GA_Nanally_QTE",
    damage_effect_id: "GE_Player_Nanally_QTE1_Damage",
    hit_delay_us: 666_410,
    recovery_us: 1_108_275,
    dispatch_model: HYPOTHESIS_ID,
    packet_model: Some(HYPOTHESIS_ID),
    extra_damage_effects: &[],
};

pub(crate) fn direct_packet_energy(
    ability: &str,
    effect: &str,
) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
    (ability == ENTRY.ability_id && effect == ENTRY.damage_effect_id).then_some(
        crate::runtime_api::DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: HYPOTHESIS_ID,
            event_kind: "nanally_support_packet_energy_grant",
        },
    )
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != Some(ENTRY.damage_effect_id)
    {
        return Err("Nanally Support requires QTE1 as its admitted primary".into());
    }
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    super::clock::cast(ctx, super::clock::Kind::Support)?;
    super::underboss::schedule_support(ctx)
}
