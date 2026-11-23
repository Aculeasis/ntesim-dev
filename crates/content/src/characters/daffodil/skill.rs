use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::DaffodilSkillBurstPacketsV1;
pub(crate) const DIRECT_PACKET_INPUTS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::DaffodilBranchEntryDirectPacketInputsV1;
pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::DaffodilNormalChainV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::DaffodilAoeAllInRangeDummyV1;

const DIRECT_PACKET_INPUTS: &[&str] = &[
    "GE_Player_Daffodill_Branch1_Damage",
    "GE_Player_Daffodill_Branch2_Damage",
    "GE_Player_Daffodill_Branch3_Damage",
    "GE_Player_Daffodill_Branch4_Damage",
    "GE_Player_Daffodill_Branch5_Damage",
    "GE_Player_Daffodill_Branch6_Damage",
    "GE_Player_Daffodill_Branch7_Damage",
    "GE_Player_Daffodill_EntryAttack1_Damage",
    "GE_Player_Daffodill_EntryAttack2_Damage",
    "GE_Player_Daffodill_EntryAttack4_Damage",
];

pub(crate) fn is_direct_packet_input(damage_effect_id: &str) -> bool {
    DIRECT_PACKET_INPUTS.contains(&damage_effect_id)
}

fn handle_direct_packet_input(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if ctx.ability_id != "GA_Daffodill_Melee"
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
        != Some("GE_Player_Daffodill_Melee1_Damage")
    {
        return Err("Daffodil Normal chain requires Melee1 as its admitted primary packet".into());
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.daffodil_mut()?;
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
            ctx.replace_native_primary_packet("GE_Player_Daffodill_Melee2_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee2_1_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        2 => {
            ctx.replace_native_primary_packet("GE_Player_Daffodill_Melee3_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee3_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee3_1_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee3_1_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee3_2_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee3_3_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        3 => {
            ctx.replace_native_primary_packet("GE_Player_Daffodill_Melee4_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee4_1_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee4_2_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        4 => {
            ctx.replace_native_primary_packet("GE_Player_Daffodill_Melee5_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee5_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee6_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee6_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee6_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee6_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Daffodill_Melee5_1_Damage",
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

    if actor_config.character_id != "character:1054" {
        return Ok(());
    }
    if super::clock::accepted(ctx.request) {
        return super::clock::cast(ctx);
    }
    if handle_direct_packet_input(ctx)? {
        return Ok(());
    }

    if ability_id == "GA_Daffodill_Melee" {
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.daffodil_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    let stacks = if ability_id == "GA_Daffodill_Skill" {
        if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str())
            != super::primary_packet(ability_id)
        {
            return Err("Daffodil Skill requires its modeled admission primary".into());
        }
        super::resonance::capture(ctx)?
    } else {
        0
    };
    let echoes = stacks == 2;
    let children: &[&str] = match ctx.ability_id {
        "GA_Daffodill_Skill" if echoes => &[
            "GE_Player_Daffodill_Skill3_Damage",
            "GE_Player_Daffodill_Skill3_Damage",
            "GE_Player_Daffodill_Skill3_Damage",
            "GE_Player_Daffodill_Skill3_Damage",
            "GE_Player_Daffodill_Skill3_Damage",
            "GE_Player_Daffodill_Skill4_Damage",
        ],
        "GA_Daffodill_Skill" => &[
            "GE_Player_Daffodill_Skill1_Damage",
            "GE_Player_Daffodill_Skill2_Damage",
            "GE_Player_Daffodill_Skill6_Damage",
            "GE_Player_Daffodill_Skill7_Damage",
            "GE_Player_Daffodill_Skill8_Damage",
            "GE_Player_Daffodill_Skill9_Damage",
            "GE_Player_Daffodill_Skill5_Damage",
        ],
        "GA_Daffodill_UltraSkill" => &[
            "GE_Player_Daffodill_UltraSkill3_Damage",
            "GE_Player_Daffodill_UltraSkill3_Damage",
            "GE_Player_Daffodill_UltraSkill2_Damage",
            "GE_Player_Daffodill_UltraSkill6_Damage",
            "GE_Player_Daffodill_UltraSkill6_Damage",
            "GE_Player_Daffodill_UltraSkill4_Damage",
            "GE_Player_Daffodill_UltraSkill4_Damage",
            "GE_Player_Daffodill_UltraSkill4_Damage",
            "GE_Player_Daffodill_UltraSkill4_Damage",
            "GE_Player_Daffodill_UltraSkill4_Damage",
            "GE_Player_Daffodill_UltraSkill4_Damage",
            "GE_Player_Daffodill_UltraSkill4_Damage",
            "GE_Player_Daffodill_UltraSkill5_Damage",
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
                "Daffodil packet train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
        if echoes {
            ctx.replace_native_primary_packet("GE_Player_Daffodill_Skill3_Damage")?;
        }
        for child in children {
            ctx.schedule_native_child_packet(child, ctx.target_index, ctx.hit_time_us)?;
        }
    }

    if ctx.request.targets.len() > 1
        && (ability_id == "GA_Daffodill_Skill" || ability_id == "GA_Daffodill_UltraSkill")
    {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        let primary = if echoes {
            "GE_Player_Daffodill_Skill3_Damage"
        } else {
            super::primary_packet(ability_id).ok_or("Daffodil primary missing")?
        };
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            ctx.schedule_native_child_packet(primary, target_index, ctx.hit_time_us)?;
            for packet in children {
                ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
            }
        }
    }

    if ability_id == "GA_Daffodill_UltraSkill" {
        super::insight::on_burst(ctx)?;
    }

    Ok(())
}
