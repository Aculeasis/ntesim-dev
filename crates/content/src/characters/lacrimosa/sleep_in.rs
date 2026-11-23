//! One ordinary cooldown and one earned extra cast per cooldown window.
use crate::runtime_api::CastContext;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::LacrimosaSleepInV1;
pub(crate) const COOLDOWN_US: u64 = 16_000_000;
const REQUIRED_SECTIONS: u8 = 5;
const SKILL: &str = "GA_Lacrimosa_Skill";

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    ordinary_ready_us: u64,
    sections: u8,
    extra_ready: bool,
    extra_used: bool,
}

impl State {
    pub(crate) fn pause_world(&mut self, from: u64, delta: u64) -> Result<(), String> {
        crate::simulation::timer_state::future(&mut self.ordinary_ready_us, from, delta)
    }
    fn expire(&mut self, now: u64) {
        if now >= self.ordinary_ready_us {
            self.sections = 0;
            self.extra_ready = false;
            self.extra_used = false;
        }
    }

    fn visible_deadline(&self) -> u64 {
        if self.extra_ready {
            0
        } else {
            self.ordinary_ready_us
        }
    }
}

pub(super) fn on_skill(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(MODEL)?;
    // Admission has committed the supplied ordinary timer. An earned extra
    // preserves the previously captured deadline instead of starting that timer.
    let admitted_ready = ctx
        .actor
        .ability_cooldowns
        .get(SKILL)
        .copied()
        .unwrap_or(ctx.time_us);
    let state = &mut ctx.actor.character.lacrimosa_mut()?.sleep_in;
    state.expire(ctx.time_us);
    if state.extra_ready {
        state.extra_ready = false;
        state.extra_used = true;
        state.sections = 0;
    } else {
        state.ordinary_ready_us = admitted_ready;
        state.sections = 0;
        state.extra_used = false;
    }
    let deadline = state.visible_deadline();
    ctx.actor.ability_cooldowns.insert(SKILL.into(), deadline);
    Ok(())
}

pub(super) fn on_normal(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let eligible = {
        let state = &mut ctx.actor.character.lacrimosa_mut()?.sleep_in;
        state.expire(ctx.time_us);
        ctx.time_us < state.ordinary_ready_us && !state.extra_used && !state.extra_ready
    };
    if !eligible {
        return Ok(());
    }
    ctx.record_hypothesis(MODEL)?;
    let state = &mut ctx.actor.character.lacrimosa_mut()?.sleep_in;
    state.sections += 1;
    if state.sections == REQUIRED_SECTIONS {
        state.extra_ready = true;
        ctx.actor.ability_cooldowns.remove(SKILL);
    }
    Ok(())
}

pub(super) fn reduce_cooldown(
    ctx: &mut crate::runtime_api::ActorHitContext<'_>,
    duration: u64,
) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(ctx.request, ctx.execution_assumptions, MODEL)?;
    let state = &mut ctx.actor.character.lacrimosa_mut()?.sleep_in;
    state.ordinary_ready_us = state.ordinary_ready_us.saturating_sub(duration);
    state.expire(ctx.time_us);
    let deadline = state.visible_deadline();
    ctx.actor.ability_cooldowns.insert(SKILL.into(), deadline);
    Ok(())
}
