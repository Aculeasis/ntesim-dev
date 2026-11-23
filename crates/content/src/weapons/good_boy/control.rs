use crate::runtime_api::CastContext;
use crate::simulation::{ExactFraction, GameControlApplication, GameEventPayload, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffDuration, BuffRecipient, BuffStackingScope};

pub(crate) fn on_control(
    ctx: &mut CastContext<'_>,
    control: &GameControlApplication,
) -> Result<(), String> {
    if ctx.action_metadata.input_role
        != Some(crate::simulation::metadata::InputActionRole::Ultimate)
    {
        return Ok(());
    }
    if control.source_actor_index != ctx.actor_index || control.causal_key()? != ctx.command_index {
        return Err("control receipt does not match its producing hit".into());
    }
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::GoodBoyTeamAtk {
            team_atk_bonus,
            control_atk_bonus,
            ..
        } = proc
        else {
            continue;
        };
        if control_atk_bonus.numerator == 0 {
            continue;
        }
        ctx.record_hypothesis(super::HYPOTHESIS_ID)?;
        let window = ctx.actor.equipment.good_boy;
        if window.command_index != Some(ctx.command_index)
            || window.upgraded
            || ctx.time_us >= window.expires_at_us
        {
            continue;
        }
        // A later same-name cast by another wearer owns the shared snapshot.
        let owns_window = ctx.buffs.instances().iter().any(|buff|
            buff.id == "buff_fork_mofeikesi_atk" && buff.source == "weapon:fork_mofeikesi"
            && buff.owner_actor == ctx.actor_index
            && matches!(buff.duration, BuffDuration::Timed { expires_at_us, .. } if expires_at_us == ctx.world_time_us + window.expires_at_us.saturating_sub(ctx.time_us))
        );
        if !owns_window {
            continue;
        }
        let total = ExactFraction::from_ratio(*team_atk_bonus)?
            .add(ExactFraction::from_ratio(*control_atk_bonus)?)?;
        let grant = crate::modifiers::BuffGrantBuilder::new(
            ctx.actor_index,
            "buff_fork_mofeikesi_atk",
            "weapon:fork_mofeikesi",
        )
        .recipient(BuffRecipient::TeamAll)
        .stacking_scope(BuffStackingScope::Shared)
        .duration_us(window.expires_at_us - ctx.time_us)
        .stat("atk_percent", total)
        .receipt(
            "good_boy_control_atk",
            GameEventPayload {
                atk_percent: Some(total),
                command_index: control.command_index,
                event_index: control.autonomous_event_index,
                target_index: Some(control.target_index),
                expires_at_us: Some(window.expires_at_us),
                ..Default::default()
            },
        )
        .into_event();
        crate::gear::schedule_cast_buff(ctx, grant)?;
        ctx.actor.equipment.good_boy.upgraded = true;
    }
    Ok(())
}
