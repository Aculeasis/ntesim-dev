use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::CastContext;
use crate::simulation::metadata::InputActionRole;
use crate::simulation::{ExactFraction, GameEventPayload, GearProcConfig};
use ntesim_core::buffs::dispatch::BuffRecipient;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsEternalWaltzV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::EternalWaltz {
            mental_bonus,
            duration_us,
            ..
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.action_metadata.input_role == Some(InputActionRole::Ultimate) {
                let bonus = ExactFraction::from_ratio(*mental_bonus)?;
                let grant = crate::modifiers::BuffGrantBuilder::new(
                    ctx.actor_index,
                    "eternal_waltz.mental_damage",
                    "weapon:fork_Arachne",
                )
                .recipient(BuffRecipient::SelfActor)
                .duration_us(*duration_us)
                .stat("mental_damage_bonus", bonus)
                .receipt(
                    "eternal_waltz_mental",
                    GameEventPayload {
                        mental_damage_bonus: Some(bonus),
                        duration_us: Some(*duration_us),
                        ..Default::default()
                    },
                )
                .into_event();
                crate::gear::schedule_cast_buff(ctx, grant)?;
            }
        }
    }
    Ok(())
}
