//! Both forms share canonical Normal input and independent native A talent.
use super::{
    clock::{self, Kind},
    state::{Form, State},
};
use crate::runtime_api::{CastContext, NormalActionOverride};
use crate::simulation::{GameHoldInput, state::ActorRuntime};
pub(crate) fn kind(state: &State, now: u64) -> Kind {
    let magic = state.form_at(now) == Form::Illusion;
    Kind::Normal(
        magic,
        crate::characters::normal::after_gap(
            if state.form != state.form_at(now) {
                0
            } else {
                state.normal_stage
            },
            state.normal_started_us,
            now,
        ) % if magic { 4 } else { 5 },
    )
}
pub(crate) fn held(
    actor: &ActorRuntime,
    input: GameHoldInput,
    now: u64,
) -> Result<Option<NormalActionOverride>, String> {
    let crate::characters::RuntimeState::Zankou(state) = &actor.character else {
        return Ok(None);
    };
    if input.duration_us < 200_000 || input.reserved_stocks.is_some() {
        return Err("Zankou held Normal requires at least200000us and no stock reservation".into());
    }
    let magic = state.form_at(now) == Form::Illusion;
    if !clock::held_ready(state.clock.as_ref(), magic, now) {
        return Err("Zankou held Normal requires the current form's Normal1 or Normal2".into());
    }
    let kind = Kind::Held(magic);
    Ok(Some(NormalActionOverride {
        packet: super::profile::hits(kind)[0].0,
        role: None,
        model: super::forms::MODEL,
        timing: Some((
            super::profile::recovery(kind),
            super::profile::hits(kind)[0].1,
        )),
    }))
}
pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if let Some(kind) = super::secondary::kind(ctx)? {
        return super::secondary::cast(ctx, kind);
    }
    ctx.record_hypothesis(super::forms::MODEL)?;
    ctx.record_hypothesis(super::skill::CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.zankou_mut()?;
    let mut selected = kind(state, ctx.time_us);
    if ctx.request.hold_inputs.contains_key(&ctx.command_index) {
        selected = Kind::Held(state.form_at(ctx.time_us) == Form::Illusion);
    }
    super::forms::stop(ctx.actor, ctx.queue, ctx.buffs, ctx.time_us)?;
    ctx.cancel_primary_delivery()?;
    let first_seq = *ctx.sequence;
    if (matches!(selected, Kind::Normal(true, _) | Kind::Held(true))
        || ctx.config.awakening.has_effect(1))
        && let Some(delay) = super::profile::spread(selected)
    {
        super::spread::schedule(ctx, delay)?;
    }
    let mut groups = Default::default();
    for &(effect, delay) in super::profile::hits(selected) {
        let at = ctx
            .time_us
            .checked_add(delay)
            .ok_or("Zankou Normal hit overflow")?;
        let group = ctx.schedule_native_aoe_group(effect, at)?;
        std::collections::BTreeMap::insert(&mut groups, group, delay);
        if matches!(selected, Kind::Normal(true, _) | Kind::Held(true)) {
            super::vile_ash::apply_heartwrench(ctx, at)?;
        }
    }
    let state = ctx.actor.character.zankou_mut()?;
    state.normal_stage = match selected {
        Kind::Normal(magic, stage) => (stage + 1) % if magic { 4 } else { 5 },
        _ => 0,
    };
    state.normal_started_us = None;
    clock::start(
        ctx,
        selected,
        groups,
        first_seq,
        Some(
            if matches!(selected, Kind::Normal(true, _) | Kind::Held(true)) {
                super::modifiers::Kind::Delusion
            } else {
                super::modifiers::Kind::Hunt
            },
        ),
    )
}
