use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{CallbackContext, DamageContext, HitModifiers};
use crate::simulation::metadata::PacketMetadata;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt, GearProcConfig};
use ntesim_core::execution::CausalPriority;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsWhaleV1;

pub(crate) const BROKEN_STATUS_TAG: &str = "ntesim.status.broken";

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) ready_at_us: u64,
}

pub(crate) fn initialize(ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    if ctx.request.actors.iter().any(|actor| {
        actor
            .gear_procs
            .iter()
            .any(|proc| matches!(proc, GearProcConfig::WhaleAtk { .. }))
    }) {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            HYPOTHESIS_ID,
        )?;
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    _packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::WhaleAtk {
            broken_damage_bonus,
            ..
        } = proc
        else {
            continue;
        };
        let broken = crate::simulation::state::target_tagged_status_stacks(
            ctx.buffs,
            ctx.target_index,
            BROKEN_STATUS_TAG,
            ctx.world_time_us,
        )?;
        if broken > 0 {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*broken_damage_bonus)?)?;
        }
    }
    Ok(modifiers)
}

fn reserve_cooldown(state: &mut State, time_us: u64, cooldown_us: u64) -> Result<bool, String> {
    if time_us < state.ready_at_us {
        return Ok(false);
    }
    state.ready_at_us = time_us
        .checked_add(cooldown_us)
        .ok_or("Song of the Whale kill-heal cooldown deadline overflow")?;
    Ok(true)
}

pub(crate) fn on_target_defeated(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
) -> Result<(), String> {
    let config = &ctx.request.actors[actor_index];
    let time_us = ctx.state.current_time_us;
    for proc in &config.gear_procs {
        let GearProcConfig::WhaleAtk {
            kill_heal,
            kill_heal_cooldown_us,
            ..
        } = proc
        else {
            continue;
        };
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            HYPOTHESIS_ID,
        )?;
        let broken = crate::simulation::state::target_tagged_status_stacks(
            &ctx.state.buffs,
            target_index,
            BROKEN_STATUS_TAG,
            ctx.state.mode_clock.relative_us(),
        )?;
        if broken == 0 {
            continue;
        }
        if !reserve_cooldown(
            &mut ctx.state.actors[actor_index].equipment.whale,
            time_us,
            *kill_heal_cooldown_us,
        )? {
            continue;
        }
        let hp_max = crate::simulation::state::effective_hp_max(
            &ctx.state.buffs,
            ctx.state.actors[actor_index].hp_max_base,
            &ctx.state.actors[actor_index].hp_max_static_bonus,
            actor_index,
            ctx.state.actors[actor_index].hp_max,
        )?;
        let base_heal =
            ExactFraction::new(hp_max.into(), 1)?.mul(ExactFraction::from_ratio(*kill_heal)?)?;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::WhaleHeal {
                        actor_index,
                        target_index,
                        base_heal,
                    },
                ),
            });
    }
    Ok(())
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    base_heal: ExactFraction,
) -> Result<(), String> {
    ctx.state.max_proc_depth_reached = ctx.state.max_proc_depth_reached.max(1);
    let before = ctx
        .state
        .actors
        .get(actor_index)
        .ok_or("Song of the Whale owner actor missing")?
        .hp;
    if before == 0 {
        let gross = base_heal.round_u64()?;
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ProcCallback".into(),
            event_kind: "whale_heal".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                target_index: Some(target_index),
                gross: Some(gross),
                applied: Some(0),
                overflow: Some(gross),
                reason: Some("dead_wearer".into()),
                ..Default::default()
            }),
        });
        return Ok(());
    }
    let settlement = ctx.settle_healing(actor_index, actor_index, base_heal)?;
    let gross_heal = settlement.gross;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "whale_heal".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            gross: Some(gross_heal),
            applied: Some(settlement.applied),
            overflow: Some(settlement.excess),
            hp_before: Some(before),
            hp_after: Some(settlement.after_hp),
            ..Default::default()
        }),
    });
    Ok(())
}
