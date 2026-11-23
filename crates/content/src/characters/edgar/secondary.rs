//! Authored airborne/critical inputs and engine-qualified Support entry.
use super::clock::{self, Kind, profile::Hit};
use crate::runtime_api::{CastContext, NormalActionOverride, SupportEntry};
use crate::simulation::{GameActionCommand, GameScenarioRequest, state::ActorRuntime};
pub(crate) const PLUNGE: &str = "GE_Player_Edgar_AirAttack_Damage";
pub(crate) const RIPOSTE: &str = "GE_Player_Edgar_PerfectEvadeAttack_Damage";
pub(crate) const QTE: &str = "GE_Player_Edgar_QTE_Damage";
pub(crate) const ENTRY: SupportEntry = SupportEntry {
    ability_id: "GA_Edgar_QTE",
    damage_effect_id: QTE,
    hit_delay_us: 716_894,
    recovery_us: 1_393_653,
    dispatch_model: clock::MODEL,
    packet_model: Some(clock::MODEL),
    extra_damage_effects: &[],
};
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if !clock::accepted(req) {
        return Ok(());
    }
    for (index, action) in req.actions.iter().enumerate() {
        if let GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            energy_cost,
            ..
        } = action
            && req.actors[*actor_index].character_id == "character:1021"
        {
            if ability_id == ENTRY.ability_id || damage_effect_id == RIPOSTE {
                return Err(
                    "Edgar Support/Riposte require engine switching/canonical Normal".into(),
                );
            }
            if damage_effect_id == PLUNGE && !req.plunge_inputs.contains_key(&index) {
                return Err("Edgar plunge requires explicit landing/height".into());
            }
            if ability_id == clock::BURST && *energy_cost != 100 {
                return Err("Edgar Burst requires100 paid energy".into());
            }
            if ability_id != clock::BURST && *energy_cost != 0 {
                return Err("Edgar ordinary actions have no Ultimate energy cost".into());
            }
        }
    }
    Ok(())
}
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character,crate::characters::RuntimeState::Edgar(s) if s.riposte_ready)
        .then_some(NormalActionOverride {
            packet: RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: clock::MODEL,
            timing: Some((901_881, 517_731)),
        })
}
pub(super) fn select_cast(
    ctx: &mut CastContext<'_>,
) -> Result<Option<(Kind, Vec<Hit>, u64)>, String> {
    let effect = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();
    let plan = match effect {
        PLUNGE => {
            let input = ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Edgar plunge profile missing")?;
            ctx.scale_primary_coefficient(
                PLUNGE,
                crate::simulation::ExactFraction::new(1, 1)?.add(
                    crate::simulation::ExactFraction::from_ratio(input.height_bonus)?,
                )?,
            )?;
            Some((Kind::Plunge, vec![(PLUNGE, 49_876)], input.landing_delay_us))
        }
        RIPOSTE => {
            if !ctx.actor.character.edgar_mut()?.riposte_ready {
                return Err("Edgar Riposte requires successful paid Critical Dodge".into());
            }
            Some((Kind::Riposte, vec![(RIPOSTE, 517_731)], 0))
        }
        QTE => {
            if !matches!(ctx.request.actions.get(ctx.command_index),Some(GameActionCommand::SwitchActor{target_actor_index,..}) if *target_actor_index==ctx.actor_index)
            {
                return Err("Edgar Support requires engine-qualified entry".into());
            }
            Some((Kind::Support, vec![(QTE, 716_894)], 0))
        }
        _ => None,
    };
    ctx.actor.character.edgar_mut()?.riposte_ready = false;
    Ok(plan)
}
