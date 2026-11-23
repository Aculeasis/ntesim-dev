use super::*;

pub(crate) const BLOSSOM_PARTY_BUD_CAP: usize = 3;
pub(super) const BLOSSOM_PISTIL_INTERVAL_US: u64 = 2_000_000;
pub(super) const BLOSSOM_PISTIL_COUNT: u32 = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PistilEvent {
    pub bud_id: u64,
    pub snapshot: ReactionSnapshot,
    pub is_copy: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct BudGeneration {
    pub id: u64,
    pub emissions_left: u32,
}

pub(in crate::simulation) fn create_blossom(
    ctx: &mut EventContext<'_>,
    now_us: u64,
    snapshot: ReactionSnapshot,
) -> Result<(), String> {
    let profile = crate::registry::blossom_emission_profile(ctx.request, ctx.state);
    let (count, interval_us) = if let Some(profile) = profile {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            profile.model,
        )?;
        (profile.count, profile.interval_us)
    } else {
        (BLOSSOM_PISTIL_COUNT, BLOSSOM_PISTIL_INTERVAL_US)
    };
    let bud_policy = crate::registry::blossom_bud_policy(ctx.request, ctx.state);
    let bud_cap = bud_policy.bud_cap;
    let num_buds = bud_policy.num_buds;
    for _ in 0..num_buds {
        // A fourth bud (or seventh under B2) replaces the oldest; its pending emissions are spent.
        if ctx.state.reactions.buds.len() >= bud_cap
            && let Some(oldest) = ctx
                .state
                .reactions
                .buds
                .iter()
                .enumerate()
                .min_by_key(|(index, bud)| (bud.id, *index))
                .map(|(index, _)| index)
        {
            ctx.state.reactions.buds.remove(oldest);
        }
        let bud_id = ctx.state.reactions.next_id;
        ctx.state.reactions.next_id += 1;
        ctx.state.reactions.buds.push(BudGeneration {
            id: bud_id,
            emissions_left: count,
        });
        let lifetime_us = u64::from(count)
            .checked_mul(interval_us)
            .ok_or("Blossom lifetime overflow")?;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push_on(
            ntesim_core::kernel::timing::ClockDomain::ObjectLifetime,
            ScheduledGameEvent {
                time_us: now_us
                    .checked_add(lifetime_us)
                    .ok_or("Blossom expiry overflow")?,
                priority: CausalPriority::SettlementAndDeath,
                seq: *ctx.sequence,
                data: GameEventData::Content(crate::callbacks::ContentEvent::BlossomBudExpiry {
                    bud_id,
                    is_copy: false,
                }),
            },
        );
        for emission in 1..=count {
            let delay_us = u64::from(emission) * interval_us;
            schedule(
                ctx,
                now_us
                    .checked_add(delay_us)
                    .ok_or("Blossom pistil deadline overflow")?,
                GameEventData::ReactionPistil(Box::new(PistilEvent {
                    bud_id,
                    snapshot: snapshot.clone(),
                    is_copy: false,
                })),
            )?;
        }
        if crate::registry::blossom_bud_copy_eligible(ctx.request) {
            schedule(
                ctx,
                now_us
                    .checked_add(3_000_000)
                    .ok_or("Bud copy check overflow")?,
                GameEventData::Content(crate::callbacks::ContentEvent::IroiBudCopyCheck {
                    bud_id,
                    snapshot: Box::new(snapshot.clone()),
                }),
            )?;
        }
        trace(
            ctx,
            "reaction_blossom_created",
            Some(snapshot.owner_index),
            Some(GameEventPayload {
                bud_id: Some(bud_id),
                level: Some(snapshot.level),
                exact_amount: Some(snapshot.cycle_intensity),
                ..Default::default()
            }),
        );
    }
    Ok(())
}

pub(crate) fn expire_bud(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    bud_id: u64,
    is_copy: bool,
) -> Result<(), String> {
    if is_copy {
        for actor in &mut ctx.state.actors {
            if let crate::characters::RuntimeState::Iroi(state) = &mut actor.character {
                state.bud_copy_ids.retain(|id| *id != bud_id);
            }
        }
    } else {
        ctx.state.reactions.buds.retain(|bud| bud.id != bud_id);
    }
    ctx.queue.retain(|event| !matches!(&event.data,
        GameEventData::ReactionPistil(pistil) if pistil.bud_id == bud_id && pistil.is_copy == is_copy));
    Ok(())
}

pub(in crate::simulation) fn settle_pistil(
    ctx: &mut EventContext<'_>,
    event: PistilEvent,
) -> Result<(), String> {
    let position = if !event.is_copy {
        let Some(pos) = ctx
            .state
            .reactions
            .buds
            .iter()
            .position(|bud| bud.id == event.bud_id)
        else {
            return Ok(());
        };
        Some(pos)
    } else {
        None
    };
    let snapshot = event.snapshot;
    let base = ntesim_data::reaction_values::base_at_level(
        ntesim_data::reaction_values::ReactionDamage::Blossom,
        snapshot.level,
    )?;
    let mut base = ExactFraction::new(base.into(), 1)?;
    if event.is_copy {
        base = base.mul(ExactFraction::new(3, 8)?)?;
    }
    let target_count = ctx.state.targets.len();
    let mut charge_eligible = false;
    for target_index in 0..target_count {
        if ctx.state.targets[target_index].hp_current == 0 {
            continue;
        }
        charge_eligible |= remora::is_active(ctx, target_index)?;
        let defense = defense_factor(
            snapshot.level,
            crate::runtime_api::target_defense(
                &ctx.state.buffs,
                &ctx.state.targets[target_index],
                target_index,
                ctx.state.mode_clock.relative_us(),
            )?,
            ExactFraction::new(0, 1)?,
            ExactFraction::new(0, 1)?,
        )?;
        let resistance = element_resistance(ctx, target_index, "NATURE")?;
        let damage = base
            .mul(intensity_factor(&snapshot)?)?
            .mul(defense)?
            .mul(resistance)?;
        let metadata = reaction_metadata(
            "ntesim.reaction.blossom",
            DamageKind::Elemental,
            Some(DamageElement::Nature),
            "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE",
        )?;
        let trace_data = GameEventPayload {
            target_index: Some(target_index),
            bud_id: Some(event.bud_id),
            copied: event.is_copy.then_some(true),
            ..Default::default()
        };
        settle_damage(
            ctx,
            target_index,
            &snapshot,
            damage,
            metadata,
            trace_data,
            false,
            None,
            crate::formula::Ratio::zero(),
        )?;
    }
    if charge_eligible {
        remora::charge(ctx, &snapshot, event.bud_id)?;
    }
    // The final emission removes the bud; earlier emissions keep it alive so
    // replacement accounting and pending-event guards stay exact.
    if let Some(pos) = position {
        let bud = &mut ctx.state.reactions.buds[pos];
        bud.emissions_left = bud
            .emissions_left
            .checked_sub(1)
            .ok_or("Blossom emission underflow")?;
        if bud.emissions_left == 0 {
            ctx.state.reactions.buds.remove(pos);
        }
    }
    Ok(())
}
