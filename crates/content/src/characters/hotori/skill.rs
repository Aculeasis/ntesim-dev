use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::HotoriNormalChainV1;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::HotoriSkillBurstPacketsV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::HotoriAoeAllInRangeDummyV1;

const SKILL_HITS: &[(&str, u64)] = &[
    ("GE_Player_Jin_Skill3_Damage", 181_953),
    ("GE_Player_Jin_Skill1_Damage", 337_641),
    ("GE_Player_Jin_Skill3_Damage", 410_283),
    ("GE_Player_Jin_Skill1_Damage", 483_224),
    ("GE_Player_Jin_Skill1_Damage", 585_734),
    ("GE_Player_Jin_Skill1_Damage", 670_257),
    ("GE_Player_Jin_Skill2_Damage", 955_666),
];

const BURST_HITS: &[(&str, u64)] = &[
    ("GE_Player_Jin_UltraSkill1_Damage", 2_316_080),
    ("GE_Player_Jin_UltraSkill1_1_Damage", 2_416_466),
    ("GE_Player_Jin_UltraSkill1_2_Damage", 2_516_466),
    ("GE_Player_Jin_UltraSkill1_3_Damage", 2_616_466),
    ("GE_Player_Jin_UltraSkill1_4_Damage", 2_716_466),
    ("GE_Player_Jin_UltraSkill1_5_Damage", 2_816_667),
    ("GE_Player_Jin_UltraSkill2_Damage", 2_900_000),
    ("GE_Player_Jin_UltraSkill4_Damage", 3_927_549),
    ("GE_Player_Jin_UltraSkill3_Damage", 3_971_990),
    ("GE_Player_Jin_UltraSkill3_Damage", 4_002_235),
];

const SKILL_RECOVERY_US: u64 = 1_408_661;
const BURST_RECOVERY_US: u64 = 4_604_238;

pub(crate) fn public_skill_timing() -> (u64, u64, u64) {
    (SKILL_RECOVERY_US, 0, SKILL_HITS[0].1)
}

pub(crate) fn public_burst_timing() -> (u64, u64, u64) {
    (BURST_RECOVERY_US, 0, BURST_HITS[0].1)
}

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet("GA_Jin_Melee")
    {
        return Err(
            "Hotori normal chain requires the modeled first packet as its admitted input".into(),
        );
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.hotori_mut()?;
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
            ctx.replace_native_primary_packet("GE_Player_Jin_Melee2_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Jin_Melee3_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Jin_Melee3_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Jin_Melee9_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        2 => {
            ctx.replace_native_primary_packet("GE_Player_Jin_Melee4_Damage")?;
            for _ in 0..4 {
                ctx.schedule_native_child_packet(
                    "GE_Player_Jin_Melee4_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
        }
        3 => {
            ctx.replace_native_primary_packet("GE_Player_Jin_Melee5_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Jin_Melee5_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Jin_Melee6_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        4 => {
            ctx.replace_native_primary_packet("GE_Player_Jin_Melee7_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Jin_Melee8_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn schedule_train(
    ctx: &mut CastContext<'_>,
    hits: &[(&str, u64)],
    hypothesis: HypothesisId,
) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err(
            "Hotori packet train requires the modeled first packet as its admitted primary".into(),
        );
    }
    ctx.record_hypothesis(hypothesis)?;
    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
    }
    ctx.cancel_primary_delivery()?;
    for (effect, offset_us) in hits {
        let time_us = ctx
            .time_us
            .checked_add(*offset_us)
            .ok_or("Hotori packet delivery deadline overflow")?;
        ctx.schedule_native_aoe_group(effect, time_us)?;
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1052" {
        return Ok(());
    }

    if ability_id == "GA_Jin_Melee" {
        if ctx
            .packet_metadata
            .is_some_and(|packet| super::katana::is_katana_packet(&packet.damage_effect_id))
        {
            return super::katana::cast(ctx);
        }
        handle_melee(ctx)?;
        return Ok(());
    }

    {
        let state = ctx.actor.character.hotori_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    match ability_id {
        "GA_Jin_Skill" => {
            super::timepiece::commit_skill(ctx)?;
            schedule_train(ctx, SKILL_HITS, PACKETS_HYPOTHESIS_ID)
        }
        "GA_Jin_UltraSkill" => {
            super::timepiece::commit_burst(ctx)?;
            schedule_train(ctx, BURST_HITS, PACKETS_HYPOTHESIS_ID)
        }
        _ => Ok(()),
    }
}
