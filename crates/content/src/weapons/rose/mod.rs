use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::metadata::{InputActionRole, PacketMetadata};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GearProcConfig};
use ntesim_core::buffs::dispatch::{
    BuffDuration, BuffManager, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsRoseV1;
pub(crate) const BREAK_EXTENSION_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::WeaponsRoseBreakExtensionV1;

pub(crate) const THORN_BUFF_ID: &str = "rose.chaos_thorn";
pub(crate) const ICD_BUFF_ID: &str = "rose.thorn_icd";
pub(crate) const SOURCE: &str = "weapon:fork_Rose";
pub(crate) const ICD_US: u64 = 300_000;
pub(crate) const MAX_THORN_STACKS: u32 = 10;

fn buff_active(buffs: &BuffManager, actor_index: usize, buff_id: &str, now_us: u64) -> bool {
    buffs.instances().iter().any(|buff| {
        buff.id == buff_id
            && buff.source == SOURCE
            && buff.owner_actor == actor_index
            && buff.recipient == BuffRecipient::SelfActor
            && matches!(
                buff.duration,
                BuffDuration::Timed { expires_at_us, .. } if expires_at_us > now_us
            )
    })
}

fn thorn_stacks(buffs: &BuffManager, actor_index: usize, now_us: u64) -> u32 {
    buffs
        .instances()
        .iter()
        .filter(|buff| {
            buff.id == THORN_BUFF_ID
                && buff.source == SOURCE
                && buff.owner_actor == actor_index
                && buff.recipient == BuffRecipient::SelfActor
                && matches!(
                    buff.duration,
                    BuffDuration::Timed { expires_at_us, .. } if expires_at_us > now_us
                )
        })
        .map(|buff| buff.stacks)
        .sum()
}

pub(crate) fn grant_thorn_stacks(
    buffs: &mut BuffManager,
    actor_index: usize,
    now_us: u64,
    stacks: u32,
    duration_us: u64,
    respect_icd: bool,
) -> Result<bool, String> {
    if respect_icd && buff_active(buffs, actor_index, ICD_BUFF_ID, now_us) {
        return Ok(false);
    }
    for _ in 0..stacks {
        crate::simulation::state::apply_timed_buff(
            buffs,
            TimedBuffDeclaration {
                id: THORN_BUFF_ID.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::SelfActor,
                duration_us,
                stacking_scope: BuffStackingScope::PerSource,
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: MAX_THORN_STACKS,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: BTreeMap::new(),
            },
            actor_index,
            now_us,
        )
        .map_err(|error| format!("Bohemian Rose thorn grant failed: {error}"))?;
    }
    if respect_icd {
        crate::simulation::state::apply_timed_buff(
            buffs,
            TimedBuffDeclaration {
                id: ICD_BUFF_ID.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::SelfActor,
                duration_us: ICD_US,
                stacking_scope: BuffStackingScope::PerSource,
                refresh_policy: RefreshPolicy::Disallow,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: BTreeMap::new(),
            },
            actor_index,
            now_us,
        )
        .map_err(|error| format!("Bohemian Rose thorn ICD grant failed: {error}"))?;
    }
    Ok(true)
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.action_metadata.input_role != Some(InputActionRole::RedirectSkill) {
        return Ok(());
    }
    grant(ctx, MAX_THORN_STACKS, false)
}

pub(crate) fn on_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !ctx.packet_metadata.is_some_and(|packet| {
        packet
            .semantic_tags
            .contains(crate::characters::packet_semantics::DOT_DAMAGE_TAG)
    }) {
        return Ok(());
    }
    grant(ctx, 1, true)
}

pub(crate) fn on_broken_target_hit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    execution_assumptions: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let Some(config) = ctx.request.actors.get(actor_index) else {
        return Err("Bohemian Rose hit actor index is out of range".into());
    };
    for proc in &config.gear_procs {
        let GearProcConfig::RoseThorn {
            broken_extension_us,
            ..
        } = proc
        else {
            continue;
        };
        let can_extend = ctx
            .state
            .targets
            .get(target_index)
            .and_then(|target| {
                target
                    .break_state
                    .as_ref()
                    .map(|break_state| (target, break_state))
            })
            .is_some_and(|(target, break_state)| {
                target.hp_current > 0
                    && break_state
                        .broken_until_us
                        .is_some_and(|deadline| deadline > ctx.state.current_time_us)
                    && !break_state.rose_extension_used
            });
        if !can_extend {
            continue;
        }
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            execution_assumptions,
            BREAK_EXTENSION_HYPOTHESIS_ID,
        )?;
        crate::simulation::break_state::extend_broken_for_rose(
            ctx.request,
            ctx.state,
            ctx.queue,
            ctx.sequence,
            ctx.trace,
            target_index,
            *broken_extension_us,
        )?;
    }
    Ok(())
}

fn grant(ctx: &mut CastContext<'_>, stacks: u32, respect_icd: bool) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::RoseThorn { duration_us, .. } = proc else {
            continue;
        };
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        grant_thorn_stacks(
            ctx.buffs,
            ctx.actor_index,
            ctx.world_time_us,
            stacks,
            *duration_us,
            respect_icd,
        )?;
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    _packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::RoseThorn {
            crit_damage_per_stack,
            ..
        } = proc
        else {
            continue;
        };
        let stacks = thorn_stacks(ctx.buffs, ctx.actor_index, ctx.world_time_us);
        if stacks > 0 {
            let total = ExactFraction::from_ratio(*crit_damage_per_stack)?
                .mul(ExactFraction::new(stacks.into(), 1)?)?;
            modifiers = modifiers.add_crit_damage(total)?;
        }
    }
    Ok(modifiers)
}
