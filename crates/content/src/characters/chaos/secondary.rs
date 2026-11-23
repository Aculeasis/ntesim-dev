//! Paid front dodge, one explicit Critical Riposte permission and authored airborne plunge.
use crate::runtime_api::{
    CastContext, DirectPacketEnergyPolicy, NormalActionOverride, PlungeInputContext,
};
use crate::simulation::{ExactFraction, GameScenarioRequest, PlayerInput, state::ActorRuntime};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChaosSecondaryActionsV1;
pub(crate) const PLUNGE: &str = "GE_Player_Chaos_Airattack_Damage";
pub(crate) const PLUNGE_ULTRA: &str = "GE_Player_Chaos_Airattack_Damage_Ultra";
pub(crate) const RIPOSTE: &str = "GE_Player_Chaos_PerfectEvadeAttack1_Damage";

pub(crate) fn public_plunge() -> crate::build::PublicAction {
    crate::build::PublicAction {
        command: crate::simulation::GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Chaos071_Melee".into(),
            damage_effect_id: PLUNGE.into(),
            array_index: 0,
            atk_rate: crate::formula::Ratio::one(),
            scaling_stat: Default::default(),
            energy_cost: 0,
            recovery_us: 159_289,
            cooldown_us: 0,
            hit_delay_us: 47_594,
        },
        advance_us: 159_289,
        model_assumptions: [MODEL, super::clock::MODEL, super::resources::MODEL]
            .into_iter()
            .map(|id| (id.into(), id.description().into()))
            .collect(),
    }
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
fn require(request: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(request) || !super::clock::accepted(request) {
        return Err(
            "Chaos secondary actions require secondary_actions_v1 and ordinary_clock_v1".into(),
        );
    }
    Ok(())
}
pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if accepted(request) {
        require(request)?;
    }
    for (index, command) in request.actions.iter().enumerate() {
        let crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            damage_effect_id,
            ..
        } = command
        else {
            continue;
        };
        if request.actors[*actor_index].character_id != "character:1071" {
            continue;
        }
        if matches!(damage_effect_id.as_str(), RIPOSTE | PLUNGE_ULTRA) {
            return Err("Chaos secondary rows are owner-selected: use canonical Normal for Riposte or ordinary authored plunge input".into());
        }
        if damage_effect_id == PLUNGE && !request.plunge_inputs.contains_key(&index) {
            return Err(
                "Chaos plunge requires an explicit positive airborne landing/height profile".into(),
            );
        }
    }
    Ok(())
}
pub(crate) fn validate_input(
    request: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if input != PlayerInput::Dash || !accepted(request) {
        return Ok(None);
    }
    require(request)?;
    if request.combat_stamina.is_none() {
        return Err("Chaos dodge requires an explicit combat stamina profile".into());
    }
    Ok(Some(MODEL))
}
pub(crate) fn validate_plunge(ctx: &PlungeInputContext<'_>) -> Result<(), String> {
    require(ctx.request)?;
    if ctx.metadata.action.ability_id != "GA_Chaos071_Melee"
        || ctx.metadata.packet.damage_effect_id != PLUNGE
    {
        return Err("Chaos plunge requires its ordinary canonical Normal-family plunge row; form selects the live counterpart".into());
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .and_then(|time| time.checked_add(159_289))
        .ok_or("Chaos plunge deadline overflow")?;
    Ok(())
}
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character,crate::characters::RuntimeState::Chaos(state) if state.riposte_ready)
        .then_some(NormalActionOverride {
            packet: RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: MODEL,
            timing: Some((366_207, 250_157)),
        })
}
pub(crate) fn validate_packet(
    actor: &ActorRuntime,
    ability: &str,
    packet: &str,
) -> Result<(), String> {
    if packet == RIPOSTE && (ability != "GA_Chaos071_Melee" || replacement(actor).is_none()) {
        return Err("Chaos Critical Riposte requires one paid successful Critical Dodge".into());
    }
    Ok(())
}
pub(crate) fn critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    require(ctx.request)?;
    ctx.record_hypothesis(MODEL)?;
    ctx.actor.character.chaos_mut()?.riposte_ready = true;
    Ok(())
}
pub(crate) fn on_input(
    request: &GameScenarioRequest,
    actor: &mut ActorRuntime,
    input: PlayerInput,
) -> Result<(), String> {
    let state = actor.character.chaos_mut()?;
    state.riposte_ready = false;
    state.entry_normal_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    if input == PlayerInput::Dash && accepted(request) {
        actor.locked_until_us = actor
            .last_action_start_us
            .checked_add(998_284)
            .ok_or("Chaos front dodge recovery overflow")?;
    }
    Ok(())
}

pub(crate) fn committed_input(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: PlayerInput,
) -> Result<(), String> {
    if input == PlayerInput::Dash && accepted(ctx.request) {
        super::clock::secondary::dodge(ctx, owner, command)
    } else {
        super::cancel::on_jump(ctx, owner)
    }
}
pub(super) fn on_cast(ctx: &mut CastContext<'_>) -> Result<Option<super::clock::Kind>, String> {
    let packet = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        .unwrap_or_default();
    let kind = match packet {
        PLUNGE => super::clock::Kind::Plunge,
        RIPOSTE => super::clock::Kind::Riposte,
        _ => return Ok(None),
    };
    require(ctx.request)?;
    if ctx.ability_id != "GA_Chaos071_Melee" {
        return Err("Chaos secondary requires its actual Normal initiator".into());
    }
    if matches!(kind, super::clock::Kind::Plunge) {
        let input = ctx
            .request
            .plunge_inputs
            .get(&ctx.command_index)
            .ok_or("Chaos plunge profile missing")?;
        let effect = if super::form::active(ctx.actor) {
            PLUNGE_ULTRA
        } else {
            PLUNGE
        };
        if effect != packet {
            ctx.replace_native_primary_packet(effect)?;
        }
        ctx.scale_primary_coefficient(
            effect,
            ExactFraction::new(1, 1)?.add(ExactFraction::from_ratio(input.height_bonus)?)?,
        )?;
        ctx.scope_primary_aoe(effect)?;
    } else {
        validate_packet(ctx.actor, ctx.ability_id, packet)?;
        ctx.scope_primary_aoe(packet)?;
    }
    ctx.record_hypothesis(MODEL)?;
    let state = ctx.actor.character.chaos_mut()?;
    state.riposte_ready = false;
    state.entry_normal_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    Ok(Some(kind))
}
pub(crate) fn energy(ability: &str, packet: &str) -> Option<DirectPacketEnergyPolicy> {
    (ability == "GA_Chaos071_Melee" && matches!(packet, PLUNGE | PLUNGE_ULTRA | RIPOSTE)).then_some(
        DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: MODEL,
            event_kind: "chaos_secondary_energy_grant",
        },
    )
}
