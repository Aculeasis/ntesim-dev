use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::CallbackContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::{ExactFraction, GameEventReceipt, GearProcConfig};
use ntesim_core::execution::CausalPriority;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsRishiV2;
pub(crate) const ARC_ABILITY: &str = "weapon:fork_rishi:arc";

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) field_until_us: u64,
    pub(crate) defeats_used: u32,
}

pub(crate) fn arc_parameters(
    actor: &crate::simulation::GameActorConfig,
) -> Result<(u64, u64), String> {
    crate::build::weapon_wearer_item(
        ntesim_data::stat_projection::frozen_tables(),
        &actor.character_id,
        "fork_rishi",
        "Eclipse",
    )?;
    actor
        .gear_procs
        .iter()
        .find_map(|proc| match proc {
            GearProcConfig::RishiAtk {
                eclipse_duration_us,
                arc_cooldown_us,
                ..
            } => Some((*eclipse_duration_us, *arc_cooldown_us)),
            _ => None,
        })
        .ok_or_else(|| "Arc: Eclipse requires its projected passive".into())
}

pub(crate) fn activate_arc(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    duration_us: u64,
) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    let time_us = ctx.state.current_time_us;
    let eclipse = &mut ctx.state.actors[actor_index].equipment.rishi;
    eclipse.field_until_us = time_us
        .checked_add(duration_us)
        .ok_or("Eclipse field deadline overflow")?;
    eclipse.defeats_used = 0;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us,
        priority: CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(ContentEvent::RishiEclipseField { actor_index }),
    });
    Ok(())
}

pub(crate) fn on_target_defeated(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
) -> Result<(), String> {
    let config = &ctx.request.actors[actor_index];
    let actor = &mut ctx.state.actors[actor_index];
    if actor.hp == 0 {
        return Ok(());
    }
    let time_us = ctx.state.current_time_us;
    for proc in &config.gear_procs {
        let GearProcConfig::RishiAtk {
            defeat_energy,
            max_defeats,
            ..
        } = proc
        else {
            continue;
        };
        let eclipse = &mut actor.equipment.rishi;
        if time_us >= eclipse.field_until_us || eclipse.defeats_used >= *max_defeats {
            continue;
        }
        eclipse.defeats_used += 1;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us,
            priority: CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::RishiEclipseEnergy {
                actor_index,
                amount: *defeat_energy,
            }),
        });
    }
    let _ = target_index;
    Ok(())
}

pub(crate) fn apply_field(ctx: &mut CallbackContext<'_>, actor_index: usize) -> Result<(), String> {
    if ctx.state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let eclipse = &state.actors[actor_index].equipment.rishi;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "rishi_eclipse_field".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            expires_at_us: Some(eclipse.field_until_us),
            count: Some(
                usize::try_from(eclipse.defeats_used).map_err(|_| "Rishi defeats overflow")?,
            ),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    amount: u64,
) -> Result<(), String> {
    if ctx.state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let actor = &mut state.actors[actor_index];
    let before = crate::simulation::energy::EnergySnapshot::capture(actor);
    let charge_basis = ExactFraction::new(amount.into(), 1)?;
    let _ = actor.grant_energy(amount)?;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "rishi_eclipse_energy".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: Some(before.receipt(
            actor,
            crate::simulation::GameEnergySource::RishiEclipse,
            actor_index,
            actor_index,
            Some(charge_basis),
        )?),
        energy_fanout: None,
        data: None,
    });
    crate::simulation::energy::notify_active_actor_energy_gain(ctx, actor_index, amount > 0)?;
    Ok(())
}
