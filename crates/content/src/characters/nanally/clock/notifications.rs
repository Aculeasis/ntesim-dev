//! Reached owner notifications; interruption never issues an unborn state.
use super::{Kind, deadline};
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Marker {
    Cooldown,
    Authority,
    Underboss,
    Held { next: Kind, progress: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub(super) owner: usize,
    pub(super) command: usize,
    marker: Marker,
}
impl Event {
    pub(super) fn progress(&self) -> u64 {
        match self.marker {
            Marker::Cooldown => 425_571,
            Marker::Authority => 429_917,
            Marker::Underboss => 3_415_870,
            Marker::Held { progress, .. } => progress,
        }
    }
}

pub(super) fn schedule(ctx: &mut CastContext<'_>, kind: Kind) -> Result<(), String> {
    let mut markers = match kind {
        Kind::Skill => vec![Marker::Cooldown, Marker::Authority],
        Kind::Burst => vec![Marker::Underboss],
        _ => vec![],
    };
    let clock = &ctx.actor.character.nanally_mut()?.clock;
    let action = clock.action.as_ref().ok_or("Nanally state clock missing")?;
    if let Some((next, progress)) = super::held::branch(kind)
        && let Some(release) = action.held_until
        && deadline(clock, action, progress)? < release
    {
        markers.push(Marker::Held { next, progress });
    }
    for marker in markers {
        let event = Event {
            owner: ctx.actor_index,
            command: ctx.command_index,
            marker,
        };
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: deadline(clock, action, event.progress())?,
            priority: crate::execution::CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::NanallyClock(event)),
        });
    }
    Ok(())
}

pub(crate) fn apply(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    let actor = &mut ctx.state.actors[event.owner];
    if actor.hp == 0 {
        return Ok(());
    }
    let Some(action) = actor
        .character
        .nanally_mut()?
        .clock
        .action
        .as_ref()
        .filter(|action| action.command == event.command)
    else {
        return Ok(());
    };
    let captured = action
        .owned_action
        .clone()
        .ok_or("Nanally state notification lost its actual action")?;
    let release = action.held_until;
    match event.marker {
        Marker::Cooldown => {
            let until = ctx
                .state
                .current_time_us
                .checked_add(super::super::removal::SKILL_COOLDOWN_US)
                .ok_or("Nanally Skill cooldown overflow")?;
            ctx.state.actors[event.owner]
                .ability_cooldowns
                .insert("GA_Nanally_Skill".into(), until);
            Ok(())
        }
        Marker::Authority => {
            captured.with_cast(ctx, event.owner, 0, super::super::skill::apply_authority)
        }
        Marker::Underboss => {
            captured.with_cast(ctx, event.owner, 0, super::super::underboss::start)
        }
        Marker::Held { next, .. } => {
            let release = release.ok_or("Nanally held section lost its release")?;
            captured.with_cast(ctx, event.owner, 0, |cast| {
                super::held::advance(cast, next, release)
            })
        }
    }
}

impl Event {
    pub(crate) fn timer_clock(&self) -> ntesim_core::kernel::timing::ClockDomain {
        crate::simulation::timer_policy::action(self.owner, self.command, false)
    }
}
