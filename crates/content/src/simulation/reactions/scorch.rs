use super::*;

pub(in crate::simulation) const SCORCH_TICK_INTERVAL_US: u64 = 990_000;
pub(in crate::simulation) const SCORCH_DURATION_US: u64 = 15_000_000;
pub(crate) const SCORCH_STATUS_ID: &str = "ntesim.reaction.scorch";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScorchTickEvent {
    pub target_index: usize,
    pub generation: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct ScorchGeneration {
    /// Tick/expiry identity. Qualified refreshes keep this so the tick phase is
    /// stable, even though each new application can create a fresh Discord.
    pub generation: u64,
    /// Identity of the Scorch application used for pair-keyed Discord dedupe.
    pub discord_generation: u64,
    pub snapshot: ReactionSnapshot,
    pub phase_origin_us: u64,
    pub scheduled_until_us: u64,
    pub expires_at_us: u64,
    pub stacks: u32,
    pub enhancement: Option<crate::runtime_api::ScorchEnhancement>,
}

pub(in crate::simulation) fn apply_scorch(
    ctx: &mut EventContext<'_>,
    now_us: u64,
    target_index: usize,
    snapshot: ReactionSnapshot,
) -> Result<(), String> {
    apply_scorch_with_application(ctx, now_us, target_index, snapshot, true)
}

pub(in crate::simulation) fn apply_scorch_with_application(
    ctx: &mut EventContext<'_>,
    now_us: u64,
    target_index: usize,
    snapshot: ReactionSnapshot,
    new_discord_application: bool,
) -> Result<(), String> {
    // An in-window reapplication replaces the offensive snapshot, refreshes
    // expiry and keeps the existing tick phase without an immediate tick; an
    // expired generation is absent and anchors its own fresh phase.
    let refresh = ctx
        .state
        .reactions
        .scorch
        .get(&target_index)
        .filter(|existing| existing.expires_at_us > now_us)
        .cloned();
    let existing_discord_generation = refresh.as_ref().map(|existing| existing.discord_generation);
    let enhancement = match &refresh {
        Some(existing) => {
            let mut enhancement = existing.enhancement.clone();
            if let Some(enh) = &mut enhancement {
                let mut assumptions = BTreeMap::new();
                if let Ok(Some(current_policy)) =
                    crate::registry::scorch_enhancement(ctx.request, ctx.state, &mut assumptions)
                {
                    enh.set_max_stacks = current_policy.set_max_stacks;
                }
            }
            enhancement
        }
        None => {
            let mut assumptions = BTreeMap::new();
            let policy =
                crate::registry::scorch_enhancement(ctx.request, ctx.state, &mut assumptions)?;
            ctx.state.execution_model_assumptions.extend(assumptions);
            policy
        }
    };
    let stack_cap = enhancement.as_ref().map_or(1, |policy| policy.stack_cap);
    let set_max = enhancement
        .as_ref()
        .is_some_and(|policy| policy.set_max_stacks);
    let stacks = if set_max {
        stack_cap
    } else {
        refresh.as_ref().map_or(1, |existing| {
            existing.stacks.saturating_add(1).min(stack_cap)
        })
    };
    let (generation, phase_origin_us, mut scheduled_until_us, expires_at_us) = match refresh {
        Some(existing) => {
            let expires_at_us = now_us
                .checked_add(SCORCH_DURATION_US)
                .ok_or("Scorch expiry overflow")?;
            (
                existing.generation,
                existing.phase_origin_us,
                existing.scheduled_until_us,
                expires_at_us,
            )
        }
        None => {
            let generation = ctx.state.reactions.next_id;
            ctx.state.reactions.next_id += 1;
            (
                generation,
                now_us,
                now_us,
                now_us
                    .checked_add(SCORCH_DURATION_US)
                    .ok_or("Scorch expiry overflow")?,
            )
        }
    };
    let discord_generation = match existing_discord_generation {
        Some(generation) if !new_discord_application => generation,
        _ => {
            let generation = ctx.state.reactions.next_id;
            ctx.state.reactions.next_id += 1;
            generation
        }
    };
    while let Some(tick_us) = scheduled_until_us.checked_add(SCORCH_TICK_INTERVAL_US) {
        if tick_us > expires_at_us {
            break;
        }
        schedule(
            ctx,
            tick_us,
            GameEventData::ReactionScorchTick(Box::new(ScorchTickEvent {
                target_index,
                generation,
            })),
        )?;
        scheduled_until_us = tick_us;
    }
    ctx.state.reactions.scorch.insert(
        target_index,
        ScorchGeneration {
            generation,
            discord_generation,
            snapshot: snapshot.clone(),
            phase_origin_us,
            scheduled_until_us,
            expires_at_us,
            stacks,
            enhancement,
        },
    );
    let target_stacks_needed = if set_max { stack_cap } else { 1 };
    for _ in 0..target_stacks_needed {
        apply_status_with_cap(
            ctx,
            target_index,
            snapshot.owner_index,
            SCORCH_STATUS_ID,
            BTreeSet::from([
                crate::characters::packet_semantics::SCORCH_STATUS_TAG.to_owned(),
                "Ability.Player.Buff.Reaction.ZhouAn".to_owned(),
            ]),
            SCORCH_DURATION_US,
            stack_cap,
        )?;
    }
    trace(
        ctx,
        "reaction_scorch_applied",
        Some(snapshot.owner_index),
        Some(GameEventPayload {
            target_index: Some(target_index),
            scorch_generation: Some(generation),
            discord_generation: Some(discord_generation),
            element: Some(snapshot.element_key.clone()),
            phase_origin_us: Some(phase_origin_us),
            exact_amount: Some(snapshot.cycle_intensity),
            ..Default::default()
        }),
    );
    use crate::characters::adler::temperance;
    if let Some(adler) = temperance::owner(ctx.request, &ctx.state.actors)? {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            temperance::MODEL,
        )?;
        let outcome = ctx
            .state
            .reactions
            .reaction_rng()?
            .uniform(temperance::RNG_STREAM, 3)?;
        let id = temperance::apply(
            &mut ctx.state.buffs,
            adler,
            target_index,
            ctx.state.mode_clock.relative_us(),
            outcome.value,
        )?;
        if outcome.value == 0 {
            // Report live target ATK even when its expiry falls in an idle gap.
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: expires_at_us,
                priority: CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: GameEventData::Content(crate::callbacks::ContentEvent::BuffLifecycleExpiry),
            });
        }
        trace(
            ctx,
            "adler_temperance_applied",
            Some(adler),
            Some(GameEventPayload {
                target_index: Some(target_index),
                buff_id: Some(id.into()),
                duration_us: Some(temperance::DURATION_US),
                ..Default::default()
            }),
        );
    }
    super::try_discord_pair(
        ctx,
        target_index,
        snapshot.owner_index,
        snapshot.command_index,
    )
}

pub(in crate::simulation) fn refresh_scorch_from_word(
    ctx: &mut EventContext<'_>,
    source_actor_index: usize,
    target_index: usize,
    word_generation: u64,
) -> Result<(), String> {
    let now_us = ctx.state.current_time_us;
    let Some(current) = ctx.state.reactions.scorch.get(&target_index) else {
        return Ok(());
    };
    if current.expires_at_us <= now_us || ctx.state.targets[target_index].hp_current == 0 {
        return Ok(());
    }
    let snapshot = current.snapshot.clone();
    apply_scorch_with_application(ctx, now_us, target_index, snapshot, false)?;
    ctx.state
        .reactions
        .record_reaction(super::ReactionKind::Scorch);
    trace(
        ctx,
        "baicang_scorch_refresh",
        Some(source_actor_index),
        Some(GameEventPayload {
            target_index: Some(target_index),
            word_generation: Some(word_generation),
            ..Default::default()
        }),
    );
    Ok(())
}

pub(crate) fn spread_scorch_to_target(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target_index: usize,
    snapshot: ReactionSnapshot,
) -> Result<(), String> {
    let now_us = ctx.state.current_time_us;
    let mut hits = Vec::new();
    let mut ledger = Vec::new();
    let mut rng = None;
    let mut event_ctx = crate::simulation::context::EventContext {
        request: ctx.request,
        prepared_packets: &[],
        state: ctx.state,
        queue: ctx.queue,
        sequence: ctx.sequence,
        trace: ctx.trace,
        rng: &mut rng,
        hits: &mut hits,
        action_metadata_ledger: &mut ledger,
    };
    apply_scorch(&mut event_ctx, now_us, target_index, snapshot)
}

pub(in crate::simulation) fn settle_scorch_tick(
    ctx: &mut EventContext<'_>,
    event: ScorchTickEvent,
) -> Result<(), String> {
    let Some(current) = ctx.state.reactions.scorch.get(&event.target_index) else {
        return Ok(());
    };
    if current.generation != event.generation
        || ctx.state.current_time_us >= current.expires_at_us
        || ctx.state.targets[event.target_index].hp_current == 0
    {
        return Ok(());
    }
    let snapshot = current.snapshot.clone();
    let stacks = current.stacks;
    let base = ntesim_data::reaction_values::base_at_level(
        ntesim_data::reaction_values::ReactionDamage::Scorch,
        snapshot.level,
    )?;
    let base = ExactFraction::new(base.into(), 1)?;
    let defense = defense_factor(
        snapshot.level,
        crate::runtime_api::target_defense(
            &ctx.state.buffs,
            &ctx.state.targets[event.target_index],
            event.target_index,
            ctx.state.mode_clock.relative_us(),
        )?,
        ExactFraction::new(0, 1)?,
        ExactFraction::new(0, 1)?,
    )?;
    let resistance = element_resistance(ctx, event.target_index, &snapshot.element_key)?;
    let damage = base
        .mul(intensity_factor(&snapshot)?)?
        .mul(ExactFraction::new(stacks.into(), 1)?)?
        .mul(defense)?
        .mul(resistance)?;
    // Fixed-50 crit samples each tick with the captured crit damage,
    // regardless of the owner's ordinary crit rate.
    let sample = ctx.state.reactions.sample_scorch(snapshot.owner_index)?;
    let critical = sample.value < 1;
    let damage_type_ex = format!(
        "EExecutionDamageTypeEX::DAMAGE_TYPE_{}",
        snapshot.element_key
    );
    let element = element_from_key(&snapshot.element_key).map(|element| match element {
        ntesim_data::constants::character::Element::Cosmos => DamageElement::Cosmos,
        ntesim_data::constants::character::Element::Nature => DamageElement::Nature,
        ntesim_data::constants::character::Element::Incantation => DamageElement::Incantation,
        ntesim_data::constants::character::Element::Chaos => DamageElement::Chaos,
        ntesim_data::constants::character::Element::Psyche => DamageElement::Psyche,
        ntesim_data::constants::character::Element::Lakshana => DamageElement::Lakshana,
    });
    let mut metadata = reaction_metadata(
        "ntesim.reaction.scorch",
        DamageKind::Elemental,
        element,
        &damage_type_ex,
    )?;
    metadata.packet.fixed_crit_rate = Some(ExactFraction::new(1, 2)?);
    settle_damage(
        ctx,
        event.target_index,
        &snapshot,
        damage,
        metadata,
        GameEventPayload {
            target_index: Some(event.target_index),
            scorch_generation: Some(event.generation),
            critical: Some(critical),
            ..Default::default()
        },
        critical,
        Some(sample),
        crate::formula::Ratio {
            numerator: 1,
            denominator: 2,
        },
    )
}
