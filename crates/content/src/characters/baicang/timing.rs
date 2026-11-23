//! Selected rate-one coordinates and local hitstop from the owner handoff.
use crate::runtime_api::CastContext;
use crate::simulation::events::GameEventData;

pub(crate) fn primary(ctx: &mut CastContext<'_>, delay: u64, recovery: u64) -> Result<(), String> {
    let deadline = ctx
        .time_us
        .checked_add(delay)
        .ok_or("Baicang impact overflow")?;
    let mut queue = ctx.queue.drain().collect::<Vec<_>>();
    let event = queue
        .iter_mut()
        .find(|event| {
            matches!(&event.data,
        GameEventData::HitDelivery { command_index, actor_index, target_index, .. }
        if *command_index == ctx.command_index && *actor_index == ctx.actor_index && *target_index == ctx.target_index)
        })
        .ok_or("Baicang primary delivery missing")?;
    event.time_us = deadline;
    ctx.queue.replace(queue);
    ctx.hit_time_us = deadline;
    ctx.actor.locked_until_us = ctx
        .time_us
        .checked_add(recovery)
        .ok_or("Baicang recovery overflow")?;
    Ok(())
}

pub(crate) const NORMAL_RECOVERY: [u64; 5] = [923724, 840277, 1218687, 1706341, 820279];
pub(crate) const NORMAL_IMPACT: [u64; 5] = [184377, 199754, 166255, 133698, 364066];

pub(crate) fn normal(ctx: &mut CastContext<'_>, stage: usize, packet: &str) -> Result<(), String> {
    let causal = super::clock::accepted(ctx.request);
    let recovery = if causal {
        [894_024, 810_577, 1_158_387, 1_498_441, 790_579][stage]
    } else {
        NORMAL_RECOVERY[stage]
    };
    primary(ctx, NORMAL_IMPACT[stage], recovery)?;
    ctx.scope_primary_aoe(packet)?;
    if stage == 3 {
        // Seven interpolated progress points, with one .0297s local pause per hit.
        for slot in 1..7 {
            let progress = 133697.57 + (649926.4 - 133697.57) * f64::from(slot) / 6.0;
            let offset = progress.round() as u64 + if causal { 0 } else { (slot as u64) * 29700 };
            ctx.schedule_native_aoe_packet(
                packet,
                ctx.time_us
                    .checked_add(offset)
                    .ok_or("Baicang Normal4 time overflow")?,
            )?;
        }
    }
    Ok(())
}
