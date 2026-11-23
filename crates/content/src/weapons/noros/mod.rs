use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::CastContext;
use crate::simulation::GearProcConfig;
use crate::simulation::metadata::InputActionRole;
use ntesim_core::buffs::dispatch::BuffRecipient;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) ready_at_us: u64,
}

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsNorosV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.action_metadata.input_role != Some(InputActionRole::RedirectSkill) {
        return Ok(());
    }
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::NorosAtk {
            atk_bonus,
            duration_us,
            cooldown_us,
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.time_us < ctx.actor.equipment.noros.ready_at_us {
                continue;
            }
            ctx.actor.equipment.noros.ready_at_us = ctx
                .time_us
                .checked_add(*cooldown_us)
                .ok_or("Noros cooldown deadline overflow")?;
            let grant = crate::modifiers::BuffGrantBuilder::new(
                ctx.actor_index,
                "buff_fork_nonos_atk",
                "weapon:fork_nonos",
            )
            .recipient(BuffRecipient::SelfActor)
            .duration_us(*duration_us)
            .stat_ratio("atk_percent", *atk_bonus)?
            .receipt(
                "noros_atk",
                crate::simulation::GameEventPayload {
                    atk_percent: Some(crate::simulation::ExactFraction::from_ratio(*atk_bonus)?),
                    duration_us: Some(*duration_us),
                    ..Default::default()
                },
            )
            .into_event();
            crate::gear::schedule_cast_buff(ctx, grant)?;
        }
    }
    Ok(())
}
