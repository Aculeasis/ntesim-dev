//! A real Lacrimosa Evade exit admits JumpZ, without an extra attack or input.
use crate::runtime_api::{CallbackContext, CastContext, OwnerEntryAction};
use crate::simulation::{
    ExactFraction, GameCausalTrigger, GameScenarioRequest, state::GameSimulatorState,
};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId =
    crate::HypothesisId::BaicangLacrimosaEntryJumpV1;
pub(crate) const EFFECT: &str = "Buff_Cang_JumpZ";

pub(crate) fn select(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    old: usize,
    new: usize,
    qualified_support: bool,
) -> Result<Option<OwnerEntryAction>, String> {
    let now = state.current_time_us;
    if !super::etd::has_passive(&request.actors[new])
        || state.actors[new].hp == 0
        || state.actors[new].locked_until_us > now
        || !crate::characters::lacrimosa::bat_exit::is_marked(&state.actors[old], now)
    {
        return Ok(None);
    }
    if request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        != Some(HYPOTHESIS_ID.description())
    {
        return Err("UNSUPPORTED_BAICANG_LACRIMOSA_ENTRY: actual Lacrimosa Evade exit requires baicang.lacrimosa_entry_jump_v1".into());
    }
    now.checked_add(200_000)
        .ok_or("Baicang JumpZ deadline overflow")?;
    Ok(Some(OwnerEntryAction::Effect {
        source_actor_index: old,
        support: super::support::select(request, state, new, qualified_support, true)?,
        apply: admit,
    }))
}

fn admit(
    ctx: &mut CallbackContext<'_>,
    source: usize,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    let key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: owner,
            ability_id: EFFECT.into(),
            time_us: now,
            trigger: GameCausalTrigger::FieldEntry {
                command_index: command,
            },
            recipient_healing: None,
        },
    )?;
    crate::simulation::state::apply_timed_buff(
        &mut ctx.state.buffs,
        crate::simulation::state::TimedBuffDeclaration::reset(
            EFFECT,
            EFFECT,
            ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(owner),
            200_000,
            ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([(
                "jump_multiplier".into(),
                ExactFraction::new(6, 5)?,
            )]),
        ),
        owner,
        ctx.state.mode_clock.relative_us(),
    )?;
    ctx.state.actors[owner]
        .character
        .baicang_mut()?
        .entry_jump_pending = true;
    let (_, index) = ctx.state.origins.identify(ctx.request, key)?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ActionAdmission".into(),
        event_kind: "baicang_lacrimosa_entry_jump".into(),
        active_actor: owner,
        hit_index: None,
        from_actor: Some(source),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            event_index: index,
            duration_us: Some(200_000),
            exact_amount: Some(ExactFraction::new(6, 5)?),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !ctx.actor.character.baicang_mut()?.entry_jump_pending {
        return Ok(());
    }
    if ctx
        .packet_metadata
        .is_some_and(|packet| packet.damage_effect_id == super::plunge::PACKET_ID)
    {
        if !ctx.request.plunge_inputs.contains_key(&ctx.command_index) {
            return Err("UNSUPPORTED_BAICANG_ENTRY_LANDING: Lacrimosa entry plunge requires authored landing_delay_us and height_bonus".into());
        }
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        ctx.actor.character.baicang_mut()?.entry_jump_pending = false;
    } else if matches!(
        ctx.request.actions.get(ctx.command_index),
        Some(crate::simulation::GameActionCommand::InvokeAbility { .. })
    ) {
        ctx.actor.character.baicang_mut()?.entry_jump_pending = false;
    }
    Ok(())
}
