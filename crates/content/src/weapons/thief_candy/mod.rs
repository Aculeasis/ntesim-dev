use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::CastContext;
use crate::simulation::state::{TimedBuffDeclaration, apply_timed_buff};
use crate::simulation::{ExactFraction, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsThiefCandyV1;

pub(crate) const CRITICAL_DODGE_TAG: &str = "ntesim.trigger.critical_dodge";
pub(crate) const BUFF_ID: &str = "thief_candy.critical_dodge_damage_up";
pub(crate) const SOURCE: &str = "weapon:fork_ThiefCandy";

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !ctx
        .action_metadata
        .gameplay_tags
        .iter()
        .any(|tag| tag == CRITICAL_DODGE_TAG)
    {
        return Ok(());
    }
    apply_critical_dodge(ctx)
}

pub(crate) fn on_critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    apply_critical_dodge(ctx)
}

fn apply_critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::ThiefCandyDodge {
            damage_bonus,
            duration_us,
        } = proc
        else {
            continue;
        };
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        let bonus = ExactFraction::from_ratio(*damage_bonus)?;
        let declaration = TimedBuffDeclaration {
            id: BUFF_ID.into(),
            source: SOURCE.into(),
            recipient: BuffRecipient::SelfActor,
            duration_us: *duration_us,
            stacking_scope: BuffStackingScope::PerSource,
            refresh_policy: RefreshPolicy::Reset,
            max_stacks: 3,
            decay_policy: DecayPolicy::AllAtOnce,
            stat_modifiers: BTreeMap::from([("damage_bonus".into(), bonus)]),
        };
        let actor_index = ctx.actor_index;
        apply_timed_buff(&mut *ctx.buffs, declaration, actor_index, ctx.world_time_us)
            .map_err(|error| format!("Thief's Candy buff failed: {error:?}"))?;
    }
    Ok(())
}
