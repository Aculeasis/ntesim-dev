use crate::runtime_api::CallbackContext;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

pub(crate) fn apply_team_heal(
    ctx: &mut CallbackContext<'_>,
    source_actor_index: usize,
    recipient_actor_index: usize,
    atk_ratio: ExactFraction,
    event_kind: &str,
    action_name: &str,
) -> Result<(), String> {
    let source_atk = ctx.live_atk(source_actor_index)?;
    apply_team_heal_atk(
        ctx,
        source_actor_index,
        recipient_actor_index,
        source_atk,
        atk_ratio,
        event_kind,
        action_name,
    )
}

pub(super) fn apply_support_heal(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    atk_ratio: ExactFraction,
) -> Result<(), String> {
    let source_atk = ctx.live_atk(owner)?;
    let regressed = &ctx.state.actors[owner].character.iroi()?.regressed_allies;
    let mut recipients = (0..ctx.request.actors.len())
        .filter(|recipient| !regressed.contains_key(recipient))
        .collect::<Vec<_>>();
    recipients.sort_by_key(|&index| (ctx.request.actors[index].slot, index));
    for recipient in recipients {
        apply_team_heal_atk(
            ctx,
            owner,
            recipient,
            source_atk,
            atk_ratio,
            "iroi_support_heal",
            "Support",
        )?;
    }
    Ok(())
}

fn apply_team_heal_atk(
    ctx: &mut CallbackContext<'_>,
    source_actor_index: usize,
    recipient_actor_index: usize,
    source_atk: ExactFraction,
    atk_ratio: ExactFraction,
    event_kind: &str,
    action_name: &str,
) -> Result<(), String> {
    let source = ctx
        .state
        .actors
        .get(source_actor_index)
        .ok_or("Iroi healing source actor missing")?;
    if source.hp == 0 || ctx.state.actors[recipient_actor_index].hp == 0 {
        return Ok(());
    }
    let base_heal = source_atk.mul(atk_ratio)?;
    let settlement = ctx.settle_healing(source_actor_index, recipient_actor_index, base_heal)?;
    ctx.state.max_proc_depth_reached = ctx.state.max_proc_depth_reached.max(1);
    let input = match action_name {
        "Skill" => "skill",
        "Support" => "support",
        _ => "burst",
    };
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: event_kind.into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(source_actor_index),
        to_actor: Some(recipient_actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            input: Some(input.into()),
            exact_raw: Some(source_atk),
            atk_ratio: Some(atk_ratio),
            gross: Some(settlement.gross),
            applied: Some(settlement.applied),
            overflow: Some(settlement.excess),
            hp_before: Some(settlement.before_hp),
            hp_after: Some(settlement.after_hp),
            ..Default::default()
        }),
    });

    if settlement.gross > 0
        && ctx.request.actors[source_actor_index].character_id == "character:1075"
    {
        let def_ignore = ExactFraction::new(1, 20)?;
        ctx.state.apply_buff(
            source_actor_index,
            TimedBuffDeclaration::reset(
                "Buff_Oneiroi_Passive2_DefIgnore",
                "character:1075",
                BuffRecipient::TeamAll,
                20_000_000,
                BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([("def_ignore".into(), def_ignore)]),
            ),
        )?;
        if ctx.request.actors[source_actor_index]
            .awakening
            .has_effect(5)
        {
            let base_atk = ctx.request.actors[source_actor_index].static_stats.base_atk as u128;
            let a5_uncapped = ExactFraction::new(base_atk * 3, 20)?;
            let a5_atk = if a5_uncapped.to_f64() > 500.0 {
                ExactFraction::new(500, 1)?
            } else {
                a5_uncapped
            };
            ctx.state.apply_buff(
                source_actor_index,
                TimedBuffDeclaration::reset(
                    "Buff_Oneiroi075_Level5",
                    "character:1075",
                    BuffRecipient::TeamAll,
                    20_000_000,
                    BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([("atk_add".into(), a5_atk)]),
                ),
            )?;
        }
    }

    crate::weapons::wrong_gate::on_healing(
        ctx,
        source_actor_index,
        recipient_actor_index,
        settlement.gross,
    )
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    source_actor_index: usize,
    atk_add: ExactFraction,
    def_ignore: ExactFraction,
    a5_atk_add: Option<ExactFraction>,
    duration_us: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    for recipient_index in 0..ctx.request.actors.len() {
        if recipient_index != source_actor_index {
            state.apply_buff(
                source_actor_index,
                TimedBuffDeclaration::reset(
                    "Buff_Oneiroi_Skill_AtkUP",
                    "character:1075",
                    BuffRecipient::SpecificActor(recipient_index),
                    duration_us,
                    BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([("atk_add".into(), atk_add)]),
                ),
            )?;
        }
    }
    state.apply_buff(
        source_actor_index,
        TimedBuffDeclaration::reset(
            "Buff_Oneiroi_Passive2_DefIgnore",
            "character:1075",
            BuffRecipient::TeamAll,
            duration_us,
            BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([("def_ignore".into(), def_ignore)]),
        ),
    )?;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "iroi_skill_buff".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(source_actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            atk_add: Some(atk_add),
            def_ignore: Some(def_ignore),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    });

    if let Some(a5_atk) = a5_atk_add {
        state.apply_buff(
            source_actor_index,
            TimedBuffDeclaration::reset(
                "Buff_Oneiroi075_Level5",
                "character:1075",
                BuffRecipient::TeamAll,
                duration_us,
                BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([("atk_add".into(), a5_atk)]),
            ),
        )?;
        trace.push(GameEventReceipt {
            time_us: state.current_time_us,
            phase: "ProcCallback".into(),
            event_kind: "iroi_a5_atk_buff".into(),
            active_actor: state.active_actor_index,
            hit_index: None,
            from_actor: Some(source_actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                atk_add: Some(a5_atk),
                duration_us: Some(duration_us),
                ..Default::default()
            }),
        });
    }

    Ok(())
}

pub(crate) fn apply_a2_assist(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "iroi_a2_assist".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: None,
    });
    Ok(())
}

pub(crate) fn apply_a6_revive(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "iroi_a6_revive_grant".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: None,
    });
    Ok(())
}
