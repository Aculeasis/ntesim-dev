//! Canonical native inputs, finite packet clocks and Authority lifecycle.
use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const SKILL_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::NanallySkillFiveHitsV1;
pub(crate) const BURST_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::NanallyBurstSevenHitsV1;
pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::NanallyNormalChainV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1010" {
        return Ok(());
    }
    if super::plunge::on_cast(ctx)? {
        return Ok(());
    }
    if ability_id != "GA_Nanally_Melee" {
        super::clock::end(ctx.actor, ctx.queue)?;
    }
    if ability_id == "GA_Nanally_QTE" {
        super::support::on_cast(ctx)?;
    }

    if ability_id == "GA_Nanally_Melee" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            == super::primary_packet(ability_id)
        {
            ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
            let state = ctx.actor.character.nanally_mut()?;
            state.normal_stage = crate::characters::normal::after_gap(
                state.normal_stage,
                state.normal_started_us,
                ctx.time_us,
            );
            let stage = state.normal_stage;
            state.normal_stage = (stage + 1) % 5;
            state.normal_started_us = Some(ctx.time_us);
            super::clock::cast(ctx, super::clock::Kind::Normal(stage))?;
            super::underboss::schedule(ctx, Some(stage))?;
        } else if ctx.packet_metadata.is_some_and(|packet| {
            packet.damage_effect_id == "GE_Player_Nanally_BranchPraojectile_Damage"
        }) {
            super::clock::cast(ctx, super::clock::Kind::Aim)?;
        }
    } else {
        let state = ctx.actor.character.nanally_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if ability_id == "GA_Nanally_Skill" {
        let state = ctx.actor.character.nanally_mut()?;
        if state.authority_until_us > ctx.time_us {
            if ctx.time_us < state.authority_recast_unlock_us {
                return Err(
                    "Nanally Skill recast during active Ichi-daime's Authority is unsupported: recast lock is active".into(),
                );
            }
            state.authority_until_us = 0;
            state.authority_command = None;
            state.authority_started_us = 0;
            state.authority_recast_unlock_us = 0;
            ctx.cancel_primary_delivery()?;
            return Ok(());
        }
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != Some("GE_Player_Nanally_Skill1_Damage")
        {
            return Err(
                "Nanally Skill train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(SKILL_PACKETS_HYPOTHESIS_ID)?;
        ctx.actor.ability_cooldowns.remove("GA_Nanally_Skill");
        super::clock::cast(ctx, super::clock::Kind::Skill)?;
        super::underboss::schedule(ctx, None)?;
    }

    if ability_id == "GA_Nanally_UltraSkill"
        && ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            == Some("GE_Player_Nanally_UltraSkill1_Damage")
    {
        ctx.record_hypothesis(BURST_PACKETS_HYPOTHESIS_ID)?;
        ctx.record_hypothesis(super::underboss::SUMMON_HYPOTHESIS_ID)?;
        let until = ctx
            .time_us
            .checked_add(super::underboss::BURST_COOLDOWN_US)
            .ok_or("Nanally Burst cooldown overflow")?;
        ctx.actor.ability_cooldowns.insert(ability_id.into(), until);
        super::clock::cast(ctx, super::clock::Kind::Burst)?;
    }
    Ok(())
}

pub(crate) fn apply_authority(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let duration_us = if ctx.config.awakening.has_effect(6) {
        if ctx.actor.character.nanally_mut()?.out_of_combat {
            ctx.record_hypothesis(super::combat::MODEL)?;
            PARAMETERS.a6_out_of_combat_authority_duration_us
        } else {
            PARAMETERS.a6_authority_duration_us
        }
    } else {
        PARAMETERS.authority_duration_us
    };
    let state = ctx.actor.character.nanally_mut()?;
    state.authority_started_us = ctx.time_us;
    state.authority_command = Some(ctx.command_index);
    state.authority_recast_unlock_us = ctx
        .time_us
        .checked_add(2_000_000)
        .ok_or("Nanally Authority recast unlock overflow")?;
    state.authority_until_us = ctx
        .time_us
        .checked_add(duration_us)
        .ok_or("Nanally Authority deadline overflow")?;
    Ok(())
}

pub(crate) fn on_field_exit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    super::clock::end(&mut ctx.state.actors[actor_index], ctx.queue)?;
    let actor = ctx
        .state
        .actors
        .get_mut(actor_index)
        .ok_or("Nanally field exit actor missing")?;
    let state = actor.character.nanally_mut()?;
    if state.authority_until_us > ctx.state.current_time_us {
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ActionAdmission".into(),
            event_kind: "nanally_authority_end".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                reason: Some("field_exit".into()),
                ..Default::default()
            }),
        });
    }
    state.authority_until_us = 0;
    state.authority_command = None;
    state.authority_started_us = 0;
    state.authority_recast_unlock_us = 0;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.a5_stacks = 0;
    state.heavy_resume = None;
    state.a5_until_us = 0;
    ctx.state
        .buffs
        .remove_owned_buff("Buff_Nanally010_Level5", "character:1010", actor_index);
    Ok(())
}
