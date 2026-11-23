//! Settlement for the owner's true remaining-HP finish; never a second ordinary hit.
use super::*;

pub(super) fn after_hit(
    ctx: &mut EventContext<'_>,
    parent: &HitDelivery,
    hp_loss: u64,
) -> Result<(), String> {
    let target = parent.identity.target_index;
    let Some(finish) = crate::registry::finishing_damage(&crate::runtime_api::FinishHitContext {
        request: ctx.request,
        state: ctx.state,
        parent_actor: parent.identity.actor_index,
        target_index: target,
        applied_hp_loss: hp_loss,
        metadata: &parent.metadata,
    })?
    else {
        return Ok(());
    };
    let owner = finish.owner;
    let (report_command_index, autonomous_event_index) = ctx
        .state
        .origins
        .identify(ctx.request, parent.identity.command_index)?;
    let id = finish.hypothesis;
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        id,
    )?;
    let remaining = ctx.state.targets[target].hp_current;
    let damage = ExactFraction::new(u128::from(remaining), 1)?;
    let zero = ExactFraction::new(0, 1)?;
    ctx.state.targets[target].hp_current = 0;
    ctx.state.add_rounded_damage(remaining)?;
    ctx.state.hits_settled = ctx
        .state
        .hits_settled
        .checked_add(1)
        .ok_or("execute hit count overflow")?;
    let execution_model_assumption_ids = ctx
        .state
        .record_receipt_assumptions([(id.as_str().into(), id.description().into())])?;
    let (metadata_id, energy_group_id) =
        ctx.state.intern_action_packet_metadata(&finish.metadata)?;
    let hit_index = ctx.hits.len();
    ctx.hits.push(GameHitReceipt {
        active_actor_index: ctx.state.active_actor_index,
        parent_hit_index: None,
        damage_base_override: None,
        autonomous_event_index,
        control_applications: Vec::new(),
        switch_origin: None,
        break_contribution: None,
        execution_model_assumption_ids,
        command_index: report_command_index,
        time_us: ctx.state.current_time_us,
        actor_index: owner,
        target_index: target,
        ability_id: finish.ability_id.into(),
        damage_effect_id: finish.effect_id.into(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        scaling_rate: crate::formula::Ratio::zero(),
        metadata_id,
        energy_group_id,
        static_general_damage_bonus: zero,
        static_element_damage_bonus: zero,
        static_mental_damage_bonus: zero,
        non_crit_damage: damage,
        crit_damage: damage,
        crit_rate: crate::formula::Ratio::zero(),
        critical: false,
        damage,
        rounded_damage: remaining,
        applied_hp_loss: remaining,
        crit_sample: None,
    });
    if ctx.state.record_detail {
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "SettlementAndDeath".into(),
            event_kind: "hp_finish".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: Some(hit_index),
            from_actor: Some(owner),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                target_index: Some(target),
                hp_after: Some(remaining),
                ..Default::default()
            }),
        });
    }
    crate::registry::on_finish_target_defeated(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        owner,
        target,
    )?;
    crate::simulation::reactions::clear_target_statuses(&mut ctx.state.buffs, target);
    Ok(())
}
