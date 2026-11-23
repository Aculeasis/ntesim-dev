use crate::runtime_api::{CastContext, DirectPacketEnergyPolicy, NormalActionOverride};
use crate::simulation::{metadata::InputActionRole, state::ActorRuntime};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::MintRiposteV1;
pub(crate) const PACKET: &str = "GE_Player_Mint_PerfectEvadeAttack_Damage";

pub(crate) fn cancel_after_defeat(effect: &str) -> bool {
    effect == PACKET
}

pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character, super::super::RuntimeState::Mint(state) if state.riposte_ready)
        .then_some(NormalActionOverride {
            packet: PACKET,
            role: Some(InputActionRole::CriticalRiposte),
            model: HYPOTHESIS_ID,
            // Rate-one source .22721595 impact and .44350475 recovery;
            // one connected .3*(1-.01) hitstop -> recovery .44350475 + .297 = .74050475s
            // rounded upward to microseconds.
            timing: Some((740_505, 227_216)),
        })
}

pub(crate) fn validate(actor: &ActorRuntime, ability: &str, packet: &str) -> Result<(), String> {
    if packet == PACKET && (ability != "GA_Mint019_Melee" || replacement(actor).is_none()) {
        return Err(
            "Mint Critical Riposte requires one paid successful Critical Dodge opportunity".into(),
        );
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if ctx
        .packet_metadata
        .is_none_or(|packet| packet.damage_effect_id != PACKET)
    {
        return Ok(false);
    }
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let state = ctx.actor.character.mint_mut()?;
    state.riposte_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    ctx.scope_primary_aoe(PACKET)?;
    Ok(true)
}

pub(crate) fn direct_packet_energy(
    ability: &str,
    packet: &str,
) -> Option<DirectPacketEnergyPolicy> {
    (ability == "GA_Mint019_Melee" && packet == PACKET).then_some(DirectPacketEnergyPolicy {
        trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
        hypothesis_id: HYPOTHESIS_ID,
        event_kind: "mint_riposte_energy_grant",
    })
}
