mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::{ExactFraction, GameEventPayload, GearProcConfig};
use ntesim_core::buffs::dispatch::BuffRecipient;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsBlastCandyV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.action_metadata.input_role
        != Some(crate::simulation::metadata::InputActionRole::Ultimate)
    {
        return Ok(());
    }
    let config = ctx.config;
    for proc in &config.gear_procs {
        if let GearProcConfig::BlastCandyAtk {
            atk_bonus,
            duration_us,
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            let atk_percent = ExactFraction::from_ratio(*atk_bonus)?;
            let grant = crate::modifiers::BuffGrantBuilder::new(
                ctx.actor_index,
                "buff_fork_BlastCandy_atk",
                "weapon:fork_BlastCandy",
            )
            .recipient(BuffRecipient::SelfActor)
            .duration_us(*duration_us)
            .stat("atk_percent", atk_percent)
            .receipt(
                "blast_candy_atk",
                GameEventPayload {
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
