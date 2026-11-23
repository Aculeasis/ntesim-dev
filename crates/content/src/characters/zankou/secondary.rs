//! Earned Support/Riposte and explicitly authored airborne landing.
use super::clock::{self, Kind};
use crate::runtime_api::{
    CallbackContext, CastContext, NormalActionOverride, PlungeInputContext, SupportEntry,
};
use crate::simulation::{
    GameActionCommand, GameScenarioRequest, PlayerInput,
    state::{ActorRuntime, GameSimulatorState},
};
pub(crate) const PLUNGE: &str = "GE_Player_Zankou_AirAttack_Damage";
pub(crate) const RIPOSTE: &str = "GE_Player_Zankou_PerfectEvadeAttack_Damage";
const QTE: &str = "GE_Player_Zankou_QTE_Damage";
const ENTRY: SupportEntry = SupportEntry {
    ability_id: "GA_Zankou_QTE",
    damage_effect_id: QTE,
    hit_delay_us: 791_367,
    recovery_us: 1_034_234,
    dispatch_model: super::forms::MODEL,
    packet_model: Some(super::forms::MODEL),
    extra_damage_effects: &[],
};
pub(crate) fn select(
    _req: &GameScenarioRequest,
    state: &GameSimulatorState,
    new: usize,
    qualified_support: bool,
) -> Result<Option<&'static SupportEntry>, String> {
    if state.actors[new].locked_until_us > state.current_time_us || !qualified_support {
        return Ok(None);
    }
    Ok(Some(&ENTRY))
}
pub(crate) fn validate_plunge(ctx: &PlungeInputContext<'_>) -> Result<(), String> {
    if ctx.metadata.action.ability_id != "GA_Zankou_Melee"
        || ctx.metadata.packet.damage_effect_id != PLUNGE
    {
        return Err("Zankou plunge requires its native Normal row".into());
    }
    if ctx.input.height_bonus.numerator > ctx.input.height_bonus.denominator {
        return Err("Zankou plunge height bonus must be in0..1".into());
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .and_then(|t| t.checked_add(281_249))
        .ok_or("Zankou landing deadline overflow")?;
    Ok(())
}
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character,crate::characters::RuntimeState::Zankou(s) if s.riposte_ready)
        .then_some(NormalActionOverride {
            packet: RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: super::forms::MODEL,
            timing: Some((387_993, 166_172)),
        })
}
pub(crate) fn critical(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(super::forms::MODEL)?;
    ctx.actor.character.zankou_mut()?.riposte_ready = true;
    Ok(())
}
pub(crate) fn validate_input(
    req: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if !matches!(input, PlayerInput::Jump | PlayerInput::Dash) {
        return Ok(None);
    }
    if input == PlayerInput::Dash && req.combat_stamina.is_none() {
        return Err("Zankou paid Dodge requires explicit combat stamina".into());
    }
    Ok(Some(super::forms::MODEL))
}
pub(crate) fn admission(
    req: &GameScenarioRequest,
    command: usize,
    actor: &ActorRuntime,
    ability: &str,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    let crate::characters::RuntimeState::Zankou(state) = &actor.character else {
        return None;
    };
    let plunge = ability == "GA_Zankou_Melee" && req.plunge_inputs.contains_key(&command);
    let message = if state.airborne && !plunge {
        "Zankou airborne state requires declared plunge and landing"
    } else if !state.airborne && plunge {
        "Zankou plunge requires a newly admitted Jump"
    } else {
        return None;
    };
    Some(crate::simulation::rejection::AdmissionFailure {
        reason: crate::simulation::GameAdmissionReason::Other {
            message: message.into(),
        },
        message: message.into(),
    })
}
pub(crate) fn kind(ctx: &mut CastContext<'_>) -> Result<Option<Kind>, String> {
    let packet = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();
    let kind = match packet {
        PLUNGE => {
            let input = ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Zankou plunge requires declared landing")?;
            if !ctx.actor.character.zankou_mut()?.airborne {
                return Err("Zankou plunge requires a new Jump".into());
            }
            Kind::Plunge(input.landing_delay_us)
        }
        RIPOSTE => {
            if replacement(ctx.actor).is_none() {
                return Err("Zankou Riposte requires earned Critical Dodge".into());
            }
            Kind::Riposte
        }
        QTE => {
            if !matches!(ctx.request.actions.get(ctx.command_index),Some(GameActionCommand::SwitchActor{target_actor_index,..}) if *target_actor_index==ctx.actor_index)
            {
                return Err("Zankou Support requires an engine-qualified switch".into());
            }
            Kind::Support
        }
        _ => return Ok(None),
    };
    ctx.record_hypothesis(super::forms::MODEL)?;
    let state = ctx.actor.character.zankou_mut()?;
    state.airborne = false;
    state.riposte_ready = false;
    Ok(Some(kind))
}
pub(crate) fn cast(ctx: &mut CastContext<'_>, kind: Kind) -> Result<(), String> {
    super::forms::stop(ctx.actor, ctx.queue, ctx.buffs, ctx.time_us)?;
    let first_seq = *ctx.sequence;
    if ctx.config.awakening.has_effect(1)
        && let Some(delay) = super::profile::spread(kind)
    {
        super::spread::schedule(ctx, delay)?;
    }
    let (effect, delay) = match kind {
        Kind::Plunge(landing) => (PLUNGE, landing + 83_928),
        Kind::Riposte => (RIPOSTE, 166_172),
        Kind::Support => (QTE, 791_367),
        _ => return Err("invalid Zankou secondary action".into()),
    };
    ctx.replace_native_primary_packet(effect)?;
    if matches!(kind, Kind::Plunge(_)) {
        let height = ctx.request.plunge_inputs[&ctx.command_index].height_bonus;
        let factor = crate::simulation::ExactFraction::new(1, 1)?
            .add(crate::simulation::ExactFraction::from_ratio(height)?)?;
        ctx.scale_primary_coefficient(PLUNGE, factor)?;
    }
    let group = ctx.scope_primary_aoe_group(effect)?;
    if kind == Kind::Support {
        ctx.actor.switch_action_locked_until_us = ctx
            .time_us
            .checked_add(1_015_890)
            .ok_or("Zankou Support switch overflow")?;
    }
    let state = ctx.actor.character.zankou_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    clock::start(ctx, kind, [(group, delay)].into(), first_seq, None)
}
pub(crate) fn committed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: PlayerInput,
) -> Result<(), String> {
    if matches!(input, PlayerInput::Jump | PlayerInput::Dash) {
        super::forms::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            &mut ctx.state.buffs,
            ctx.state.current_time_us,
        )?;
        let state = ctx.state.actors[owner].character.zankou_mut()?;
        state.armor_pressure_halves = None;
        state.normal_stage = 0;
        state.normal_started_us = None;
        state.riposte_ready = false;
        if input == PlayerInput::Jump {
            state.airborne = true;
        }
        if input == PlayerInput::Dash {
            clock::dodge(ctx, owner, command)?;
        }
    }
    Ok(())
}
