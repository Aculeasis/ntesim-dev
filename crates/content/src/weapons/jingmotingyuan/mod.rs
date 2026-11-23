use crate::HypothesisId;
pub(crate) mod arc;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::CallbackContext;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsJingmotingyuanV1;

pub(crate) const BUFF_ID: &str = "jingmotingyuan.hp_cost_crit_damage";
pub(crate) const SOURCE: &str = "weapon:fork_jingmotingyuan";
pub(crate) const MAX_STACKS: u32 = 4;

pub(crate) fn on_hp_consumed(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    actual_hp_loss: u64,
) -> Result<(), String> {
    if actual_hp_loss == 0 {
        return Ok(());
    }
    for proc in &ctx.request.actors[actor_index].gear_procs {
        let GearProcConfig::JingmotingyuanCrit {
            crit_damage_bonus,
            duration_us,
        } = proc
        else {
            continue;
        };
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            HYPOTHESIS_ID,
        )?;
        let bonus = ExactFraction::from_ratio(*crit_damage_bonus)?;
        let declaration = TimedBuffDeclaration {
            id: BUFF_ID.into(),
            source: SOURCE.into(),
            recipient: BuffRecipient::SelfActor,
            duration_us: *duration_us,
            stacking_scope: BuffStackingScope::PerSource,
            refresh_policy: RefreshPolicy::Reset,
            max_stacks: MAX_STACKS,
            decay_policy: DecayPolicy::AllAtOnce,
            stat_modifiers: BTreeMap::from([("crit_damage_bonus".into(), bonus)]),
        };
        ctx.state.apply_buff(actor_index, declaration)?;
        let deadline = ctx
            .state
            .current_time_us
            .checked_add(*duration_us)
            .ok_or("Silent Garden expiry overflow")?;
        if deadline <= ctx.request.absolute_limit_us {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue
                .push(crate::simulation::events::ScheduledGameEvent {
                    time_us: deadline,
                    priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                    seq: *ctx.sequence,
                    data: crate::simulation::events::GameEventData::Content(
                        crate::callbacks::ContentEvent::BuffLifecycleExpiry,
                    ),
                });
        }
        ctx.state.max_proc_depth_reached = ctx.state.max_proc_depth_reached.max(1);
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ProcCallback".into(),
            event_kind: "jingmotingyuan_crit_stack".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                consumed: Some(u32::try_from(actual_hp_loss).unwrap_or(u32::MAX)),
                expires_at_us: Some(deadline),
                ..Default::default()
            }),
        });
    }
    Ok(())
}
