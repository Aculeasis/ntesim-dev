//! Ordinary caster progress; packet contact delays only this owner's montage.
use crate::runtime_api::{ActorHitContext, CastContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{GameScenarioRequest, events::GameEventData, state::ActorRuntime};
use ntesim_core::kernel::timing::{HitlagTracker, HitlagWindow};
use std::collections::{BTreeMap, BTreeSet};
mod profile;
use profile::Gate;
pub(crate) use profile::Kind;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::BlackbirdOrdinaryClockV1;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    tracker: HitlagTracker,
    action: Option<Action>,
    contacts: BTreeSet<String>,
}

/// Owner input families with their own documented earliest cancellation gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cancel {
    Jump,
    Skill,
    Movement,
}

#[derive(Clone, Debug)]
struct Action {
    kind: Kind,
    owner: usize,
    start_us: u64,
    recovery: u64,
    switch_lock: Option<u64>,
    jump: Gate,
    skill: Gate,
    movement: Gate,
    montage: Vec<(u64, u64)>,
    spawn: Option<SpawnPulses>,
}

#[derive(Clone, Debug)]
struct SpawnPulses {
    offset: u64,
    period_us: u64,
    groups: Vec<Vec<u64>>,
    spawned: bool,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let Some(selected) = profile::select(ctx)? else {
        return Ok(());
    };
    ctx.record_hypothesis(MODEL)?;
    let (montage, spawn) = capture(ctx, &selected)?;
    let mut clock = std::mem::take(&mut ctx.actor.character.blackbird_mut()?.clock);
    if let Some(previous) = clock.action.take() {
        remove_action_events(ctx.queue, &previous, &clock.tracker, ctx.time_us)?;
    }
    let mut carried = HitlagTracker::default();
    for window in clock.tracker.windows() {
        if window.start_real_us.saturating_add(window.duration_real_us) > ctx.time_us {
            carried
                .add_window(window.clone())
                .map_err(|e| format!("Blackbird clock: {e:?}"))?;
        }
    }
    clock.tracker = carried;
    clock.contacts.clear();
    let hold_us = ctx
        .request
        .hold_inputs
        .get(&ctx.command_index)
        .filter(|input| input.duration_us >= super::branches::HOLD_US)
        .map_or(0, |input| input.duration_us);
    clock.action = Some(Action {
        kind: selected.kind,
        owner: ctx.actor_index,
        start_us: ctx
            .time_us
            .checked_add(hold_us)
            .ok_or("Blackbird hold start overflow")?,
        recovery: selected.recovery,
        switch_lock: selected.switch_lock,
        jump: selected.jump,
        skill: selected.skill,
        movement: selected.movement,
        montage,
        spawn,
    });
    project(ctx.actor, ctx.queue, &mut clock, ctx.time_us)?;
    ctx.actor.character.blackbird_mut()?.clock = clock;
    if ctx.ability_id == "GA_BlackBird_Skill" {
        ctx.actor.ability_cooldowns.insert(
            ctx.ability_id.into(),
            ctx.time_us
                .checked_add(16_000_000)
                .ok_or("Blackbird Skill cooldown overflow")?,
        );
    }
    Ok(())
}

fn capture(
    ctx: &CastContext<'_>,
    profile: &profile::Profile,
) -> Result<(Vec<(u64, u64)>, Option<SpawnPulses>), String> {
    let mut groups = BTreeMap::<String, Vec<u64>>::new();
    for event in ctx.queue.iter() {
        if let GameEventData::HitDelivery {
            command_index,
            actor_index,
            ability_id,
            metadata,
            ..
        } = &event.data
            && *command_index == ctx.command_index
            && *actor_index == ctx.actor_index
            && ability_id == ctx.ability_id
        {
            let group = metadata
                .binding
                .energy_group_id
                .as_ref()
                .ok_or("Blackbird clock requires causal packet groups")?;
            groups.entry(group.clone()).or_default().push(event.seq);
        }
    }
    let mut groups: Vec<Vec<u64>> = groups.into_values().collect();
    groups.sort_by_key(|sequences| *sequences.iter().min().expect("nonempty packet group"));
    let (mut montage, spawn) = match profile.spawn {
        Some(spawn) => {
            if !profile.offsets.is_empty() || groups.len() != spawn.count {
                return Err(
                    "Blackbird clock packet count disagrees with the native actor spawn".into(),
                );
            }
            (
                Vec::new(),
                Some(SpawnPulses {
                    offset: spawn.offset,
                    period_us: spawn.period_us,
                    groups,
                    spawned: false,
                }),
            )
        }
        None => {
            if groups.len() != profile.offsets.len() {
                return Err("Blackbird clock packet count disagrees with the native action".into());
            }
            let mut montage = Vec::new();
            for (sequences, &offset) in groups.into_iter().zip(&profile.offsets) {
                montage.extend(sequences.into_iter().map(|sequence| (sequence, offset)));
            }
            (montage, None)
        }
    };
    for event in ctx.queue.iter() {
        if let GameEventData::Content(crate::callbacks::ContentEvent::Blackbird(event_data)) =
            &event.data
            && let Some(offset) = event_data.clock_anchor(ctx.actor_index, ctx.command_index)
        {
            montage.push((event.seq, offset));
        }
    }
    Ok((montage, spawn))
}

fn remove_action_events(
    queue: &mut BinaryHeap,
    action: &Action,
    tracker: &HitlagTracker,
    now: u64,
) -> Result<(), String> {
    let spawned = action
        .spawn
        .as_ref()
        .map(|spawn| {
            resolve(tracker, action.owner, action.start_us, spawn.offset)
                .map(|deadline| spawn.spawned || now >= deadline)
        })
        .transpose()?
        .unwrap_or(false);
    let spawn_bound = action.spawn.as_ref().filter(|_| !spawned).map(|spawn| {
        spawn
            .groups
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>()
    });
    let events = queue.drain().collect::<Vec<_>>();
    let mut kept = Vec::with_capacity(events.len());
    for event in events {
        let montage_bound = action
            .montage
            .iter()
            .any(|(sequence, _)| *sequence == event.seq);
        let spawn_bound = spawn_bound
            .as_ref()
            .is_some_and(|sequences| sequences.contains(&event.seq));
        if !montage_bound && !spawn_bound {
            kept.push(event);
        }
    }
    queue.replace(kept);
    Ok(())
}

fn resolve(
    tracker: &HitlagTracker,
    owner: usize,
    start_us: u64,
    offset: u64,
) -> Result<u64, String> {
    tracker
        .resolve_real_deadline(owner, start_us, offset)
        .map_err(|e| format!("Blackbird caster deadline: {e:?}"))
}

fn gate_deadline(clock: &State, action: &Action, gate: Gate) -> Result<u64, String> {
    let offset = match gate {
        Gate::At(offset) => offset,
        Gate::AnySkill => action.recovery,
    };
    resolve(
        &clock.tracker,
        action.owner,
        action.start_us,
        offset.min(action.recovery),
    )
}

fn project(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    clock: &mut State,
    now: u64,
) -> Result<(), String> {
    let Some(action) = &mut clock.action else {
        return Ok(());
    };
    let resolve = |offset| {
        clock
            .tracker
            .resolve_real_deadline(action.owner, action.start_us, offset)
            .map_err(|e| format!("Blackbird caster deadline: {e:?}"))
    };
    let mut deadlines = BTreeMap::new();
    for &(_, offset) in &action.montage {
        if let std::collections::btree_map::Entry::Vacant(entry) = deadlines.entry(offset) {
            entry.insert(resolve(offset)?);
        }
    }
    let mut events = queue.drain().collect::<Vec<_>>();
    let indices: BTreeMap<_, _> = events
        .iter()
        .enumerate()
        .map(|(index, event)| (event.seq, index))
        .collect();
    action
        .montage
        .retain(|(sequence, _)| indices.contains_key(sequence));
    for &(sequence, offset) in &action.montage {
        let event = &mut events[indices[&sequence]];
        // Other recipients of an already emitted simultaneous packet keep its time.
        if event.time_us > now || now == action.start_us {
            event.time_us = deadlines[&offset];
        }
    }
    if let Some(spawn) = &mut action.spawn
        && !spawn.spawned
    {
        let deadline = resolve(spawn.offset)?;
        if now >= deadline {
            // The actor exists; its remaining actor-local hits keep their real times.
            spawn.spawned = true;
        } else {
            for (index, group) in spawn.groups.iter().enumerate() {
                let time = spawn
                    .period_us
                    .checked_mul(index as u64)
                    .and_then(|offset| deadline.checked_add(offset))
                    .ok_or("Blackbird actor pulse deadline overflow")?;
                for sequence in group {
                    if let Some(&event) = indices.get(sequence)
                        && (events[event].time_us > now || now == action.start_us)
                    {
                        events[event].time_us = time;
                    }
                }
            }
        }
    }
    actor.locked_until_us = resolve(action.recovery)?;
    if let Some(offset) = action.switch_lock {
        actor.character.blackbird_mut()?.qte_switch_until_us = resolve(offset)?;
    }
    if matches!(
        action.kind,
        Kind::OrdinaryNormal(_) | Kind::EvolvedNormal(_)
    ) {
        let state = actor.character.blackbird_mut()?;
        state.normal_started_us = Some(actor.locked_until_us);
        state.evolve_melee_started_us = Some(actor.locked_until_us);
    }
    queue.replace(events);
    Ok(())
}

/// Earliest applicable cancellation deadline of the unfinished owner montage.
pub(crate) fn gate(actor: &ActorRuntime, now: u64, cancel: Cancel) -> Result<Option<u64>, String> {
    let crate::characters::RuntimeState::Blackbird(state) = &actor.character else {
        return Ok(None);
    };
    if now > actor.locked_until_us {
        return Ok(None);
    }
    let Some(action) = &state.clock.action else {
        return Ok(None);
    };
    let gate = match cancel {
        Cancel::Jump => action.jump,
        Cancel::Skill => action.skill,
        Cancel::Movement => action.movement,
    };
    gate_deadline(&state.clock, action, gate).map(Some)
}

/// Earliest deadline across all cancel families; a held branch cannot precede it.
pub(crate) fn earliest(actor: &ActorRuntime, now: u64) -> Result<Option<u64>, String> {
    let crate::characters::RuntimeState::Blackbird(state) = &actor.character else {
        return Ok(None);
    };
    if now > actor.locked_until_us {
        return Ok(None);
    }
    let Some(action) = &state.clock.action else {
        return Ok(None);
    };
    let mut earliest = u64::MAX;
    for gate in [action.jump, action.skill, action.movement] {
        earliest = earliest.min(gate_deadline(&state.clock, action, gate)?);
    }
    Ok(Some(earliest))
}

/// Kind of the still-unfinished montage, if any. Released montages cannot be branched or canceled.
pub(crate) fn unfinished(actor: &ActorRuntime, now: u64) -> Result<Option<Kind>, String> {
    let crate::characters::RuntimeState::Blackbird(state) = &actor.character else {
        return Ok(None);
    };
    let Some(action) = &state.clock.action else {
        return Ok(None);
    };
    let end = gate_deadline(&state.clock, action, Gate::AnySkill)?;
    Ok((now < end).then_some(action.kind))
}

/// Cancel the unfinished montage's pending requests; spawned actors and captured extras remain.
pub(crate) fn cancel_pending(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.blackbird_mut()?;
    if let Some(action) = state.clock.action.take() {
        remove_action_events(
            ctx.queue,
            &action,
            &state.clock.tracker,
            ctx.state.current_time_us,
        )?;
    }
    Ok(())
}

pub(crate) fn on_exit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    cancel_pending(ctx, owner)
}

pub(crate) fn on_input(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: crate::simulation::PlayerInput,
) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    cancel_pending(ctx, owner)?;
    if input == crate::simulation::PlayerInput::Dash && super::secondary::accepted(ctx.request) {
        let actor = &mut ctx.state.actors[owner];
        let back = ctx.request.dash_directions.get(&command)
            == Some(&crate::simulation::GameDashDirection::Back);
        let recovery = if back {
            if super::form::active(actor) {
                1_008_023
            } else {
                1_009_134
            }
        } else {
            1_004_760
        };
        let mut clock = std::mem::take(&mut actor.character.blackbird_mut()?.clock);
        clock.action = Some(Action {
            kind: Kind::Dodge,
            owner,
            start_us: ctx.state.current_time_us,
            recovery,
            switch_lock: None,
            jump: Gate::At(if back { 0 } else { 83_202 }),
            skill: Gate::At(if back { 531_889 } else { 333_319 }),
            movement: Gate::AnySkill,
            montage: Vec::new(),
            spawn: None,
        });
        project(actor, ctx.queue, &mut clock, ctx.state.current_time_us)?;
        actor.character.blackbird_mut()?.clock = clock;
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let burst = ctx.damage_effect_id == "GE_Player_BlackBird_UltraSkill_Damage";
    if burst && !super::form::accepted(ctx.request) {
        return Ok(());
    }

    if ctx.damage_effect_id == super::support::PACKET && !super::support::accepted(ctx.request) {
        return Ok(());
    }
    if ctx.damage_effect_id == super::support::EVOLVED_PACKET
        && !super::support::evolved_accepted(ctx.request)
    {
        return Ok(());
    }

    if !accepted(ctx.request)
        || ctx.applied_hp_loss == 0
        || !matches!(
            ctx.damage_effect_id,
            "GE_Player_BlackBird_Melee1_Damage"
                | "GE_Player_BlackBird_Melee2_Damage"
                | "GE_Player_BlackBird_Melee3_Damage"
                | "GE_Player_BlackBird_Melee4_Damage"
                | "GE_Player_BlackBird_Melee5_Damage"
                | super::branches::ROOSTCALL_PACKET
                | super::branches::BESTOWED_GRACE_PACKET
                | super::branches::TIDINGS_PACKET
                | "GE_Player_BlackBird_Skill_Damage"
                | super::secondary::PLUNGE
                | super::secondary::RIPOSTE
                | super::secondary::EVOLVE_PLUNGE
                | super::secondary::EVOLVE_RIPOSTE
                | super::support::PACKET
                | super::support::EVOLVED_PACKET
                | "GE_Player_BlackBird_UltraSkill_Damage"
                | "GE_Player_BlackBird_Evolve_Melee1_Damage"
                | "GE_Player_BlackBird_Evolve_Melee2_Damage"
                | "GE_Player_BlackBird_Evolve_Melee3_Damage"
                | "GE_Player_BlackBird_Evolve_Melee4_Damage"
                | "GE_Player_BlackBird_Evolve_Melee5_Damage"
        )
    {
        return Ok(());
    }
    let group = ctx
        .packet_group_id
        .ok_or("Blackbird contact requires a causal packet group")?;
    let is_melee5 = ctx.damage_effect_id == "GE_Player_BlackBird_Evolve_Melee5_Damage";
    let mut clock = std::mem::take(&mut ctx.actor.character.blackbird_mut()?.clock);
    if clock.contacts.insert(group.into()) {
        clock
            .tracker
            .add_window(HitlagWindow {
                actor_index: ctx.actor_index,
                start_real_us: ctx.time_us,
                duration_real_us: if burst {
                    67_000
                } else if matches!(
                    ctx.damage_effect_id,
                    super::secondary::PLUNGE
                        | super::secondary::RIPOSTE
                        | super::secondary::EVOLVE_PLUNGE
                        | super::secondary::EVOLVE_RIPOSTE
                        | super::support::PACKET
                        | super::support::EVOLVED_PACKET
                ) {
                    130_000
                } else {
                    30_000
                },
                dilated_rate_ppm: if burst {
                    300_000
                } else if is_melee5 {
                    500_000
                } else {
                    10_000
                },
            })
            .map_err(|e| format!("Blackbird hitstop: {e:?}"))?;
        project(ctx.actor, ctx.queue, &mut clock, ctx.time_us)?;
    }
    ctx.actor.character.blackbird_mut()?.clock = clock;
    Ok(())
}

impl State {
    pub(crate) fn global_freeze(
        &mut self,
        owner: usize,
        start: u64,
        duration: u64,
    ) -> Result<(), String> {
        self.tracker
            .add_freeze(owner, start, duration)
            .map_err(|error| error.to_string())
    }
}
