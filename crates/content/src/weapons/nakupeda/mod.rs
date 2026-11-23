use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::simulation::{ExactFraction, GearProcConfig};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsNakupedaV1;
pub(crate) const GROUP_ID: &str = "ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE";
const SHIELD_ID: &str = "weapon:fork_Nakupeda:shield";

#[derive(Clone, Debug)]
pub(crate) struct State {
    pub(crate) ready_at_us: u64,
    pub(crate) rng: crate::rng::Rng,
    pub(crate) receipts: Vec<crate::simulation::GameProcSelectionReceipt>,
}

impl State {
    pub(crate) fn new(seed: u64, actor_index: usize) -> Result<Self, String> {
        let selector = rng_selector(actor_index)?;
        let rng = crate::rng::Rng::new(
            seed,
            std::collections::BTreeMap::from([(
                format!("weapon:nakupeda:{actor_index}"),
                selector,
            )]),
        )?;
        Ok(Self {
            ready_at_us: 0,
            rng,
            receipts: Vec::new(),
        })
    }
}

pub(crate) fn rng_selector(actor_index: usize) -> Result<u64, String> {
    (1_u64 << 62)
        .checked_add(u64::try_from(actor_index).map_err(|_| "actor index exceeds u64")?)
        .ok_or_else(|| "Nacupeda selector overflow".into())
}

pub(crate) fn rng_snapshot(rng: &crate::rng::Rng) -> Result<serde_json::Value, String> {
    let mut snapshot = serde_json::to_value(rng).map_err(|error| error.to_string())?;
    let streams = snapshot["streams"]
        .as_object_mut()
        .ok_or("Nacupeda RNG snapshot has no streams")?;
    for stream in streams.values_mut() {
        for field in ["sequence", "state"] {
            let value = stream[field].as_u64().ok_or("invalid RNG snapshot word")?;
            stream[field] = serde_json::Value::String(value.to_string());
        }
    }
    Ok(snapshot)
}

pub(crate) fn on_cast(ctx: &mut crate::runtime_api::CastContext<'_>) -> Result<(), String> {
    use crate::simulation::metadata::InputActionRole;
    if ctx.action_metadata.input_role != Some(InputActionRole::Ultimate) {
        return Ok(());
    }
    let Some((
        lowest_hp_heal_ratio,
        shield_ratio,
        shield_duration_us,
        team_heal_ratio,
        cooldown_us,
    )) = ctx.config.gear_procs.iter().find_map(|proc| match proc {
        GearProcConfig::Nakupeda {
            lowest_hp_heal_ratio,
            shield_ratio,
            shield_duration_us,
            team_heal_ratio,
            cooldown_us,
            ..
        } => Some((
            *lowest_hp_heal_ratio,
            *shield_ratio,
            *shield_duration_us,
            *team_heal_ratio,
            *cooldown_us,
        )),
        _ => None,
    })
    else {
        return Ok(());
    };
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let runtime = ctx
        .actor
        .equipment
        .nakupeda
        .as_mut()
        .ok_or("equipped Nacupeda runtime state missing")?;
    if ctx.time_us < runtime.ready_at_us {
        return Ok(());
    }
    let ready_at_us = ctx
        .time_us
        .checked_add(cooldown_us)
        .ok_or("Nacupeda cooldown deadline overflow")?;
    runtime.ready_at_us = ready_at_us;
    let stream = format!("weapon:nakupeda:{}", ctx.actor_index);
    let sample = runtime.rng.uniform(&stream, 3)?;
    let outcome_index = sample.value;
    runtime
        .receipts
        .push(crate::simulation::GameProcSelectionReceipt {
            time_us: ctx.time_us,
            actor_index: ctx.actor_index,
            stream,
            outcome_index,
            sample,
            cooldown_until_us: ready_at_us,
        });
    let (recipients, ratio) = match outcome_index {
        0 => {
            let Some(recipient) = lowest_hp_recipient(ctx.team_hp_snapshot) else {
                return Ok(());
            };
            (vec![recipient], lowest_hp_heal_ratio)
        }
        1 => (vec![ctx.actor_index], shield_ratio),
        2 => {
            let living = ctx
                .team_hp_snapshot
                .iter()
                .enumerate()
                .filter_map(|(index, (hp, _))| (*hp > 0).then_some(index))
                .collect::<Vec<_>>();
            (living, team_heal_ratio)
        }
        _ => return Err("Nacupeda outcome outside 0..3".into()),
    };
    let bases = recipients
        .iter()
        .map(|recipient| {
            let max_hp = ctx.team_hp_snapshot[*recipient].1;
            ExactFraction::new(max_hp.into(), 1)?.mul(ExactFraction::from_ratio(ratio)?)
        })
        .collect::<Result<Vec<_>, String>>()?;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: ntesim_core::execution::CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::NakupedaProc {
                    actor_index: ctx.actor_index,
                    outcome_index,
                    recipients,
                    base_amounts: bases,
                    shield_duration_us,
                },
            ),
        });
    Ok(())
}

pub(crate) fn lowest_hp_recipient(team: &[(u64, u64)]) -> Option<usize> {
    let mut lowest: Option<(usize, u64, u64)> = None;
    for (index, (hp, max)) in team.iter().copied().enumerate() {
        if hp > 0
            && lowest.is_none_or(|(_, best_hp, best_max)| {
                u128::from(hp) * u128::from(best_max) < u128::from(best_hp) * u128::from(max)
            })
        {
            lowest = Some((index, hp, max));
        }
    }
    lowest.map(|(index, _, _)| index)
}

pub(crate) fn apply(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    outcome_index: u64,
    recipients: &[usize],
    base_amounts: &[ExactFraction],
    shield_duration_us: u64,
) -> Result<(), String> {
    if recipients.len() != base_amounts.len() {
        return Err("Nacupeda callback recipient/base count mismatch".into());
    }
    ctx.state.max_proc_depth_reached = ctx.state.max_proc_depth_reached.max(1);
    if ctx.state.actors[actor_index].hp == 0 {
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ProcCallback".into(),
            event_kind: "nakupeda_cancelled".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                outcome: Some(outcome_index.to_string()),
                reason: Some("owner_defeated".into()),
                ..Default::default()
            }),
        });
        return Ok(());
    }
    match outcome_index {
        0 | 2 => apply_healing(ctx, actor_index, outcome_index, recipients, base_amounts),
        1 => apply_shield(ctx, actor_index, base_amounts, shield_duration_us),
        _ => Err("Nacupeda callback outcome outside 0..3".into()),
    }
}

fn apply_healing(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    outcome_index: u64,
    recipients: &[usize],
    base_amounts: &[ExactFraction],
) -> Result<(), String> {
    for (recipient_index, base_heal) in recipients.iter().copied().zip(base_amounts.iter().copied())
    {
        if ctx.state.actors[recipient_index].hp == 0 {
            continue;
        }
        let result = ctx.settle_healing(actor_index, recipient_index, base_heal)?;
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ProcCallback".into(),
            event_kind: "nakupeda_heal".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(recipient_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                outcome: Some(outcome_index.to_string()),
                applied: Some(result.applied),
                overflow: Some(result.excess),
                hp_before: Some(result.before_hp),
                hp_after: Some(result.after_hp),
                ..Default::default()
            }),
        });
    }
    Ok(())
}

fn apply_shield(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    base_amounts: &[ExactFraction],
    shield_duration_us: u64,
) -> Result<(), String> {
    let base_amount = base_amounts
        .first()
        .ok_or("Nacupeda shield amount missing")?;
    let outgoing_shield_bonus = ctx.outgoing_shield_bonus(actor_index, actor_index)?;
    let amount = base_amount
        .mul(ExactFraction::new(1, 1)?.add(outgoing_shield_bonus)?)?
        .round_u64()?;
    let now_us = ctx.state.current_time_us;
    let max_hp = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        ctx.state.actors[actor_index].hp_max_base,
        &ctx.state.actors[actor_index].hp_max_static_bonus,
        actor_index,
        ctx.state.actors[actor_index].hp_max,
    )?;
    let actor = &mut ctx.state.actors[actor_index];
    if actor.hp == 0 {
        return Ok(());
    }
    let expires_at_us = crate::simulation::state::replace_actor_shield(
        actor,
        max_hp,
        now_us,
        SHIELD_ID,
        "weapon:fork_Nakupeda",
        amount,
        shield_duration_us,
    )?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now_us,
        phase: "ProcCallback".into(),
        event_kind: "nakupeda_shield".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            amount: Some(amount),
            expires_at_us: Some(expires_at_us),
            ..Default::default()
        }),
    });
    Ok(())
}
