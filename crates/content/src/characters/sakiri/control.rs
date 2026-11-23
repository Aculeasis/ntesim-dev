use super::parameters::PARAMETERS;
use crate::runtime_api::ActorHitContext;
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};

pub(crate) fn preflight(ctx: &mut crate::runtime_api::CastContext<'_>) -> Result<(), String> {
    if ctx.ability_id != "GA_Sagiri_UltraSkill" {
        return Ok(());
    }
    ctx.record_hypothesis(crate::HypothesisId::SakiriSuppressV1)?;
    ctx.record_hypothesis(crate::HypothesisId::CombatControlEffectsV1)?;
    ctx.hit_time_us
        .checked_add(PARAMETERS.control_def_down_us)
        .ok_or("Sakiri control DEF deadline overflow")?;
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.ability_id != "GA_Sagiri_UltraSkill" {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        ctx.execution_assumptions,
        crate::HypothesisId::SakiriSuppressV1,
    )?;
    if !ctx
        .actor
        .character
        .sakiri_mut()?
        .suppression_inputs
        .insert((ctx.command_index, ctx.target_index))
    {
        return Ok(());
    }
    let duration_us = if ctx.config.awakening.has_effect(1) {
        PARAMETERS.a1_suppress_us
    } else {
        PARAMETERS.suppress_us
    };
    if !ctx.apply_control(crate::simulation::GameControlKind::Suppress, duration_us)? {
        return Ok(());
    }
    ctx.buffs
        .apply_buff(
            BuffDefinition {
                id: "sakiri.impish_trick_def".into(),
                source: ctx.config.character_id.clone(),
                recipient: BuffRecipient::SpecificTarget(ctx.target_index.into()),
                stacking_scope: BuffStackingScope::Shared,
                source_lifetime: Default::default(),
                semantic_tags: std::collections::BTreeSet::from([
                    "ntesim.status.defense_down".into()
                ]),
                duration_us: Some(PARAMETERS.control_def_down_us),
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: std::collections::BTreeMap::from([(
                    "def_percent_reduction".into(),
                    PARAMETERS.control_def_down,
                )]),
            },
            ctx.actor_index,
            ctx.world_time_us,
        )
        .map_err(|e| format!("Sakiri control DEF application failed: {e:?}"))?;
    Ok(())
}
