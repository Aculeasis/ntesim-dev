use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::HathorNormalChainV1;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::HathorSkillBurstPacketsV1;

pub(crate) const DIRECT_PACKET_INPUTS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::HathorBranchAirattackDirectPacketInputsV1;

pub(crate) const POWER_STATE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::HathorExpressDeliveryStateV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::HathorAoeAllInRangeDummyV1;

const DIRECT_PACKET_INPUTS: &[&str] = &[
    "GE_Player_Hathor_Branch_Damage",
    "GE_Player_Hathor_AirAttack_Damage",
];

pub(crate) fn is_direct_packet_input(damage_effect_id: &str) -> bool {
    DIRECT_PACKET_INPUTS.contains(&damage_effect_id)
}

fn handle_direct_packet_input(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if ctx.ability_id != "GA_Hathor_Melee"
        || !ctx
            .packet_metadata
            .is_some_and(|packet| is_direct_packet_input(&packet.damage_effect_id))
    {
        return Ok(false);
    }
    ctx.record_hypothesis(DIRECT_PACKET_INPUTS_HYPOTHESIS_ID)?;
    Ok(true)
}

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet("GA_Hathor_Melee")
    {
        return Err(
            "Hathor normal chain requires the modeled first packet as its admitted input".into(),
        );
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.hathor_mut()?;
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
            ctx.replace_native_primary_packet("GE_Player_Hathor_Melee2_Damage")?;
            for _ in 0..4 {
                ctx.schedule_native_child_packet(
                    "GE_Player_Hathor_Melee2_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
        }
        2 => {
            ctx.replace_native_primary_packet("GE_Player_Hathor_Melee3_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Hathor_Melee3_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        3 => {
            ctx.replace_native_primary_packet("GE_Player_Hathor_Melee4_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Hathor_Melee4_1_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Hathor_Melee4_2_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        4 => {
            ctx.replace_native_primary_packet("GE_Player_Hathor_Melee5_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Hathor_Melee5_1_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1025" {
        return Ok(());
    }
    if super::clock::accepted(ctx.request) {
        return super::clock::cast(ctx);
    }
    if handle_direct_packet_input(ctx)? {
        return Ok(());
    }

    let emergency_delivery_is_active = matches!(
        &ctx.actor.character,
        crate::characters::RuntimeState::Hathor(state)
            if state.emergency_delivery_until_us > ctx.time_us
    );
    if ability_id == "GA_Hathor_Skill" && emergency_delivery_is_active {
        return Err(
            "Hathor Aerial Command is unavailable during Emergency Delivery; its Cyclone Strike replacement is unsupported"
                .into(),
        );
    }

    if ability_id == "GA_Hathor_Melee" {
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.hathor_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    let children: &[&str] = match ctx.ability_id {
        "GA_Hathor_Skill" => &[
            "GE_Player_Hathor_Skill1_Damage",
            "GE_Player_Hathor_Skill11_Damage",
        ],
        "GA_Hathor_UltraSkill" => &[
            "GE_Player_Hathor_UltraSkill1_Damage",
            "GE_Player_Hathor_UltraSkill2_Damage",
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
                "Hathor packet train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
        for child in children {
            ctx.schedule_native_child_packet(child, ctx.target_index, ctx.hit_time_us)?;
        }
    }

    if ctx.request.targets.len() > 1
        && (ability_id == "GA_Hathor_Skill" || ability_id == "GA_Hathor_UltraSkill")
    {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        let extra_packets: &[&str] = match ability_id {
            "GA_Hathor_Skill" => &[
                "GE_Player_Hathor_Skill1_Damage",
                "GE_Player_Hathor_Skill1_Damage",
                "GE_Player_Hathor_Skill11_Damage",
            ],
            "GA_Hathor_UltraSkill" => &[
                "GE_Player_Hathor_UltraSkill1_Damage",
                "GE_Player_Hathor_UltraSkill1_Damage",
                "GE_Player_Hathor_UltraSkill2_Damage",
            ],
            _ => &[],
        };
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            for packet in extra_packets {
                ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
            }
        }
    }

    if ability_id == "GA_Hathor_Skill" {
        ctx.record_hypothesis(POWER_STATE_HYPOTHESIS_ID)?;
        let hathor = ctx.actor.character.hathor_mut()?;
        hathor.express_delivery_power = hathor
            .express_delivery_power
            .checked_add(2)
            .ok_or("Hathor Express Delivery Power overflow")?;
    }

    if ability_id == "GA_Hathor_UltraSkill" {
        ctx.record_hypothesis(POWER_STATE_HYPOTHESIS_ID)?;
        let hathor = ctx.actor.character.hathor_mut()?;
        hathor.emergency_delivery_until_us = ctx
            .time_us
            .checked_add(PARAMETERS.emergency_duration_us)
            .ok_or("Hathor emergency_delivery deadline overflow")?;
    }

    Ok(())
}
