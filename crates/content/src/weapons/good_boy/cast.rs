use crate::runtime_api::CastContext;
use crate::simulation::GearProcConfig;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.action_metadata.input_role
        != Some(crate::simulation::metadata::InputActionRole::Ultimate)
    {
        return Ok(());
    }

    let actor_config = ctx.config;
    let actor_idx = ctx.actor_index;

    for proc in &actor_config.gear_procs {
        if let GearProcConfig::GoodBoyTeamAtk {
            team_atk_bonus,
            duration_us,
            control_atk_bonus,
        } = proc
        {
            if control_atk_bonus.numerator > 0 {
                ctx.record_hypothesis(super::HYPOTHESIS_ID)?;
                ctx.actor.equipment.good_boy = super::State {
                    command_index: Some(ctx.command_index),
                    expires_at_us: ctx
                        .time_us
                        .checked_add(*duration_us)
                        .ok_or("Good Boy deadline overflow")?,
                    upgraded: false,
                };
            }
            let atk_percent = crate::simulation::ExactFraction::from_ratio(*team_atk_bonus)?;
            let grant = crate::modifiers::BuffGrantBuilder::new(
                actor_idx,
                "buff_fork_mofeikesi_atk",
                "weapon:fork_mofeikesi",
            )
            .recipient(ntesim_core::buffs::dispatch::BuffRecipient::TeamAll)
            .stacking_scope(ntesim_core::buffs::dispatch::BuffStackingScope::Shared)
            .duration_us(*duration_us)
            .stat("atk_percent", atk_percent)
            .receipt(
                "good_boy_team_atk",
                crate::simulation::GameEventPayload {
                    atk_percent: Some(atk_percent),
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
