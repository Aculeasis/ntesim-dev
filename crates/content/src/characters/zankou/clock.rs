//! Finite owner montage. Cooldowns, form expiry and launched DoTs use real time.
use crate::runtime_api::{ActorHitContext, CastContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::kernel::timing::{HitlagTracker, HitlagWindow};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Normal(bool, usize),
    Held(bool),
    Skill(super::forms::Variant),
    Plunge(u64),
    Riposte,
    Support,
    Dodge(bool),
    Burst(super::burst::Variant),
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    lag: HitlagTracker,
    active: Option<Action>,
}
#[derive(Clone, Debug)]
struct Action {
    command: usize,
    owner: usize,
    start: u64,
    kind: Kind,
    modifier: Option<super::modifiers::Kind>,
    groups: BTreeMap<String, u64>,
    notifies: BTreeMap<u64, u64>,
    contacts: BTreeSet<String>,
}
fn due(clock: &Clock, action: &Action, progress: u64) -> Result<u64, String> {
    clock
        .lag
        .resolve_real_deadline(action.owner, action.start, progress)
        .map_err(|e| format!("Zankou montage deadline: {e:?}"))
}
pub(crate) fn entry_ready(clock: Option<&Clock>, now: u64) -> bool {
    let Some(clock) = clock else {
        return false;
    };
    let Some(action) = &clock.active else {
        return false;
    };
    matches!(
        action.kind,
        Kind::Normal(false, 3 | 4) | Kind::Held(false) | Kind::Riposte
    ) && due(clock, action, super::profile::skill(action.kind)).is_ok_and(|gate| now >= gate)
        && due(clock, action, super::profile::recovery(action.kind)).is_ok_and(|end| now <= end)
}
pub(crate) fn held_ready(clock: Option<&Clock>, magic: bool, now: u64) -> bool {
    let Some(clock) = clock else {
        return false;
    };
    let Some(action) = &clock.active else {
        return false;
    };
    matches!(action.kind, Kind::Normal(form,0 | 1) if form == magic)
        && due(clock, action, super::profile::recovery(action.kind)).is_ok_and(|end| now <= end)
}
fn belongs(event: &ScheduledGameEvent, action: &Action) -> bool {
    action.notifies.contains_key(&event.seq)
        || matches!(&event.data, GameEventData::HitDelivery { metadata, .. } if metadata.binding.energy_group_id.as_ref().is_some_and(|g| action.groups.contains_key(g)))
}
fn project(
    clock: &Clock,
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
    fresh: bool,
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
) -> Result<(), String> {
    let Some(action) = &clock.active else {
        return Ok(());
    };
    let mut events = queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if (!fresh && event.time_us <= now) || !belongs(event, action) {
            continue;
        }
        let progress = action
            .notifies
            .get(&event.seq)
            .copied()
            .or_else(|| match &event.data {
                GameEventData::HitDelivery { metadata, .. } => metadata
                    .binding
                    .energy_group_id
                    .as_ref()
                    .and_then(|g| action.groups.get(g))
                    .copied(),
                _ => None,
            });
        if let Some(progress) = progress {
            event.time_us = due(clock, action, progress)?;
        }
    }
    actor.locked_until_us = due(clock, action, super::profile::recovery(action.kind))?;
    let switch = match action.kind {
        Kind::Support => Some(1_015_890),
        Kind::Skill(super::forms::Variant::Return(false)) => Some(2_250_017),
        Kind::Skill(super::forms::Variant::Return(true)) => Some(2_249_976),
        Kind::Burst(super::burst::Variant::Bloodfeast) => Some(5_649_767),
        Kind::Burst(_) => Some(4_799_427),
        _ => None,
    };
    if let Some(progress) = switch {
        actor.switch_action_locked_until_us = due(clock, action, progress)?;
    }
    if matches!(action.kind, Kind::Normal(..)) {
        actor.character.zankou_mut()?.normal_started_us = Some(actor.locked_until_us);
    }
    queue.replace(events);
    super::modifiers::action(
        actor,
        action.owner,
        buffs,
        action.modifier,
        actor.locked_until_us,
        now,
    )
}
pub(crate) fn start(
    ctx: &mut CastContext<'_>,
    kind: Kind,
    groups: BTreeMap<String, u64>,
    first_seq: u64,
    modifier: Option<super::modifiers::Kind>,
) -> Result<(), String> {
    ctx.record_hypothesis(super::modifiers::MODEL)?;
    let notifies = ctx
        .queue
        .iter()
        .filter(|e| e.seq > first_seq && matches!(e.data, GameEventData::Content(_)))
        .map(|e| (e.seq, e.time_us - ctx.time_us))
        .collect();
    let mut clock = ctx
        .actor
        .character
        .zankou_mut()?
        .clock
        .take()
        .unwrap_or_default();
    ctx.actor.character.zankou_mut()?.armor_pressure_halves = None;
    clock.active = Some(Action {
        command: ctx.command_index,
        owner: ctx.actor_index,
        start: ctx.time_us,
        kind,
        modifier,
        groups,
        notifies,
        contacts: Default::default(),
    });
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true, ctx.buffs)?;
    ctx.actor.character.zankou_mut()?.clock = Some(clock);
    super::combat_passives::arm_heal(ctx)?;
    Ok(())
}
pub(crate) fn command(actor: &ActorRuntime) -> Option<usize> {
    let crate::characters::RuntimeState::Zankou(state) = &actor.character else {
        return None;
    };
    state.clock.as_ref()?.active.as_ref().map(|a| a.command)
}
pub(crate) fn dodge(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let mut clock = actor
        .character
        .zankou_mut()?
        .clock
        .take()
        .unwrap_or_default();
    let back = ctx.request.dash_directions.get(&command)
        == Some(&crate::simulation::GameDashDirection::Back);
    clock.active = Some(Action {
        command,
        owner,
        start: now,
        kind: Kind::Dodge(back),
        modifier: None,
        groups: Default::default(),
        notifies: Default::default(),
        contacts: Default::default(),
    });
    project(&clock, actor, ctx.queue, now, true, &mut ctx.state.buffs)?;
    actor.character.zankou_mut()?.clock = Some(clock);
    Ok(())
}
pub(crate) fn stop(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    now: u64,
) -> Result<(), String> {
    if let Some(clock) = &mut actor.character.zankou_mut()?.clock
        && let Some(action) = clock.active.take()
    {
        queue.retain(|e| !belongs(e, &action));
        if matches!(
            action.kind,
            Kind::Burst(super::burst::Variant::Inferno | super::burst::Variant::FreeInferno)
        ) {
            actor.character.zankou_mut()?.kindling_active_for_ultra = false;
        }
        super::modifiers::action(actor, action.owner, buffs, None, now, now)?;
    }
    Ok(())
}
pub(crate) fn contact(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let pause = super::profile::pause(ctx.damage_effect_id);
    if pause == 0 || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let Some(mut clock) = ctx.actor.character.zankou_mut()?.clock.take() else {
        return Ok(());
    };
    if let Some(group) = ctx.packet_group_id
        && clock
            .active
            .as_mut()
            .is_some_and(|a| a.groups.contains_key(group) && a.contacts.insert(group.into()))
    {
        clock
            .lag
            .add_window(HitlagWindow {
                actor_index: ctx.actor_index,
                start_real_us: ctx.time_us,
                duration_real_us: pause,
                dilated_rate_ppm: 0,
            })
            .map_err(|e| format!("Zankou contact pause: {e:?}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us, false, ctx.buffs)?;
    }
    ctx.actor.character.zankou_mut()?.clock = Some(clock);
    Ok(())
}
pub(crate) fn recovery(
    actor: &ActorRuntime,
    input: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    let crate::characters::RuntimeState::Zankou(state) = &actor.character else {
        return None;
    };
    let Some(clock) = &state.clock else {
        return (input == "jump").then_some((now, super::forms::MODEL));
    };
    let Some(action) = &clock.active else {
        return (input == "jump").then_some((now, super::forms::MODEL));
    };
    let progress = match input {
        "switch_actor" => match action.kind {
            Kind::Skill(super::forms::Variant::Entry(_)) => 0,
            Kind::Skill(super::forms::Variant::Return(false)) => 2_250_017,
            Kind::Skill(super::forms::Variant::Return(true)) => 2_249_976,
            Kind::Support => 1_015_890,
            Kind::Burst(super::burst::Variant::Bloodfeast) => 5_649_767,
            Kind::Burst(_) => 4_799_427,
            _ => super::profile::recovery(action.kind),
        },
        "GA_Zankou_Skill" => super::profile::skill(action.kind),
        "jump" => match action.kind {
            Kind::Skill(super::forms::Variant::Entry(false)) => 737_301,
            Kind::Skill(super::forms::Variant::Entry(true)) => 748_330,
            _ => super::profile::jump(action.kind),
        },
        "dash" => super::profile::movement(action.kind),
        _ => super::profile::recovery(action.kind),
    };
    Some((due(clock, action, progress).ok()?, super::forms::MODEL))
}

pub(crate) fn carries(actor: &ActorRuntime, kind: super::modifiers::Kind, now: u64) -> bool {
    let crate::characters::RuntimeState::Zankou(state) = &actor.character else {
        return false;
    };
    if state.a1
        || (if kind == super::modifiers::Kind::Hunt {
            state.hunt_until_us
        } else {
            state.delusion_until_us
        }) > now
    {
        return true;
    }
    let Some(clock) = &state.clock else {
        return false;
    };
    let Some(action) = &clock.active else {
        return false;
    };
    action.modifier == Some(kind)
        && due(clock, action, super::profile::recovery(action.kind)).is_ok_and(|end| now == end)
}

impl Clock {
    pub(crate) fn global_freeze(
        &mut self,
        owner: usize,
        start: u64,
        duration: u64,
    ) -> Result<(), String> {
        self.lag
            .add_freeze(owner, start, duration)
            .map_err(|error| error.to_string())
    }
}
