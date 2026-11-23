use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::BaicangPlungeV1;
pub(crate) const PACKET_ID: &str = "GE_Player_Cang_AirAttack_Damage";
pub(crate) const HEIGHT_ID: HypothesisId = HypothesisId::BaicangHeightAwarePlungeV1;

pub(crate) fn validate_input(
    ctx: &crate::runtime_api::PlungeInputContext<'_>,
) -> Result<(), String> {
    if ctx.metadata.action.ability_id != "GA_Cang_Melee"
        || ctx.metadata.packet.damage_effect_id != PACKET_ID
    {
        return Err("no supported owner plunge binding".into());
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .and_then(|time| time.checked_add(198_548))
        .ok_or("Baicang plunge deadline overflow")?;
    Ok(())
}

pub(crate) fn public_action() -> crate::build::PublicAction {
    crate::build::PublicAction {
        command: crate::simulation::GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Cang_Melee".into(),
            damage_effect_id: PACKET_ID.into(),
            array_index: 0,
            atk_rate: ntesim_core::formula::Ratio::one(),
            scaling_stat: Default::default(),
            energy_cost: 0,
            recovery_us: 198_548,
            cooldown_us: 0,
            hit_delay_us: 49_876,
        },
        advance_us: 198_548,
        model_assumptions: [HEIGHT_ID]
            .into_iter()
            .map(|id| (id.as_str().into(), id.description().into()))
            .collect(),
    }
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    let packet_id = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();

    if packet_id != PACKET_ID {
        return Ok(false);
    }

    let landing = if let Some(input) = ctx.request.plunge_inputs.get(&ctx.command_index) {
        ctx.record_hypothesis(HEIGHT_ID)?;
        let factor = crate::simulation::ExactFraction::from_ratio(input.height_bonus)?
            .add(crate::simulation::ExactFraction::new(1, 1)?)?;
        ctx.scale_primary_coefficient(PACKET_ID, factor)?;
        input.landing_delay_us
    } else {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        0
    };
    super::timing::primary(
        ctx,
        landing
            .checked_add(49_876)
            .ok_or("plunge impact overflow")?,
        landing
            .checked_add(198_548)
            .ok_or("plunge recovery overflow")?,
    )?;
    ctx.scope_primary_aoe(PACKET_ID)?;

    let state = ctx.actor.character.baicang_mut()?;
    state.riposte_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;

    Ok(true)
}
