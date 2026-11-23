//! Declared flight is real time; native landing progress begins at touchdown.
use crate::runtime_api::{CastContext, PlungeInputContext};

pub(crate) const PACKET: &str = "GE_Player_Nanally_AirAttack1_Damage";
pub(crate) const CAT_PACKET: &str = "GE_Player_Nanally_Cat_AirAttack_Damage";

pub(crate) fn validate(request: &crate::simulation::GameScenarioRequest) -> Result<(), String> {
    for (command, action) in request.actions.iter().enumerate() {
        let crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            damage_effect_id,
            ..
        } = action
        else {
            continue;
        };
        if request.actors[*actor_index].character_id != "character:1010"
            || damage_effect_id != PACKET
        {
            continue;
        }
        if !request.plunge_inputs.contains_key(&command) {
            return Err(
                "Nanally plunge requires an explicit airborne landing/height profile".into(),
            );
        }
        if request.hold_inputs.contains_key(&command) {
            return Err("Nanally plunge cannot also hold".into());
        }
    }
    Ok(())
}

pub(crate) fn validate_input(ctx: &PlungeInputContext<'_>) -> Result<(), String> {
    if ctx.metadata.action.ability_id != "GA_Nanally_Melee"
        || ctx.metadata.packet.damage_effect_id != PACKET
    {
        return Err("Nanally plunge requires its native landing packet".into());
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .and_then(|time| time.checked_add(808_561))
        .ok_or("Nanally plunge deadline overflow")?;
    Ok(())
}

pub(crate) fn public_action() -> crate::build::PublicAction {
    crate::build::PublicAction {
        command: crate::simulation::GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Nanally_Melee".into(),
            damage_effect_id: PACKET.into(),
            array_index: 0,
            atk_rate: crate::formula::Ratio::one(),
            scaling_stat: Default::default(),
            energy_cost: 0,
            recovery_us: 255_141,
            cooldown_us: 0,
            hit_delay_us: 50_170,
        },
        advance_us: 255_141,
        model_assumptions: [super::clock::MODEL]
            .into_iter()
            .map(|id| (id.into(), id.description().into()))
            .collect(),
    }
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if ctx
        .packet_metadata
        .is_none_or(|packet| packet.damage_effect_id != PACKET)
    {
        return Ok(false);
    }
    let input = *ctx
        .request
        .plunge_inputs
        .get(&ctx.command_index)
        .ok_or("Nanally plunge requires an explicit airborne landing/height profile")?;
    if ctx.request.hold_inputs.contains_key(&ctx.command_index) {
        return Err("Nanally plunge cannot also hold".into());
    }
    let factor = crate::simulation::ExactFraction::from_ratio(input.height_bonus)?
        .add(crate::simulation::ExactFraction::new(1, 1)?)?;
    ctx.scale_primary_coefficient(PACKET, factor)?;
    let state = ctx.actor.character.nanally_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    super::clock::cast_after(ctx, super::clock::Kind::Plunge, input.landing_delay_us)?;
    super::underboss::schedule_plunge(ctx)?;
    Ok(true)
}
