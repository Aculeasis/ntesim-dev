//! One captured incoming split, before shields and without recursive children.
use crate::runtime_api::{CallbackContext, IncomingDamagePlan, IncomingDamageShare};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::FadiaIncomingShareV1;
pub(crate) fn accepted(req: &crate::simulation::GameScenarioRequest) -> bool {
    super::effects::accepted(req)
        || req
            .model_assumptions
            .get(MODEL.as_str())
            .map(String::as_str)
            == Some(MODEL.description())
}
pub(crate) fn plan(
    ctx: &mut CallbackContext<'_>,
    target: usize,
    damage: u64,
) -> Result<IncomingDamagePlan, String> {
    let mut plan = IncomingDamagePlan::single(target, damage);
    if !accepted(ctx.request)
        || ctx.state.actors[target].hp == 0
        || crate::registry::incoming_damage_immune(
            &ctx.request.actors[target],
            &ctx.state.actors[target],
            ctx.state.current_time_us,
        )
    {
        return Ok(plan);
    }
    let owner = if ctx.request.actors[target].character_id == "character:1039" {
        Some(target)
    } else {
        ctx.request
            .actors
            .iter()
            .enumerate()
            .find(|(index, config)| {
                config.character_id == "character:1039" && ctx.state.actors[*index].hp > 1
                    && !matches!(&ctx.state.actors[*index].character, crate::characters::RuntimeState::Fadia(s) if s.out_of_combat)
            })
            .map(|(index, _)| index)
    };
    let Some(owner) = owner else {
        return Ok(plan);
    };
    if matches!(&ctx.state.actors[owner].character, crate::characters::RuntimeState::Fadia(s) if s.out_of_combat)
    {
        return Ok(plan);
    }
    let others: Vec<_> = ctx
        .state
        .actors
        .iter()
        .enumerate()
        .filter_map(|(index, actor)| (index != owner && actor.hp > 0).then_some(index))
        .collect();
    if others.is_empty() {
        return Ok(plan);
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        if super::effects::accepted(ctx.request) {
            super::effects::MODEL
        } else {
            MODEL
        },
    )?;
    let hp = ctx.state.actors[owner].hp;
    let numerator = if ctx.request.actors[owner].awakening.has_effect(1) {
        3u128
    } else {
        2u128
    };
    let pool: u64 = (u128::from(damage) * numerator / 4)
        .try_into()
        .map_err(|_| "Fadia incoming share overflow")?;
    if target != owner && hp > 1 {
        plan.primary.amount -= pool;
        plan.redirected.push(IncomingDamageShare {
            actor_index: owner,
            amount: pool,
            hp_floor: 1,
        });
    } else if target == owner {
        plan.primary.hp_floor = 1;
        if hp == 1 {
            plan.primary.amount -= pool;
            let count = others.len() as u64;
            for (position, actor_index) in others.into_iter().enumerate() {
                let amount = pool / count + u64::from((position as u64) < pool % count);
                plan.redirected.push(IncomingDamageShare {
                    actor_index,
                    amount,
                    hp_floor: u64::from(
                        ctx.request.actors[actor_index].character_id == "character:1039",
                    ),
                });
            }
        }
    }
    Ok(plan)
}
