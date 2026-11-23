use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::BaicangHoldBranchesV1;

pub(crate) const BRANCH1_PACKET: &str = "GE_Player_Cang_Branch1_Damage";
pub(crate) const BRANCH1_1_PACKET: &str = "GE_Player_Cang_Branch1_1_Damage";
pub(crate) const BRANCH2_PACKET: &str = "GE_Player_Cang_Branch2_Damage";
pub(crate) const BRANCH2_1_PACKET: &str = "GE_Player_Cang_Branch2_1_Damage";

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    let packet_id = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();

    let is_branch1 = packet_id == BRANCH1_PACKET;
    let is_branch2 = packet_id == BRANCH2_PACKET;

    if !is_branch1 && !is_branch2 {
        return Ok(false);
    }

    ctx.record_hypothesis(HYPOTHESIS_ID)?;

    let explosion_packet = if is_branch1 {
        BRANCH1_1_PACKET
    } else {
        BRANCH2_1_PACKET
    };

    let hold_us = ctx
        .request
        .hold_inputs
        .get(&ctx.command_index)
        .map_or(0, |input| input.duration_us);
    super::timing::primary(
        ctx,
        hold_us
            .checked_add(if is_branch1 { 416_667 } else { 251_127 })
            .ok_or("Baicang held impact overflow")?,
        hold_us
            .checked_add(if is_branch1 && super::clock::accepted(ctx.request) {
                1_341_224
            } else if is_branch1 {
                1_481_224
            } else {
                1_329_568
            })
            .ok_or("Baicang held recovery overflow")?,
    )?;
    ctx.scope_primary_aoe(packet_id)?;

    // Schedule 3 explosions
    for offset in if is_branch1 {
        [if super::clock::accepted(ctx.request) {
            666_667
        } else {
            736_667
        }; 3]
    } else {
        [600_174, 617_079, 633_235]
    } {
        ctx.schedule_native_aoe_packet(
            explosion_packet,
            ctx.time_us
                .checked_add(hold_us)
                .and_then(|time| time.checked_add(offset))
                .ok_or("Branch explosion time overflow")?,
        )?;
    }

    // Arm derived skill stage 2 (Such Crime) for 2s
    let state = ctx.actor.character.baicang_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.riposte_ready = false;
    state.derived_skill_stage = 2;
    state.derived_skill_stage_expires_at_us = ctx.actor.locked_until_us.saturating_add(2_000_000);

    Ok(true)
}
