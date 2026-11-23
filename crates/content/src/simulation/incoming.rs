use super::{GameEventPayload, GameEventReceipt};

pub(super) fn settle(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    event_index: usize,
    actor_index: usize,
    delivered_damage: u64,
) -> Result<super::GameActorDamageReceipt, String> {
    let mitigation =
        super::incoming_defense::evaluate(ctx, event_index, actor_index, delivered_damage)?;
    let delivered_damage = mitigation
        .as_ref()
        .map_or(delivered_damage, |(_, amount)| *amount);
    let plan = crate::registry::incoming_damage_plan(ctx, actor_index, delivered_damage)?;
    validate_plan(&plan, actor_index, delivered_damage, ctx.state.actors.len())?;
    let mut receipt = settle_share(ctx, event_index, plan.primary)?;
    for share in plan.redirected {
        receipt
            .redirected
            .push(settle_share(ctx, event_index, share)?);
    }
    receipt.delivered_damage = delivered_damage;
    receipt.defense_mitigation = mitigation.map(|(receipt, _)| receipt);
    super::dummy::contact(ctx, event_index, &receipt)?;
    Ok(receipt)
}

fn validate_plan(
    plan: &crate::runtime_api::IncomingDamagePlan,
    actor: usize,
    delivered: u64,
    actor_count: usize,
) -> Result<(), String> {
    if plan.primary.actor_index != actor {
        return Err("incoming plan changed the authored primary recipient".into());
    }
    let mut recipients = std::collections::BTreeSet::new();
    let mut total = 0u64;
    for share in std::iter::once(&plan.primary).chain(&plan.redirected) {
        if share.actor_index >= actor_count || !recipients.insert(share.actor_index) {
            return Err("incoming plan has invalid/duplicate recipients".into());
        }
        total = total
            .checked_add(share.amount)
            .ok_or("incoming plan total overflow")?;
    }
    if total != delivered {
        return Err("incoming plan does not conserve delivered damage".into());
    }
    Ok(())
}

fn absorb_broken_blessings(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    broken_blessings: &[String],
    residual: &mut u64,
) -> Result<Vec<super::GameIncomingEffectAbsorptionReceipt>, String> {
    if broken_blessings.is_empty() {
        return Ok(Vec::new());
    }
    crate::runtime_api::record_hypothesis_id(
        request,
        &mut state.execution_model_assumptions,
        crate::characters::adler::skill::A4_OVERFLOW_HYPOTHESIS_ID,
    )?;
    let mut absorbed_by_effects = Vec::new();
    let mut seen_sources = std::collections::BTreeSet::new();
    for broken_shield in broken_blessings {
        let Some(source_actor_index) = crate::characters::adler::skill::shield_owner(broken_shield)
        else {
            continue;
        };
        if !seen_sources.insert(source_actor_index) {
            continue;
        }
        let Some(cap) =
            crate::characters::adler::skill::a4_overflow_cap(request, state, source_actor_index)?
        else {
            continue;
        };
        let amount = (*residual).min(cap);
        *residual -= amount;
        absorbed_by_effects.push(super::GameIncomingEffectAbsorptionReceipt {
            effect_id: crate::characters::adler::skill::A4_OVERFLOW_HYPOTHESIS_ID.into(),
            source_actor_index,
            broken_shield: broken_shield.clone(),
            amount,
        });
    }
    Ok(absorbed_by_effects)
}

fn settle_living_share(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    share: crate::runtime_api::IncomingDamageShare,
    max_hp: u64,
    hp_before: u64,
) -> Result<IncomingShareResolution, String> {
    let actor_index = share.actor_index;
    let delivered_damage = share.amount;
    let now_us = ctx.state.current_time_us;
    let mut vitals = ctx.state.actors[actor_index]
        .restore_vitals(max_hp, now_us, None)
        .map_err(|error| format!("incoming actor restore failed: {error}"))?;
    let immune = crate::registry::incoming_damage_immune(
        &ctx.request.actors[actor_index],
        &ctx.state.actors[actor_index],
        now_us,
    );
    let absorbed = vitals
        .absorb_shields(if immune { 0 } else { delivered_damage })
        .map_err(|error| format!("incoming shield absorption failed: {error:?}"))?;
    let broken_blessings = absorbed
        .iter()
        .filter(|entry| {
            !vitals
                .shields()
                .iter()
                .any(|shield| shield.id == entry.shield)
                && crate::characters::adler::skill::is_a4_shield(ctx.request, &entry.shield)
        })
        .map(|entry| entry.shield.clone())
        .collect::<Vec<_>>();
    let shield_absorbed = absorbed.iter().map(|entry| entry.amount).sum::<u64>();
    let mut residual = if immune {
        0
    } else {
        delivered_damage - shield_absorbed
    };
    let absorbed_by_effects =
        absorb_broken_blessings(ctx.request, ctx.state, &broken_blessings, &mut residual)?;
    let resolution = crate::registry::settle_incoming_hp(
        crate::runtime_api::IncomingHpContext {
            request: ctx.request,
            state: ctx.state,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
            actor_index,
            hp_before,
            residual: if share.hp_floor > 0 {
                residual.min(hp_before.saturating_sub(share.hp_floor))
            } else {
                residual
            },
        },
        &mut vitals,
    )?;
    ctx.state.actors[actor_index].save_vitals(&vitals, now_us);
    ctx.state.actors[actor_index].record_hp_loss(resolution.actual_hp_loss, max_hp);
    if resolution.defeated {
        ctx.state.actors[actor_index].shields.clear();
    }
    Ok(IncomingShareResolution {
        actual_hp_loss: resolution.actual_hp_loss,
        absorbed_by_shields: absorbed
            .into_iter()
            .map(|entry| super::GameShieldAbsorptionReceipt {
                shield: entry.shield,
                amount: entry.amount,
            })
            .collect(),
        absorbed_by_effects,
        overflow: resolution.overflow,
        defeated: resolution.defeated,
    })
}

struct IncomingShareResolution {
    actual_hp_loss: u64,
    absorbed_by_shields: Vec<super::GameShieldAbsorptionReceipt>,
    absorbed_by_effects: Vec<super::GameIncomingEffectAbsorptionReceipt>,
    overflow: u64,
    defeated: bool,
}

fn settle_share(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    event_index: usize,
    share: crate::runtime_api::IncomingDamageShare,
) -> Result<super::GameActorDamageReceipt, String> {
    let actor_index = share.actor_index;
    let delivered_damage = share.amount;
    let max_hp = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        ctx.state.actors[actor_index].hp_max_base,
        &ctx.state.actors[actor_index].hp_max_static_bonus,
        actor_index,
        ctx.state.actors[actor_index].hp_max,
    )?;
    let hp_before = ctx.state.actors[actor_index].hp;
    let settled = if hp_before == 0 {
        ctx.state.actors[actor_index].shields.clear();
        IncomingShareResolution {
            actual_hp_loss: 0,
            absorbed_by_shields: Vec::new(),
            absorbed_by_effects: Vec::new(),
            overflow: delivered_damage,
            defeated: true,
        }
    } else {
        settle_living_share(ctx, share, max_hp, hp_before)?
    };
    let receipt = super::GameActorDamageReceipt {
        defense_mitigation: None,
        event_index,
        time_us: ctx.state.current_time_us,
        actor_index,
        delivered_damage,
        redirected: Vec::new(),
        hp_before,
        prevented_hp_loss: delivered_damage.saturating_sub(
            settled.actual_hp_loss
                + settled.overflow
                + settled
                    .absorbed_by_shields
                    .iter()
                    .map(|s| s.amount)
                    .sum::<u64>()
                + settled
                    .absorbed_by_effects
                    .iter()
                    .map(|s| s.amount)
                    .sum::<u64>(),
        ),
        actual_hp_loss: settled.actual_hp_loss,
        absorbed_by_shields: settled.absorbed_by_shields.clone(),
        absorbed_by_effects: settled.absorbed_by_effects,
        overflow: settled.overflow,
        hp_after: ctx.state.actors[actor_index].hp,
        defeated: settled.defeated,
    };
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "HitDelivery".into(),
        event_kind: "actor_damage".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            event_index: Some(event_index),
            delivered_damage: Some(delivered_damage),
            applied: Some(settled.actual_hp_loss),
            overflow: Some(settled.overflow),
            absorbed_by_shields: Some(
                settled
                    .absorbed_by_shields
                    .iter()
                    .map(|entry| entry.amount)
                    .sum(),
            ),
            absorbed_by_effects: Some(
                receipt
                    .absorbed_by_effects
                    .iter()
                    .map(|entry| entry.amount)
                    .sum(),
            ),
            defeated: settled.defeated.then_some(true),
            ..Default::default()
        }),
    });
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        super::events::INCOMING_DAMAGE_HYPOTHESIS_ID,
    )?;
    if settled.actual_hp_loss > 0 {
        crate::gear::on_damage_received(ctx, actor_index, settled.actual_hp_loss)?;
    }
    if settled.defeated {
        crate::registry::on_actor_defeated(ctx, actor_index)?;
    }
    Ok(receipt)
}
