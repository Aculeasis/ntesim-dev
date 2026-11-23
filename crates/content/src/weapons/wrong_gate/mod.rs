use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsTheWrongGateHealingV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::WrongGate { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn on_healing(
    ctx: &mut CallbackContext<'_>,
    source_actor: usize,
    recipient_actor: usize,
    rounded_pre_cap_heal: u64,
) -> Result<(), String> {
    if rounded_pre_cap_heal == 0
        || ctx
            .state
            .actors
            .get(source_actor)
            .is_none_or(|actor| actor.hp == 0)
        || ctx
            .state
            .actors
            .get(recipient_actor)
            .is_none_or(|actor| actor.hp == 0)
    {
        return Ok(());
    }
    let Some(GearProcConfig::WrongGate {
        anima_damage_bonus,
        ally_damage_bonus,
        duration_us,
        ..
    }) = ctx.request.actors[source_actor]
        .gear_procs
        .iter()
        .find(|proc| matches!(proc, GearProcConfig::WrongGate { .. }))
    else {
        return Ok(());
    };
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    let time_us = ctx.state.current_time_us;
    for owner in 0..ctx.state.actors.len() {
        ctx.state
            .buffs
            .remove_owned_buff("the_wrong_gate.anima_damage", "weapon:fork_Door", owner);
        ctx.state
            .buffs
            .remove_owned_buff("the_wrong_gate.ally_damage", "weapon:fork_Door", owner);
    }
    let anima = ExactFraction::from_ratio(*anima_damage_bonus)?;
    let ally = ExactFraction::from_ratio(*ally_damage_bonus)?;
    crate::simulation::state::apply_timed_buff(
        &mut ctx.state.buffs,
        TimedBuffDeclaration::reset(
            "the_wrong_gate.anima_damage",
            "weapon:fork_Door",
            BuffRecipient::SpecificActor(source_actor),
            *duration_us,
            BuffStackingScope::Shared,
            BTreeMap::from([(
                "damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE".into(),
                anima,
            )]),
        ),
        source_actor,
        ctx.state.mode_clock.relative_us(),
    )?;
    let mut recipients = Vec::new();
    for (index, actor) in ctx.state.actors.iter().enumerate() {
        if index == source_actor || actor.hp == 0 {
            continue;
        }
        crate::simulation::state::apply_timed_buff(
            &mut ctx.state.buffs,
            TimedBuffDeclaration::reset(
                "the_wrong_gate.ally_damage",
                "weapon:fork_Door",
                BuffRecipient::SpecificActor(index),
                *duration_us,
                BuffStackingScope::Shared,
                BTreeMap::from([("damage_bonus".into(), ally)]),
            ),
            source_actor,
            ctx.state.mode_clock.relative_us(),
        )?;
        recipients.push(index);
    }
    let expires_at_us = time_us
        .checked_add(*duration_us)
        .ok_or("The Wrong Gate deadline overflow")?;
    ctx.trace.push(GameEventReceipt {
        time_us,
        phase: "ProcCallback".into(),
        event_kind: "the_wrong_gate_triggered".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(source_actor),
        to_actor: Some(recipient_actor),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            amount: Some(rounded_pre_cap_heal),
            recipients: (!recipients.is_empty()).then_some(recipients),
            expires_at_us: Some(expires_at_us),
            ..Default::default()
        }),
    });
    Ok(())
}
