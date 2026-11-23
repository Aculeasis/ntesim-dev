use crate::HypothesisId;
use crate::runtime_api::CallbackContext;
use crate::simulation::reactions::ReactionSnapshot;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiBudCopyB2V1;

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn iroi_has_b2(request: &crate::simulation::GameScenarioRequest) -> Option<usize> {
    if !accepted(request) {
        return None;
    }
    for (index, actor_config) in request.actors.iter().enumerate() {
        if actor_config.character_id == "character:1075" {
            let breakthrough = ntesim_data::skill_levels::progression(
                &actor_config.character_id,
                actor_config.character_level,
                actor_config.character_max_level,
            )
            .ok()
            .and_then(|p| p["breakthrough"].as_u64())
            .unwrap_or(0);
            if breakthrough >= 2 {
                return Some(index);
            }
        }
    }
    None
}

pub(crate) fn on_bud_copy_check(
    ctx: &mut CallbackContext<'_>,
    bud_id: u64,
    snapshot: ReactionSnapshot,
) -> Result<(), String> {
    let Some(iroi_index) = iroi_has_b2(ctx.request) else {
        return Ok(());
    };
    if ctx.state.actors[iroi_index].hp == 0 {
        return Ok(());
    }

    // Check if parent bud still exists
    let bud_exists = ctx
        .state
        .reactions
        .buds()
        .iter()
        .any(|bud| bud.id == bud_id);
    if !bud_exists {
        return Ok(());
    }

    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;

    let state = ctx.state.actors[iroi_index].character.iroi_mut()?;
    if state.bud_copy_ids.contains(&bud_id) {
        return Ok(());
    }
    let replaced = (state.bud_copy_ids.len() == 3).then(|| state.bud_copy_ids.pop_front().unwrap());
    state.bud_copy_ids.push_back(bud_id);
    if let Some(replaced) = replaced {
        crate::simulation::reactions::expire_bud(ctx, replaced, true)?;
    }

    // A copy has its own ten ObjectLifetime seconds. Parent expiry must not
    // remove it; a Time Stop can consume this lifetime without banking volleys.
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push_on(
        ntesim_core::kernel::timing::ClockDomain::ObjectLifetime,
        crate::simulation::events::ScheduledGameEvent {
            time_us: ctx
                .state
                .current_time_us
                .checked_add(10_000_000)
                .ok_or("Iroi copy lifetime overflow")?,
            priority: ntesim_core::execution::CausalPriority::SettlementAndDeath,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::BlossomBudExpiry {
                    bud_id,
                    is_copy: true,
                },
            ),
        },
    );
    // Schedule 20 copied pistil ticks every 0.5s starting at now + 0.5s
    let now = ctx.state.current_time_us;
    for emission in 1..=20 {
        let delay_us = emission * 500_000;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: now + delay_us,
                priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::ReactionPistil(Box::new(
                    crate::simulation::reactions::PistilEvent {
                        bud_id,
                        snapshot: snapshot.clone(),
                        is_copy: true,
                    },
                )),
            });
    }

    Ok(())
}
