//! B2 Friendship: captured base ATK, source/target caps and combat rollback.
use crate::runtime_api::CallbackContext;
use crate::simulation::{ExactFraction as F, GameScenarioRequest};
use crate::simulation::{GameAtkTransferCause as Cause, GameAtkTransferReceipt};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HanielNovaExtractionV1;
const BUFF: &str = "haniel.friendship_atk";
const SOURCE: &str = "character:1020";

#[derive(Clone, Debug)]
pub(crate) struct Membership {
    base: F,
    gain: F,
    reductions: BTreeMap<usize, F>,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    super::secondary::accepted(request)
        || request
            .model_assumptions
            .get(MODEL.as_str())
            .map(String::as_str)
            == Some(MODEL.description())
}

pub(crate) fn validate_nova(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    target: usize,
) -> Result<(), String> {
    if accepted(request) && state.actors.iter().any(|actor| {
        matches!(
            &actor.character, crate::characters::RuntimeState::Haniel(s) if s.membership.is_some()
        )
    })
        && request
            .targets
            .get(target)
            .is_some_and(|t| t.original_atk.is_none())
    {
        return Err("Haniel B2 Nova extraction requires explicit target original_atk".into());
    }
    Ok(())
}

pub(crate) fn enter(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if !accepted(ctx.request)
        || ctx.state.actors[owner]
            .character
            .haniel_mut()?
            .membership
            .is_some()
    {
        return Ok(());
    }
    let config = &ctx.request.actors[owner];
    let tier = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?["breakthrough"]
        .as_u64()
        .unwrap_or(0);
    if tier < 2 {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    ctx.state.actors[owner].character.haniel_mut()?.membership = Some(Membership {
        base: F::new(config.static_stats.base_atk.into(), 1)?,
        gain: F::new(0, 1)?,
        reductions: BTreeMap::new(),
    });
    Ok(())
}

fn min(a: F, b: F) -> Result<F, String> {
    Ok(if a.try_cmp(b).map_err(String::from)?.is_lt() {
        a
    } else {
        b
    })
}

pub(crate) fn nova_end(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    generation: u64,
) -> Result<(), String> {
    if !accepted(ctx.request) || ctx.state.targets[target].hp_current == 0 {
        return Ok(());
    }
    let Some(member) = ctx.state.actors[owner]
        .character
        .haniel_mut()?
        .membership
        .clone()
    else {
        return Ok(());
    };
    let original = ctx.request.targets[target]
        .original_atk
        .ok_or("Haniel B2 Nova extraction requires explicit target original_atk")?;
    let original = F::new(original.into(), 1)?;
    let before = ctx.state.targets[target]
        .attack
        .ok_or("Haniel target ATK missing")?;
    let lost = original.sub(before)?;
    let available = original.mul(F::new(1, 5)?)?.sub(lost)?.mul(F::new(2, 1)?)?;
    let amount = min(
        min(
            member.base.mul(F::new(2, 25)?)?,
            member.base.mul(F::new(4, 25)?)?.sub(member.gain)?,
        )?,
        available,
    )?;
    if amount.is_zero() {
        return Ok(());
    }
    let gain = member.gain.add(amount)?;
    let drain = amount.mul(F::new(1, 2)?)?;
    let after = before.sub(drain)?;
    let reduction = member
        .reductions
        .get(&target)
        .copied()
        .unwrap_or(F::new(0, 1)?)
        .add(drain)?;
    // Prepare and apply the fallible buff before committing either side of the ledger.
    use ntesim_core::buffs::dispatch::*;
    ctx.state
        .buffs
        .apply_buff(
            BuffDefinition {
                id: BUFF.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::TeamAll,
                duration_us: None,
                stacking_scope: BuffStackingScope::PerSource,
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                source_lifetime: BuffSourceLifetime::Snapshot,
                semantic_tags: Default::default(),
                stat_modifiers: BTreeMap::from([(
                    "atk_add".into(),
                    crate::formula::Ratio {
                        numerator: gain
                            .numerator
                            .try_into()
                            .map_err(|_| "Haniel gain exceeds u64")?,
                        denominator: gain
                            .denominator
                            .try_into()
                            .map_err(|_| "Haniel gain denominator exceeds u64")?,
                    },
                )]),
            },
            owner,
            ctx.state.mode_clock.relative_us(),
        )
        .map_err(|e| format!("Haniel B2 buff failed: {e:?}"))?;
    let ledger = ctx.state.actors[owner]
        .character
        .haniel_mut()?
        .membership
        .as_mut()
        .ok_or("Haniel B2 membership missing")?;
    ledger.gain = gain;
    ledger.reductions.insert(target, reduction);
    ctx.state.targets[target].attack = Some(after);
    ctx.state.atk_transfers.push(GameAtkTransferReceipt {
        time_us: ctx.state.current_time_us,
        source_actor_index: owner,
        target_index: target,
        cause: Cause::NovaEnd,
        nova_generation: Some(generation),
        captured_base_atk: member.base,
        party_flat_atk_before: member.gain,
        party_flat_atk_after: gain,
        target_atk_before: before,
        target_atk_after: after,
    });
    Ok(())
}

pub(crate) fn leave(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let Some(member) = ctx.state.actors[owner]
        .character
        .haniel_mut()?
        .membership
        .clone()
    else {
        return Ok(());
    };
    let mut gain = member.gain;
    for (&target, &drain) in &member.reductions {
        let before = ctx.state.targets[target]
            .attack
            .ok_or("Haniel target ATK missing")?;
        let after = before.add(drain)?;
        let next_gain = gain.sub(drain.mul(F::new(2, 1)?)?)?;
        ctx.state.atk_transfers.push(GameAtkTransferReceipt {
            time_us: ctx.state.current_time_us,
            source_actor_index: owner,
            target_index: target,
            cause: Cause::CombatExit,
            nova_generation: None,
            captured_base_atk: member.base,
            party_flat_atk_before: gain,
            party_flat_atk_after: next_gain,
            target_atk_before: before,
            target_atk_after: after,
        });
        ctx.state.targets[target].attack = Some(after);
        gain = next_gain;
    }
    ctx.state.buffs.remove_owned_buff(BUFF, SOURCE, owner);
    ctx.state.actors[owner].character.haniel_mut()?.membership = None;
    Ok(())
}

pub(crate) fn transition(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<crate::simulation::GameActorCombatTransitionReceipt, String> {
    let leaving = ctx.request.combat_transition_events[event].leaving;
    if ctx.state.actors[owner].hp == 0
        || ctx.state.actors[owner]
            .character
            .haniel_mut()?
            .out_of_combat
            == leaving
    {
        return Err(
            "Haniel combat transition requires a living actor and alternating state".into(),
        );
    }
    if leaving {
        leave(ctx, owner)?;
        super::hootie::on_defeat(ctx, owner)?;
    } else {
        enter(ctx, owner)?;
    }
    ctx.state.actors[owner]
        .character
        .haniel_mut()?
        .out_of_combat = leaving;
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
