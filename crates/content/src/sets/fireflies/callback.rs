use crate::callbacks::ContentEvent;
use crate::runtime_api::CallbackContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{GameEventPayload, GameEventReceipt};
use ntesim_core::execution::CausalPriority;

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    stack_amount: ntesim_core::formula::Ratio,
    max_stacks: u32,
    duration_us: u64,
) -> Result<(), String> {
    if ctx
        .state
        .actors
        .get(actor_index)
        .is_none_or(|actor| actor.hp == 0)
    {
        return Ok(());
    }
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    let request = ctx.request;
    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let admitted = crate::gear::active_stacks(&state.buffs, actor_index) < max_stacks;
    if admitted {
        state.apply_buff(
            actor_index,
            TimedBuffDeclaration {
                id: "fireflies.crit_damage".into(),
                source: "set:fireflies".into(),
                recipient: ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(actor_index),
                duration_us,
                stacking_scope: ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy::Independent,
                max_stacks,
                decay_policy: ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
                stat_modifiers: std::collections::BTreeMap::from([("crit_damage_bonus".into(), {
                    crate::simulation::ExactFraction::from_ratio(stack_amount)?
                })]),
            },
        )?;
        let expires_at_us = state
            .current_time_us
            .checked_add(duration_us)
            .ok_or("Fireflies expiry overflow")?;
        if expires_at_us <= request.absolute_limit_us {
            *seq = seq.checked_add(1).ok_or("event sequence overflow")?;
            queue.push(ScheduledGameEvent {
                time_us: expires_at_us,
                priority: CausalPriority::BuffTickOrExpiry,
                seq: *seq,
                data: GameEventData::Content(ContentEvent::BuffLifecycleExpiry),
            });
        }
    }

    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: if admitted {
            "fireflies_crit_stack"
        } else {
            "fireflies_stack_capped"
        }
        .into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            admitted: Some(admitted),
            stacks: Some(crate::gear::active_stacks(&state.buffs, actor_index)),
            ..Default::default()
        }),
    });
    Ok(())
}
