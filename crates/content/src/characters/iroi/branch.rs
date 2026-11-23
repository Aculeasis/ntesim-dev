use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::ExactFraction;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiDaydreamBranchV1;

pub(crate) const BRANCH1: &str = "GE_Player_Oneiroi_Branch1_Damage";
pub(crate) const BRANCH2: &str = "GE_Player_Oneiroi_Branch2_Damage";
pub(crate) const BRANCH3: &str = "GE_Player_Oneiroi_Branch3_Damage";
pub(crate) const BRANCH5: &str = "GE_Player_Oneiroi_Branch5_Damage";

#[allow(dead_code)]
pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn is_branch_packet(packet: &str) -> bool {
    matches!(packet, BRANCH1 | BRANCH2 | BRANCH3 | BRANCH5)
}

pub(crate) fn handle_branch(ctx: &mut CastContext<'_>, packet: &str) -> Result<bool, String> {
    if !is_branch_packet(packet) {
        return Ok(false);
    }

    let (in_dream, has_sufficient_moon, required_moon) = {
        let state = ctx.actor.character.iroi_mut()?;
        let in_dream = state.dream_active;
        let required_moon = if packet == BRANCH5 {
            ExactFraction::new(10, 1)?
        } else {
            ExactFraction::new(20, 1)?
        };
        let has_sufficient_moon = state.moon.numerator * required_moon.denominator
            >= required_moon.numerator * state.moon.denominator;
        (in_dream, has_sufficient_moon, required_moon)
    };

    if !in_dream && !has_sufficient_moon {
        // Fallback to Melee1 (Normal1), pays 0 Moon.
        ctx.replace_native_primary_packet("GE_Player_Oneiroi_Melee1_Damage")?;
        let state = ctx.actor.character.iroi_mut()?;
        state.normal_stage = 1;
        state.normal_started_us = Some(ctx.time_us);
        return Ok(true);
    }

    ctx.record_hypothesis(HYPOTHESIS_ID)?;

    {
        let state = ctx.actor.character.iroi_mut()?;
        if !in_dream {
            state.moon = state.moon.sub(required_moon)?;
        }
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    let hit_time_us = ctx.hit_time_us;

    match packet {
        BRANCH1 => {
            if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(BRANCH1) {
                ctx.replace_native_primary_packet(BRANCH1)?;
            }
        }
        BRANCH2 => {
            if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(BRANCH1) {
                ctx.replace_native_primary_packet(BRANCH1)?;
            }
            ctx.schedule_native_child_packet(BRANCH2, ctx.target_index, hit_time_us)?;
        }
        BRANCH3 => {
            if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(BRANCH1) {
                ctx.replace_native_primary_packet(BRANCH1)?;
            }
            ctx.schedule_native_child_packet(BRANCH2, ctx.target_index, hit_time_us)?;
            ctx.schedule_native_child_packet(BRANCH3, ctx.target_index, hit_time_us)?;
        }
        BRANCH5 => {
            if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(BRANCH5) {
                ctx.replace_native_primary_packet(BRANCH5)?;
            }
            for _ in 1..35 {
                ctx.schedule_native_child_packet(BRANCH5, ctx.target_index, hit_time_us)?;
            }
        }
        _ => {}
    }

    Ok(true)
}
