use crate::HypothesisId;
use crate::runtime_api::{CastContext, DirectPacketEnergyPolicy, NormalActionOverride};
use crate::simulation::{metadata::InputActionRole, state::ActorRuntime};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::AureliaRiposteV1;
pub(crate) const PACKET: &str = "GE_Player_Mitsuki_PerfectEvadeAtk_Damage";

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character, super::super::RuntimeState::Aurelia(state) if state.riposte_ready)
        .then_some(NormalActionOverride {
            packet: PACKET,
            role: Some(InputActionRole::CriticalRiposte),
            model: HYPOTHESIS_ID,
            // Nominal damage offset: .35614303s + .05s = .406143s (406_143 us); recovery 1.3132069s + .297s = 1.6102069s (1_610_207 us)
            timing: Some((1_610_207, 406_143)),
        })
}

pub(crate) fn validate(actor: &ActorRuntime, ability: &str, packet: &str) -> Result<(), String> {
    if packet == PACKET && (ability != "GA_Mitsuki_Melee" || replacement(actor).is_none()) {
        return Err(
            "Aurelia Critical Riposte requires one paid successful Critical Dodge opportunity"
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
    let state = ctx.actor.character.aurelia_mut()?;
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
    (ability == "GA_Mitsuki_Melee" && packet == PACKET).then_some(DirectPacketEnergyPolicy {
        trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
        hypothesis_id: HYPOTHESIS_ID,
        event_kind: "aurelia_riposte_energy_grant",
    })
}
