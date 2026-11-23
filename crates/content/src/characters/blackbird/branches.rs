//! Held-Normal branch replacement: one Roostcall row and the two evolved branches.
use crate::runtime_api::{CastContext, NormalActionOverride};
use crate::simulation::{GameHoldInput, GameScenarioRequest, state::ActorRuntime};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::BlackbirdBranchesV1;
pub(crate) const HOLD_US: u64 = 200_000;
pub(crate) const ROOSTCALL_PACKET: &str = "GE_Player_BlackBird_Branch1_Damage";
pub(crate) const BESTOWED_GRACE_PACKET: &str = "GE_Player_BlackBird_Evolve_Branch1_Damage";
pub(crate) const TIDINGS_PACKET: &str = "GE_Player_BlackBird_Evolve_Branch2_Damage";
const ROOSTCALL_GROUPS: usize = 1;
const BESTOWED_GRACE_GROUPS: usize = 5;
const TIDINGS_GROUPS: usize = 5;
const ROOSTCALL_TIMING: (u64, u64) = (757_210, 200_476);
const BESTOWED_GRACE_TIMING: (u64, u64) = (1_345_821, 617_712);
const TIDINGS_TIMING: (u64, u64) = (975_109, 143_845);

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

fn require(request: &GameScenarioRequest) -> Result<(), String> {
    if !super::clock::accepted(request) || !super::malice::accepted(request) {
        return Err("Blackbird branches require ordinary clock and exact Malice models".into());
    }
    Ok(())
}

fn require_evolved(request: &GameScenarioRequest) -> Result<(), String> {
    if !super::skill::evolve_accepted(request) || !super::form::accepted(request) {
        return Err(
            "Blackbird evolved branches require evolve_actions_v1 and primary_form_v1".into(),
        );
    }
    Ok(())
}

pub(crate) fn validate_inputs(
    request: &GameScenarioRequest,
    packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
) -> Result<(), String> {
    for (index, packet) in packets.iter().enumerate() {
        if packet.as_ref().is_some_and(|metadata| {
            matches!(
                metadata.packet.damage_effect_id.as_str(),
                ROOSTCALL_PACKET | BESTOWED_GRACE_PACKET | TIDINGS_PACKET
            )
        }) {
            return Err(format!(
                "Blackbird branch command {index} must use a held canonical Normal input"
            ));
        }
    }
    for (&index, input) in &request.hold_inputs {
        if crate::characters::hathor::branch::entry_hold(request, index) {
            continue;
        }
        let Some(crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            timestamp_us,
            ..
        }) = request.actions.get(index)
        else {
            return Err(format!(
                "hold_inputs index {index} must identify an InvokeAbility"
            ));
        };
        let character = request.actors[*actor_index].character_id.as_str();
        if matches!(
            character,
            "character:1023"
                | "character:1010"
                | "character:1071"
                | "character:1073"
                | "character:1054"
                | "character:1036"
                | "character:1021"
                | "character:1020"
                | "character:1025"
                | "character:1019"
                | "character:1075"
        ) {
            continue;
        }
        if character != "character:1042" {
            return Err(format!(
                "hold_inputs index {index} has no supported owner binding"
            ));
        }
        if input.moving {
            return Err("Blackbird held Normal has no movement-intent branch".into());
        }
        if input.duration_us >= HOLD_US && !accepted(request) {
            return Err("Blackbird held branches require branches_v1".into());
        }
        let metadata = packets[index]
            .as_ref()
            .ok_or("hold packet metadata missing")?;
        if !matches!(
            metadata.action.ability_id.as_str(),
            "GA_BlackBird_Melee" | "GA_BlackBird_Evolve_Melee"
        ) || super::primary_packet(&metadata.action.ability_id)
            != Some(metadata.packet.damage_effect_id.as_str())
            || request.plunge_inputs.contains_key(&index)
        {
            return Err(format!(
                "hold_inputs index {index} has no supported owner Normal binding"
            ));
        }
        timestamp_us
            .checked_add(input.duration_us)
            .and_then(|time| time.checked_add(1_500_000))
            .ok_or("Blackbird hold deadline overflow")?;
        if accepted(request) {
            require(request)?;
        }
    }
    Ok(())
}

pub(crate) fn replacement(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: GameHoldInput,
    now: u64,
) -> Result<Option<NormalActionOverride>, String> {
    if input.duration_us < HOLD_US {
        return Ok(None);
    }
    let crate::characters::RuntimeState::Blackbird(state) = &actor.character else {
        return Err("unsupported held-input owner".into());
    };
    let unfinished = super::clock::unfinished(actor, now)?;
    let packet = if state.form.is_some() {
        require(request)?;
        require_evolved(request)?;
        match unfinished {
            Some(super::clock::Kind::BestowedGrace) | Some(super::clock::Kind::EvolvedSkill) => {
                TIDINGS_PACKET
            }
            Some(super::clock::Kind::EvolvedNormal(1)) if state.evolve_melee_stage == 2 => {
                BESTOWED_GRACE_PACKET
            }
            _ => {
                return Err(
                    "Blackbird evolved hold requires an unfinished evolved Normal2, Bestowed Grace or Evolved Skill montage"
                        .into(),
                );
            }
        }
    } else {
        require(request)?;
        match unfinished {
            Some(super::clock::Kind::OrdinaryNormal(stage)) if stage < 3 => ROOSTCALL_PACKET,
            _ => {
                return Err(
                    "Blackbird hold requires an unfinished ordinary Normal1/2/3 montage".into(),
                );
            }
        }
    };
    let timing = match packet {
        ROOSTCALL_PACKET => ROOSTCALL_TIMING,
        BESTOWED_GRACE_PACKET => BESTOWED_GRACE_TIMING,
        _ => TIDINGS_TIMING,
    };
    Ok(Some(NormalActionOverride {
        packet,
        role: None,
        model: MODEL,
        timing: Some((
            input
                .duration_us
                .checked_add(timing.0)
                .ok_or("Blackbird branch recovery overflow")?,
            input
                .duration_us
                .checked_add(timing.1)
                .ok_or("Blackbird branch hit overflow")?,
        )),
    }))
}

/// A held branch cannot start before the current montage's earliest cancel gate.
pub(crate) fn input_window(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    now: u64,
) -> Result<u64, String> {
    if !accepted(request) {
        return Err("Blackbird hold input window requires branches_v1".into());
    }
    replacement(
        request,
        actor,
        GameHoldInput {
            duration_us: HOLD_US,
            moving: false,
            reserved_stocks: None,
        },
        now,
    )?;
    super::clock::earliest(actor, now)?
        .ok_or("Blackbird branch requires an unfinished owner montage".into())
}

pub(crate) fn recovery_override(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    command_index: usize,
    now: u64,
) -> Result<Option<(u64, crate::HypothesisId)>, String> {
    let Some(input) = request.hold_inputs.get(&command_index).copied() else {
        return Ok(None);
    };
    if replacement(request, actor, input, now)?.is_none() {
        return Ok(None);
    }
    let deadline = super::clock::earliest(actor, now)?
        .ok_or("Blackbird branch requires an unfinished owner montage")?;
    Ok(Some((deadline, MODEL)))
}

fn reset_stages(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let state = ctx.actor.character.blackbird_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.evolve_melee_stage = 0;
    state.evolve_melee_started_us = None;
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    let Some(packet) = ctx
        .packet_metadata
        .map(|metadata| metadata.damage_effect_id.as_str())
    else {
        return Ok(false);
    };
    let (groups, evolved, expected) = match packet {
        ROOSTCALL_PACKET => (ROOSTCALL_GROUPS, false, "GA_BlackBird_Melee"),
        BESTOWED_GRACE_PACKET => (BESTOWED_GRACE_GROUPS, true, "GA_BlackBird_Evolve_Melee"),
        TIDINGS_PACKET => (TIDINGS_GROUPS, true, "GA_BlackBird_Evolve_Melee"),
        _ => return Ok(false),
    };
    require(ctx.request)?;
    if evolved {
        require_evolved(ctx.request)?;
        if !super::form::active(ctx.actor) {
            return Err("Blackbird evolved branch requires active transformation form".into());
        }
    } else if super::form::active(ctx.actor) {
        return Err("Blackbird ordinary branch cannot run in transformation form".into());
    }
    if ctx.ability_id != expected {
        return Err("Blackbird branch requires its Normal input family".into());
    }
    ctx.record_hypothesis(MODEL)?;
    ctx.scope_primary_aoe(packet)?;
    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(super::skill::ALL_IN_RANGE_HYPOTHESIS_ID)?;
        ctx.record_hypothesis(super::skill::GROUPS_HYPOTHESIS_ID)?;
    }
    for _ in 1..groups {
        ctx.schedule_native_aoe_packet(packet, ctx.hit_time_us)?;
    }
    reset_stages(ctx)?;
    Ok(true)
}
