//! Finite ordinary montage contacts and notifications on the real combat axis.
use super::owned_packet::Source;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameActionCommand, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) mod controls;
mod profile;
pub(super) type Plan = (Kind, Vec<Hit>, u64, u64);
pub(crate) use controls::{committed, dodging, recovery};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HanielOrdinaryClockV1;
const SKILL: &str = "GA_Haniel_Skill";
const BURST: &str = "GA_Haniel_UltraSkill";
type Hit = (&'static str, u64);
const NORMAL: [&[Hit]; 5] = [
    &[("GE_Player_Haniel_Melee1_Damage", 179_633)],
    &[
        ("GE_Player_Haniel_Melee2_Damage", 227_892),
        ("GE_Player_Haniel_Melee2_Damage", 430_246),
    ],
    &[("GE_Player_Haniel_Melee3_Damage", 119_031)],
    &[
        ("GE_Player_Haniel_Melee4_Damage", 332_637),
        ("GE_Player_Haniel_Melee4_Damage", 373_916),
        ("GE_Player_Haniel_Melee4_Damage", 412_088),
        ("GE_Player_Haniel_Melee4_Damage", 455_757),
    ],
    &[("GE_Player_Haniel_Melee5_Damage", 608_633)],
];
const RECOVERY: [u64; 5] = [887_510, 722_267, 417_606, 655_768, 950_032];

#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    active: Option<Action>,
}
#[derive(Clone, Debug)]
struct Action {
    owner: usize,
    command: usize,
    groups: BTreeSet<String>,
    skill: bool,
    cooldown_started: bool,
    charge: Option<super::cannon::Charge>,
    start: u64,
    kind: Kind,
    pause: u64,
    contacts: BTreeSet<String>,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Kind {
    Normal(usize),
    Entry,
    Skill,
    Burst,
    Charge,
    FastCharge,
    Plunge,
    Riposte,
    Support,
    Dodge(bool),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Boundary {
    SkillCooldown,
    Ace(Box<Source>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    owner: usize,
    command: usize,
    boundary: Boundary,
}

pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    super::cannon::accepted(req)
        || req
            .model_assumptions
            .get(MODEL.as_str())
            .map(String::as_str)
            == Some(MODEL.description())
}
pub(crate) fn cycle(
    req: &GameScenarioRequest,
    effect: &str,
) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
    if !accepted(req) {
        return Ok(None);
    }
    let Some(raw) =
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
    else {
        return Ok(None);
    };
    Ok(Some(crate::runtime_api::PacketCycleGrant {
        raw: crate::simulation::ExactFraction::new(raw.numerator(), raw.denominator())?,
        hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
    }))
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(req) {
        return Ok(());
    }
    for actor in req
        .actors
        .iter()
        .filter(|a| a.character_id == "character:1020")
    {
        if actor.energy_capacity != 120 {
            return Err("Haniel ordinary clock requires Ultimate capacity120".into());
        }
    }
    for (index, action) in req.actions.iter().enumerate() {
        if let GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            energy_cost,
            ..
        } = action
            && req.actors[*actor_index].character_id == "character:1020"
        {
            let plunge = super::secondary::accepted(req)
                && damage_effect_id == super::secondary::PLUNGE
                && ability_id == "GA_Haniel_Melee"
                && req.plunge_inputs.contains_key(&index);
            if plunge && req.hold_inputs.contains_key(&index) {
                return Err("Haniel plunge cannot also be held Cannon input".into());
            }
            if !matches!(ability_id.as_str(), "GA_Haniel_Melee" | SKILL | BURST)
                || (!plunge && super::primary_packet(ability_id) != Some(damage_effect_id.as_str()))
                || *energy_cost
                    != if ability_id == BURST {
                        super::BURST_ENERGY_COST
                    } else {
                        0
                    }
            {
                return Err("Haniel ordinary clock requires canonical Normal/Skill/Burst and native energy cost".into());
            }
            if (req.hold_inputs.contains_key(&index)
                && (!super::cannon::accepted(req) || ability_id != "GA_Haniel_Melee"))
                || (req.plunge_inputs.contains_key(&index) && !plunge)
            {
                return Err(
                    "Haniel charged/plunge inputs require their remaining owner bindings".into(),
                );
            }
        }
    }
    Ok(())
}
fn belongs(event: &ScheduledGameEvent, action: &Action) -> bool {
    match &event.data {
        GameEventData::HitDelivery { metadata, .. } => metadata
            .binding
            .energy_group_id
            .as_ref()
            .is_some_and(|group| action.groups.contains(group)),
        GameEventData::Content(crate::callbacks::ContentEvent::HanielClock(event)) => {
            event.owner == action.owner && event.command == action.command
        }
        GameEventData::Content(crate::callbacks::ContentEvent::Haniel(
            super::hootie::Event::Spawn(spawn),
        )) => spawn.belongs(action.owner, action.command),
        GameEventData::Content(crate::callbacks::ContentEvent::HanielCannon(event)) => {
            event.belongs(action.owner, action.command)
        }
        _ => false,
    }
}
pub(crate) fn stop(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    let Some(clock) = actor.character.haniel_mut()?.clock.as_mut() else {
        return Ok(());
    };
    if let Some(action) = clock.active.take() {
        queue.retain(|event| {
            // A projectile that already impacted owns its same-time spawn callback.
            let emitted = event.time_us <= now
                && matches!(
                    &event.data,
                    GameEventData::Content(crate::callbacks::ContentEvent::Haniel(
                        super::hootie::Event::Spawn(_)
                    ))
                );
            emitted || !belongs(event, &action)
        });
        if action.skill && !action.cooldown_started {
            actor.ability_cooldowns.insert(
                SKILL.into(),
                now.checked_add(3_000_000)
                    .ok_or("Haniel canceled Skill cooldown overflow")?,
            );
        }
    }
    Ok(())
}
fn schedule(ctx: &mut CastContext<'_>, progress: u64, boundary: Boundary) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx
            .time_us
            .checked_add(progress)
            .ok_or("Haniel notify deadline overflow")?,
        priority: ntesim_core::execution::CausalPriority::ActionAdmission,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::HanielClock(Event {
            owner: ctx.actor_index,
            command: ctx.command_index,
            boundary,
        })),
    });
    Ok(())
}
pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(MODEL)?;
    if super::secondary::accepted(ctx.request) {
        ctx.record_hypothesis(super::secondary::MODEL)?;
    }
    let charged = super::cannon::input(ctx)?;
    let continued = crate::characters::normal::continues_at(
        ctx.actor.character.haniel_mut()?.normal_started_us,
        ctx.time_us,
    );
    stop(ctx.actor, ctx.queue, ctx.time_us)?;
    let (kind, hits, recovery, landing) = profile::select(ctx, charged, continued)?;
    let start = ctx
        .time_us
        .checked_add(landing)
        .ok_or("Haniel landing overflow")?;
    if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(hits[0].0) {
        ctx.replace_native_primary_packet(hits[0].0)?;
    }
    let mut groups = BTreeMap::from([(ctx.scope_primary_aoe_group(hits[0].0)?, hits[0].1)]);
    for &(effect, progress) in &hits[1..] {
        groups.insert(
            ctx.schedule_native_aoe_group(effect, ctx.time_us)?,
            progress,
        );
    }
    let mut events = ctx.queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if let GameEventData::HitDelivery { metadata, .. } = &event.data
            && let Some(progress) = metadata
                .binding
                .energy_group_id
                .as_ref()
                .and_then(|g| groups.get(g))
        {
            event.time_us = start
                .checked_add(*progress)
                .ok_or("Haniel impact deadline overflow")?;
        }
    }
    ctx.queue.replace(events);
    ctx.actor.locked_until_us = start
        .checked_add(recovery)
        .ok_or("Haniel recovery overflow")?;
    let state = ctx.actor.character.haniel_mut()?;
    state.normal_started_us =
        matches!(kind, Kind::Normal(_) | Kind::Entry).then_some(ctx.actor.locked_until_us);
    if ctx.ability_id != "GA_Haniel_Melee" {
        state.normal_stage = 0;
    }
    state
        .clock
        .as_mut()
        .ok_or("Haniel ordinary clock not initialized")?
        .active = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        groups: groups.into_keys().collect(),
        skill: ctx.ability_id == SKILL,
        cooldown_started: false,
        charge: (charged || matches!(kind, Kind::Riposte)).then(Default::default),
        start,
        kind,
        pause: 0,
        contacts: Default::default(),
    });
    match ctx.ability_id {
        SKILL => {
            ctx.actor.ability_cooldowns.remove(SKILL);
            schedule(ctx, 684_620, Boundary::SkillCooldown)?;
            let grant = super::skill::flat_team_atk(ctx)?;
            super::spawn::at(
                ctx,
                grant,
                ctx.time_us
                    .checked_add(719_512)
                    .ok_or("Haniel spawn overflow")?,
            )?;
        }
        BURST => {
            ctx.actor.ability_cooldowns.insert(
                BURST.into(),
                ctx.time_us
                    .checked_add(20_000_000)
                    .ok_or("Haniel Burst cooldown overflow")?,
            );
            ctx.actor.switch_action_locked_until_us = ctx
                .time_us
                .checked_add(4_037_446)
                .ok_or("Haniel Burst switch lock overflow")?;
            schedule(
                ctx,
                2_869_166,
                Boundary::Ace(Box::new(Source::capture(ctx)?)),
            )?;
        }
        _ => {}
    }
    if charged {
        super::cannon::schedule(ctx)?;
    }
    if matches!(kind, Kind::Riposte) {
        super::cannon::riposte(ctx)?;
    }
    Ok(())
}

pub(super) fn charge(
    actor: &mut ActorRuntime,
    command: usize,
) -> Result<Option<&mut super::cannon::Charge>, String> {
    Ok(actor
        .character
        .haniel_mut()?
        .clock
        .as_mut()
        .and_then(|c| c.active.as_mut())
        .filter(|a| a.command == command)
        .and_then(|a| a.charge.as_mut()))
}
pub(super) fn accelerate(actor: &mut ActorRuntime, command: usize, now: u64) -> Result<(), String> {
    if let Some(action) = actor
        .character
        .haniel_mut()?
        .clock
        .as_mut()
        .and_then(|c| c.active.as_mut())
        .filter(|a| a.command == command)
    {
        action.kind = Kind::FastCharge;
        action.start = now;
    }
    Ok(())
}
pub(super) fn pause(
    actor: &mut ActorRuntime,
    command: usize,
    group: &str,
    loss: u64,
) -> Result<(), String> {
    if let Some(action) = actor
        .character
        .haniel_mut()?
        .clock
        .as_mut()
        .and_then(|c| c.active.as_mut())
        .filter(|a| a.command == command)
        && action.contacts.insert(group.into())
    {
        action.pause = action
            .pause
            .checked_add(loss)
            .ok_or("Haniel packet pause overflow")?;
        actor.locked_until_us = actor
            .locked_until_us
            .checked_add(loss)
            .ok_or("Haniel recovery pause overflow")?;
    }
    Ok(())
}
pub(crate) fn contact(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    super::cannon::contact(ctx)?;
    if !super::secondary::accepted(ctx.request) || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let loss = match ctx.damage_effect_id {
        super::secondary::RIPOSTE => 297_000,
        super::secondary::QTE => 210_000,
        _ => return Ok(()),
    };
    let group = ctx
        .packet_group_id
        .ok_or("Haniel secondary requires causal packet group")?;
    pause(ctx.actor, ctx.command_index, group, loss)
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    let actor = &mut ctx.state.actors[event.owner];
    if actor.hp == 0
        || actor
            .character
            .haniel_mut()?
            .clock
            .as_ref()
            .and_then(|c| c.active.as_ref())
            .is_none_or(|a| a.command != event.command)
    {
        return Ok(());
    }
    match event.boundary {
        Boundary::SkillCooldown => {
            let duration = if ctx.request.actors[event.owner].awakening.has_effect(6) {
                16_000_000
            } else {
                20_000_000
            };
            actor.ability_cooldowns.insert(
                SKILL.into(),
                ctx.state
                    .current_time_us
                    .checked_add(duration)
                    .ok_or("Haniel Skill cooldown overflow")?,
            );
            actor
                .character
                .haniel_mut()?
                .clock
                .as_mut()
                .unwrap()
                .active
                .as_mut()
                .unwrap()
                .cooldown_started = true;
            Ok(())
        }
        Boundary::Ace(source) => source.activate_ace(ctx, event.owner),
    }
}
pub(crate) fn interrupt(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let before = actor.locked_until_us;
    let cooldown = actor.ability_cooldowns.get(SKILL).copied();
    let command = actor
        .character
        .haniel_mut()?
        .clock
        .as_ref()
        .and_then(|c| c.active.as_ref())
        .filter(|_| now < before)
        .map(|a| a.command);
    let count = ctx
        .queue
        .iter()
        .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
        .count();
    if command.is_some() {
        stop(actor, ctx.queue, now)?;
        actor.locked_until_us = now;
        actor.character.haniel_mut()?.normal_stage = 0;
        actor.character.haniel_mut()?.normal_started_us = None;
    }
    Ok(crate::simulation::GameActorInterruptReceipt {
        event_index: event,
        actor_index: owner,
        time_us: now,
        interrupted_command_index: command,
        cancelled_hit_deliveries: count
            - ctx
                .queue
                .iter()
                .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
                .count(),
        recovery_before_us: before,
        recovery_after_us: actor.locked_until_us,
        skill_cooldown_before_us: cooldown,
        skill_cooldown_after_us: actor.ability_cooldowns.get(SKILL).copied(),
        field_cancelled: false,
    })
}

impl Event {
    pub(crate) fn timer_clock(&self) -> ntesim_core::kernel::timing::ClockDomain {
        crate::simulation::timer_policy::action(self.owner, self.command, false)
    }
}

impl Clock {
    pub(crate) fn global_freeze(
        &mut self,
        _owner: usize,
        _start: u64,
        duration: u64,
    ) -> Result<(), String> {
        if let Some(action) = &mut self.active {
            action.pause = action
                .pause
                .checked_add(duration)
                .ok_or("Haniel global pause overflow")?;
        }
        Ok(())
    }
}
