//! Pure input gates and committed cancellation share the finite owner clock.
use super::*;
use crate::simulation::{GameActorInterruptReceipt, PlayerInput};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::DaffodilActionCancelsV1;
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if accepted(req) && !super::accepted(req) {
        return Err("Daffodil action cancels require ordinary_clock_v1".into());
    }
    Ok(())
}
pub(super) fn gate(kind: Kind, input: &str) -> Option<u64> {
    let recovery = profile::recovery(kind);
    let progress = match (kind, input) {
        (Kind::Normal(stage), "jump") => [216_490, 161_534, 901_691, 291_004, 892_380][stage],
        (Kind::Normal(stage), "GA_Daffodill_Skill") => {
            [283_178, 427_051, 1_178_939, 770_375, 985_107][stage]
        }
        (Kind::Normal(stage), "dash") => [383_436, 600_274, 1_849_347, 1_560_915, 1_412_737][stage],
        (Kind::Skill(false), "jump") => 288_606,
        (Kind::Skill(true), "jump") => 208_484,
        (Kind::Skill(true), "GA_Daffodill_Skill") => 863_285,
        (Kind::Burst, "jump") => 5_086_241,
        (Kind::Branch, "jump") => 1_266_199,
        (Kind::Branch, "GA_Daffodill_Skill") => 1_299_907,
        (Kind::Branch, "dash") => 1_712_559,
        (Kind::Plunge, "jump") => 149_206,
        (Kind::Riposte, "jump" | "GA_Daffodill_Skill") => 0,
        (Kind::Support, "jump") => recovery,
        (Kind::Dodge(back), "jump") => {
            if back {
                0
            } else {
                82_544
            }
        }
        (Kind::Dodge(back), "GA_Daffodill_Skill") => {
            if back {
                494_283
            } else {
                333_754
            }
        }
        (Kind::Skill(false), "dash") => 1_713_067,
        (Kind::Skill(true), "dash") => 1_599_487,
        (Kind::Burst, "dash") => 5_084_873,
        (Kind::Plunge, "dash") => 750_726,
        (Kind::Support, "dash") => 1_291_110,
        (Kind::Dodge(true), "dash") => 836_966,
        (_, "dash" | "GA_Daffodill_Melee") => recovery,
        (_, "GA_Daffodill_Skill") => recovery,
        _ => return None,
    };
    Some(progress.min(recovery))
}
pub(crate) fn recovery(
    req: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if !accepted(req) {
        return None;
    }
    let crate::characters::RuntimeState::Daffodil(state) = &actor.character else {
        return None;
    };
    let clock = state.clock.as_ref()?;
    if super::super::held::accepted(req) {
        if let Some((start, _)) = clock.displaced.filter(|(_, end)| now < *end) {
            return Some((
                start.checked_add(gate(Kind::Dodge(false), input)?)?,
                super::super::held::MODEL,
            ));
        }
        let back = clock.active.as_ref().is_some_and(|action| {
            req.actions.iter().enumerate().any(|(index, command)| {
                matches!(command, crate::simulation::GameActionCommand::PlayerInput {
                    timestamp_us, actor_index, input: PlayerInput::Dash, ..
                } if *timestamp_us == now && *actor_index == action.owner)
                    && req.dash_directions.get(&index)
                        == Some(&crate::simulation::GameDashDirection::Back)
            })
        });
        if input == "dash"
            && !back
            && let Some(gate) = super::normal_jump_gate(actor, now)
        {
            return Some((gate, super::super::held::MODEL));
        }
    }
    let Some(action) = &clock.active else {
        return (input == "jump").then_some((now, MODEL));
    };
    Some((due(clock, action, gate(action.kind, input)?).ok()?, MODEL))
}
pub(crate) fn committed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    input: PlayerInput,
) -> Result<(), String> {
    if input != PlayerInput::Jump || !accepted(ctx.request) {
        return Ok(());
    }
    stop(
        &mut ctx.state.actors[owner],
        ctx.queue,
        ctx.state.current_time_us,
    )?;
    let state = ctx.state.actors[owner].character.daffodil_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    Ok(())
}
pub(crate) fn interrupt(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<GameActorInterruptReceipt, String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let cooldown = actor.ability_cooldowns.get(SKILL).copied();
    let mut receipt = GameActorInterruptReceipt {
        event_index,
        actor_index: owner,
        time_us: now,
        interrupted_command_index: None,
        cancelled_hit_deliveries: 0,
        recovery_before_us: actor.locked_until_us,
        recovery_after_us: actor.locked_until_us,
        skill_cooldown_before_us: cooldown,
        skill_cooldown_after_us: cooldown,
        field_cancelled: false,
    };
    if actor.hp == 0 || now >= actor.locked_until_us {
        return Ok(receipt);
    }
    receipt.interrupted_command_index = actor
        .character
        .daffodil_mut()?
        .clock
        .as_ref()
        .and_then(|c| c.active.as_ref())
        .map(|a| a.command);
    let before = ctx
        .queue
        .iter()
        .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
        .count();
    stop(actor, ctx.queue, now)?;
    let after = ctx
        .queue
        .iter()
        .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
        .count();
    receipt.cancelled_hit_deliveries = before - after;
    actor.locked_until_us = now;
    let state = actor.character.daffodil_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    receipt.recovery_after_us = now;
    receipt.skill_cooldown_after_us = actor.ability_cooldowns.get(SKILL).copied();
    Ok(receipt)
}
