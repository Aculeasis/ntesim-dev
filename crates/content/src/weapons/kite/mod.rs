use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::metadata::{DamageElement, InputActionRole, PacketMetadata};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GearProcConfig};
use ntesim_core::buffs::dispatch::{
    BuffDuration, BuffManager, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsKiteV2;

const BUFF_ID: &str = "kite.redirect_skill_atk_up";
const SOURCE: &str = "weapon:fork_Kite";

fn window_active(buffs: &BuffManager, actor_index: usize, now_us: u64) -> bool {
    buffs.instances().iter().any(|buff| {
        buff.id == BUFF_ID
            && buff.source == SOURCE
            && buff.owner_actor == actor_index
            && buff.recipient == BuffRecipient::SelfActor
            && matches!(
                buff.duration,
                BuffDuration::Timed { expires_at_us, .. } if expires_at_us > now_us
            )
    })
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::KiteAtk {
            atk_bonus,
            duration_us,
            ..
        } = proc
        else {
            continue;
        };
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        if ctx.action_metadata.input_role != Some(InputActionRole::RedirectSkill) {
            continue;
        }
        let bonus = ExactFraction::from_ratio(*atk_bonus)?;
        crate::simulation::state::apply_timed_buff(
            ctx.buffs,
            TimedBuffDeclaration {
                id: BUFF_ID.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::SelfActor,
                duration_us: *duration_us,
                stacking_scope: BuffStackingScope::PerSource,
                refresh_policy: RefreshPolicy::Disallow,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: BTreeMap::from([("atk_percent".into(), bonus)]),
            },
            ctx.actor_index,
            ctx.world_time_us,
        )
        .map_err(|error| format!("Watch Your Heads! buff grant failed: {error}"))?;
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::KiteAtk { lakshana_bonus, .. } = proc else {
            continue;
        };
        if packet.element != Some(DamageElement::Lakshana) {
            continue;
        }
        if !window_active(ctx.buffs, ctx.actor_index, ctx.world_time_us) {
            continue;
        }
        for tag in [
            crate::simulation::reactions::REMORA_STATUS_TAG,
            crate::simulation::reactions::STAIN_STATUS_TAG,
        ] {
            if crate::simulation::state::target_tagged_status_stacks(
                ctx.buffs,
                ctx.target_index,
                tag,
                ctx.world_time_us,
            )? > 0
            {
                modifiers = modifiers.add_damage(ExactFraction::from_ratio(*lakshana_bonus)?)?;
                break;
            }
        }
    }
    Ok(modifiers)
}
