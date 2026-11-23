use crate::callbacks::ContentEvent;
use crate::runtime_api::CastContext;
use crate::simulation::GearProcConfig;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use ntesim_core::execution::CausalPriority;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let actor_idx = ctx.actor_index;
    let is_redirect_skill = ctx.action_metadata.input_role
        == Some(crate::simulation::metadata::InputActionRole::RedirectSkill);
    let actor = &mut *ctx.actor;
    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;

    for proc in &actor_config.gear_procs {
        #[allow(clippy::collapsible_if)]
        if let GearProcConfig::MindRoyaleEnergy {
            energy_grant,
            cooldown_us,
        } = proc
        {
            if is_redirect_skill && ctx.time_us >= actor.equipment.mind_royale.ready_at_us {
                actor.equipment.mind_royale.ready_at_us = ctx
                    .time_us
                    .checked_add(*cooldown_us)
                    .ok_or("Mind Royale cooldown deadline overflow")?;
                crate::runtime_api::increment_sequence(seq)?;
                queue.push(ScheduledGameEvent {
                    time_us: ctx.time_us,
                    priority: CausalPriority::ProcCallback,
                    seq: *seq,
                    data: GameEventData::Content(ContentEvent::MindRoyaleEnergyGrant {
                        actor_index: actor_idx,
                        amount: *energy_grant,
                    }),
                });
            }
        }
    }
    Ok(())
}
