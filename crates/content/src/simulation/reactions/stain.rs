use super::{EventContext, ExactFraction, REACTION_SOURCE, ReactionSnapshot, trace};
use crate::runtime_api::DamageContext;
use crate::simulation::GameEventPayload;
use crate::simulation::metadata::{DamageElement, PacketMetadata};
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffManager, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};
use ntesim_core::combat::DamageKind;
use std::collections::BTreeSet;

pub(super) const DURATION_US: u64 = 12_000_000;
pub(crate) const STATUS_TAG: &str = "ntesim.status.stain";
const STATUS_ID: &str = "ntesim.reaction.stain";
const NATIVE_STATUS_TAG: &str = "Ability.Player.Buff.Reaction.HunXiang";
const BASE_BONUS_STAT: &str = "reaction_stain_base_bonus";

pub(super) fn apply(ctx: &mut EventContext<'_>, snapshot: ReactionSnapshot) -> Result<(), String> {
    let target = ctx.request.aim_target(snapshot.command_index);
    let mut assumptions = Default::default();
    let policy = crate::registry::stain_enhancement(ctx.request, ctx.state, &mut assumptions)?;
    let duration_us = policy
        .as_ref()
        .map_or(DURATION_US, |policy| policy.duration_us);
    let bonus = policy.map_or(crate::formula::Ratio::zero(), |policy| policy.base_bonus);
    // Keep one definition shape so a later unenhanced generation can replace
    // an enhanced one after the source dies, with no stale bonus or new stack.
    let stats = std::collections::BTreeMap::from([(BASE_BONUS_STAT.into(), bonus)]);
    ctx.state
        .buffs
        .apply_buff(
            BuffDefinition {
                stacking_scope: BuffStackingScope::Shared,
                source_lifetime: Default::default(),
                source: REACTION_SOURCE.into(),
                recipient: BuffRecipient::SpecificTarget(target.into()),
                id: STATUS_ID.into(),
                duration_us: Some(duration_us),
                stat_modifiers: stats,
                semantic_tags: BTreeSet::from([STATUS_TAG.into(), NATIVE_STATUS_TAG.into()]),
                max_stacks: 1,
                refresh_policy: RefreshPolicy::Reset,
                decay_policy: DecayPolicy::AllAtOnce,
            },
            snapshot.owner_index,
            ctx.state.mode_clock.relative_us(),
        )
        .map_err(|error| format!("Stain status application failed: {error:?}"))?;
    ctx.state.execution_model_assumptions.extend(assumptions);
    trace(
        ctx,
        "reaction_stain_applied",
        Some(snapshot.owner_index),
        Some(GameEventPayload {
            target_index: Some(target),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    );
    Ok(())
}

pub(crate) fn damage_multiplier(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
) -> Result<ExactFraction, String> {
    let one = ExactFraction::new(1, 1)?;
    if packet.damage_kind != DamageKind::Elemental
        || !matches!(
            packet.element,
            Some(DamageElement::Psyche | DamageElement::Lakshana)
        )
        || crate::simulation::state::target_tagged_status_stacks(
            ctx.buffs,
            ctx.target_index,
            STATUS_TAG,
            ctx.world_time_us,
        )? == 0
    {
        return Ok(one);
    }
    let live_ci = ctx
        .buff_stats
        .get("cycle_intensity")
        .copied()
        .unwrap_or(crate::formula::Ratio::zero());
    let ci = ExactFraction::from_ratio(ctx.config.static_stats.cycle_intensity)?
        .add(ExactFraction::from_ratio(live_ci)?)?;
    // Stain is a separate zone using the damage actor's live E(CI), not Q(CI)
    // or the pair winner's captured intensity.
    let efficiency = one.add(
        ci.mul(ExactFraction::new(1, 5)?)?
            .div(ci.add(ExactFraction::new(180, 1)?)?)?,
    )?;
    let target_stats = ctx
        .buffs
        .get_stat_modifiers_for_target(&ctx.target_index.to_string(), ctx.world_time_us)
        .map_err(|error| format!("Stain target modifier query failed: {error:?}"))?;
    let bonus = target_stats
        .get(BASE_BONUS_STAT)
        .copied()
        .unwrap_or(crate::formula::Ratio::zero());
    ExactFraction::new(6, 5)?
        .add(ExactFraction::from_ratio(bonus)?)?
        .mul(efficiency)
}

pub(crate) fn clear_target(buffs: &mut BuffManager, target_index: usize) {
    buffs.remove_target_buff(STATUS_ID, REACTION_SOURCE, &target_index.to_string());
}
