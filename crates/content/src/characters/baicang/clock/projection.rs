use super::*;
use crate::callbacks::ContentEvent;

#[derive(Clone, Debug)]
struct Anchor {
    seq: u64,
    offset: u64,
    travel: u64,
    emitted_at: u64,
    committed: bool,
}

#[derive(Clone, Debug)]
pub(super) struct Action {
    command: usize,
    start: u64,
    recovery: u64,
    skill: Option<u64>,
    jump: Option<u64>,
    is_skill: bool,
    first_emission: Option<u64>,
    field_generation: Option<u64>,
    anchors: Vec<Anchor>,
}

impl Action {
    pub(super) fn capture(ctx: &CastContext<'_>) -> Result<Self, String> {
        let crate::characters::RuntimeState::Baicang(state) = &ctx.actor.character else {
            return Err("Baicang clock state missing".into());
        };
        let window = state
            .cancel_window
            .ok_or("Baicang clock requires an admitted producer")?;
        let mut anchors = Vec::new();
        for event in ctx.queue.iter() {
            let emitted = match &event.data {
                GameEventData::HitDelivery {
                    actor_index,
                    command_index,
                    ability_id,
                    damage_effect_id,
                    ..
                } if *actor_index == ctx.actor_index
                    && *command_index == ctx.command_index
                    && ability_id == window.ability_id
                    && !damage_effect_id.starts_with("GE_Player_Cang_World")
                    && damage_effect_id != "GE_Player_Cang_UltraSkill_Damage" =>
                {
                    if window.emission == super::super::cancel::EmissionKind::Branch1
                        && damage_effect_id == super::super::branches::BRANCH1_1_PACKET
                    {
                        Some(
                            ctx.time_us
                                .checked_add(window.hold_us)
                                .and_then(|t| t.checked_add(416_667))
                                .ok_or("Baicang branch emitter overflow")?,
                        )
                    } else {
                        Some(window.emitted_at(damage_effect_id, event.time_us)?)
                    }
                }
                GameEventData::Content(
                    ContentEvent::BaicangFieldWords {
                        actor_index,
                        generation,
                        ..
                    }
                    | ContentEvent::BaicangDomainTick {
                        actor_index,
                        generation,
                        ..
                    }
                    | ContentEvent::BaicangDomainEnd {
                        actor_index,
                        generation,
                    },
                ) if *actor_index == ctx.actor_index
                    && *generation == state.domain_generation
                    && window.emission == super::super::cancel::EmissionKind::Burst =>
                {
                    Some(state.domain_start_us)
                }
                _ => None,
            };
            if let Some(emitted_at) = emitted {
                anchors.push(Anchor {
                    seq: event.seq,
                    offset: emitted_at
                        .checked_sub(ctx.time_us)
                        .ok_or("Baicang emitter before input")?,
                    travel: event
                        .time_us
                        .checked_sub(emitted_at)
                        .ok_or("Baicang impact before emission")?,
                    emitted_at,
                    committed: false,
                });
            }
        }
        let is_skill = window.ability_id == "GA_Cang_Skill";
        let first_emission = if is_skill {
            anchors.iter().map(|a| a.emitted_at).min()
        } else {
            None
        };
        Ok(Self {
            command: ctx.command_index,
            start: ctx.time_us,
            recovery: ctx.actor.locked_until_us - ctx.time_us,
            skill: window.skill_at_us.map(|t| t - ctx.time_us),
            jump: window.jump_at_us.map(|t| t - ctx.time_us),
            is_skill,
            first_emission,
            field_generation: (window.emission == super::super::cancel::EmissionKind::Burst)
                .then_some(state.domain_generation),
            anchors,
        })
    }

    pub(super) fn emitted_at(&self, seq: u64) -> Option<u64> {
        self.anchors
            .iter()
            .find(|a| a.seq == seq)
            .map(|a| a.emitted_at)
    }

    pub(super) fn first_skill_emission(&self) -> Option<u64> {
        self.first_emission
    }
}

pub(super) fn project(
    actor: &mut crate::simulation::state::ActorRuntime,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    clock: &mut State,
    actor_index: usize,
    now: u64,
) -> Result<(), String> {
    let Some(action) = &mut clock.current else {
        return Ok(());
    };
    let old_recovery = actor.locked_until_us;
    let state = actor.character.baicang_mut()?;
    if state
        .cancel_window
        .is_none_or(|w| w.command_index != action.command)
        || old_recovery < now
    {
        clock.current = None;
        return Ok(());
    }
    let resolve = |offset| {
        clock
            .tracker
            .resolve_real_deadline(actor_index, action.start, offset)
            .map_err(|e| format!("Baicang caster deadline: {e:?}"))
    };
    let recovery = resolve(action.recovery)?;
    if action.is_skill && action.first_emission.is_some_and(|time| time > now) {
        let offset = action
            .anchors
            .iter()
            .map(|a| a.offset)
            .min()
            .ok_or("Baicang Skill emitter missing")?;
        action.first_emission = Some(resolve(offset)?);
    }
    let mut events = queue.drain().collect::<Vec<_>>();
    let indices: std::collections::BTreeMap<_, _> = events
        .iter()
        .enumerate()
        .map(|(index, event)| (event.seq, index))
        .collect();
    action
        .anchors
        .retain(|anchor| indices.contains_key(&anchor.seq));
    for anchor in &mut action.anchors {
        if anchor.emitted_at <= now {
            anchor.committed = true;
        }
        if anchor.committed {
            continue;
        }
        anchor.emitted_at = resolve(anchor.offset)?;
        if let Some(&index) = indices.get(&anchor.seq) {
            let event = &mut events[index];
            event.time_us = anchor
                .emitted_at
                .checked_add(anchor.travel)
                .ok_or("Baicang arrival overflow")?;
        }
    }
    if let Some(window) = &mut state.cancel_window {
        window.skill_at_us = action.skill.map(resolve).transpose()?;
        window.jump_at_us = action.jump.map(resolve).transpose()?;
    }
    if state.normal_started_us == Some(old_recovery) {
        state.normal_started_us = Some(recovery);
    }
    if state.derived_skill_stage_expires_at_us == old_recovery.saturating_add(2_000_000) {
        state.derived_skill_stage_expires_at_us = recovery
            .checked_add(2_000_000)
            .ok_or("Baicang derived deadline overflow")?;
    }
    if action.field_generation == Some(state.domain_generation) && state.domain_start_us > now {
        let field_start = resolve(2_664_744)?;
        let duration = state.domain_until_us - state.domain_start_us;
        state.domain_start_us = field_start;
        state.domain_until_us = field_start
            .checked_add(duration)
            .ok_or("Baicang field deadline overflow")?;
        for event in &mut events {
            if let GameEventData::Content(ContentEvent::BaicangDomainTick {
                actor_index: source,
                generation,
                end_time_us,
                ..
            }) = &mut event.data
                && *source == actor_index
                && *generation == state.domain_generation
            {
                *end_time_us = state.domain_until_us;
            }
        }
    }
    actor.locked_until_us = recovery;
    queue.replace(events);
    Ok(())
}
