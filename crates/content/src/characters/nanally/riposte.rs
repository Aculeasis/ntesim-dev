use crate::runtime_api::{CastContext, DirectPacketEnergyPolicy, NormalActionOverride};
use crate::simulation::{metadata::InputActionRole, state::ActorRuntime};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::NanallyRiposteV1;
pub(crate) const PACKET: &str = "GE_Player_Nanally_PerfectEvadeAttack_Damage";
pub(crate) const CAT_PACKET: &str = "GE_Player_Nanally_Cat_PerfectEvadeAttack_Damage";

pub(crate) fn cancel_after_defeat(effect: &str) -> bool {
    matches!(effect, PACKET | CAT_PACKET)
}

pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character, super::super::RuntimeState::Nanally(state) if state.riposte_ready)
        .then_some(NormalActionOverride {
            packet: PACKET,
            role: Some(InputActionRole::CriticalRiposte),
            model: HYPOTHESIS_ID,
            // Contact pause is projected only after positive actual HP loss.
            timing: Some((841_593, 267_562)),
        })
}

pub(crate) fn validate(actor: &ActorRuntime, ability: &str, packet: &str) -> Result<(), String> {
    if packet == PACKET && (ability != "GA_Nanally_Melee" || replacement(actor).is_none()) {
        return Err(
            "Nanally Critical Riposte requires one paid successful Critical Dodge opportunity"
                .into(),
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
    let state = ctx.actor.character.nanally_mut()?;
    state.riposte_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.heavy_resume = None;
    super::clock::cast(ctx, super::clock::Kind::Riposte)?;
    super::underboss::schedule_riposte(ctx)?;
    Ok(true)
}

pub(crate) fn direct_packet_energy(
    ability: &str,
    packet: &str,
) -> Option<DirectPacketEnergyPolicy> {
    (ability == "GA_Nanally_Melee" && packet == PACKET).then_some(DirectPacketEnergyPolicy {
        trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
        hypothesis_id: HYPOTHESIS_ID,
        event_kind: "nanally_riposte_energy_grant",
    })
}
