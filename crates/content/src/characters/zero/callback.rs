use crate::runtime_api::CallbackContext;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

pub(crate) fn on_paid_cosmos_cycle(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    let config = &ctx.request.actors[owner];
    if config.character_id != "character:1046" || !config.awakening.has_resonance_6() {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        crate::HypothesisId::ZeroCosmosCycleAtkV1,
    )?;
    let parameters = &super::parameters::PARAMETERS;
    let now = ctx.state.mode_clock.relative_us();
    for (recipient, actor) in ctx.state.actors.iter().enumerate() {
        if actor.hp > 0 {
            crate::simulation::state::apply_timed_buff(
                &mut ctx.state.buffs,
                TimedBuffDeclaration::reset(
                    "zero.cosmos_cycle_atk",
                    &config.character_id,
                    BuffRecipient::SpecificActor(recipient),
                    parameters.cosmos_cycle_atk_duration_us,
                    BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([(
                        "atk_percent".into(),
                        parameters.cosmos_cycle_atk_percent,
                    )]),
                ),
                owner,
                now,
            )?;
        }
    }
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "zero_cosmos_cycle_atk".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            buff_id: Some("zero.cosmos_cycle_atk".into()),
            atk_percent: Some(parameters.cosmos_cycle_atk_percent),
            duration_us: Some(parameters.cosmos_cycle_atk_duration_us),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn apply_a2_energy(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    amount: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let actor = state
        .actors
        .get_mut(actor_index)
        .ok_or("Zero actor not found")?;
    let before = crate::simulation::energy::EnergySnapshot::capture(actor);
    let charge_basis = ExactFraction::new(amount.into(), 1)?;
    let _ = actor.grant_energy(amount)?;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "zero_a2_energy_grant".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: Some(before.receipt(
            actor,
            crate::simulation::GameEnergySource::ZeroA2,
            actor_index,
            actor_index,
            Some(charge_basis),
        )?),
        energy_fanout: None,
        data: None,
    });
    crate::simulation::energy::notify_active_actor_energy_gain(ctx, actor_index, amount > 0)?;
    Ok(())
}

pub(crate) fn apply_a5_buff(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    duration_us: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    // 10% ATK buff per DT_Female051EffectFigure Female051_LV5 = 0.1
    let atk_percent = super::parameters::PARAMETERS.support_atk_percent;
    state.apply_buff(
        actor_index,
        TimedBuffDeclaration::reset(
            "Buff_Female051_Level5",
            "character:1046",
            BuffRecipient::SelfActor,
            duration_us,
            BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([("atk_percent".into(), atk_percent)]),
        ),
    )?;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "zero_a5_atk_buff".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            atk_percent: Some(atk_percent),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn on_charge_grant(
    ctx: &mut CallbackContext<'_>,
    recipient: usize,
) -> Result<(), String> {
    let zero_actor =
        ctx.request
            .actors
            .iter()
            .enumerate()
            .find(|(i, a)| a.character_id == "character:1051" && ctx.state.actors[*i].hp > 0)
            .or_else(|| {
                ctx.request.actors.iter().enumerate().find(|(i, a)| {
                    a.character_id == "character:1046" && ctx.state.actors[*i].hp > 0
                })
            });

    let Some((zero_index, config)) = zero_actor else {
        return Ok(());
    };

    let base_atk = config.static_stats.base_atk;
    let base_heal = ExactFraction::new(base_atk.into(), 2)?;
    let settlement = ctx.settle_healing(zero_index, recipient, base_heal)?;

    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "zero_charge_heal".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(zero_index),
        to_actor: Some(recipient),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            gross: Some(settlement.gross),
            ..Default::default()
        }),
    });

    Ok(())
}
