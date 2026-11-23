//! Continuous held button: reached section gates, never caller-selected stages.
use super::Kind;
use crate::runtime_api::{CastContext, NormalActionOverride};
use crate::simulation::{GameActionCommand, GameScenarioRequest, state::ActorRuntime};
const RESUME_HOLD_US: u64 = 200_000;

pub(super) fn fresh(state: &crate::characters::nanally::awakenings::State, now: u64) -> bool {
    state
        .normal_started_us
        .is_some_and(|start| now.saturating_sub(start) <= crate::characters::normal::RESET_GAP_US)
}

fn resume_kind(actor: &ActorRuntime, now: u64) -> Option<Kind> {
    let crate::characters::RuntimeState::Nanally(state) = &actor.character else {
        return None;
    };
    if !fresh(state, now) {
        return None;
    }
    state
        .heavy_resume
        .filter(|stage| matches!(stage, 1 | 2))
        .map(Kind::Heavy)
}

pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    for action in &request.actions {
        if let GameActionCommand::InvokeAbility {
            actor_index,
            damage_effect_id,
            ..
        } = action
            && request.actors[*actor_index].character_id == "character:1010"
            && super::profile::HEAVY
                .iter()
                .flat_map(|hits| hits.iter())
                .any(|(packet, _)| packet == damage_effect_id)
        {
            return Err("Nanally heavy rows are owner-selected; use canonical held Normal".into());
        }
    }
    for (&command, input) in &request.hold_inputs {
        let Some(GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            timestamp_us,
            ..
        }) = request.actions.get(command)
        else {
            continue;
        };
        if request.actors[*actor_index].character_id != "character:1010" {
            continue;
        }
        if ability_id != "GA_Nanally_Melee"
            || damage_effect_id != "GE_Player_Nanally_Melee1_Damage"
            || input.moving
            || input.reserved_stocks.is_some()
            || request.plunge_inputs.contains_key(&command)
        {
            return Err(
                "Nanally held input requires canonical stationary unreserved Normal without plunge"
                    .into(),
            );
        }
        timestamp_us
            .checked_add(input.duration_us)
            .and_then(|time| time.checked_add(3_706_608))
            .ok_or("Nanally held action deadline overflow")?;
    }
    Ok(())
}

pub(crate) fn replacement(
    actor: &ActorRuntime,
    input: crate::simulation::GameHoldInput,
    now: u64,
) -> Result<Option<NormalActionOverride>, String> {
    if matches!(&actor.character, crate::characters::RuntimeState::Nanally(state) if state.riposte_ready)
    {
        return Ok(None);
    }
    if actor.aiming && input.duration_us > 0 {
        return Err("Nanally held melee requires leaving Aim first".into());
    }
    if input.duration_us >= RESUME_HOLD_US
        && let Some(kind) = resume_kind(actor, now)
    {
        return Ok(Some(NormalActionOverride {
            packet: super::profile::hits(kind)[0].0,
            role: None,
            model: super::MODEL,
            timing: Some((
                RESUME_HOLD_US + super::profile::windows(kind).recovery,
                RESUME_HOLD_US + super::profile::hits(kind)[0].1,
            )),
        }));
    }
    Ok(None)
}

pub(crate) fn resume_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    let Some(packet) = ctx.packet_metadata else {
        return Ok(false);
    };
    let Some(stage) = super::profile::HEAVY
        .iter()
        .position(|hits| hits[0].0 == packet.damage_effect_id)
    else {
        return Ok(false);
    };
    let input = ctx
        .request
        .hold_inputs
        .get(&ctx.command_index)
        .ok_or("Nanally resumed heavy requires a new held input")?;
    let kind = Kind::Heavy(stage);
    if input.duration_us < RESUME_HOLD_US || resume_kind(ctx.actor, ctx.time_us) != Some(kind) {
        return Err("Nanally resumed heavy requires a fresh preserved dodge continuation".into());
    }
    let state = ctx.actor.character.nanally_mut()?;
    state.heavy_resume = None;
    state.normal_stage = 0;
    state.normal_started_us = Some(
        ctx.time_us
            .checked_add(RESUME_HOLD_US)
            .ok_or("Nanally heavy entry overflow")?,
    );
    super::cast_after(ctx, kind, RESUME_HOLD_US)?;
    crate::characters::nanally::underboss::schedule_heavy(ctx, stage)?;
    Ok(true)
}

pub(super) fn release(ctx: &CastContext<'_>, kind: Kind) -> Result<Option<u64>, String> {
    if !matches!(kind, Kind::Normal(_) | Kind::Riposte | Kind::Heavy(_)) {
        return Ok(None);
    }
    ctx.request
        .hold_inputs
        .get(&ctx.command_index)
        .map(|input| {
            ctx.time_us
                .checked_add(input.duration_us)
                .ok_or_else(|| "Nanally hold release overflow".into())
        })
        .transpose()
}

pub(super) fn branch(kind: Kind) -> Option<(Kind, u64)> {
    match kind {
        Kind::Normal(0 | 4) => Some((Kind::Heavy(0), 300_001)),
        Kind::Normal(1) => Some((Kind::Heavy(0), 350_001)),
        Kind::Normal(2) => Some((Kind::Heavy(0), 600_001)),
        // The fourth section requests Normal5, never an extra heavy packet.
        Kind::Normal(3) => Some((Kind::Normal(4), 900_001)),
        Kind::Riposte => Some((Kind::Heavy(0), 900_001)),
        Kind::Heavy(0) => Some((Kind::Heavy(1), 600_001)),
        Kind::Heavy(1) => Some((Kind::Heavy(2), 980_001)),
        _ => None,
    }
}

pub(super) fn advance(ctx: &mut CastContext<'_>, next: Kind, release: u64) -> Result<(), String> {
    if release <= ctx.time_us {
        return Ok(());
    }
    let state = ctx.actor.character.nanally_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = Some(ctx.time_us);
    super::continue_held(ctx, next, release)?;
    match next {
        Kind::Heavy(stage) => crate::characters::nanally::underboss::schedule_heavy(ctx, stage),
        Kind::Normal(stage) => crate::characters::nanally::underboss::schedule(ctx, Some(stage)),
        _ => Err("Nanally held transition selected an unsupported section".into()),
    }
}
