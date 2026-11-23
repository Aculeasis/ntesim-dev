//! Explicit airborne landing, paid Dodge/Riposte and qualified engine Support.
use super::clock::Kind;
use crate::runtime_api::{
    CallbackContext, CastContext, NormalActionOverride, PlungeInputContext, SupportEntry,
};
use crate::simulation::{
    ExactFraction, GameScenarioRequest, PlayerInput,
    state::{ActorRuntime, GameSimulatorState},
};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizSecondaryActionsV1;
pub(crate) const PLUNGE: &str = "GE_Player_Chiichan073_AirAttack1_Damage";
pub(crate) const RIPOSTE: &str = "GE_Player_Chiichan073_PerfectEvadeAttack1_Damage";
const RIPOSTE2: &str = "GE_Player_Chiichan073_PerfectEvadeAttack2_Damage";
const QTE: &str = "GE_Player_Chiichan073_QTE_Damage";
const MELEE: &str = "GA_Chiichan073_Melee";
const ENTRY: SupportEntry = SupportEntry {
    ability_id: "GA_Chiichan073_QTE",
    damage_effect_id: QTE,
    hit_delay_us: 800_160,
    recovery_us: 1_736_720,
    dispatch_model: MODEL,
    packet_model: Some(MODEL),
    extra_damage_effects: &[],
};
pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if accepted(request) && !super::cancel::accepted(request) {
        return Err("Chiz secondary actions require action_cancels_v1".into());
    }
    for (index, command) in request.actions.iter().enumerate() {
        if let crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            damage_effect_id,
            ..
        } = command
            && request.actors[*actor_index].character_id == "character:1073"
            && accepted(request)
        {
            if matches!(damage_effect_id.as_str(), RIPOSTE | RIPOSTE2 | QTE) {
                return Err("Chiz secondary rows require canonical next-Normal or qualified switch authorization".into());
            }
            if damage_effect_id == PLUNGE && !request.plunge_inputs.contains_key(&index) {
                return Err("Chiz plunge requires explicit airborne landing and height".into());
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_plunge(ctx: &PlungeInputContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request)
        || ctx.metadata.action.ability_id != MELEE
        || ctx.metadata.packet.damage_effect_id != PLUNGE
    {
        return Err(
            "Chiz plunge requires secondary_actions_v1 and its native Normal-family row".into(),
        );
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .and_then(|time| time.checked_add(648_515))
        .ok_or("Chiz plunge deadline overflow")?;
    Ok(())
}
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character,crate::characters::RuntimeState::Chiz(state) if state.riposte_ready)
        .then_some(NormalActionOverride {
            packet: RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: MODEL,
            timing: Some((1_026_147, 150_950)),
        })
}
pub(crate) fn critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(MODEL)?;
    ctx.actor.character.chiz_mut()?.riposte_ready = true;
    Ok(())
}
pub(crate) fn validate_input(
    request: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if input != PlayerInput::Dash || !accepted(request) {
        return Ok(None);
    }
    if request.combat_stamina.is_none() {
        return Err("Chiz paid Dodge requires explicit combat stamina".into());
    }
    Ok(Some(MODEL))
}
pub(crate) fn on_input(
    request: &GameScenarioRequest,
    actor: &mut ActorRuntime,
    _input: PlayerInput,
) -> Result<(), String> {
    if accepted(request) {
        super::clock::inherit_pressed(actor, actor.last_action_start_us)?;
        actor.character.chiz_mut()?.riposte_ready = false;
        actor.character.chiz_mut()?.normal_stage = 0;
        actor.character.chiz_mut()?.normal_started_us = None;
    }
    Ok(())
}
pub(crate) fn committed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: PlayerInput,
) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    super::clock::stop(&mut ctx.state.actors[owner], ctx.queue, now)?;
    if input == PlayerInput::Dash {
        let back = ctx.request.dash_directions.get(&command)
            == Some(&crate::simulation::GameDashDirection::Back);
        super::clock::dodge(ctx, owner, command, back)?;
    }
    Ok(())
}
pub(crate) fn select(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    new: usize,
    qualified_support: bool,
) -> Result<Option<&'static SupportEntry>, String> {
    if !accepted(request)
        || state.actors[new].locked_until_us > state.current_time_us
        || !qualified_support
    {
        return Ok(None);
    }
    Ok(Some(&ENTRY))
}
pub(crate) fn prepare(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if accepted(ctx.request) {
        super::clock::inherit_pressed(ctx.actor, ctx.time_us)?;
        if ctx.ability_id != MELEE {
            ctx.actor.character.chiz_mut()?.riposte_ready = false;
        }
    }
    Ok(())
}
pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if !accepted(ctx.request) {
        return Ok(false);
    }
    let effect = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        .unwrap_or_default();
    let (kind, landing, hits): (Kind, u64, &[(&str, u64)]) = match effect {
        PLUNGE => (
            Kind::Plunge,
            ctx.request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Chiz airborne profile missing")?
                .landing_delay_us,
            &[(PLUNGE, 199_456)],
        ),
        RIPOSTE => {
            if replacement(ctx.actor).is_none() {
                return Err("Chiz Riposte requires paid successful Critical Dodge".into());
            }
            (Kind::Riposte, 0, &[(RIPOSTE, 150_950), (RIPOSTE2, 333_387)])
        }
        QTE => {
            if !matches!(
                ctx.request.actions.get(ctx.command_index),
                Some(crate::simulation::GameActionCommand::SwitchActor { .. })
            ) {
                return Err("Chiz Support requires actual qualified switch".into());
            }
            (Kind::Support, 0, &[(QTE, 800_160)])
        }
        _ => return Ok(false),
    };
    ctx.record_hypothesis(MODEL)?;
    super::clock::inherit_pressed(ctx.actor, ctx.time_us)?;
    if kind == Kind::Plunge {
        let height = ctx.request.plunge_inputs[&ctx.command_index].height_bonus;
        ctx.scale_primary_coefficient(
            PLUNGE,
            ExactFraction::new(1, 1)?.add(ExactFraction::from_ratio(height)?)?,
        )?;
    }
    let mut groups =
        std::collections::BTreeMap::from([(ctx.scope_primary_aoe_group(effect)?, (hits[0].1, 0))]);
    for &(row, progress) in &hits[1..] {
        groups.insert(
            ctx.schedule_native_aoe_group(row, ctx.time_us)?,
            (progress, 0),
        );
    }
    let state = ctx.actor.character.chiz_mut()?;
    state.riposte_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    super::pressed::reset(ctx.actor)?;
    super::clock::start(ctx, kind, landing, groups)?;
    if kind == Kind::Support {
        ctx.actor.switch_action_locked_until_us = ctx
            .time_us
            .checked_add(1_650_798)
            .ok_or("Chiz Support lock overflow")?;
    }
    Ok(true)
}
pub(crate) fn energy(
    ability: &str,
    packet: &str,
) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
    (ability == ENTRY.ability_id && packet == QTE).then_some(
        crate::runtime_api::DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: MODEL,
            event_kind: "chiz_support_energy",
        },
    )
}
