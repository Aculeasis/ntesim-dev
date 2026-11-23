//! Grounded paid responses and engine-qualified Support; airborne branches are separate.
use super::clock::{Kind, Plan};
use crate::runtime_api::{CastContext, NormalActionOverride, SupportEntry};
use crate::simulation::{GameActionCommand, GameScenarioRequest, PlayerInput, state::ActorRuntime};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorSecondaryActionsV1;
pub(crate) const RIPOSTE: &str = "GE_Player_Hathor_PerfectEvadeAttack_Damage";
pub(crate) const RIPOSTE2: &str = "GE_Player_Hathor_PerfectEvadeAttack1_Damage";
pub(crate) const QTE: &str = "GE_Player_Hathor_QTE1_Damage";
pub(crate) const QTE2: &str = "GE_Player_Hathor_QTE2_Damage";
const ENTRY: SupportEntry = SupportEntry {
    ability_id: "GA_Hathor_QTE",
    damage_effect_id: QTE,
    hit_delay_us: 633_462,
    recovery_us: 1_350_067,
    dispatch_model: MODEL,
    packet_model: Some(MODEL),
    extra_damage_effects: &[],
};
pub(crate) fn accepted(r: &GameScenarioRequest) -> bool {
    r.model_assumptions.get(MODEL.as_str()).map(String::as_str) == Some(MODEL.description())
}
pub(crate) fn validate(r: &GameScenarioRequest) -> Result<(), String> {
    if accepted(r) && (!super::combat::accepted(r) || !super::cyclone::accepted(r)) {
        return Err(
            "Hathor secondary_actions_v1 requires combat_lifecycle_v1 and cyclone_v1".into(),
        );
    }
    Ok(())
}
pub(crate) fn validate_input(
    r: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if !accepted(r) {
        return Ok(None);
    }
    if !matches!(input, PlayerInput::Dash | PlayerInput::Jump) || r.combat_stamina.is_none() {
        return Err(
            "Hathor grounded controls support paid Dodge/Jump with explicit stamina".into(),
        );
    }
    Ok(Some(MODEL))
}
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character,crate::characters::RuntimeState::Hathor(s) if s.riposte_ready_us.is_some())
        .then_some(NormalActionOverride{packet:RIPOSTE,role:Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model:MODEL,timing:Some((1_183_548,341_403))})
}
pub(super) fn select(ctx: &mut CastContext<'_>) -> Result<Option<Plan>, String> {
    let packet = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();
    let ready = ctx.actor.character.hathor_mut()?.riposte_ready_us.take();
    let result = if packet == RIPOSTE {
        if ready.is_none_or(|t| t > ctx.time_us) {
            return Err("Hathor Riposte requires one ready paid response".into());
        }
        Some(Plan {
            kind: Kind::Riposte,
            hits: vec![(RIPOSTE, 341_403), (RIPOSTE2, 649_874)],
            recovery: 1_183_548,
            jump: 0,
            switch: 1_183_548,
            notifies: vec![],
        })
    } else if packet == QTE {
        if !matches!(ctx.request.actions.get(ctx.command_index),Some(GameActionCommand::SwitchActor{target_actor_index,..}) if *target_actor_index==ctx.actor_index)
        {
            return Err("Hathor Support requires an engine-qualified switch".into());
        }
        Some(Plan {
            kind: Kind::Support,
            hits: vec![(QTE, 633_462), (QTE2, 824_235)],
            recovery: 1_350_067,
            jump: 1_183_819,
            switch: 1_210_463,
            notifies: vec![],
        })
    } else if ctx.ability_id == "GA_Hathor_Melee"
        && matches!(
            ctx.request.actions.get(ctx.command_index),
            Some(GameActionCommand::SwitchActor { .. })
        )
    {
        Some(Plan {
            kind: Kind::Entry,
            hits: vec![("GE_Player_Hathor_Melee1_Damage", 216_389)],
            recovery: 1_100_818,
            jump: 316_681,
            switch: 1_100_818,
            notifies: vec![],
        })
    } else {
        None
    };
    if result.is_some() {
        ctx.record_hypothesis(MODEL)?;
    }
    Ok(result)
}
pub(crate) fn entry(
    r: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    new: usize,
    qualified_support: bool,
) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
    if !accepted(r)
        || state.actors[new].locked_until_us > state.current_time_us
        || matches!(&state.actors[new].character,crate::characters::RuntimeState::Hathor(s) if s.out_of_combat)
    {
        return Ok(None);
    }
    if qualified_support {
        return Ok(Some(crate::runtime_api::OwnerEntryAction::Support(&ENTRY)));
    }
    Ok(None)
}
pub(crate) fn parry(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<crate::simulation::GameActorParryReceipt, String> {
    let now = ctx.state.current_time_us;
    if ctx.state.active_actor_index != owner
        || ctx.state.actors[owner].hp == 0
        || !super::clock::controls::dodging(&ctx.state.actors[owner], now)
    {
        return Err(
            "Hathor explicit parry requires an active living actor during paid Dodge".into(),
        );
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .riposte_ready_us = Some(now);
    Ok(crate::simulation::GameActorParryReceipt {
        event_index: event,
        actor_index: owner,
        time_us: now,
        malice_granted: crate::simulation::ExactFraction::new(0, 1)?,
        skill_permission_until_us: None,
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Reward {
    owner: usize,
    battle: u64,
}
pub(crate) fn contact(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request)
        || ctx.applied_hp_loss == 0
        || !matches!(ctx.damage_effect_id, RIPOSTE | RIPOSTE2)
    {
        return Ok(());
    }
    let s = ctx.actor.character.hathor_mut()?;
    if ctx.time_us < s.riposte_reward_until_us {
        return Ok(());
    }
    s.riposte_reward_until_us = ctx
        .time_us
        .checked_add(5_000_000)
        .ok_or("Hathor Riposte reward deadline overflow")?;
    let reward = Reward {
        owner: ctx.actor_index,
        battle: s.battle_generation,
    };
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: ntesim_core::execution::CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::HathorRiposte(reward),
            ),
        });
    Ok(())
}
pub(crate) fn reward(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    event: Reward,
) -> Result<(), String> {
    if ctx.state.actors[event.owner]
        .character
        .hathor_mut()?
        .battle_generation
        != event.battle
    {
        return Ok(());
    }
    super::power::grant(ctx, event.owner, super::power::Cause::Riposte)
}
