use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::ChizNormalChainV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::ChizAoeAllInRangeDummyV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1073" {
        return Ok(());
    }
    super::secondary::prepare(ctx)?;
    if super::barren::on_cast(ctx)? {
        return Ok(());
    }
    if super::secondary::on_cast(ctx)? {
        return Ok(());
    }

    if super::pressed::accepted(ctx.request) {
        if ability_id == "GA_Chiichan073_Skill" {
            return super::pressed::on_cast(ctx);
        }
        super::pressed::reset(ctx.actor)?;
    }

    if super::clock::accepted(ctx.request) {
        if ability_id == "GA_Chiichan073_Melee" {
            return super::clock::on_cast(ctx);
        }
        super::clock::stop(ctx.actor, ctx.queue, ctx.time_us)?;
    }

    if ability_id == "GA_Chiichan073_Melee" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            == super::primary_packet(ability_id)
        {
            ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
            let state = ctx.actor.character.chiz_mut()?;
            state.normal_stage = crate::characters::normal::after_gap(
                state.normal_stage,
                state.normal_started_us,
                ctx.time_us,
            );
            let stage = state.normal_stage;
            state.normal_stage = (stage + 1) % 5;
            state.normal_started_us = Some(ctx.time_us);
            match stage {
                0 => {}
                1 => {
                    ctx.replace_native_primary_packet("GE_Player_Chiichan073_Melee2_Damage")?;
                    for _ in 0..2 {
                        ctx.schedule_native_child_packet(
                            "GE_Player_Chiichan073_Melee2_Damage",
                            ctx.target_index,
                            ctx.hit_time_us,
                        )?;
                    }
                }
                2 => {
                    ctx.replace_native_primary_packet("GE_Player_Chiichan073_Melee3_Damage")?;
                }
                3 => {
                    ctx.replace_native_primary_packet("GE_Player_Chiichan073_Melee4_Damage")?;
                    for _ in 0..4 {
                        ctx.schedule_native_child_packet(
                            "GE_Player_Chiichan073_Melee4_Damage",
                            ctx.target_index,
                            ctx.hit_time_us,
                        )?;
                    }
                }
                4 => {
                    ctx.replace_native_primary_packet("GE_Player_Chiichan073_Melee5_Damage")?;
                    ctx.schedule_native_child_packet(
                        "GE_Player_Chiichan073_Melee5_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                _ => unreachable!(),
            }
        }
        ctx.record_hypothesis(HypothesisId::ChizNormalGrainV1)?;
    } else {
        let state = ctx.actor.character.chiz_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if ctx.request.targets.len() > 1 && ability_id == "GA_Chiichan073_Skill" {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Chiichan073_Skill1_1Damage",
                target_index,
                ctx.hit_time_us,
            )?;
        }
    }

    // per DT_ChiichanEffectFigure.json UltraSkill_Dur_Show = 7.0
    if ability_id == "GA_Chiichan073_UltraSkill" {
        return super::market::on_cast(ctx);
    }

    Ok(())
}

pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    super::pressed::on_hit(ctx)?;
    super::clock::on_hit(ctx)?;
    if ctx.applied_hp_loss > 0 {
        super::resource::grant_normal(
            ctx.config,
            ctx.actor,
            ctx.damage_effect_id,
            ctx.command_index,
        )?;
        super::finance::after_hit(ctx)?;
    }
    if ctx.config.awakening.has_resonance_6() && !super::pressed::accepted(ctx.request) {
        let reductions = std::collections::BTreeMap::from([(
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS".into(),
            super::parameters::PARAMETERS.homecoming_resistance_reduction,
        )]);
        // The serialized resonance has no expiry: this battle-scoped hypothesis stays visible.
        ctx.refresh_status("chiz.homecoming", u64::MAX - ctx.time_us, reductions)?;
    }
    Ok(())
}

pub(crate) fn before_hit(ctx: &mut crate::runtime_api::HitContext<'_>) -> Result<(), String> {
    if super::pressed::accepted(ctx.request) && ctx.config.awakening.has_resonance_6() {
        ctx.refresh_status(
            "chiz.homecoming",
            super::parameters::PARAMETERS.homecoming_duration_us,
            std::collections::BTreeMap::from([(
                "ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS".into(),
                super::parameters::PARAMETERS.homecoming_resistance_reduction,
            )]),
        )?;
    }
    Ok(())
}
