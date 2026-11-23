//! Explicit paid secondary inputs and qualified Support; no inferred combat success.
use super::clock::profile::Plan;
use crate::runtime_api::{CastContext, NormalActionOverride, SupportEntry};
use crate::simulation::{GameActionCommand, GameScenarioRequest, PlayerInput, state::ActorRuntime};
pub(crate) const PLUNGE: &str = "GE_Player_Fadia_AirAttack_Damage";
pub(crate) const RIPOSTE: &str = "GE_Player_Fadia_PerfectEvadeAttack_Damage";
pub(crate) const QTE: &str = "GE_Player_Fadia_QTE1_Damage";
pub(crate) const ENTRY: SupportEntry = SupportEntry {
    ability_id: "GA_Fadia_QTE",
    damage_effect_id: QTE,
    hit_delay_us: 685983,
    recovery_us: 1476672,
    dispatch_model: super::effects::MODEL,
    packet_model: Some(super::effects::MODEL),
    extra_damage_effects: &[],
};
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    for (index, command) in req.actions.iter().enumerate() {
        if let GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            ..
        } = command
            && req.actors[*actor_index].character_id == "character:1039"
        {
            if ability_id == ENTRY.ability_id || damage_effect_id == RIPOSTE {
                return Err(
                    "Fadia Support/Riposte require qualified switch/canonical Normal".into(),
                );
            }
            if damage_effect_id == PLUNGE && !req.plunge_inputs.contains_key(&index) {
                return Err("Fadia plunge requires explicit landing and height".into());
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_input(
    req: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if super::effects::accepted(req) && matches!(input, PlayerInput::Dash | PlayerInput::Jump) {
        if req.combat_stamina.is_none() {
            return Err("Fadia Dodge/Jump requires an explicit stamina profile".into());
        }
        return Ok(Some(super::effects::MODEL));
    }
    Ok(None)
}
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character,crate::characters::RuntimeState::Fadia(s) if s.full_kit && s.riposte_ready).then_some(NormalActionOverride {
        packet:RIPOSTE,role:Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
        model:super::effects::MODEL,timing:Some((874201,200709)),
    })
}
pub(super) fn select(ctx: &mut CastContext<'_>) -> Result<Option<Plan>, String> {
    let packet = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();
    let state = ctx.actor.character.fadia_mut()?;
    let armed = state.riposte_ready;
    state.riposte_ready = false;
    let in_form = state.lilith_until_us.is_some_and(|end| ctx.time_us < end);
    let a5 = in_form && ctx.config.awakening.has_effect(5);
    let (hits, recovery, jump, landing) = match packet {
        PLUNGE => {
            let input = ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Fadia plunge profile missing")?;
            ctx.scale_primary_coefficient(
                PLUNGE,
                crate::simulation::ExactFraction::new(1, 1)?.add(
                    crate::simulation::ExactFraction::from_ratio(input.height_bonus)?,
                )?,
            )?;
            (
                &[(PLUNGE, 32851)][..],
                195920,
                115245,
                input.landing_delay_us,
            )
        }
        RIPOSTE => {
            if !armed {
                return Err("Fadia Riposte requires one successful paid Critical Dodge".into());
            }
            (&[(RIPOSTE, 200709)][..], 874201, 0, 0)
        }
        QTE => {
            if !matches!(ctx.request.actions.get(ctx.command_index),Some(GameActionCommand::SwitchActor {target_actor_index,..}) if *target_actor_index==ctx.actor_index)
            {
                return Err("Fadia Support requires qualified engine entry".into());
            }
            (&[(QTE, 685983)][..], 1476672, 1476672, 0)
        }
        _ => return Ok(None),
    };
    let state = ctx.actor.character.fadia_mut()?;
    state.normal_stage = 0;
    if packet != RIPOSTE {
        state.lilith_until_us = None;
        state.high_q = false;
    }
    Ok(Some(Plan {
        hits,
        recovery,
        skill: false,
        enhanced: false,
        a5: a5 && packet == RIPOSTE,
        jump,
        skill_gate: recovery,
        landing,
    }))
}
