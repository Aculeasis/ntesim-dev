//! Paid controls, explicit landing and engine-owned Support/entry selection.
use crate::runtime_api::{CastContext, NormalActionOverride, SupportEntry};
use crate::simulation::{GameActionCommand, GameScenarioRequest, PlayerInput, state::ActorRuntime};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HanielFullKitV1;
pub(crate) const PLUNGE: &str = "GE_Player_Haniel_Airattack_Damage";
pub(crate) const RIPOSTE: &str = "GE_Player_Haniel020_PerfectEvadeAttack_Damage";
pub(crate) const QTE: &str = "GE_Player_Haniel_QTE_Damage";
pub(crate) const ENTRY: SupportEntry = SupportEntry {
    ability_id: "GA_Haniel_QTE",
    damage_effect_id: QTE,
    hit_delay_us: 712_638,
    recovery_us: 1_177_904,
    dispatch_model: MODEL,
    packet_model: Some(MODEL),
    extra_damage_effects: &[],
};
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate_input(
    req: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if input == PlayerInput::Jump && !accepted(req) {
        return Err("Haniel Jump requires full_kit_v1".into());
    }
    if accepted(req) && matches!(input, PlayerInput::Dash | PlayerInput::Jump) {
        if req.combat_stamina.is_none() {
            return Err("Haniel Dodge/Jump requires an explicit stamina profile".into());
        }
        return Ok(Some(MODEL));
    }
    Ok(None)
}
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character, crate::characters::RuntimeState::Haniel(s) if s.riposte_ready_us.is_some())
        .then_some(NormalActionOverride { packet: RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: MODEL, timing: Some((957_007,400_000)) })
}
pub(super) fn select(ctx: &mut CastContext<'_>) -> Result<Option<super::clock::Plan>, String> {
    let effect = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();
    let armed = ctx.actor.character.haniel_mut()?.riposte_ready_us.take();
    let plan = match effect {
        PLUNGE => {
            let input = ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Haniel plunge requires landing/height")?;
            ctx.scale_primary_coefficient(
                PLUNGE,
                crate::simulation::ExactFraction::new(1, 1)?.add(
                    crate::simulation::ExactFraction::from_ratio(input.height_bonus)?,
                )?,
            )?;
            Some((
                super::clock::Kind::Plunge,
                vec![(PLUNGE, 65_670)],
                175_616,
                input.landing_delay_us,
            ))
        }
        RIPOSTE => {
            if armed.is_none_or(|ready| ready > ctx.time_us) {
                return Err("Haniel Riposte requires one ready paid Dodge/parry response".into());
            }
            if ctx.request.hold_inputs.contains_key(&ctx.command_index) {
                return Err("Haniel Riposte Cannon bypasses held input".into());
            }
            Some((
                super::clock::Kind::Riposte,
                vec![(RIPOSTE, 400_000)],
                957_007,
                0,
            ))
        }
        QTE => {
            if !matches!(ctx.request.actions.get(ctx.command_index),Some(GameActionCommand::SwitchActor{target_actor_index,..}) if *target_actor_index==ctx.actor_index)
            {
                return Err("Haniel Support requires engine-qualified switch entry".into());
            }
            ctx.actor.switch_action_locked_until_us = ctx
                .time_us
                .checked_add(1_112_165)
                .ok_or("Haniel Support switch deadline overflow")?;
            Some((
                super::clock::Kind::Support,
                vec![(QTE, 712_638)],
                1_177_904,
                0,
            ))
        }
        _ => None,
    };
    Ok(plan)
}
pub(crate) fn entry(
    req: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    new: usize,
    qualified_support: bool,
) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
    if !accepted(req)
        || state.actors[new].locked_until_us > state.current_time_us
        || matches!(&state.actors[new].character,crate::characters::RuntimeState::Haniel(s) if s.out_of_combat)
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
    event_index: usize,
) -> Result<crate::simulation::GameActorParryReceipt, String> {
    let now = ctx.state.current_time_us;
    if ctx.state.active_actor_index != owner
        || ctx.state.actors[owner].hp == 0
        || !super::clock::dodging(&ctx.state.actors[owner], now)
    {
        return Err(
            "Haniel explicit parry requires a living active actor during a paid Dodge".into(),
        );
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    ctx.state.actors[owner]
        .character
        .haniel_mut()?
        .riposte_ready_us = Some(
        now.checked_add(20_000)
            .ok_or("Haniel parry request overflow")?,
    );
    Ok(crate::simulation::GameActorParryReceipt {
        event_index,
        actor_index: owner,
        time_us: now,
        malice_granted: crate::simulation::ExactFraction::new(0, 1)?,
        skill_permission_until_us: None,
    })
}
