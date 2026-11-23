use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::AureliaNormalChainV1;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::AureliaSkillBurstPacketsV1;
pub(crate) const DIRECT_PACKET_INPUTS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::AureliaDirectPacketInputsV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::AureliaAoeAllInRangeDummyV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1070" {
        return Ok(());
    }

    if ctx.ability_id == "GA_Mitsuki_Melee" {
        handle_melee_chain(ctx)?;
    } else {
        let state = ctx.actor.character.aurelia_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    handle_burst_children(ctx)?;
    handle_multi_target_aoe(ctx)?;
    if ctx.ability_id == "GA_Mitsuki_Skill" {
        grant_skill_cadenza(ctx)?;
        let finish_time_us = ctx
            .time_us
            .checked_add(1_128_081)
            .ok_or("skill finish time overflow")?;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: finish_time_us,
                priority: ntesim_core::execution::CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::AureliaSkillFinish {
                        actor_index: ctx.actor_index,
                    },
                ),
            });
    } else if ctx.ability_id == "GA_Mitsuki_QTE" {
        super::cadenza::handle_a2_trigger(ctx)?;
    }

    Ok(())
}

fn handle_melee_chain(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let packet_id = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str());
    if matches!(
        packet_id,
        Some("GE_Player_Mitsuki_AirAttack_Damage" | "GE_Player_Mitsuki_EvadeBranch_Damage")
    ) {
        ctx.record_hypothesis(DIRECT_PACKET_INPUTS_HYPOTHESIS_ID)?;
        return Ok(());
    }
    if packet_id != super::primary_packet(ctx.ability_id) {
        return Err(
            "Aurelia normal chain requires the modeled first packet as its admitted input".into(),
        );
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let time_us = ctx.time_us;
    let (stage, cadenza_until) = {
        let state = ctx.actor.character.aurelia_mut()?;
        state.normal_stage = crate::characters::normal::after_gap(
            state.normal_stage,
            state.normal_started_us,
            time_us,
        );
        let stage = state.normal_stage;
        state.normal_stage = (stage + 1) % 4;
        state.normal_started_us = Some(time_us);
        (stage, state.cadenza_until_us)
    };
    if stage == 2
        && ctx
            .request
            .model_assumptions
            .contains_key(HypothesisId::AureliaCadenzaPowerSelectV1.as_str())
    {
        ctx.record_hypothesis(HypothesisId::AureliaCadenzaPowerSelectV1)?;
        let p0 = if cadenza_until > time_us.saturating_add(151_628) {
            "GE_Player_Mitsuki_Melee3_Power_Damage"
        } else {
            "GE_Player_Mitsuki_Melee3_Damage"
        };
        let p1 = if cadenza_until > time_us.saturating_add(270_998) {
            "GE_Player_Mitsuki_Melee3_Power_Damage"
        } else {
            "GE_Player_Mitsuki_Melee3_Damage"
        };
        let p2 = if cadenza_until > time_us.saturating_add(421_645) {
            "GE_Player_Mitsuki_Melee3_Power_Damage"
        } else {
            "GE_Player_Mitsuki_Melee3_Damage"
        };
        ctx.replace_native_primary_packet(p0)?;
        if p0 == "GE_Player_Mitsuki_Melee3_Power_Damage" {
            ctx.scope_primary_aoe(p0)?;
        }
        if p1 == "GE_Player_Mitsuki_Melee3_Power_Damage" {
            ctx.schedule_native_aoe_packet(p1, ctx.hit_time_us)?;
        } else {
            ctx.schedule_native_child_packet(p1, ctx.target_index, ctx.hit_time_us)?;
        }
        if p2 == "GE_Player_Mitsuki_Melee3_Power_Damage" {
            ctx.schedule_native_aoe_packet(p2, ctx.hit_time_us)?;
        } else {
            ctx.schedule_native_child_packet(p2, ctx.target_index, ctx.hit_time_us)?;
        }
    } else {
        let packet = [
            "GE_Player_Mitsuki_Melee1_Damage",
            "GE_Player_Mitsuki_Melee2_Damage",
            "GE_Player_Mitsuki_Melee3_Damage",
            "GE_Player_Mitsuki_Melee4_Damage",
        ][stage];
        if stage != 0 {
            ctx.replace_native_primary_packet(packet)?;
        }
        for _ in 1..[1, 1, 3, 4][stage] {
            ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
        }
    }
    Ok(())
}

fn handle_burst_children(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let children: &[&str] = match ctx.ability_id {
        "GA_Mitsuki_UltraSkill" => &[
            "GE_Player_Mitsuki_UltraSkill_Damage",
            "GE_Player_Mitsuki_UltraSkill_Damage",
            "GE_Player_Mitsuki_UltraSkill_Damage",
            "GE_Player_Mitsuki_UltraSkill_Damage",
            "GE_Player_Mitsuki_UltraSkill_Damage2",
        ],
        _ => &[],
    };
    if !children.is_empty() {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != super::primary_packet(ctx.ability_id)
        {
            return Err(
                "Aurelia packet train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
        for child in children {
            ctx.schedule_native_child_packet(child, ctx.target_index, ctx.hit_time_us)?;
        }
    }
    Ok(())
}

fn handle_multi_target_aoe(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let ability_id = ctx.ability_id;
    if ctx.request.targets.len() > 1
        && (ability_id == "GA_Mitsuki_Skill" || ability_id == "GA_Mitsuki_UltraSkill")
    {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            let all_packets: &[&str] = match ability_id {
                "GA_Mitsuki_Skill" => &["GE_Player_Mitsuki_Skill_Damage"],
                "GA_Mitsuki_UltraSkill" => &[
                    "GE_Player_Mitsuki_UltraSkill_Damage",
                    "GE_Player_Mitsuki_UltraSkill_Damage",
                    "GE_Player_Mitsuki_UltraSkill_Damage",
                    "GE_Player_Mitsuki_UltraSkill_Damage",
                    "GE_Player_Mitsuki_UltraSkill_Damage",
                    "GE_Player_Mitsuki_UltraSkill_Damage2",
                ],
                _ => &[],
            };
            for packet in all_packets {
                ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
            }
        }
    }
    Ok(())
}

fn grant_skill_cadenza(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.actor.character.aurelia_mut()?.cadenza_until_us = ctx
        .time_us
        .checked_add(PARAMETERS.cadenza_duration_us)
        .ok_or("Aurelia Cadenza deadline overflow")?;
    super::cadenza::trigger_r6_if_eligible(ctx)
}

pub(crate) fn handle_skill_finish(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    if ctx.state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    let config = &ctx.request.actors[actor_index];
    let now_us = ctx.state.current_time_us;

    // A6: Crescendo cap 20 and +10 stacks at Skill REALFINISH
    if config.awakening.has_effect(6) {
        let character_id = config.character_id.clone();
        let state = ctx.state.actors[actor_index].character.aurelia_mut()?;
        super::crescendo::apply_a6_skill_finish(
            state,
            &mut ctx.state.buffs,
            actor_index,
            &character_id,
            now_us,
            ctx.state.mode_clock.relative_us(),
        )?;
    }

    // Shadow Creed 4pc at Skill REALFINISH
    if ctx
        .request
        .model_assumptions
        .contains_key(HypothesisId::AureliaShadowCreedSkillFinishV1.as_str())
    {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            HypothesisId::AureliaShadowCreedSkillFinishV1,
        )?;
        for proc in &config.gear_procs {
            if let crate::simulation::GearProcConfig::ShadowCreedAtk {
                atk_bonus,
                duration_us,
            } = proc
            {
                let atk_percent = crate::simulation::ExactFraction::from_ratio(*atk_bonus)?;
                crate::simulation::state::apply_timed_buff(
                    &mut ctx.state.buffs,
                    crate::simulation::state::TimedBuffDeclaration::reset(
                        "shadow_creed.atk",
                        &config.character_id,
                        ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
                        *duration_us,
                        ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                        std::collections::BTreeMap::from([("atk_percent".into(), atk_percent)]),
                    ),
                    actor_index,
                    ctx.state.mode_clock.relative_us(),
                )?;
            }
        }
    }

    Ok(())
}
