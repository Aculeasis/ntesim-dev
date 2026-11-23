//! Finite intrinsic Gaze generations on the real combat axis.
use crate::runtime_api::CallbackContext;
use crate::simulation::{GameDummyCombatEvent, GameEventReceipt, GameScenarioRequest};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorGazeV1;
const INTERVAL: u64 = 5_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    Ordinary,
    Enhanced,
}
impl Mark {
    fn code(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::Enhanced => "enhanced",
        }
    }
}
#[derive(Clone, Debug)]
struct Target {
    proximity: bool,
    generation: u64,
    mark: Option<Mark>,
    deadline: Option<u64>,
}
#[derive(Clone, Debug)]
pub(crate) struct State {
    targets: Vec<Target>,
}
impl State {
    pub(crate) fn pause_world(&mut self, from: u64, delta: u64) -> Result<(), String> {
        for target in &mut self.targets {
            if let Some(deadline) = &mut target.deadline {
                crate::simulation::timer_state::future(deadline, from, delta)?;
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    owner: usize,
    target: usize,
    generation: u64,
    mark: Option<Mark>,
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(req) {
        return if req.dummy_combat_events.is_empty() {
            Ok(())
        } else {
            Err(
                "dummy combat observations require an accepted content observer (hathor.gaze_v1)"
                    .into(),
            )
        };
    }
    if !super::power::accepted(req)
        || !req
            .actors
            .iter()
            .any(|a| a.character_id == "character:1025")
    {
        return Err("hathor.gaze_v1 requires party Hathor and tap_power_v1".into());
    }
    Ok(())
}
fn schedule(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    mark: Option<Mark>,
    phase: &str,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .gaze
        .as_mut()
        .ok_or("Hathor Gaze state missing")?;
    let entry = &mut state.targets[target];
    entry.generation = entry
        .generation
        .checked_add(1)
        .ok_or("Hathor Gaze generation overflow")?;
    let generation = entry.generation;
    let due = now
        .checked_add(INTERVAL)
        .ok_or("Hathor Gaze deadline overflow")?;
    entry.deadline = Some(due);
    ctx.trace.push(GameEventReceipt {
        time_us: now,
        phase: phase.into(),
        event_kind: "hathor_gaze_deadline".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            target_index: Some(target),
            mark: Some(mark.map_or("proximity", Mark::code).into()),
            expires_at_us: Some(due),
            ..Default::default()
        }),
    });
    if due <= ctx.request.absolute_limit_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: due,
                priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::HathorGaze(Event {
                        owner,
                        target,
                        generation,
                        mark,
                    }),
                ),
            });
    }
    Ok(())
}
fn trace(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    phase: &str,
    mark: Mark,
    action: &str,
) {
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: phase.into(),
        event_kind: "hathor_gaze".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            target_index: Some(target),
            mark: Some(mark.code().into()),
            action: Some(action.into()),
            ..Default::default()
        }),
    });
}
pub(crate) fn initialize(ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    for owner in 0..ctx.request.actors.len() {
        if ctx.request.actors[owner].character_id != "character:1025"
            || ctx.state.actors[owner].hp == 0
        {
            continue;
        }
        initialize_owner(ctx, owner, "ModePause")?;
    }
    Ok(())
}
pub(super) fn initialize_owner(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    phase: &str,
) -> Result<(), String> {
    let mut proximity = vec![true; ctx.state.targets.len()];
    for event in &ctx.request.dummy_combat_events {
        if let GameDummyCombatEvent::Proximity {
            timestamp_us,
            target_index,
            in_proximity,
        } = event
            && *timestamp_us <= ctx.state.current_time_us
        {
            proximity[*target_index] = *in_proximity;
        }
    }
    ctx.state.actors[owner].character.hathor_mut()?.gaze = Some(Box::new(State {
        targets: proximity
            .iter()
            .map(|&in_range| Target {
                proximity: in_range,
                generation: 0,
                mark: None,
                deadline: None,
            })
            .collect(),
    }));
    for (target, in_range) in proximity.into_iter().enumerate() {
        if in_range && ctx.state.targets[target].hp_current > 0 {
            schedule(ctx, owner, target, None, phase)?;
        }
    }
    Ok(())
}
fn remove(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    end: super::power::GazeEnd,
) -> Result<(), String> {
    let entry = &mut ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .gaze
        .as_mut()
        .ok_or("Hathor Gaze state missing")?
        .targets[target];
    let Some(mark) = entry.mark.take() else {
        return Ok(());
    };
    entry.deadline = None;
    entry.generation = entry
        .generation
        .checked_add(1)
        .ok_or("Hathor Gaze generation overflow")?;
    let rearm = entry.proximity;
    trace(
        ctx,
        owner,
        target,
        end.phase(),
        mark,
        if end.natural() {
            "naturally_expired"
        } else {
            "consumed"
        },
    );
    if !end.natural() || ctx.request.actors[owner].awakening.has_effect(2) {
        super::power::grant(
            ctx,
            owner,
            super::power::Cause::Gaze {
                end,
                enhanced: mark == Mark::Enhanced,
            },
        )?;
    }
    if rearm {
        schedule(ctx, owner, target, None, end.phase())?;
    }
    Ok(())
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    let Event {
        owner,
        target,
        generation,
        mark,
    } = event;
    if ctx.state.actors[owner].hp == 0 || ctx.state.targets[target].hp_current == 0 {
        return Ok(());
    }
    let Some(state) = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .gaze
        .as_mut()
    else {
        return Ok(());
    };
    let entry = &mut state.targets[target];
    if entry.generation != generation
        || entry.deadline != Some(ctx.state.current_time_us)
        || entry.mark != mark
        || (mark.is_none() && !entry.proximity)
    {
        return Ok(());
    }
    if mark.is_some() {
        return remove(ctx, owner, target, super::power::GazeEnd::Natural);
    }
    entry.mark = Some(Mark::Ordinary);
    trace(
        ctx,
        owner,
        target,
        "BuffTickOrExpiry",
        Mark::Ordinary,
        "applied",
    );
    schedule(ctx, owner, target, Some(Mark::Ordinary), "BuffTickOrExpiry")
}
/// Each actual held timer requests this application even when its damage rounds to0.
pub(crate) fn enhance(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
) -> Result<(), String> {
    let entry = &mut ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .gaze
        .as_mut()
        .ok_or("Hathor enhanced Gaze state missing")?
        .targets[target];
    if entry.mark == Some(Mark::Enhanced) {
        return Ok(());
    }
    entry.mark = Some(Mark::Enhanced);
    trace(
        ctx,
        owner,
        target,
        "ActionAdmission",
        Mark::Enhanced,
        "replaced",
    );
    schedule(ctx, owner, target, Some(Mark::Enhanced), "ActionAdmission")
}
/// Only explicit provenance and actual HP/shield contact can use this route.
pub(crate) fn contact(ctx: &mut CallbackContext<'_>, target: usize) -> Result<(), String> {
    for owner in 0..ctx.state.actors.len() {
        if ctx.request.actors[owner].character_id != "character:1025"
            || ctx.state.actors[owner].hp == 0
            || ctx.state.targets[target].hp_current == 0
        {
            continue;
        }
        let enhanced = ctx.state.actors[owner]
            .character
            .hathor_mut()?
            .gaze
            .as_ref()
            .is_some_and(|s| s.targets[target].mark == Some(Mark::Enhanced));
        if enhanced {
            remove(ctx, owner, target, super::power::GazeEnd::Contact)?;
        }
    }
    Ok(())
}
pub(crate) fn observe(ctx: &mut CallbackContext<'_>, index: usize) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let event = ctx.request.dummy_combat_events[index].clone();
    let target = event.target_index();
    let recipient_alive = match event {
        GameDummyCombatEvent::AttackStart { actor_index, .. } => {
            ctx.state.actors[actor_index].hp > 0
        }
        _ => false,
    };
    if ctx.state.targets[target].hp_current == 0 {
        return Ok(());
    }
    for owner in 0..ctx.state.actors.len() {
        if ctx.request.actors[owner].character_id != "character:1025"
            || ctx.state.actors[owner].hp == 0
        {
            continue;
        }
        let Some(state) = ctx.state.actors[owner]
            .character
            .hathor_mut()?
            .gaze
            .as_mut()
        else {
            continue;
        };
        let entry = &mut state.targets[target];
        match event {
            GameDummyCombatEvent::Proximity { in_proximity, .. } => {
                if entry.proximity == in_proximity {
                    continue;
                }
                entry.proximity = in_proximity;
                if entry.mark.is_none() {
                    entry.deadline = None;
                    entry.generation = entry
                        .generation
                        .checked_add(1)
                        .ok_or("Hathor Gaze generation overflow")?;
                    if in_proximity {
                        schedule(ctx, owner, target, None, "ModePause")?;
                    }
                }
            }
            GameDummyCombatEvent::AttackStart { .. } => {
                if entry.mark.is_none() || !recipient_alive {
                    continue;
                }
                remove(ctx, owner, target, super::power::GazeEnd::Attack)?;
            }
        }
    }
    Ok(())
}
pub(crate) fn defeat(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.request.actors[owner].character_id == "character:1025" {
        ctx.state.actors[owner].character.hathor_mut()?.gaze = None;
        ctx.queue.retain(|e|!matches!(&e.data,
            crate::simulation::events::GameEventData::Content(crate::callbacks::ContentEvent::HathorGaze(x)) if x.owner==owner));
    }
    Ok(())
}

pub(crate) fn target_defeated(ctx: &mut CallbackContext<'_>, target: usize) -> Result<(), String> {
    for actor in &mut ctx.state.actors {
        if let crate::characters::RuntimeState::Hathor(state) = &mut actor.character
            && let Some(gaze) = &mut state.gaze
        {
            let entry = &mut gaze.targets[target];
            entry.mark = None;
            entry.deadline = None;
            entry.proximity = false;
            entry.generation = entry
                .generation
                .checked_add(1)
                .ok_or("Hathor Gaze generation overflow")?;
        }
    }
    Ok(())
}
