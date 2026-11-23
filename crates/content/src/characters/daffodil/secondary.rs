//! Authored airborne/critical assumptions and one engine-qualified Support.
use super::clock::{Kind, profile::Hit};
use crate::runtime_api::{
    CallbackContext, CastContext, NormalActionOverride, PlungeInputContext, SupportEntry,
};
use crate::simulation::{
    ExactFraction, GameActionCommand, GameScenarioRequest, PlayerInput,
    state::{ActorRuntime, GameSimulatorState},
};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::DaffodilSecondaryActionsV1;
pub(crate) const PLUNGE: &str = "GE_Player_Daffodill_AirAttack_Damage";
pub(crate) const RIPOSTE: &str = "GE_Player_Daffodill_PerfectEvadeAttack_Damage";
pub(crate) const QTE: &str = "GE_Player_Daffodill_QTE_Damage";
const MELEE: &str = "GA_Daffodill_Melee";
const ENTRY: SupportEntry = SupportEntry {
    ability_id: "GA_Daffodill_QTE",
    damage_effect_id: QTE,
    hit_delay_us: 745_033,
    recovery_us: 900_691,
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
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(req) {
        return Ok(());
    }
    if !super::clock::cancel::accepted(req) {
        return Err("Daffodil secondary actions require action_cancels_v1".into());
    }
    for (index, action) in req.actions.iter().enumerate() {
        if let GameActionCommand::InvokeAbility {
            actor_index,
            damage_effect_id,
            ability_id,
            ..
        } = action
            && req.actors[*actor_index].character_id == "character:1054"
        {
            if matches!(damage_effect_id.as_str(), RIPOSTE | QTE) || ability_id == ENTRY.ability_id
            {
                return Err("Daffodil Riposte/Support require canonical Normal or engine-qualified switching".into());
            }
            if damage_effect_id == PLUNGE && !req.plunge_inputs.contains_key(&index) {
                return Err("Daffodil plunge requires declared landing and height".into());
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
            "Daffodil plunge requires secondary_actions_v1 and its native Normal row".into(),
        );
    }
    if ctx.input.height_bonus.numerator > ctx.input.height_bonus.denominator {
        return Err("Daffodil plunge height must be in0..1".into());
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .and_then(|t| t.checked_add(320_849))
        .ok_or("Daffodil plunge deadline overflow")?;
    Ok(())
}
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character,crate::characters::RuntimeState::Daffodil(s) if s.riposte_ready)
        .then_some(NormalActionOverride {
            packet: RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: MODEL,
            timing: Some((874_201, 229_761)),
        })
}
pub(crate) fn critical(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(MODEL)?;
    ctx.actor.character.daffodil_mut()?.riposte_ready = true;
    Ok(())
}
pub(crate) fn validate_input(
    req: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if input != PlayerInput::Dash || !accepted(req) {
        return Ok(None);
    }
    if req.combat_stamina.is_none() {
        return Err("Daffodil paid Dodge requires explicit combat stamina".into());
    }
    Ok(Some(MODEL))
}
pub(crate) fn committed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: PlayerInput,
) -> Result<(), String> {
    if accepted(ctx.request) && input == PlayerInput::Dash {
        let back = ctx.request.dash_directions.get(&command)
            == Some(&crate::simulation::GameDashDirection::Back);
        super::clock::dodge(ctx, owner, command, back)?;
    }
    Ok(())
}
pub(crate) fn select(
    req: &GameScenarioRequest,
    state: &GameSimulatorState,
    new: usize,
    qualified_support: bool,
) -> Result<Option<&'static SupportEntry>, String> {
    if !accepted(req)
        || state.actors[new].locked_until_us > state.current_time_us
        || !qualified_support
    {
        return Ok(None);
    }
    Ok(Some(&ENTRY))
}
pub(crate) fn select_cast(
    ctx: &mut CastContext<'_>,
) -> Result<Option<(Kind, &'static [Hit], u64)>, String> {
    if !accepted(ctx.request) {
        return Ok(None);
    }
    let effect = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();
    let selected = match effect {
        PLUNGE => {
            let input = ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Daffodil plunge profile missing")?;
            ctx.scale_primary_coefficient(
                PLUNGE,
                ExactFraction::new(1, 1)?.add(ExactFraction::from_ratio(input.height_bonus)?)?,
            )?;
            (
                Kind::Plunge,
                &[(PLUNGE, 53_784)][..],
                input.landing_delay_us,
            )
        }
        RIPOSTE => {
            if replacement(ctx.actor).is_none() {
                return Err("Daffodil Close Call requires paid successful Critical Dodge".into());
            }
            (Kind::Riposte, &[(RIPOSTE, 229_761)][..], 0)
        }
        QTE => {
            if !matches!(ctx.request.actions.get(ctx.command_index),Some(GameActionCommand::SwitchActor {target_actor_index,..}) if *target_actor_index==ctx.actor_index)
            {
                return Err("Daffodil Support is engine-owned qualified entry".into());
            }
            (Kind::Support, &[(QTE, 745_033)][..], 0)
        }
        _ => {
            ctx.actor.character.daffodil_mut()?.riposte_ready = false;
            return Ok(None);
        }
    };
    ctx.record_hypothesis(MODEL)?;
    ctx.actor.character.daffodil_mut()?.riposte_ready = false;
    Ok(Some(selected))
}
pub(crate) fn energy(
    ability: &str,
    packet: &str,
) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
    (ability == ENTRY.ability_id && packet == QTE).then_some(
        crate::runtime_api::DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: MODEL,
            event_kind: "daffodil_support_energy",
        },
    )
}
