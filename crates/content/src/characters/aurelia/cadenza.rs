use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;
use ntesim_core::buffs::dispatch::BuffRecipient;
use std::collections::BTreeMap;

pub(crate) const A2_HYPOTHESIS_ID: HypothesisId = HypothesisId::AureliaA2CadenzaV1;
pub(crate) const R6_HYPOTHESIS_ID: HypothesisId = HypothesisId::AureliaR6PsycheV1;

pub(crate) fn handle_a2_trigger(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !ctx.config.awakening.has_effect(2) {
        return Ok(());
    }
    if !ctx
        .request
        .model_assumptions
        .contains_key(A2_HYPOTHESIS_ID.as_str())
    {
        return Ok(());
    }
    ctx.record_hypothesis(A2_HYPOTHESIS_ID)?;

    let now_us = ctx.time_us;
    let aurelia = ctx.actor.character.aurelia_mut()?;
    let previous_until = aurelia.cadenza_until_us;

    let remaining = previous_until.saturating_sub(now_us);
    let duration = remaining
        .saturating_add(PARAMETERS.a2_extension_us)
        .min(PARAMETERS.cadenza_duration_us);
    let updated_until = now_us
        .checked_add(duration)
        .ok_or("Aurelia Cadenza extension overflow")?;

    aurelia.cadenza_until_us = updated_until;

    if updated_until > previous_until {
        trigger_r6_if_eligible(ctx)?;
    }

    Ok(())
}

pub(crate) fn trigger_r6_if_eligible(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.awakening.has_resonance_6() {
        if ctx
            .request
            .model_assumptions
            .contains_key(R6_HYPOTHESIS_ID.as_str())
        {
            ctx.record_hypothesis(R6_HYPOTHESIS_ID)?;
        }
        ctx.refresh_buff(
            "aurelia.res6_psyche_damage",
            BuffRecipient::TeamAll,
            PARAMETERS.resonance_duration_us,
            BTreeMap::from([(
                "damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_PSYCHE".into(),
                PARAMETERS.resonance_psyche_damage_bonus,
            )]),
        )?;
    }
    Ok(())
}
