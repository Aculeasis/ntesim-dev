//! Successful baseline mesh callback and finite on-field real-time form.
use super::events::Event;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{
    GameEventReceipt, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffRecipient, BuffSourceLifetime, BuffStackingScope, DecayPolicy,
    RefreshPolicy,
};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::BlackbirdPrimaryFormV1;
const FORM_US: u64 = 21_500_000;
const BUFF_MARKER_US: u64 = 3_166_927;
const DEF_BUFF: &str = "blackbird.form_def";
#[derive(Clone, Debug)]
pub(crate) struct Form {
    remaining_us: u64,
    field_started_us: Option<u64>,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn active(actor: &ActorRuntime) -> bool {
    matches!(&actor.character,crate::characters::RuntimeState::Blackbird(state) if state.form.is_some())
}
pub(crate) fn validate_inputs(request: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(request) {
        return Ok(());
    }
    if !super::clock::accepted(request)
        || !super::malice::accepted(request)
        || !super::bird::accepted(request)
        || !super::secondary::accepted(request)
        || !super::support::accepted(request)
    {
        return Err("Blackbird primary form requires ordinary clock, Malice, bird, secondary and Support models".into());
    }
    for action in &request.actions {
        if let crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            energy_cost,
            ..
        } = action
            && request.actors[*actor_index].character_id == "character:1042"
        {
            if ability_id == "GA_BlackBird_UltraSkill" && *energy_cost != 100 {
                return Err(
                    "Blackbird transformation Burst requires its native100 Ultimate energy cost"
                        .into(),
                );
            }
            if ability_id == "GA_BlackBird_Evolve_UltraSkill" && *energy_cost != 0 {
                return Err("Blackbird finisher Burst requires 0 Ultimate energy cost".into());
            }
        }
    }
    Ok(())
}
fn enqueue(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    time: u64,
    event: Event,
    priority: ntesim_core::execution::CausalPriority,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: time,
        priority,
        seq: *sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Blackbird(event)),
    });
    Ok(())
}
fn deadline(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.blackbird_mut()?;
    let Some(form) = &state.form else {
        return Ok(());
    };
    let Some(start) = form.field_started_us else {
        return Ok(());
    };
    let time = start
        .checked_add(form.remaining_us)
        .ok_or("Blackbird form deadline overflow")?;
    if time <= ctx.request.absolute_limit_us {
        enqueue(
            ctx.queue,
            ctx.sequence,
            time,
            Event::FormDeadline {
                actor_index: owner,
                generation: state.form_generation,
            },
            ntesim_core::execution::CausalPriority::ModePause,
        )?;
    }
    Ok(())
}
pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.ability_id != "GA_BlackBird_UltraSkill" || !accepted(ctx.request) {
        return Ok(());
    }
    ctx.record_hypothesis(MODEL)?;
    let generation = {
        let state = ctx.actor.character.blackbird_mut()?;
        state.form_generation = state
            .form_generation
            .checked_add(1)
            .ok_or("Blackbird form generation overflow")?;
        state.form = Some(Form {
            remaining_us: FORM_US,
            field_started_us: Some(ctx.time_us),
        });
        if ctx.config.awakening.count() >= 3 {
            state
                .malice
                .as_mut()
                .ok_or("Blackbird transformation has no Malice pool")?
                .grant(crate::simulation::ExactFraction::new(20, 1)?)?;
        }
        state.form_generation
    };
    ctx.actor.ability_cooldowns.insert(
        ctx.ability_id.into(),
        ctx.time_us
            .checked_add(20_000_000)
            .ok_or("Blackbird Burst cooldown overflow")?,
    );
    if ctx.config.awakening.has_effect(2) {
        ctx.buffs
            .apply_buff(
                BuffDefinition {
                    id: DEF_BUFF.into(),
                    source: ctx.config.character_id.clone(),
                    recipient: BuffRecipient::SelfActor,
                    source_lifetime: BuffSourceLifetime::WhileAlive,
                    stacking_scope: BuffStackingScope::PerSource,
                    duration_us: None,
                    max_stacks: 1,
                    refresh_policy: RefreshPolicy::Reset,
                    decay_policy: DecayPolicy::AllAtOnce,
                    semantic_tags: Default::default(),
                    stat_modifiers: BTreeMap::from([(
                        "def_percent".into(),
                        super::parameters::PARAMETERS.a2_form_def_percent,
                    )]),
                },
                ctx.actor_index,
                ctx.world_time_us,
            )
            .map_err(|error| format!("Blackbird form DEF: {error:?}"))?;
    }
    let end = ctx
        .time_us
        .checked_add(FORM_US)
        .ok_or("Blackbird form deadline overflow")?;
    if end <= ctx.request.absolute_limit_us {
        enqueue(
            ctx.queue,
            ctx.sequence,
            end,
            Event::FormDeadline {
                actor_index: ctx.actor_index,
                generation,
            },
            ntesim_core::execution::CausalPriority::ModePause,
        )?;
    }
    enqueue(
        ctx.queue,
        ctx.sequence,
        ctx.time_us
            .checked_add(BUFF_MARKER_US)
            .ok_or("Blackbird Burst cue overflow")?,
        Event::BurstBuff {
            actor_index: ctx.actor_index,
            command_index: ctx.command_index,
            progress_us: BUFF_MARKER_US,
        },
        ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
    )
}
fn trace(
    ctx: &mut CallbackContext<'_>,
    kind: &str,
    to_actor: Option<usize>,
    data: Option<crate::simulation::GameEventPayload>,
) {
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ModePause".into(),
        event_kind: kind.into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor,
        energy_change: None,
        energy_fanout: None,
        data,
    });
}
pub(crate) fn end(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.blackbird_mut()?;
    if state.form.take().is_none() {
        return Ok(());
    }
    state.form_generation = state
        .form_generation
        .checked_add(1)
        .ok_or("Blackbird form generation overflow")?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.evolve_melee_stage = 0;
    state.evolve_melee_started_us = None;
    state.skill_permission_until_us = None;
    state.finisher_extra_commands.clear();
    state.riposte_ready = false;
    ctx.state
        .buffs
        .remove_owned_buff(DEF_BUFF, &ctx.request.actors[owner].character_id, owner);
    trace(ctx, "blackbird_form_end", Some(owner), None);
    Ok(())
}
pub(crate) fn end_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let owner = ctx.actor_index;
    let state = ctx.actor.character.blackbird_mut()?;
    if state.form.take().is_none() {
        return Ok(());
    }
    state.form_generation = state
        .form_generation
        .checked_add(1)
        .ok_or("Blackbird form generation overflow")?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.evolve_melee_stage = 0;
    state.evolve_melee_started_us = None;
    state.skill_permission_until_us = None;
    state.finisher_extra_commands.clear();
    state.riposte_ready = false;
    ctx.buffs
        .remove_owned_buff(DEF_BUFF, &ctx.config.character_id, owner);
    Ok(())
}
pub(crate) fn expire(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.blackbird_mut()?;
    if state.form_generation == generation
        && state
            .form
            .as_ref()
            .is_some_and(|form| form.field_started_us.is_some())
    {
        end(ctx, owner)?;
    }
    Ok(())
}
pub(crate) fn exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return end(ctx, owner);
    }
    let state = ctx.state.actors[owner].character.blackbird_mut()?;
    let Some(form) = &mut state.form else {
        return Ok(());
    };
    if let Some(start) = form.field_started_us.take() {
        form.remaining_us = form
            .remaining_us
            .saturating_sub(ctx.state.current_time_us - start);
        state.form_generation = state
            .form_generation
            .checked_add(1)
            .ok_or("Blackbird form generation overflow")?;
        let remaining = form.remaining_us;
        trace(
            ctx,
            "blackbird_form_pause",
            Some(owner),
            Some(crate::simulation::GameEventPayload {
                duration_us: Some(remaining),
                ..Default::default()
            }),
        );
    }
    Ok(())
}
pub(crate) fn enter(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.blackbird_mut()?;
    let Some(form) = &mut state.form else {
        return Ok(());
    };
    if form.field_started_us.is_none() {
        form.field_started_us = Some(ctx.state.current_time_us);
        state.form_generation = state
            .form_generation
            .checked_add(1)
            .ok_or("Blackbird form generation overflow")?;
        trace(ctx, "blackbird_form_resume", Some(owner), None);
        deadline(ctx, owner)?;
    }
    Ok(())
}

impl Form {
    pub(crate) fn pause_world(&mut self, delta: u64) -> Result<(), String> {
        if let Some(start) = &mut self.field_started_us {
            crate::simulation::timer_state::anchor(start, delta)?;
        }
        Ok(())
    }
}
