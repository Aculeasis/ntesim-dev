use super::owned_packet::Source;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::ActionPacketMetadata;
use ntesim_core::execution::CausalPriority;

pub(crate) const HYPOTHESIS: crate::HypothesisId = crate::HypothesisId::HanielHarmonyEnsembleV1;
pub(crate) const ENSEMBLE: &str = "GE_Player_Haniel_Skill2_Damage";
pub(crate) const SYMPHONY: &str = "GE_Player_Haniel_Ultraskill2_Damage";
pub(crate) const EXTRA: &str = "GE_Player_Haniel_UltraSkill_Extra_Damage";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Token {
    pub(crate) until: u64,
    pub(crate) source: Box<Source>,
    pub(crate) doubled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Spawn(Box<super::spawn::Spawn>),
    Expire {
        owner: usize,
        until: u64,
    },
    Ensemble {
        owner: usize,
        trigger: usize,
        source: Box<Source>,
    },
    Extra {
        owner: usize,
        target: usize,
        token: Box<Token>,
    },
}

fn enqueue(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.state.current_time_us,
        priority: CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Haniel(event)),
    });
    Ok(())
}

fn is_direct(metadata: &ActionPacketMetadata) -> bool {
    !matches!(
        metadata.packet.damage_effect_id.as_str(),
        ENSEMBLE | SYMPHONY | EXTRA
    ) && metadata
        .packet
        .native_damage_source_category
        .as_deref()
        .is_some_and(|category| {
            ["_A", "_E", "_Q", "_H", "_X", "_Z"]
                .iter()
                .any(|suffix| category.ends_with(suffix))
        })
        && !metadata
            .packet
            .semantic_tags
            .contains(crate::characters::packet_semantics::DOT_DAMAGE_TAG)
        && metadata
            .action
            .input_role
            .is_some_and(|role| role != crate::simulation::metadata::InputActionRole::Other)
        && metadata.packet.native_ga_name.as_deref() == Some(metadata.action.ability_id.as_str())
}

/// Called once per positive recipient. A source-wide gate deduplicates AoE/pellets.
pub(crate) fn observe(
    ctx: &mut CallbackContext<'_>,
    actor: usize,
    target: usize,
    metadata: &ActionPacketMetadata,
) -> Result<(), String> {
    if actor != ctx.state.active_actor_index
        || ctx.state.actors[actor].hp == 0
        || !is_direct(metadata)
    {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    for owner in 0..ctx.state.actors.len() {
        if ctx.request.actors[owner].character_id != "character:1020"
            || ctx.state.actors[owner].hp == 0
        {
            continue;
        }
        let state = ctx.state.actors[owner].character.haniel_mut()?;
        // Consume the previous token before Harmony can arm another one.
        let token = if ctx.state.targets[target].hp_current > 0 {
            state.token.take().filter(|token| token.until > now)
        } else {
            None
        };
        if let Some(token) = token {
            enqueue(
                ctx,
                Event::Extra {
                    owner,
                    target,
                    token,
                },
            )?;
        }
        let state = ctx.state.actors[owner].character.haniel_mut()?;
        if !state.hootie_until_us.is_some_and(|until| until > now) {
            state.harmony = 0;
            state.harmony_ready_us = 0;
            continue;
        }
        if now < state.harmony_ready_us {
            continue;
        }
        state.harmony_ready_us = now
            .checked_add(500_000)
            .ok_or("Haniel Harmony gate overflow")?;
        state.harmony += 1;
        if state.harmony == 4 {
            state.harmony = 0;
            let source = state
                .hootie_source
                .clone()
                .ok_or("Haniel Ensemble parent missing")?;
            enqueue(
                ctx,
                Event::Ensemble {
                    owner,
                    trigger: actor,
                    source,
                },
            )?;
        }
    }
    Ok(())
}

pub(crate) fn apply(event: Event, ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    match event {
        Event::Spawn(spawn) => super::spawn::apply(*spawn, ctx)?,
        Event::Expire { owner, until } => {
            let state = ctx.state.actors[owner].character.haniel_mut()?;
            if state.hootie_until_us == Some(until) {
                state.hootie_until_us = None;
                state.hootie_source = None;
                state.hootie_atk_add = None;
                state.harmony = 0;
                state.harmony_ready_us = 0;
                ctx.state.buffs.remove_owned_buff(
                    "haniel.hootie_team_atk",
                    "character:1020",
                    owner,
                );
                ctx.queue.retain(|event| !matches!(&event.data, GameEventData::Content(crate::callbacks::ContentEvent::Haniel(
                    Event::Ensemble { owner: source, .. })) if *source == owner));
            }
        }
        Event::Extra {
            owner,
            target,
            token,
        } => {
            if ctx.state.actors[owner].hp > 0 && ctx.state.targets[target].hp_current > 0 {
                token
                    .source
                    .emit(ctx, owner, EXTRA, Some(target), token.doubled)?;
            }
        }
        Event::Ensemble {
            owner,
            trigger,
            source,
        } => {
            if ctx.state.actors[owner].hp == 0 {
                return Ok(());
            }
            let now = ctx.state.current_time_us;
            let config = &ctx.request.actors[owner];
            let bond = !super::secondary::accepted(ctx.request)
                || ntesim_data::skill_levels::progression(
                    &config.character_id,
                    config.character_level,
                    config.character_max_level,
                )?["breakthrough"]
                    .as_u64()
                    .unwrap_or(0)
                    >= 4;
            let state = ctx.state.actors[owner].character.haniel_mut()?;
            let ace = state
                .paranormal_ace_until_us
                .is_some_and(|until| until > now);
            let cap = super::resource::capacity(&config.awakening);
            state.plot_cap = cap;
            if bond {
                state.plot = (state.plot + if ace { 2 } else { 1 }).min(cap);
            }
            let burst = if ace { state.ace_source.clone() } else { None };
            if let Some(source) = burst.clone() {
                state.token = Some(Box::new(Token {
                    until: now
                        .checked_add(3_000_000)
                        .ok_or("Haniel token deadline overflow")?,
                    source,
                    doubled: config.awakening.has_effect(4) && owner == trigger,
                }));
            }
            source.emit(ctx, owner, ENSEMBLE, None, false)?;
            if let Some(source) = burst {
                source.emit(ctx, owner, SYMPHONY, None, false)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn capture_burst(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let source = Source::capture(ctx)?;
    ctx.actor.character.haniel_mut()?.ace_source = Some(Box::new(source));
    ctx.record_hypothesis(HYPOTHESIS)
}

pub(crate) fn schedule_expiry(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let Some(until) = ctx.actor.character.haniel_mut()?.hootie_until_us else {
        return Ok(());
    };
    if until > ctx.request.absolute_limit_us {
        return Ok(());
    }
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: until,
        priority: CausalPriority::SettlementAndDeath,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Haniel(Event::Expire {
            owner: ctx.actor_index,
            until,
        })),
    });
    Ok(())
}

pub(crate) fn on_defeat(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.request.actors[owner].character_id != "character:1020" {
        return Ok(());
    }
    super::clock::stop(
        &mut ctx.state.actors[owner],
        ctx.queue,
        ctx.state.current_time_us,
    )?;
    let state = ctx.state.actors[owner].character.haniel_mut()?;
    let plot_cap = state.plot_cap;
    let membership = state.membership.take();
    let clock = state.clock.take();
    let out_of_combat = state.out_of_combat;
    *state = Default::default();
    state.plot_cap = plot_cap;
    state.membership = membership;
    state.clock = clock;
    state.out_of_combat = out_of_combat;
    for id in [
        "haniel.hootie_team_atk",
        "haniel.paranormal_ace_team_atk",
        "haniel.paranormal_ace_self",
        "haniel.paranormal_ace_team",
    ] {
        ctx.state
            .buffs
            .remove_owned_buff(id, "character:1020", owner);
    }
    Ok(())
}
