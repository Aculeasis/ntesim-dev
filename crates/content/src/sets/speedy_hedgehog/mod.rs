mod config;
use crate::runtime_api::CastContext;
use crate::simulation::{ExactFraction, GameEventPayload, GearProcConfig};
pub(crate) use config::{validate_config, validate_deadline};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.action_metadata.input_role
        != Some(crate::simulation::metadata::InputActionRole::Ultimate)
    {
        return Ok(());
    }
    let config = ctx.config;
    for proc in &config.gear_procs {
        if let GearProcConfig::SpeedyHedgehogTeamAtk {
            team_atk_bonus,
            duration_us,
        } = proc
        {
            let atk_percent = ExactFraction::from_ratio(*team_atk_bonus)?;
            let expires_at_us = ctx
                .time_us
                .checked_add(*duration_us)
                .ok_or("Speedy Hedgehog deadline overflow")?;
            let grant = crate::modifiers::BuffGrantBuilder::new(
                ctx.actor_index,
                "speedy_hedgehog_team_atk",
                "set:speedy_hedgehog",
            )
            .recipient(BuffRecipient::TeamAll)
            .stacking_scope(BuffStackingScope::Shared)
            .duration_us(*duration_us)
            .stat("atk_percent", atk_percent)
            .receipt(
                "speedy_hedgehog_team_atk",
                GameEventPayload {
                    atk_percent: Some(atk_percent),
                    expires_at_us: Some(expires_at_us),
                    ..Default::default()
                },
            )
            .into_event();
            crate::gear::schedule_cast_buff(ctx, grant)?;
        }
    }
    Ok(())
}
