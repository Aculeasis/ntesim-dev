use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) gnaw_until_us: u64,
    pub(crate) gnaw_generation: u64,
    pub(crate) gnaw_marked: bool,
    pub(crate) gnaw_command: usize,
}

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::SkiaNormalChainV1;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::SkiaSkillBurstPacketsV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1008" {
        return Ok(());
    }
    super::entry_state::before_cast(ctx)?;
    let ability_id = ctx.ability_id;

    if ability_id == "GA_Skia_Melee" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != super::primary_packet(ability_id)
        {
            return Err(
                "Skia normal chain requires the modeled first packet as its admitted input".into(),
            );
        }
        ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
        let state = ctx.actor.character.skia_mut()?;
        state.normal_stage = crate::characters::normal::after_gap(
            state.normal_stage,
            state.normal_started_us,
            ctx.time_us,
        );
        let stage = state.normal_stage;
        state.normal_stage = (stage + 1) % 5;
        state.normal_started_us = Some(ctx.time_us);
        let packets: &[&str] = match stage {
            0 => &["GE_Player_Skia_Melee1_Damage"],
            1 => &[
                "GE_Player_Skia_Melee2_Damage",
                "GE_Player_Skia_Melee2_Damage",
                "GE_Player_Skia_Melee4_Damage",
            ],
            2 => &[
                "GE_Player_Skia_Melee5_Damage",
                "GE_Player_Skia_Melee6_Damage",
            ],
            3 => &["GE_Player_Skia_Melee7_Damage"],
            _ => &[
                "GE_Player_Skia_Melee8_Damage",
                "GE_Player_Skia_Melee9_Damage",
            ],
        };
        if stage != 0 {
            ctx.replace_native_primary_packet(packets[0])?;
        }
        for packet in &packets[1..] {
            ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
        }
    } else {
        let state = ctx.actor.character.skia_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    let children: &[&str] = match ctx.ability_id {
        "GA_Skia_Skill" => &[
            "GE_Player_Skia_Skill1_Damage",
            "GE_Player_Skia_Skill1_Damage",
            "GE_Player_Skia_Skill1_Damage",
            "GE_Player_Skia_Skill2_Damage",
            "GE_Player_Skia_Skill3_Damage",
        ],
        "GA_Skia_UltraSkill" => &[
            "GE_Player_Skia_UltraSkill2_Damage",
            "GE_Player_Skia_UltraSkill2_Damage",
            "GE_Player_Skia_UltraSkill2_Damage",
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
                "Skia packet train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
        if ability_id == "GA_Skia_UltraSkill" {
            // The spread slash reaches all living dummies before the three
            // converge bites; those bites retain their primary-only recipient.
            ctx.scope_primary_aoe("GE_Player_Skia_UltraSkill1_Damage")?;
        }
        for child in children {
            ctx.schedule_native_child_packet(child, ctx.target_index, ctx.hit_time_us)?;
        }
    }

    // per 1008AwakenEffect.json / DT_SkiaEffectFigure.json Skia_Leve2CritBase = 0.10, Skia_Leve2Duaration = 15.0
    if ability_id == "GA_Skia_Skill" && ctx.config.awakening.has_effect(2) {
        ctx.refresh_buff(
            "skia.a2_crit_rate",
            ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
            PARAMETERS.a2_duration_us,
            std::collections::BTreeMap::from([("crit_rate_bonus".into(), PARAMETERS.a2_crit_rate)]),
        )?;
    }

    super::entry_state::on_cast(ctx)
}

pub(crate) fn before_hit(ctx: &mut crate::runtime_api::HitContext<'_>) -> Result<(), String> {
    if ctx.ability_id == "GA_Skia_Skill" {
        let reductions = if ctx.config.awakening.has_resonance_6() {
            std::collections::BTreeMap::from([(
                "ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA".into(),
                PARAMETERS.resonance_resistance_reduction,
            )])
        } else {
            Default::default()
        };
        ctx.refresh_status(
            "skia.fang_thrust",
            if ctx.config.awakening.has_effect(4) {
                PARAMETERS.a4_mark_duration_us
            } else {
                PARAMETERS.mark_duration_us
            },
            reductions,
        )?;
    }
    Ok(())
}

pub(crate) fn skill_coordinate(
    config: &crate::simulation::GameActorConfig,
    ability: &str,
) -> Result<usize, String> {
    let purchased = config
        .talent_levels
        .map(|levels| levels[1])
        .unwrap_or(config.purchased_skill_level);
    let (bonus, _) = crate::characters::resonance::bonus(
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        ability,
        &config.awakening,
    )?;
    usize::try_from(
        purchased
            .checked_add(bonus)
            .and_then(|level| level.checked_sub(1))
            .ok_or("input talent coordinate overflow")?,
    )
    .map_err(|_| "input talent coordinate overflow".into())
}

pub(crate) fn schedule_shadow_atk(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    request: &crate::simulation::GameScenarioRequest,
    actor_index: usize,
    target_index: usize,
    time_us: u64,
    command_index: usize,
) -> Result<(), String> {
    let packet = shadow_packet(request, actor_index, target_index, command_index)?;
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(crate::simulation::events::ScheduledGameEvent {
        time_us,
        priority: ntesim_core::execution::CausalPriority::HitDelivery,
        seq: *sequence,
        data: crate::simulation::events::GameEventData::HitDelivery {
            command_index: packet.command_index,
            actor_index: packet.actor_index,
            target_index: packet.target_index,
            ability_id: packet.ability_id,
            damage_effect_id: packet.damage_effect_id,
            atk_rate: packet.atk_rate,
            scaling_stat: packet.scaling_stat,
            metadata: packet.metadata,
        },
    });
    Ok(())
}

fn shadow_packet(
    request: &crate::simulation::GameScenarioRequest,
    actor_index: usize,
    target_index: usize,
    command_index: usize,
) -> Result<crate::simulation::events::HitPacket, String> {
    let config = &request.actors[actor_index];
    let coordinate = skill_coordinate(config, "GA_Skia_Skill")?;
    let (atk_rate, scaling_stat) = crate::simulation::inputs::coefficient(
        request,
        config,
        "GA_Skia_Skill",
        "GE_Player_Skia_SkillShadowAtk_Damage",
        coordinate,
    )?;
    let metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        "GA_Skia_Skill",
        "GE_Player_Skia_SkillShadowAtk_Damage",
        None,
        &request.model_assumptions,
    )?;
    Ok(crate::simulation::events::HitPacket {
        command_index,
        actor_index,
        target_index,
        ability_id: "GA_Skia_Skill".into(),
        damage_effect_id: "GE_Player_Skia_SkillShadowAtk_Damage".into(),
        atk_rate,
        scaling_stat,
        metadata: Box::new(metadata),
    })
}

pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1008" {
        return Ok(());
    }
    if ctx.ability_id == "GA_Skia_Skill"
        || ctx.damage_effect_id == "GE_Player_Skia_SkillShadowAtk_Damage"
    {
        return Ok(());
    }
    if !ctx.target_alive {
        return Ok(());
    }
    let target_index = ctx.target_index;
    let has_mark_to_consume = match &ctx.actor.character {
        crate::characters::RuntimeState::Skia(state) => state.gnaw_marked,
        _ => false,
    };
    if has_mark_to_consume {
        let has_mark = crate::simulation::state::target_has_status(
            ctx.buffs,
            ctx.actor_index,
            &ctx.config.character_id,
            target_index,
            "skia.fang_thrust",
        );
        if has_mark {
            ctx.buffs.remove_target_buff(
                "skia.fang_thrust",
                &ctx.config.character_id,
                &target_index.to_string(),
            );
            if let Ok(skia_state) = ctx.actor.character.skia_mut() {
                skia_state.gnaw_marked = false;
            }
            let packet = shadow_packet(
                ctx.request,
                ctx.actor_index,
                target_index,
                ctx.command_index,
            )?;
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue
                .push(crate::simulation::events::ScheduledGameEvent {
                    time_us: ctx.time_us,
                    priority: ntesim_core::execution::CausalPriority::HitDelivery,
                    seq: *ctx.sequence,
                    data: crate::simulation::events::GameEventData::FollowupHit {
                        parent_hit_index: ctx.hit_index,
                        packet: Box::new(packet),
                    },
                });
        }
    }
    Ok(())
}
