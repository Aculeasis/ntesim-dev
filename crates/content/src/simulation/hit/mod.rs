use crate::simulation::context::EventContext;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt, GameHitReceipt};

mod calculation;
mod effects;
mod eligibility;
mod execute;
mod scaling;

pub(crate) use calculation::PreparedDamageBonuses;

pub(crate) fn effective_def(
    actor_config: &crate::simulation::GameActorConfig,
    buff_stats: &impl crate::runtime_api::BuffStatsView,
) -> Result<ExactFraction, String> {
    calculation::effective_def(actor_config, buff_stats)
}

#[derive(Clone, Copy)]
pub(super) struct HitIdentity {
    pub parent_hit_index: Option<usize>,
    pub command_index: usize,
    pub actor_index: usize,
    pub target_index: usize,
}

struct HitDelivery {
    identity: HitIdentity,
    ability_id: String,
    damage_effect_id: String,
    atk_rate: crate::formula::Ratio,
    scaling_stat: crate::simulation::NativeScalingStat,
    metadata: crate::simulation::metadata::ActionPacketMetadata,
}

pub(super) fn settle(
    ctx: &mut EventContext<'_>,
    identity: HitIdentity,
    ability_id: String,
    damage_effect_id: String,
    atk_rate: crate::formula::Ratio,
    scaling_stat: crate::simulation::NativeScalingStat,
    metadata: crate::simulation::metadata::ActionPacketMetadata,
) -> Result<(), String> {
    let delivery = HitDelivery {
        identity,
        ability_id,
        damage_effect_id,
        atk_rate,
        scaling_stat,
        metadata,
    };
    if !eligibility::can_deliver(ctx, &delivery)? {
        return Ok(());
    }
    crate::gear::validate_hit_delivery(
        ctx.request,
        &ctx.request.actors[delivery.identity.actor_index],
        &delivery.metadata,
    )?;
    let prepared = eligibility::prepare(ctx, &delivery)?;
    let buffs_ref = prepared.buffs.as_ref().unwrap_or(&ctx.state.buffs);
    let calculated = calculation::evaluate(ctx.request, ctx.state, &delivery, buffs_ref)?;
    super::reactions::validate_dot_replacement(
        ctx,
        identity.target_index,
        &delivery.metadata.packet,
    )?;
    let rolled = calculation::roll(ctx, &delivery, &calculated)?;
    let trigger_hit_index = ctx.hits.len();
    let outcome = effects::apply_damage(ctx, &delivery, prepared, &calculated, rolled)?;
    crate::registry::observe_applied_hp(
        ctx.request,
        ctx.state,
        identity.target_index,
        identity.command_index,
        &delivery.metadata,
        outcome.applied_hp_loss,
    )?;
    super::cycle::on_packet(
        ctx,
        identity.command_index,
        identity.actor_index,
        identity.target_index,
        outcome.applied_hp_loss,
        &delivery.metadata,
    )?;
    effects::after_damage(ctx, &delivery, &outcome, &calculated)?;
    if !outcome.connected {
        crate::registry::on_zero_periodic_delivery(
            &mut crate::runtime_api::CallbackContext {
                state: ctx.state,
                request: ctx.request,
                queue: ctx.queue,
                sequence: ctx.sequence,
                trace: ctx.trace,
            },
            identity.actor_index,
            identity.target_index,
            &delivery.damage_effect_id,
        )?;
    }
    if outcome.connected {
        super::reactions::on_dot_delivery(ctx, trigger_hit_index)?;
    }
    execute::after_hit(ctx, &delivery, outcome.applied_hp_loss)?;
    if outcome.applied_hp_loss > 0 {
        super::reactions::hexed_on_hp_damage(
            ctx,
            identity.target_index,
            identity.actor_index,
            identity.command_index,
            outcome.applied_hp_loss,
            &delivery.metadata.packet,
            trigger_hit_index,
        )?;
        crate::registry::observe_party_hit(
            &mut crate::runtime_api::CallbackContext {
                state: ctx.state,
                request: ctx.request,
                queue: ctx.queue,
                sequence: ctx.sequence,
                trace: ctx.trace,
            },
            trigger_hit_index,
            ctx.hits,
        )?;
    }
    Ok(())
}
