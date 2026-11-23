use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::CastContext;
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::{TimedBuffDeclaration, apply_timed_buff};
use crate::simulation::{ExactFraction, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsCrowbarV1;
pub(crate) const BUFF_ID: &str = "crowbar.break_intensity";
pub(crate) const SOURCE: &str = "weapon:fork_Crowbar";

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.action_metadata.input_role != Some(InputActionRole::RedirectSkill) {
        return Ok(());
    }
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::CrowbarBreak {
            break_intensity_bonus,
            duration_us,
        } = proc
        else {
            continue;
        };
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        let bonus = ExactFraction::from_ratio(*break_intensity_bonus)?;
        let declaration = TimedBuffDeclaration {
            id: BUFF_ID.into(),
            source: SOURCE.into(),
            recipient: BuffRecipient::SpecificActor(ctx.actor_index),
            duration_us: *duration_us,
            stacking_scope: BuffStackingScope::PerSource,
            refresh_policy: RefreshPolicy::Disallow,
            max_stacks: 1,
            decay_policy: DecayPolicy::AllAtOnce,
            stat_modifiers: BTreeMap::from([("break_intensity".into(), bonus)]),
        };
        let actor_index = ctx.actor_index;
        apply_timed_buff(&mut *ctx.buffs, declaration, actor_index, ctx.world_time_us)
            .map_err(|error| format!("Time Bandit buff failed: {error:?}"))?;
    }
    Ok(())
}
