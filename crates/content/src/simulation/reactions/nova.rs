//! Nova has one damage deadline and a separate, generation-checked removal.
use super::*;

pub(in crate::simulation) fn apply(
    ctx: &mut EventContext<'_>,
    now_us: u64,
    target_index: usize,
    snapshot: ReactionSnapshot,
) -> Result<(), String> {
    apply_with_lineage(ctx, now_us, target_index, snapshot, true)
}

fn apply_with_lineage(
    ctx: &mut EventContext<'_>,
    now_us: u64,
    target_index: usize,
    snapshot: ReactionSnapshot,
    reapply: bool,
) -> Result<(), String> {
    let tick_at = now_us
        .checked_add(NOVA_PACKET_DELAY_US)
        .ok_or("Nova packet deadline overflow")?;
    let end_at = now_us
        .checked_add(NOVA_STATUS_US)
        .ok_or("Nova expiry deadline overflow")?;
    // Refresh ends the previous generation even if its5s damage already fired.
    // The end callback flushes only still-pending damage, before notifying owners.
    if let Some(pending) = ctx.state.reactions.nova.remove(&target_index) {
        schedule(
            ctx,
            now_us,
            GameEventData::ReactionNovaEnd(Box::new(NovaEvent {
                target_index,
                generation: pending.generation,
                snapshot: pending.snapshot,
            })),
        )?;
    }
    let generation = ctx.state.reactions.next_id;
    ctx.state.reactions.next_id += 1;
    ctx.state.reactions.nova.insert(
        target_index,
        NovaGeneration {
            generation,
            snapshot: snapshot.clone(),
            expires_at_us: end_at,
        },
    );
    let event = NovaEvent {
        target_index,
        generation,
        snapshot,
    };
    schedule(
        ctx,
        tick_at,
        GameEventData::ReactionNovaDetonate(Box::new(event.clone())),
    )?;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: end_at,
        priority: CausalPriority::BuffTickOrExpiry,
        seq: *ctx.sequence,
        data: GameEventData::ReactionNovaEnd(Box::new(event.clone())),
    });
    apply_status(
        ctx,
        target_index,
        event.snapshot.owner_index,
        NOVA_STATUS_ID,
        BTreeSet::from(["Ability.Player.Buff.Reaction.AnHun".to_owned()]),
        NOVA_STATUS_US,
    )?;
    crate::registry::on_nova_created(
        &mut crate::runtime_api::CallbackContext {
            request: ctx.request,
            state: ctx.state,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        target_index,
        generation,
        event.snapshot.command_index,
        end_at,
        reapply,
    )?;
    super::try_discord_pair(
        ctx,
        target_index,
        event.snapshot.owner_index,
        event.snapshot.command_index,
    )
}

pub(in crate::simulation) fn settle(
    ctx: &mut EventContext<'_>,
    event: NovaEvent,
) -> Result<(), String> {
    if !ctx.state.reactions.fired_novas.insert(event.generation)
        || ctx.state.targets[event.target_index].hp_current == 0
    {
        return Ok(());
    }
    let snapshot = event.snapshot;
    let base = ntesim_data::reaction_values::base_at_level(
        ntesim_data::reaction_values::ReactionDamage::Nova,
        snapshot.level,
    )?;
    let resistance = resistance_factor(
        ExactFraction::from_ratio(ctx.request.targets[event.target_index].resistance)?,
        crate::simulation::state::target_resistance_reduction(
            &ctx.state.buffs,
            event.target_index,
            "MENTAL",
        )?,
        NegativeResistanceModel::StandardRational,
    )?;
    let damage = ExactFraction::new(base.into(), 1)?
        .mul(intensity_factor(&snapshot)?)?
        .mul(resistance)?;
    let metadata = reaction_metadata(
        "ntesim.reaction.nova",
        DamageKind::Mental,
        None,
        "EExecutionDamageTypeEX::DAMAGE_TYPE_MENTAL",
    )?;
    settle_damage(
        ctx,
        event.target_index,
        &snapshot,
        damage,
        metadata,
        GameEventPayload {
            target_index: Some(event.target_index),
            nova_generation: Some(event.generation),
            ..Default::default()
        },
        false,
        None,
        crate::formula::Ratio::zero(),
    )
}

pub(in crate::simulation) fn end(
    ctx: &mut EventContext<'_>,
    event: NovaEvent,
) -> Result<(), String> {
    if !ctx.state.reactions.ended_novas.insert(event.generation) {
        return Ok(());
    }
    settle(ctx, event.clone())?;
    if ctx
        .state
        .reactions
        .nova
        .get(&event.target_index)
        .is_some_and(|current| current.generation == event.generation)
    {
        ctx.state.reactions.nova.remove(&event.target_index);
        ctx.state.buffs.remove_target_buff(
            NOVA_STATUS_ID,
            REACTION_SOURCE,
            &event.target_index.to_string(),
        );
    }
    let effects = crate::registry::on_nova_removed(
        &mut crate::runtime_api::CallbackContext {
            request: ctx.request,
            state: ctx.state,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        event.target_index,
        event.generation,
        event.snapshot.command_index,
    )?;
    let reapply = effects.iter().any(|effect| effect.reapply);
    for effect in effects {
        settle_extension(ctx, &event, effect)?;
    }
    if ctx.state.targets[event.target_index].hp_current == 0 {
        return Ok(());
    }
    trace(
        ctx,
        "reaction_nova_ended",
        Some(event.snapshot.owner_index),
        Some(GameEventPayload {
            target_index: Some(event.target_index),
            nova_generation: Some(event.generation),
            ..Default::default()
        }),
    );
    // A simultaneous authored switch has already installed its new generation.
    // It wins over this automatic child, whose lineage cannot make a grandchild.
    let owner_index = event.snapshot.owner_index;
    if reapply && !ctx.state.reactions.nova.contains_key(&event.target_index) {
        apply_with_lineage(
            ctx,
            ctx.state.current_time_us,
            event.target_index,
            event.snapshot,
            false,
        )?;
        ctx.state.reactions.record_reaction(ReactionKind::Nova);
        trace(
            ctx,
            "reaction_nova_reapplied",
            Some(owner_index),
            Some(GameEventPayload {
                target_index: Some(event.target_index),
                nova_generation: Some(event.generation),
                ..Default::default()
            }),
        );
    }
    Ok(())
}

fn settle_extension(
    ctx: &mut EventContext<'_>,
    event: &NovaEvent,
    effect: crate::runtime_api::NovaEndEffect,
) -> Result<(), String> {
    if effect.pending_damage.is_zero() {
        return Ok(());
    }
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: effect.hypothesis.as_str().into(),
        description: effect.hypothesis.description().into(),
    };
    let mut metadata = reaction_metadata(
        effect.damage_effect_id,
        DamageKind::Mental,
        None,
        "EExecutionDamageTypeEX::DAMAGE_TYPE_MENTAL",
    )?;
    metadata.action.ability_id = effect.ability_id.into();
    metadata.action.provenance = provenance.clone();
    metadata.action.input_role_provenance = Some(provenance.clone());
    metadata.packet.provenance = provenance.clone();
    metadata.binding.initiating_ability_id = effect.ability_id.into();
    metadata.binding.provenance = Some(provenance);
    let mut snapshot = event.snapshot.clone();
    snapshot.owner_index = effect.owner_actor_index;
    // This pending value is already scaled and capped by the content owner.
    // Fixed settlement bypasses another crit/DEF/RES/bonus calculation.
    settle_damage(
        ctx,
        event.target_index,
        &snapshot,
        effect.pending_damage,
        metadata,
        GameEventPayload {
            target_index: Some(event.target_index),
            nova_generation: Some(event.generation),
            fixed: Some(true),
            ..Default::default()
        },
        false,
        None,
        crate::formula::Ratio::zero(),
    )
}
