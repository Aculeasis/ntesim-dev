use super::{hootie::Event, owned_packet::Source};
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{
    ExactFraction,
    events::{GameEventData, ScheduledGameEvent},
};
use ntesim_core::{
    buffs::dispatch::{BuffRecipient, BuffStackingScope},
    execution::CausalPriority,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Spawn {
    owner: usize,
    source: Source,
    grant: ExactFraction,
}

impl Spawn {
    pub(crate) fn timer_clock(&self) -> ntesim_core::kernel::timing::ClockDomain {
        crate::simulation::timer_policy::action(self.owner, self.source.command(), false)
    }
    pub(super) fn belongs(&self, owner: usize, command: usize) -> bool {
        self.owner == owner && self.source.command() == command
    }
}

pub(crate) fn schedule(ctx: &mut CastContext<'_>, grant: ExactFraction) -> Result<(), String> {
    at(ctx, grant, ctx.hit_time_us)
}

pub(super) fn at(
    ctx: &mut CastContext<'_>,
    grant: ExactFraction,
    time_us: u64,
) -> Result<(), String> {
    let spawn = Spawn {
        owner: ctx.actor_index,
        source: Source::capture(ctx)?,
        grant,
    };
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us,
        priority: CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Haniel(Event::Spawn(
            Box::new(spawn),
        ))),
    });
    ctx.record_hypothesis(super::hootie::HYPOTHESIS)
}

pub(crate) fn apply(spawn: Spawn, ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    if ctx.state.actors[spawn.owner].hp == 0 {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[spawn.owner].character.haniel_mut()?;
    if !state.hootie_until_us.is_some_and(|until| until > now) {
        state.harmony = 0;
        state.harmony_ready_us = 0;
    }
    let ace_until = state
        .paranormal_ace_until_us
        .filter(|until| *until > now)
        .unwrap_or(now);
    let life = if ctx.request.actors[spawn.owner].awakening.has_effect(2) {
        12_000_000
    } else {
        8_000_000
    };
    let until = ace_until
        .checked_add(life)
        .ok_or("Haniel Hootie deadline overflow")?;
    state.hootie_until_us = Some(until);
    state.hootie_source = Some(Box::new(spawn.source));
    state.hootie_atk_add = Some(spawn.grant);
    crate::simulation::state::apply_timed_buff(
        &mut ctx.state.buffs,
        crate::simulation::state::TimedBuffDeclaration::reset(
            "haniel.hootie_team_atk",
            "character:1020",
            BuffRecipient::TeamAll,
            until - now,
            BuffStackingScope::PerSource,
            BTreeMap::from([("atk_add".into(), spawn.grant)]),
        ),
        spawn.owner,
        ctx.state.mode_clock.relative_us(),
    )?;
    if until <= ctx.request.absolute_limit_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: until,
            priority: CausalPriority::SettlementAndDeath,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::Haniel(Event::Expire {
                owner: spawn.owner,
                until,
            })),
        });
    }
    Ok(())
}
