use crate::runtime_api::{CastContext, DirectPacketEnergyPolicy, NormalActionOverride};
use crate::simulation::{metadata::InputActionRole, state::ActorRuntime};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::SakiriRiposteV1;
pub(crate) const PACKET: &str = "GE_Player_Sagiri_PerfectEvadeAttack_Damage";

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character, super::super::RuntimeState::Sakiri(state) if state.riposte_ready)
        .then_some(NormalActionOverride {
            packet: PACKET,
            role: Some(InputActionRole::CriticalRiposte),
            model: HYPOTHESIS_ID,
            // Rate-one source .180s impact and .750s recovery.
            timing: Some((750_000, 180_000)),
        })
}

pub(crate) fn validate(actor: &ActorRuntime, ability: &str, packet: &str) -> Result<(), String> {
    if packet == PACKET && (ability != "GA_Sagiri_Melee" || replacement(actor).is_none()) {
        return Err(
            "Sakiri Critical Riposte requires one paid successful Critical Dodge opportunity"
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
    let state = ctx.actor.character.sakiri_mut()?;
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
    (ability == "GA_Sagiri_Melee" && packet == PACKET).then_some(DirectPacketEnergyPolicy {
        trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
        hypothesis_id: HYPOTHESIS_ID,
        event_kind: "sakiri_riposte_energy_grant",
    })
}
