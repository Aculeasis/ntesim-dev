//! Combat membership and Nova's capped, reversible structural MaxHP transfer.
use crate::runtime_api::CallbackContext;
use crate::simulation::{
    ExactFraction as F, GameHpStatCause as Cause, GameHpStatRecipient as Recipient,
    GameStructuralHpChange,
};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub(crate) struct Membership {
    base_before: F,
    combat_base: F,
    b4: bool,
    gain: F,
    reductions: BTreeMap<usize, F>,
}
fn unlocked(ctx: &CallbackContext<'_>, owner: usize, tier: u64) -> Result<bool, String> {
    let config = &ctx.request.actors[owner];
    Ok(ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?["breakthrough"]
        .as_u64()
        .unwrap_or(0)
        >= tier)
}
fn min(a: F, b: F) -> Result<F, String> {
    Ok(if a.try_cmp(b).map_err(String::from)?.is_lt() {
        a
    } else {
        b
    })
}
fn flat(ctx: &CallbackContext<'_>, actor: usize) -> Result<F, String> {
    let mut value = F::from_ratio(ctx.request.actors[actor].static_stats.hp_flat)?;
    for runtime in &ctx.state.actors {
        if let crate::characters::RuntimeState::Fadia(state) = &runtime.character
            && let Some(member) = &state.membership
        {
            value = value.add(member.gain)?;
        }
    }
    Ok(value)
}
fn change_actor(
    ctx: &mut CallbackContext<'_>,
    source: usize,
    actor: usize,
    cause: Cause,
    change: impl FnOnce(&mut crate::simulation::state::ActorRuntime) -> Result<(), String>,
) -> Result<(), String> {
    let runtime = &mut ctx.state.actors[actor];
    let before = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        runtime.hp_max_base,
        &runtime.hp_max_static_bonus,
        actor,
        runtime.hp_max,
    )?;
    let hp_before = runtime.hp;
    change(runtime)?;
    runtime.hp_max = runtime
        .hp_max_base
        .mul(F::new(1, 1)?.add(runtime.hp_max_static_bonus)?)?
        .round_u64()?
        .max(1);
    let after = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        runtime.hp_max_base,
        &runtime.hp_max_static_bonus,
        actor,
        runtime.hp_max,
    )?;
    runtime.hp = runtime.hp.min(after);
    if before != after || hp_before != runtime.hp {
        ctx.state
            .structural_hp_changes
            .push(GameStructuralHpChange {
                time_us: ctx.state.current_time_us,
                source_actor_index: source,
                recipient: Recipient::Actor(actor),
                cause,
                max_hp_before: before,
                max_hp_after: after,
                hp_before,
                hp_after: runtime.hp,
            });
    }
    Ok(())
}
fn base(ctx: &mut CallbackContext<'_>, owner: usize, value: F, cause: Cause) -> Result<(), String> {
    let flat = flat(ctx, owner)?;
    change_actor(ctx, owner, owner, cause, |actor| {
        actor.hp_max_static_bonus = actor
            .hp_max_static_bonus
            .sub(flat.div(actor.hp_max_base)?)?
            .add(flat.div(value)?)?;
        actor.hp_max_base = value;
        Ok(())
    })
}
fn percent(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    actor: usize,
    remove: bool,
) -> Result<(), String> {
    change_actor(
        ctx,
        owner,
        actor,
        if remove {
            Cause::CombatExit
        } else {
            Cause::PartyMembership
        },
        |runtime| {
            runtime.hp_max_static_bonus = if remove {
                runtime.hp_max_static_bonus.sub(F::new(1, 10)?)?
            } else {
                runtime.hp_max_static_bonus.add(F::new(1, 10)?)?
            };
            Ok(())
        },
    )
}
fn add_flat(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    actor: usize,
    amount: F,
    remove: bool,
) -> Result<(), String> {
    change_actor(
        ctx,
        owner,
        actor,
        if remove {
            Cause::CombatExit
        } else {
            Cause::Transfer
        },
        |runtime| {
            let ratio = amount.div(runtime.hp_max_base)?;
            runtime.hp_max_static_bonus = if remove {
                runtime.hp_max_static_bonus.sub(ratio)?
            } else {
                runtime.hp_max_static_bonus.add(ratio)?
            };
            Ok(())
        },
    )
}
pub(crate) fn enter(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if !super::effects::accepted(ctx.request)
        || ctx.state.actors[owner]
            .character
            .fadia_mut()?
            .membership
            .is_some()
    {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        super::effects::MODEL,
    )?;
    let before = ctx.state.actors[owner].hp_max_base;
    if ctx.request.actors[owner].awakening.has_effect(3) {
        base(ctx, owner, before.mul(F::new(13, 10)?)?, Cause::BaseHp)?;
    }
    let combat_base = ctx.state.actors[owner].hp_max_base;
    let b4 = unlocked(ctx, owner, 4)?;
    ctx.state.actors[owner].character.fadia_mut()?.membership = Some(Box::new(Membership {
        base_before: before,
        combat_base,
        b4,
        gain: F::new(0, 1)?,
        reductions: BTreeMap::new(),
    }));
    if b4 {
        for actor in 0..ctx.state.actors.len() {
            percent(ctx, owner, actor, false)?;
        }
    }
    if ctx.request.actors[owner].awakening.has_resonance_6() {
        use ntesim_core::buffs::dispatch::*;
        ctx.state
            .buffs
            .apply_buff(
                BuffDefinition {
                    id: "fadia.r6_party_atk".into(),
                    source: "character:1039".into(),
                    recipient: BuffRecipient::TeamAll,
                    source_lifetime: BuffSourceLifetime::Snapshot,
                    stacking_scope: BuffStackingScope::PerSource,
                    duration_us: None,
                    max_stacks: 1,
                    refresh_policy: RefreshPolicy::Disallow,
                    decay_policy: DecayPolicy::AllAtOnce,
                    semantic_tags: Default::default(),
                    stat_modifiers: BTreeMap::from([(
                        "atk_percent".into(),
                        crate::formula::Ratio {
                            numerator: 1,
                            denominator: 10,
                        },
                    )]),
                },
                owner,
                ctx.state.mode_clock.relative_us(),
            )
            .map_err(|e| format!("Fadia R6 membership: {e:?}"))?;
    }
    Ok(())
}
fn reductions(ctx: &CallbackContext<'_>, target: usize) -> Result<F, String> {
    let mut sum = F::new(0, 1)?;
    for actor in &ctx.state.actors {
        if let crate::characters::RuntimeState::Fadia(state) = &actor.character
            && let Some(member) = &state.membership
            && let Some(value) = member.reductions.get(&target)
        {
            sum = sum.add(*value)?;
        }
    }
    Ok(sum)
}
fn target(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    cause: Cause,
) -> Result<(), String> {
    let after = F::new(ctx.request.targets[target].hp_max.into(), 1)?
        .sub(reductions(ctx, target)?)?
        .round_u64()?
        .max(1);
    let runtime = &mut ctx.state.targets[target];
    let before = runtime.hp_max;
    let hp_before = runtime.hp_current;
    runtime.hp_max = after;
    runtime.hp_current = runtime.hp_current.min(after);
    ctx.state
        .structural_hp_changes
        .push(GameStructuralHpChange {
            time_us: ctx.state.current_time_us,
            source_actor_index: owner,
            recipient: Recipient::Target(target),
            cause,
            max_hp_before: before,
            max_hp_after: after,
            hp_before,
            hp_after: runtime.hp_current,
        });
    Ok(())
}
pub(crate) fn nova_end(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    index: usize,
) -> Result<(), String> {
    if !super::effects::accepted(ctx.request)
        || !unlocked(ctx, owner, 2)?
        || ctx.state.targets[index].hp_current == 0
    {
        return Ok(());
    }
    let Some(member) = ctx.state.actors[owner]
        .character
        .fadia_mut()?
        .membership
        .clone()
    else {
        return Ok(());
    };
    let remaining =
        F::new(ctx.request.targets[index].hp_max.into(), 1)?.sub(reductions(ctx, index)?)?;
    let available = remaining.sub(F::new(1, 1)?)?.div(F::new(20, 1)?)?;
    let cap = member.combat_base.mul(F::new(1, 2)?)?.sub(member.gain)?;
    let amount = min(
        min(member.combat_base.mul(F::new(1, 10)?)?, cap)?,
        available,
    )?;
    if amount.is_zero() {
        return Ok(());
    }
    let ledger = ctx.state.actors[owner]
        .character
        .fadia_mut()?
        .membership
        .as_mut()
        .ok_or("Fadia Nova membership missing")?;
    ledger.gain = ledger.gain.add(amount)?;
    let old = ledger
        .reductions
        .get(&index)
        .copied()
        .unwrap_or(F::new(0, 1)?);
    ledger
        .reductions
        .insert(index, old.add(amount.mul(F::new(20, 1)?)?)?);
    target(ctx, owner, index, Cause::Transfer)?;
    for actor in 0..ctx.state.actors.len() {
        add_flat(ctx, owner, actor, amount, false)?;
    }
    Ok(())
}
pub(crate) fn leave(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let Some(member) = ctx.state.actors[owner]
        .character
        .fadia_mut()?
        .membership
        .clone()
    else {
        return Ok(());
    };
    // Remove only this source's flat grants, then restore its base. Other sources remain.
    for actor in 0..ctx.state.actors.len() {
        if !member.gain.is_zero() {
            add_flat(ctx, owner, actor, member.gain, true)?;
        }
        if member.b4 {
            percent(ctx, owner, actor, true)?;
        }
    }
    ctx.state.actors[owner].character.fadia_mut()?.membership = None;
    for index in member.reductions.keys() {
        target(ctx, owner, *index, Cause::CombatExit)?;
    }
    if ctx.state.actors[owner].hp_max_base != member.base_before {
        base(ctx, owner, member.base_before, Cause::CombatExit)?;
    }
    ctx.state
        .buffs
        .remove_owned_buff("fadia.r6_party_atk", "character:1039", owner);
    Ok(())
}
pub(crate) fn transition(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<crate::simulation::GameActorCombatTransitionReceipt, String> {
    let leaving = ctx.request.combat_transition_events[event].leaving;
    if ctx.state.actors[owner].hp == 0
        || ctx.state.actors[owner].character.fadia_mut()?.out_of_combat == leaving
    {
        return Err("Fadia combat transition requires a living actor and alternating state".into());
    }
    ctx.state.actors[owner].character.fadia_mut()?.out_of_combat = leaving;
    if leaving {
        super::clock::stop(&mut ctx.state.actors[owner], ctx.queue)?;
        let state = ctx.state.actors[owner].character.fadia_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
        state.lilith_until_us = None;
        state.high_q = false;
        state.riposte_ready = false;
        ctx.state.actors[owner].locked_until_us = ctx.state.current_time_us;
        super::effects::clear_marks(ctx, owner)?;
        // Purge packets settle first at HitDelivery; stat rollback follows at ProcCallback.
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: ctx.state.current_time_us,
                priority: ntesim_core::execution::CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::FadiaEffects(
                        super::effects::Event::CombatExit { owner },
                    ),
                ),
            });
    } else {
        enter(ctx, owner)?;
    }
    Ok(crate::simulation::GameActorCombatTransitionReceipt {
        event_index: event,
        actor_index: owner,
        time_us: ctx.state.current_time_us,
        leaving,
        malice_granted: F::new(0, 1)?,
        leave_grant_due_us: None,
        resource_deposits: Default::default(),
    })
}
