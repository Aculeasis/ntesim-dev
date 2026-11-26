//! Paid Evade, Riposte, Falling Dream Plunge, Memory Anchor Aim, and Interference Held Normal.
use crate::runtime_api::{
    CastContext, DirectPacketEnergyPolicy, NormalActionOverride, PlungeInputContext,
};
use crate::simulation::{ExactFraction, GameScenarioRequest, PlayerInput, state::ActorRuntime};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ElymsSecondaryAndAimV1;
pub(crate) const PLUNGE: &str = "GE_Player_Elyms_AirAttack_Damage";
pub(crate) const RIPOSTE: &str = "GE_Player_Elyms_PerfectEvadeAttack_Damage";
pub(crate) const AIM: &str = "GE_Player_Elyms_Aim_Damage";
pub(crate) const BRANCH1: &str = "GE_Player_Elyms_Branch1_Damage";
pub(crate) const BRANCH2: &str = "GE_Player_Elyms_Branch2_Damage";
pub(crate) const BRANCH3: &str = "GE_Player_Elyms_Branch3_Damage";

pub(crate) fn public_plunge() -> crate::build::PublicAction {
    crate::build::PublicAction {
        command: crate::simulation::GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Elyms_Melee".into(),
            damage_effect_id: PLUNGE.into(),
            array_index: 0,
            atk_rate: crate::formula::Ratio::one(),
            scaling_stat: Default::default(),
            energy_cost: 0,
            recovery_us: super::clock::profile::PLUNGE_RECOVERY,
            cooldown_us: 0,
            hit_delay_us: super::clock::profile::PLUNGE_HIT,
        },
        advance_us: super::clock::profile::PLUNGE_RECOVERY,
        model_assumptions: [MODEL, super::clock::MODEL]
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
            "Elyms secondary and Aim actions require elyms.secondary_and_aim_v1 and elyms.ordinary_clock_v1"
                .into(),
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
        if request.actors[*actor_index].character_id != "character:1090" {
            continue;
        }
        if damage_effect_id == RIPOSTE {
            return Err(
                "Elyms Riposte is owner-selected: use canonical Normal after Critical Dodge".into(),
            );
        }
        if matches!(damage_effect_id.as_str(), BRANCH1 | BRANCH2 | BRANCH3) {
            return Err("Elyms Interference is owner-selected: use canonical held Normal".into());
        }
        if damage_effect_id == PLUNGE && !request.plunge_inputs.contains_key(&index) {
            return Err(
                "Elyms Falling Dream plunge requires an explicit positive airborne landing/height profile"
                    .into(),
            );
        }
    }
    for (&command, hold) in &request.hold_inputs {
        let Some(crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            ..
        }) = request.actions.get(command)
        else {
            continue;
        };
        if request.actors[*actor_index].character_id != "character:1090" {
            continue;
        }
        if ability_id != "GA_Elyms_Melee"
            || damage_effect_id != "GE_Player_Elyms_Melee1_Damage"
            || hold.moving
            || hold.reserved_stocks.is_some()
            || request.plunge_inputs.contains_key(&command)
        {
            return Err(
                "Elyms held Normal requires canonical stationary unreserved Normal without plunge"
                    .into(),
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
        return Err("Elyms Evade requires an explicit combat stamina profile".into());
    }
    Ok(Some(MODEL))
}

pub(crate) fn validate_plunge(ctx: &PlungeInputContext<'_>) -> Result<(), String> {
    require(ctx.request)?;
    if ctx.metadata.action.ability_id != "GA_Elyms_Melee"
        || ctx.metadata.packet.damage_effect_id != PLUNGE
    {
        return Err("Elyms Falling Dream requires Normal-family plunge row".into());
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .and_then(|time| time.checked_add(super::clock::profile::PLUNGE_RECOVERY))
        .ok_or("Elyms Falling Dream deadline overflow")?;
    Ok(())
}

pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character, crate::characters::RuntimeState::Elyms(state) if state.riposte_ready)
        .then_some(NormalActionOverride {
            packet: RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: MODEL,
            timing: Some((
                super::clock::profile::RIPOSTE_RECOVERY,
                super::clock::profile::RIPOSTE_HIT,
            )),
        })
}

pub(crate) fn held_replacement(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: crate::simulation::GameHoldInput,
) -> Result<Option<NormalActionOverride>, String> {
    if input.duration_us < super::parameters::PARAMETERS.hold_threshold_us {
        return Ok(None);
    }
    if actor.aiming {
        return Err("Elyms held Interference requires leaving Aim first".into());
    }
    if matches!(&actor.character, crate::characters::RuntimeState::Elyms(state) if state.riposte_ready)
    {
        return Ok(None);
    }
    require(request)?;
    Ok(Some(NormalActionOverride {
        packet: BRANCH1,
        role: None,
        model: MODEL,
        timing: Some((
            super::clock::profile::BRANCH_RECOVERY,
            super::clock::profile::BRANCH_HITS[0].1,
        )),
    }))
}

pub(crate) fn validate_packet(
    actor: &ActorRuntime,
    ability: &str,
    packet: &str,
) -> Result<(), String> {
    if packet == RIPOSTE && (ability != "GA_Elyms_Melee" || replacement(actor).is_none()) {
        return Err("Elyms Riposte requires one paid successful Critical Dodge".into());
    }
    Ok(())
}

pub(crate) fn critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    require(ctx.request)?;
    ctx.record_hypothesis(MODEL)?;
    ctx.actor.character.elyms_mut()?.riposte_ready = true;
    Ok(())
}

pub(crate) fn on_input(
    request: &GameScenarioRequest,
    actor: &mut ActorRuntime,
    input: PlayerInput,
) -> Result<(), String> {
    let state = actor.character.elyms_mut()?;
    state.riposte_ready = false;
    if input == PlayerInput::Dash && accepted(request) {
        state.entry_normal_ready = false;
        actor.locked_until_us = actor
            .last_action_start_us
            .checked_add(super::clock::profile::EVADE_FORWARD_RECOVERY)
            .ok_or("Elyms Evade recovery overflow")?;
    } else if input == PlayerInput::Jump {
        state.entry_normal_ready = false;
        state.normal_stage = 0;
        state.normal_started_us = None;
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
        super::clock::dodge(ctx, owner, command)
    } else if input == PlayerInput::Jump {
        super::clock::cancel::on_jump(ctx, owner)
    } else {
        Ok(())
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
        AIM => super::clock::Kind::Aim,
        BRANCH1 => super::clock::Kind::Branch,
        _ => return Ok(None),
    };
    require(ctx.request)?;
    if ctx.ability_id != "GA_Elyms_Melee" {
        return Err("Elyms secondary/Aim action requires GA_Elyms_Melee initiator".into());
    }
    ctx.record_hypothesis(MODEL)?;
    match kind {
        super::clock::Kind::Plunge => {
            if ctx.targets.len() > 1 {
                ctx.record_hypothesis(super::skill::ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            let input = ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Elyms Falling Dream profile missing")?;
            ctx.scale_primary_coefficient(
                PLUNGE,
                ExactFraction::new(1, 1)?.add(ExactFraction::from_ratio(input.height_bonus)?)?,
            )?;
            ctx.scope_primary_aoe_group(PLUNGE)?;
        }
        super::clock::Kind::Riposte => {
            if ctx.targets.len() > 1 {
                ctx.record_hypothesis(super::skill::ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            ctx.scope_primary_aoe_group(RIPOSTE)?;
        }
        super::clock::Kind::Aim => {
            ctx.scope_primary_target_group(AIM)?;
        }
        super::clock::Kind::Branch => {
            if ctx.targets.len() > 1 {
                ctx.record_hypothesis(super::skill::ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            ctx.scope_primary_aoe_group(BRANCH1)?;
            ctx.schedule_native_aoe_group(BRANCH2, ctx.hit_time_us)?;
            ctx.schedule_native_aoe_group(BRANCH1, ctx.hit_time_us)?;
            ctx.schedule_native_aoe_group(BRANCH3, ctx.hit_time_us)?;
        }
        _ => unreachable!(),
    }
    let state = ctx.actor.character.elyms_mut()?;
    state.riposte_ready = false;
    state.entry_normal_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    Ok(Some(kind))
}

pub(crate) fn energy(ability: &str, packet: &str) -> Option<DirectPacketEnergyPolicy> {
    match (ability, packet) {
        ("GA_Elyms_PerfectEvade", RIPOSTE)
        | ("GA_Elyms_Melee", RIPOSTE | PLUNGE)
        | ("GA_Elyms_QTE", "GE_Player_Elyms_QTE_Damage") => Some(DirectPacketEnergyPolicy {
            hypothesis_id: crate::HypothesisId::ElymsOrdinaryHitEnergyV1,
            event_kind: "elyms_direct_packet_energy_grant",
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
        }),
        _ => None,
    }
}
