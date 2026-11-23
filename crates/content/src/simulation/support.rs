//! Engine-owned Support Skill dispatch on an admitted entry switch (B3/Q2).
//! An ordinary admission rejection skips the incoming action. Unresolved
//! dependencies propagate through the simulator's incomplete-run boundary.

use super::context::EventContext;
use super::{GameActionCommand, GameEventPayload, GameEventReceipt};

/// Inspect the same owner entry selection used by a switch, without applying
/// its effects or admitting a Support. Entry attacks are not Support skills.
pub(super) fn ready(
    request: &super::GameScenarioRequest,
    state: &super::state::GameSimulatorState,
    actor_index: usize,
) -> Result<bool, String> {
    let old = state.active_actor_index;
    let actor = &state.actors[actor_index];
    if actor_index == old || actor.hp == 0 || actor.locked_until_us > state.current_time_us {
        return Ok(false);
    }
    let qualified =
        super::reactions::validate_on_switch(request, state, old, actor_index, 0)?.is_some();
    if !qualified {
        return Ok(false);
    }
    let entry = match crate::registry::entry_action(request, state, old, actor_index, qualified)? {
        Some(crate::runtime_api::OwnerEntryAction::Support(entry)) => Some(entry),
        Some(crate::runtime_api::OwnerEntryAction::Effect { support, .. }) => support,
        Some(crate::runtime_api::OwnerEntryAction::Attack(_)) | None => None,
    };
    let Some(entry) = entry else {
        return Ok(false);
    };
    super::talents::qte_coordinate(&request.actors[actor_index])?;
    Ok(super::availability::ability(
        request,
        state,
        actor_index,
        entry.ability_id,
        entry.damage_effect_id,
        0,
    )
    .ready)
}

fn prepare_packet(
    request: &super::GameScenarioRequest,
    config: &super::GameActorConfig,
    ability_id: &str,
    effect: &str,
    array_index: u32,
) -> Result<
    (
        crate::formula::Ratio,
        super::NativeScalingStat,
        super::metadata::ActionPacketMetadata,
    ),
    String,
> {
    let tables = ntesim_data::skill_levels::frozen::tables();
    let projection = crate::build::project_native_packet_coefficient(
        tables,
        &crate::build::PacketCoefficientRequest {
            content_id: &request.content_id,
            character_id: &config.character_id,
            ability_id,
            damage_effect_id: effect,
            array_index,
            research_metadata: None,
            model_assumptions: &request.model_assumptions,
        },
    )?;
    let atk_rate = serde_json::from_value(projection["compiled_atk_rate"].clone())
        .map_err(|error| format!("Support compiled coefficient invalid: {error}"))?;
    let scaling_stat = serde_json::from_value(projection["compiled_scaling_stat"].clone())
        .map_err(|error| format!("Support compiled scaling stat invalid: {error}"))?;
    let metadata = super::metadata::resolve_requested_action_packet(
        &request.content_id,
        tables,
        &config.character_id,
        ability_id,
        effect,
        None,
        &request.model_assumptions,
    )?;
    Ok((atk_rate, scaling_stat, metadata))
}

/// The entry switch is already admitted. A later owner/gear observer failure
/// must roll back only this attempted Support, including its queued children.
fn admit_support(
    ctx: &mut EventContext<'_>,
    command_index: usize,
    command: &GameActionCommand,
    metadata: super::metadata::ActionPacketMetadata,
) -> Result<Option<super::rejection::AdmissionFailure>, String> {
    let state = ctx.state.clone();
    let queue = ctx.queue.clone();
    let sequence = *ctx.sequence;
    let trace_len = ctx.trace.len();
    let ledger_len = ctx.action_metadata_ledger.len();
    let result = super::admission::admit_ability(
        ctx,
        command_index,
        command,
        Some(metadata),
        super::admission::AdmissionOrigin::SupportDispatch,
    );
    if !matches!(result, Ok(None)) {
        *ctx.state = state;
        *ctx.queue = queue;
        *ctx.sequence = sequence;
        ctx.trace.truncate(trace_len);
        ctx.action_metadata_ledger.truncate(ledger_len);
    }
    result
}

pub(super) fn dispatch_owner_entry(
    ctx: &mut EventContext<'_>,
    command_index: usize,
    actor_index: usize,
    entry: crate::runtime_api::OwnerEntryAction,
) -> Result<(), String> {
    let entry = match entry {
        crate::runtime_api::OwnerEntryAction::Attack(entry) => entry,
        crate::runtime_api::OwnerEntryAction::Support(entry) => {
            return dispatch_entry(ctx, command_index, actor_index, entry);
        }
        crate::runtime_api::OwnerEntryAction::Effect {
            source_actor_index,
            support,
            apply,
        } => {
            apply(
                &mut crate::runtime_api::CallbackContext {
                    state: ctx.state,
                    request: ctx.request,
                    queue: ctx.queue,
                    sequence: ctx.sequence,
                    trace: ctx.trace,
                },
                source_actor_index,
                actor_index,
                command_index,
            )?;
            return if let Some(entry) = support {
                dispatch_entry(ctx, command_index, actor_index, entry)
            } else {
                Ok(())
            };
        }
    };
    let now = ctx.state.current_time_us;
    let config = &ctx.request.actors[actor_index];
    if ctx.state.actors[actor_index].locked_until_us > now {
        ctx.trace.push(GameEventReceipt {
            time_us: now,
            phase: "ActionAdmission".into(),
            event_kind: "owner_entry_skipped".into(),
            active_actor: actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                reason: Some("recovery_lock".into()),
                cooldown_until_us: Some(ctx.state.actors[actor_index].locked_until_us),
                ..Default::default()
            }),
        });
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions.clone(),
        entry.model,
    )?;
    let coordinate =
        u32::try_from(entry.coordinate).map_err(|_| "entry talent coordinate overflow")?;
    let (rate, scaling, mut metadata) =
        prepare_packet(ctx.request, config, entry.ability, entry.effect, coordinate)?;
    metadata.action.input_role = Some(super::metadata::InputActionRole::Other);
    metadata.action.primary_slot = None;
    let command = GameActionCommand::InvokeAbility {
        timestamp_us: now,
        actor_index,
        ability_id: entry.ability.into(),
        damage_effect_id: entry.effect.into(),
        array_index: entry.coordinate,
        atk_rate: rate,
        scaling_stat: scaling,
        energy_cost: 0,
        recovery_us: entry.recovery_us,
        cooldown_us: 0,
        hit_delay_us: entry.hit_delay_us,
    };
    let (kind, data) = match admit_support(ctx, command_index, &command, metadata)? {
        None => (
            "owner_entry",
            GameEventPayload {
                ability_id: Some(entry.ability.into()),
                damage_effect_id: Some(entry.effect.into()),
                command_index: Some(command_index),
                ..Default::default()
            },
        ),
        Some(failure) => (
            "owner_entry_skipped",
            GameEventPayload {
                ability_id: Some(entry.ability.into()),
                damage_effect_id: Some(entry.effect.into()),
                command_index: Some(command_index),
                reason: Some(failure.message),
                ..Default::default()
            },
        ),
    };
    ctx.trace.push(GameEventReceipt {
        time_us: now,
        phase: "ActionAdmission".into(),
        event_kind: kind.into(),
        active_actor: actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(data),
    });
    Ok(())
}

fn dispatch_entry(
    ctx: &mut EventContext<'_>,
    command_index: usize,
    actor_index: usize,
    entry: &crate::runtime_api::SupportEntry,
) -> Result<(), String> {
    let dispatch_model = entry.dispatch_model;
    let request = ctx.request;
    let config = &request.actors[actor_index];
    let now_us = ctx.state.current_time_us;
    let skip = |ctx: &mut EventContext<'_>, reason: String| {
        ctx.trace.push(GameEventReceipt {
            time_us: now_us,
            phase: "ActionAdmission".into(),
            event_kind: "support_dispatch_skipped".into(),
            active_actor: actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                ability_id: Some(entry.ability_id.into()),
                reason: Some(reason),
                ..Default::default()
            }),
        });
    };
    if ctx.state.actors[actor_index].locked_until_us > now_us {
        skip(ctx, "recovery_lock".into());
        return Ok(());
    }
    let array_index = super::talents::qte_coordinate(config)?;
    // Validate evidence policy before admission can commit costs, packets or
    // team observers. Strict mode requires acceptance of every reached model.
    let mut assumptions = ctx.state.execution_model_assumptions.clone();
    crate::runtime_api::record_hypothesis_id(request, &mut assumptions, dispatch_model)?;
    if let Some(id) = entry.packet_model {
        crate::runtime_api::record_hypothesis_id(request, &mut assumptions, id)?;
    }
    for effect in entry.extra_damage_effects {
        prepare_packet(request, config, entry.ability_id, effect, array_index)?;
    }
    let (atk_rate, scaling_stat, metadata) = prepare_packet(
        request,
        config,
        entry.ability_id,
        entry.damage_effect_id,
        array_index,
    )?;
    let command = GameActionCommand::InvokeAbility {
        actor_index,
        ability_id: entry.ability_id.into(),
        array_index: array_index as usize,
        damage_effect_id: entry.damage_effect_id.into(),
        atk_rate,
        scaling_stat,
        energy_cost: 0,
        recovery_us: entry.recovery_us,
        cooldown_us: 0,
        hit_delay_us: entry.hit_delay_us,
        timestamp_us: now_us,
    };
    match admit_support(ctx, command_index, &command, metadata) {
        Ok(None) => {
            crate::runtime_api::record_hypothesis_id(
                request,
                &mut ctx.state.execution_model_assumptions,
                dispatch_model,
            )?;
            ctx.trace.push(GameEventReceipt {
                time_us: now_us,
                phase: "ActionAdmission".into(),
                event_kind: "support_dispatch".into(),
                active_actor: actor_index,
                hit_index: None,
                from_actor: Some(actor_index),
                to_actor: None,
                energy_change: None,
                energy_fanout: None,
                data: Some(GameEventPayload {
                    ability_id: Some(entry.ability_id.into()),
                    damage_effect_id: Some(entry.damage_effect_id.into()),
                    command_index: Some(command_index),
                    ..Default::default()
                }),
            });
            Ok(())
        }
        Ok(Some(failure)) => {
            skip(ctx, failure.message);
            Ok(())
        }
        Err(reason) => Err(reason),
    }
}
