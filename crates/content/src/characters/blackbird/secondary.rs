//! Explicit ordinary airborne profile and one paid Critical Dodge opportunity.
use crate::runtime_api::{CastContext, NormalActionOverride, PlungeInputContext};
use crate::simulation::{ExactFraction, GameScenarioRequest, PlayerInput, state::ActorRuntime};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::BlackbirdOrdinarySecondaryV1;
pub(crate) const EVOLVED_MODEL: crate::HypothesisId =
    crate::HypothesisId::BlackbirdEvolvedSecondaryV1;
pub(crate) const PLUNGE: &str = "GE_Player_BlackBird_AirAttack_Damage";
pub(crate) const RIPOSTE: &str = "GE_Player_BlackBird_PerfectEvade_Damage";
pub(crate) const EVOLVE_PLUNGE: &str = "GE_Player_BlackBird_Evolve_AirAttack_Damage";
pub(crate) const EVOLVE_RIPOSTE: &str = "GE_Player_BlackBird_Evolve_PerfectEvade_Damage";
pub(crate) const EVOLVE_RIPOSTE_EXTRA: &str =
    "GE_Player_BlackBird_Evolve_PerfectEvade_Extra_Damage";

pub(crate) fn cancel_after_defeat(effect: &str) -> bool {
    matches!(effect, RIPOSTE | EVOLVE_RIPOSTE)
}

pub(crate) fn critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.actor.character.blackbird_mut()?.riposte_ready = true;
    Ok(())
}

pub(crate) fn on_exit(actor: &mut ActorRuntime) -> Result<(), String> {
    actor.character.blackbird_mut()?.riposte_ready = false;
    Ok(())
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn evolved_accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(EVOLVED_MODEL.as_str())
        .map(String::as_str)
        == Some(EVOLVED_MODEL.description())
}
fn require(request: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(request) || !super::clock::accepted(request) || !super::malice::accepted(request) {
        return Err("Blackbird secondary actions require ordinary_secondary_v1, ordinary_clock_v1 and ordinary_malice_v1".into());
    }
    Ok(())
}
fn require_evolved(request: &GameScenarioRequest) -> Result<(), String> {
    if !evolved_accepted(request)
        || !super::skill::evolve_accepted(request)
        || !super::clock::accepted(request)
        || !super::malice::accepted(request)
    {
        return Err("Blackbird evolved secondary actions require evolved_secondary_v1, evolve_actions_v1, ordinary_clock_v1 and ordinary_malice_v1".into());
    }
    Ok(())
}
pub(crate) fn validate_plunge(ctx: &PlungeInputContext<'_>) -> Result<(), String> {
    let packet = ctx.metadata.packet.damage_effect_id.as_str();
    match (packet, ctx.metadata.action.ability_id.as_str()) {
        (PLUNGE, "GA_BlackBird_Melee") => {
            require(ctx.request)?;
            ctx.timestamp_us
                .checked_add(ctx.input.landing_delay_us)
                .and_then(|time| time.checked_add(409_949))
                .ok_or("Blackbird plunge deadline overflow")?;
        }
        (EVOLVE_PLUNGE, "GA_BlackBird_Evolve_Melee") => {
            require_evolved(ctx.request)?;
            ctx.timestamp_us
                .checked_add(ctx.input.landing_delay_us)
                .and_then(|time| time.checked_add(409_949))
                .ok_or("Blackbird evolved plunge deadline overflow")?;
        }
        _ => return Err("no supported owner plunge binding".into()),
    }
    Ok(())
}
pub(crate) fn validate_packet(
    actor: &ActorRuntime,
    ability: &str,
    packet: &str,
) -> Result<(), String> {
    let expected = match packet {
        RIPOSTE => "GA_BlackBird_Melee",
        EVOLVE_RIPOSTE => "GA_BlackBird_Evolve_Melee",
        _ => return Ok(()),
    };
    if ability != expected || replacement(actor).is_none() {
        return Err(
            "Blackbird Critical Riposte requires one paid successful Critical Dodge".into(),
        );
    }
    Ok(())
}
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    let crate::characters::RuntimeState::Blackbird(state) = &actor.character else {
        return None;
    };
    if !state.riposte_ready {
        return None;
    }
    if state.form.is_some() {
        Some(NormalActionOverride {
            packet: EVOLVE_RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: EVOLVED_MODEL,
            timing: Some((839_275, 225_959)),
        })
    } else {
        Some(NormalActionOverride {
            packet: RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: MODEL,
            timing: Some((903_675, 282_278)),
        })
    }
}
pub(crate) fn validate_input(
    request: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if input == PlayerInput::Dash && accepted(request) {
        require(request)?;
        if request.combat_stamina.is_none() {
            return Err("Blackbird dodge requires an explicit combat stamina profile".into());
        }
        return Ok(Some(MODEL));
    }
    Ok(None)
}
pub(crate) fn on_input(
    request: &GameScenarioRequest,
    actor: &mut ActorRuntime,
    input: PlayerInput,
) -> Result<(), String> {
    let state = actor.character.blackbird_mut()?;
    state.riposte_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.evolve_melee_stage = 0;
    state.evolve_melee_started_us = None;
    if input == PlayerInput::Dash && accepted(request) {
        actor.locked_until_us = actor
            .last_action_start_us
            .checked_add(1_004_760)
            .ok_or("Blackbird dodge recovery overflow")?;
    }
    Ok(())
}
pub(crate) fn timing(ctx: &CastContext<'_>) -> Result<Option<(u64, u64)>, String> {
    match ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) {
        Some(PLUNGE) => {
            let landing = ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Blackbird plunge requires an explicit airborne landing/height profile")?
                .landing_delay_us;
            Ok(Some((
                landing
                    .checked_add(53_784)
                    .ok_or("Blackbird plunge impact overflow")?,
                landing
                    .checked_add(281_249)
                    .ok_or("Blackbird plunge recovery overflow")?,
            )))
        }
        Some(EVOLVE_PLUNGE) => {
            let landing = ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Blackbird plunge requires an explicit airborne landing/height profile")?
                .landing_delay_us;
            Ok(Some((
                landing
                    .checked_add(100_643)
                    .ok_or("Blackbird evolved plunge impact overflow")?,
                landing
                    .checked_add(281_249)
                    .ok_or("Blackbird evolved plunge recovery overflow")?,
            )))
        }
        Some(RIPOSTE) => Ok(Some((282_278, 903_675))),
        Some(EVOLVE_RIPOSTE) => Ok(Some((225_959, 839_275))),
        _ => Ok(None),
    }
}
pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    let packet = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();
    let evolved = matches!(packet, EVOLVE_PLUNGE | EVOLVE_RIPOSTE);
    if !matches!(packet, PLUNGE | RIPOSTE) && !evolved {
        return Ok(false);
    }
    let expected = if evolved {
        require_evolved(ctx.request)?;
        if !super::form::active(ctx.actor) {
            return Err("Blackbird evolved secondary requires active transformation form".into());
        }
        "GA_BlackBird_Evolve_Melee"
    } else {
        require(ctx.request)?;
        "GA_BlackBird_Melee"
    };
    ctx.record_hypothesis(if evolved { EVOLVED_MODEL } else { MODEL })?;
    if ctx.ability_id != expected {
        return Err("Blackbird secondary action requires its Normal input family".into());
    }
    if matches!(packet, PLUNGE | EVOLVE_PLUNGE) {
        let input = ctx
            .request
            .plunge_inputs
            .get(&ctx.command_index)
            .ok_or("Blackbird plunge requires an explicit airborne landing/height profile")?;
        let multiplier =
            ExactFraction::from_ratio(input.height_bonus)?.add(ExactFraction::new(1, 1)?)?;
        ctx.scale_primary_coefficient(packet, multiplier)?;
    }
    ctx.scope_primary_aoe(packet)?;
    let state = ctx.actor.character.blackbird_mut()?;
    state.riposte_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.evolve_melee_stage = 0;
    state.evolve_melee_started_us = None;
    Ok(true)
}
pub(crate) fn energy(
    ability: &str,
    packet: &str,
) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
    if ability == "GA_BlackBird_Melee" && matches!(packet, PLUNGE | RIPOSTE) {
        return Some(crate::runtime_api::DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: MODEL,
            event_kind: "blackbird_secondary_energy_grant",
        });
    }
    (ability == "GA_BlackBird_Evolve_Melee" && matches!(packet, EVOLVE_PLUNGE | EVOLVE_RIPOSTE))
        .then_some(crate::runtime_api::DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: EVOLVED_MODEL,
            event_kind: "blackbird_evolved_secondary_energy_grant",
        })
}
