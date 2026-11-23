//! Ordinary resource-bearing packets retain one identity across all eligible dummies.
use crate::runtime_api::CastContext;

const NORMALS: [&[&str]; 5] = [
    &["GE_Player_Chaos_Melee1_Damage"],
    &[
        "GE_Player_Chaos_Melee2_Damage",
        "GE_Player_Chaos_Melee2_1_Damage",
    ],
    &[
        "GE_Player_Chaos_Melee3_Damage",
        "GE_Player_Chaos_Melee3_Damage",
    ],
    &[
        "GE_Player_Chaos_Melee4_Damage",
        "GE_Player_Chaos_Melee4_1_Damage",
        "GE_Player_Chaos_Melee4_1_Damage",
        "GE_Player_Chaos_Melee4_2_Damage",
    ],
    &[
        "GE_Player_Chaos_Melee5_Damage",
        "GE_Player_Chaos_Melee5_Damage",
        "GE_Player_Chaos_Melee5_1_Damage",
    ],
];
const SKILL: &[&str] = &[
    "GE_Player_Chaos_Skill1_Damage",
    "GE_Player_Chaos_Skill2_Damage",
];
const TRANSFORMED: [&[&str]; 5] = [
    &["GE_Player_Chaos_Melee1_Damage_Ultra"],
    &[
        "GE_Player_Chaos_Melee2_Damage_Ultra",
        "GE_Player_Chaos_Melee2_1_Damage_Ultra",
    ],
    &[
        "GE_Player_Chaos_Melee3_Damage_Ultra",
        "GE_Player_Chaos_Melee3_Damage_Ultra",
    ],
    &[
        "GE_Player_Chaos_Melee4_Damage_Ultra",
        "GE_Player_Chaos_Melee4_1_Damage_Ultra",
        "GE_Player_Chaos_Melee4_1_Damage_Ultra",
        "GE_Player_Chaos_Melee4_2_Damage_Ultra",
    ],
    &[
        "GE_Player_Chaos_Melee5_Damage_Ultra",
        "GE_Player_Chaos_Melee5_Damage_Ultra",
        "GE_Player_Chaos_Melee5_1_Damage_Ultra",
    ],
];
const BURST: &[&str] = &[
    "GE_Player_Chaos_UltraSkill1_Damage",
    "GE_Player_Chaos_UltraSkill1_Damage",
    "GE_Player_Chaos_UltraSkill1_Damage",
    "GE_Player_Chaos_UltraSkill1_Damage",
    "GE_Player_Chaos_UltraSkill2_Damage",
];

pub(super) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if !matches!(
        ctx.ability_id,
        "GA_Chaos071_Melee" | "GA_Chaos071_Skill" | "GA_Chaos071_UltraSkill"
    ) {
        return Ok(false);
    }
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err("Chaos ordinary Crime model requires the canonical first primary input; alternate actions remain separate bindings".into());
    }
    ctx.record_hypothesis(super::resources::MODEL)?;
    let normal = ctx.ability_id == "GA_Chaos071_Melee";
    let transformed = super::form::active(ctx.actor);
    let state = ctx.actor.character.chaos_mut()?;
    let (packets, model) = if normal {
        if state.entry_normal_ready {
            state.normal_stage = 0;
        }
        state.normal_stage = crate::characters::normal::after_gap(
            state.normal_stage,
            state.normal_started_us,
            ctx.time_us,
        );
        let stage = state.normal_stage;
        state.normal_stage = (stage + 1) % 5;
        state.normal_started_us = Some(ctx.actor.locked_until_us);
        (
            if transformed {
                TRANSFORMED[stage]
            } else {
                NORMALS[stage]
            },
            super::skill::CHAIN_HYPOTHESIS_ID,
        )
    } else {
        state.entry_normal_ready = false;
        state.normal_stage = 0;
        state.normal_started_us = None;
        if ctx.ability_id == "GA_Chaos071_Skill" {
            (SKILL, super::skill::SKILL_PACKETS_HYPOTHESIS_ID)
        } else {
            (BURST, super::skill::BURST_PACKETS_HYPOTHESIS_ID)
        }
    };
    ctx.record_hypothesis(model)?;
    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(super::skill::ALL_IN_RANGE_HYPOTHESIS_ID)?;
    }
    if Some(packets[0]) != super::primary_packet(ctx.ability_id) {
        ctx.replace_native_primary_packet(packets[0])?;
    }
    ctx.scope_primary_aoe(packets[0])?;
    for packet in &packets[1..] {
        ctx.schedule_native_aoe_packet(packet, ctx.hit_time_us)?;
    }
    Ok(true)
}
