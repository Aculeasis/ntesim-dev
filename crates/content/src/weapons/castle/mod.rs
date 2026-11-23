use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::CastContext;
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsCallOfTheTwistedCityV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::CastleHealingBonus {
            healing_bonus,
            duration_us,
        } = proc
        else {
            continue;
        };
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        if ctx.action_metadata.input_role == Some(InputActionRole::RedirectSkill) {
            let bonus = ExactFraction::from_ratio(*healing_bonus)?;
            crate::simulation::state::apply_timed_buff(
                ctx.buffs,
                TimedBuffDeclaration::reset(
                    "call_of_the_twisted_city.healing_bonus",
                    "weapon:fork_Castle",
                    BuffRecipient::SelfActor,
                    *duration_us,
                    BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([("healing_bonus".into(), bonus)]),
                ),
                ctx.actor_index,
                ctx.world_time_us,
            )
            .map_err(|error| format!("Call of the Twisted City buff grant failed: {error}"))?;
        }
    }
    Ok(())
}
