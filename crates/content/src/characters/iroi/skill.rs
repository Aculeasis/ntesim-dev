//! Preserved partial cast behavior; no additional game applicability is inferred.
use super::parameters::{HealKind, heal_ratio};
use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::CastContext;
use crate::simulation::{
    ExactFraction,
    events::{GameEventData, ScheduledGameEvent},
};
use ntesim_core::execution::CausalPriority;

pub(crate) const SKILL_TRAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiSkillNinePacketTrainV1;
pub(crate) const SKILL_HEAL_HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiLiveAtkHealingV1;
pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiNormalChainV1;
pub(crate) const BURST_HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiBurstPrimaryPacketV1;
pub(crate) const BURST_COOLDOWN_US: u64 = 16_000_000;
pub(crate) const BURST_HEAL_HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiLiveAtkHealingV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiAoeAllInRangeDummyV1;

pub(super) fn heal_coordinate(ctx: &CastContext<'_>, action_name: &str) -> Result<u32, String> {
    let array_index = ctx.native_array_index.ok_or_else(|| {
        format!("Iroi {action_name} healing requires an admitted native coordinate")
    })?;
    array_index
        .checked_add(1)
        .ok_or_else(|| format!("Iroi {action_name} healing talent coordinate overflow"))?
        .try_into()
        .map_err(|_| format!("Iroi {action_name} healing talent coordinate exceeds u32"))
}

fn schedule_team_heal(
    ctx: &mut CastContext<'_>,
    atk_ratio: ExactFraction,
    time_us: u64,
    burst: bool,
) -> Result<(), String> {
    let actor_idx = ctx.actor_index;
    let mut recipients = (0..ctx.request.actors.len()).collect::<Vec<_>>();
    recipients.sort_by_key(|recipient| (ctx.request.actors[*recipient].slot, *recipient));
    for recipient_actor_index in recipients {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        let event = if burst {
            ContentEvent::IroiBurstHeal {
                command_index: ctx.command_index,
                source_actor_index: actor_idx,
                recipient_actor_index,
                atk_ratio,
            }
        } else {
            ContentEvent::IroiSkillHeal {
                source_actor_index: actor_idx,
                recipient_actor_index,
                atk_ratio,
            }
        };
        ctx.queue.push(ScheduledGameEvent {
            time_us,
            priority: CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::Content(event),
        });
    }
    Ok(())
}

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet("GA_Oneiroi_Melee")
    {
        return Err(
            "Iroi normal chain requires the modeled first packet as its admitted input".into(),
        );
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.iroi_mut()?;
    state.normal_stage = crate::characters::normal::after_gap(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let stage = state.normal_stage;
    state.normal_stage = (stage + 1) % 5;
    state.normal_started_us = Some(ctx.time_us);
    let packet = [
        "GE_Player_Oneiroi_Melee1_Damage",
        "GE_Player_Oneiroi_Melee2_Damage",
        "GE_Player_Oneiroi_Melee3_Damage",
        "GE_Player_Oneiroi_Melee4_Damage",
        "GE_Player_Oneiroi_Melee5_Damage",
    ][stage];
    if stage != 0 {
        ctx.replace_native_primary_packet(packet)?;
    }
    for _ in 1..[1, 3, 8, 1, 3][stage] {
        ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
    }
    Ok(())
}

fn handle_skill(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err("Iroi Skill requires the modeled Gun packet as its admitted input".into());
    }
    ctx.record_hypothesis(SKILL_TRAIN_HYPOTHESIS_ID)?;
    ctx.record_hypothesis(SKILL_HEAL_HYPOTHESIS_ID)?;
    let coordinate = heal_coordinate(ctx, "Skill")?;
    let atk_ratio = heal_ratio(HealKind::Skill, coordinate)?;
    schedule_team_heal(ctx, atk_ratio, ctx.time_us, false)?;

    // Moon grant on Skill heal: 16 once per cast (if not in Dream)
    let state = ctx.actor.character.iroi_mut()?;
    if !state.dream_active {
        let base_moon = ExactFraction::new(16, 1)?;
        let gain = if ctx.config.awakening.has_effect(1) {
            base_moon.mul(ExactFraction::new(13, 10)?)?
        } else {
            base_moon
        };
        let cap = ExactFraction::new(100, 1)?;
        let new_moon = state.moon.add(gain)?;
        if new_moon.numerator > 100 * new_moon.denominator {
            let overflow = new_moon.sub(cap)?;
            state.moon_overflow = state.moon_overflow.add(overflow)?;
            state.moon = cap;
        } else {
            state.moon = new_moon;
        }
    }

    let hit_time_us = ctx.hit_time_us;
    for _ in 0..7 {
        ctx.schedule_native_child_packet(
            "GE_Player_Oneiroi_Skill_Syringe_Damage",
            ctx.target_index,
            hit_time_us,
        )?;
    }
    ctx.schedule_native_child_packet(
        "GE_Player_Oneiroi_Skill_Trumpet_Damage",
        ctx.target_index,
        hit_time_us,
    )?;
    if ctx.config.awakening.has_effect(2) {
        ctx.schedule_native_child_packet(
            "GE_Damage_sheep_gun_awaken2",
            ctx.target_index,
            hit_time_us,
        )?;
    }
    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;

    // per 1075AwakenEffect.json / GE_Damage_sheep_gun_awaken2
    if ctx.config.awakening.has_effect(2) {
        crate::runtime_api::increment_sequence(seq)?;
        queue.push(ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: CausalPriority::ProcCallback,
            seq: *seq,
            data: GameEventData::Content(ContentEvent::IroiA2Assist {
                actor_index: ctx.actor_index,
            }),
        });
    }

    let base_atk = ctx.config.static_stats.base_atk as u128;
    let uncapped_grant = ExactFraction::new(base_atk, 5)?;
    let cap = ExactFraction::new(500, 1)?;
    let grant = if uncapped_grant.to_f64() > 500.0 {
        cap
    } else {
        uncapped_grant
    };
    let def_ignore = ExactFraction::new(1, 20)?;

    // per 1075AwakenEffect.json / DT_OneiroiEffectFigure.json Calc_Oneiroi_Passive2_AtkAdd_LV4 = 0.15
    let a5_atk_add = if ctx.config.awakening.has_effect(5) {
        let a5_uncapped = ExactFraction::new(base_atk * 3, 20)?;
        Some(if a5_uncapped.to_f64() > 500.0 {
            cap
        } else {
            a5_uncapped
        })
    } else {
        None
    };

    crate::runtime_api::increment_sequence(seq)?;
    queue.push(ScheduledGameEvent {
        time_us: ctx.time_us,
        priority: CausalPriority::BuffTickOrExpiry,
        seq: *seq,
        data: GameEventData::Content(ContentEvent::IroiSkillBuffGrant {
            source_actor_index: ctx.actor_index,
            atk_add: grant,
            def_ignore,
            a5_atk_add,
            duration_us: 20_000_000,
        }),
    });
    Ok(())
}

/// Reached native Skill.1 application: live healing and one Moon grant,
/// independent of integer damage, recipient count and overheal.
pub(super) fn native_skill_heal(ctx: &mut CastContext<'_>) -> Result<ExactFraction, String> {
    ctx.record_hypothesis(SKILL_HEAL_HYPOTHESIS_ID)?;
    let coordinate = heal_coordinate(ctx, "Skill")?;
    schedule_team_heal(
        ctx,
        heal_ratio(HealKind::Skill, coordinate)?,
        ctx.time_us,
        false,
    )?;
    let state = ctx.actor.character.iroi_mut()?;
    if !state.dream_active {
        grant_moon(ctx.config, state, ExactFraction::new(16, 1)?)?;
    }
    let raw = ExactFraction::new(u128::from(ctx.config.static_stats.base_atk), 5)?;
    let cap = ExactFraction::new(500, 1)?;
    Ok(if raw.try_cmp(cap).map_err(|e| e.to_string())?.is_gt() {
        cap
    } else {
        raw
    })
}

pub(super) fn burst_entry(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.awakening.has_effect(6) {
        let queue = &mut *ctx.queue;
        let seq = &mut *ctx.sequence;
        crate::runtime_api::increment_sequence(seq)?;
        queue.push(ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: CausalPriority::ProcCallback,
            seq: *seq,
            data: GameEventData::Content(ContentEvent::IroiA6ReviveGrant {
                actor_index: ctx.actor_index,
            }),
        });
    }

    Ok(())
}

pub(super) fn native_burst_heal(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(BURST_HEAL_HYPOTHESIS_ID)?;
    let coordinate = heal_coordinate(ctx, "Burst")?;
    schedule_team_heal(
        ctx,
        heal_ratio(HealKind::Burst, coordinate)?,
        ctx.time_us,
        true,
    )
}

fn handle_ultra_skill(ctx: &mut CastContext<'_>) -> Result<(), String> {
    burst_entry(ctx)?;
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err("Iroi Burst requires its modeled direct UltraSkill packet as input".into());
    }
    ctx.record_hypothesis(BURST_HYPOTHESIS_ID)?;
    ctx.record_hypothesis(BURST_HEAL_HYPOTHESIS_ID)?;
    let coordinate = heal_coordinate(ctx, "Burst")?;
    let atk_ratio = heal_ratio(HealKind::Burst, coordinate)?;
    schedule_team_heal(ctx, atk_ratio, ctx.hit_time_us, true)?;

    if super::dream::accepted(ctx.request) {
        let is_ivory = super::dream::start_dream(ctx, coordinate)?;

        if !is_ivory {
            ctx.cancel_primary_delivery()?;
        } else if ctx.request.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            let max_targets = ctx.request.targets.len().min(20);
            for target_index in (0..ctx.targets.len())
                .filter(|&index| index != ctx.target_index)
                .take(max_targets - 1)
                .collect::<Vec<_>>()
            {
                ctx.schedule_native_child_packet(
                    "GE_Player_Oneiroi_UltraSkill_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
            }
        }
    } else if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Oneiroi_UltraSkill_Damage",
                target_index,
                ctx.hit_time_us,
            )?;
        }
    }

    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1075" {
        return Ok(());
    }

    if ability_id == "GA_Oneiroi_Melee" {
        if super::riposte::on_cast(ctx)? {
            return Ok(());
        }
        if let Some(packet) = ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) {
            if super::plunge::is_plunge_packet(packet) {
                return super::plunge::on_cast(ctx);
            }
            if super::branch::is_branch_packet(packet) {
                return super::branch::handle_branch(ctx, packet).map(|_| ());
            }
        }
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.iroi_mut()?;
        state.riposte_ready = false;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if ability_id == "GA_Oneiroi_Skill" {
        handle_skill(ctx)?;
    }

    if ability_id == "GA_Oneiroi_UltraSkill" {
        handle_ultra_skill(ctx)?;
    }

    Ok(())
}

pub(super) fn packet_moon(effect: &str) -> Result<Option<ExactFraction>, String> {
    Ok(match effect {
        "GE_Player_Oneiroi_Melee1_Damage" => Some(ExactFraction::new(2, 1)?),
        "GE_Player_Oneiroi_Melee2_Damage" => Some(ExactFraction::new(1, 1)?),
        "GE_Player_Oneiroi_Melee3_Damage" => Some(ExactFraction::new(3, 8)?),
        "GE_Player_Oneiroi_Melee4_Damage" => Some(ExactFraction::new(4, 1)?),
        "GE_Player_Oneiroi_Melee5_Damage" => Some(ExactFraction::new(267, 100)?),
        "GE_Player_Oneiroi_PerfectEvadeAttack_Damage" => Some(ExactFraction::new(4, 1)?),
        "GE_Player_Oneiroi_AirAttack_Damage" => Some(ExactFraction::new(2, 1)?),
        "GE_Player_Oneiroi_QTE_Damage" => Some(ExactFraction::new(10, 1)?),
        "GE_Player_Oneiroi_GunAI_Skill1_Damage"
        | "GE_Player_Oneiroi_SyringeAI_Skill1_Damage"
        | "GE_Player_Oneiroi_TrumpetAI_Skill1_Damage" => Some(ExactFraction::new(3, 1)?),
        "GE_Player_Oneiroi_GunAI_Skill2_Damage" => Some(ExactFraction::new(6, 5)?),
        "GE_Player_Oneiroi_SyringeAI_Skill2_Damage"
        | "GE_Player_Oneiroi_TrumpetAI_Skill2_Damage" => Some(ExactFraction::new(6, 1)?),
        "GE_Player_Oneiroi_GunAI_QTE_Guang_Damage"
        | "GE_Player_Oneiroi_GunAI_QTE_Zhou_Damage"
        | "GE_Player_Oneiroi_SyringeAI_QTE_Guang_Damage"
        | "GE_Player_Oneiroi_SyringeAI_QTE_Zhou_Damage"
        | "GE_Player_Oneiroi_TrumpetAI_QTE_Guang_Damage"
        | "GE_Player_Oneiroi_TrumpetAI_QTE_Zhou_Damage" => Some(ExactFraction::new(5, 1)?),
        _ => None,
    })
}

pub(super) fn grant_moon(
    config: &crate::simulation::GameActorConfig,
    state: &mut super::awakenings::State,
    base: ExactFraction,
) -> Result<(), String> {
    let gain = if config.awakening.has_effect(1) {
        base.mul(ExactFraction::new(13, 10)?)?
    } else {
        base
    };
    let cap = ExactFraction::new(100, 1)?;
    let new_moon = state.moon.add(gain)?;
    if new_moon.numerator > 100 * new_moon.denominator {
        let overflow = new_moon.sub(cap)?;
        state.moon_overflow = state.moon_overflow.add(overflow)?;
        state.moon = cap;
    } else {
        state.moon = new_moon;
    }
    Ok(())
}

pub(crate) fn on_hp_loss_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1075" {
        return Ok(());
    }
    let Some(packet) = ctx.packet_metadata else {
        return Ok(());
    };
    if (packet.damage_effect_id == super::clock::QTE_PACKET
        && super::clock::active_support(ctx.actor, ctx.command_index)?)
        || (super::clock::accepted(ctx.request)
            && super::clock::normal_effect(&packet.damage_effect_id))
        || ctx.target_index != ctx.request.aim_target(ctx.command_index)
        || ctx.actor.character.iroi_mut()?.dream_active
    {
        return Ok(());
    }
    if let Some(base) = packet_moon(&packet.damage_effect_id)? {
        grant_moon(ctx.config, ctx.actor.character.iroi_mut()?, base)?;
    }
    Ok(())
}
