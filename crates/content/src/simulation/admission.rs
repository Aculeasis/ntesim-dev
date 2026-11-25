use super::events::{GameEventData, ScheduledGameEvent};
use super::{
    ExactFraction, GameActionCommand, GameEventPayload, GameEventReceipt, NativeScalingStat,
};
use crate::execution::CausalPriority;
use crate::formula::Ratio;
mod normal_input;
mod utility;

/// Whether an admitted invocation came from the rotation or from the
/// engine-owned Support dispatch on a switch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AdmissionOrigin {
    Authored,
    SupportDispatch,
}

pub(super) fn admit(
    ctx: &mut super::context::EventContext<'_>,
    command_index: usize,
) -> Result<Option<super::rejection::AdmissionFailure>, String> {
    if let GameActionCommand::TargetGauge {
        target_index,
        gauge,
        ..
    } = ctx.request.actions[command_index]
    {
        return super::break_state::admit_authored(ctx, command_index, target_index, gauge);
    }
    if let GameActionCommand::PlayerInput {
        actor_index,
        input,
        critical_success,
        ..
    } = ctx.request.actions[command_index]
    {
        return super::inputs::admit(ctx, command_index, actor_index, input, critical_success);
    }
    let request = ctx.request;
    admit_ability(
        ctx,
        command_index,
        &request.actions[command_index],
        None,
        AdmissionOrigin::Authored,
    )
}

fn unpack_command(command: &GameActionCommand) -> AdmittedInvocation {
    match command {
        GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            array_index,
            damage_effect_id,
            atk_rate,
            scaling_stat,
            energy_cost,
            recovery_us,
            cooldown_us,
            hit_delay_us,
            ..
        } => AdmittedInvocation {
            actor_idx: *actor_index,
            ability_id: ability_id.clone(),
            arr_idx: *array_index,
            damage_effect_id: damage_effect_id.clone(),
            atk_rate: *atk_rate,
            scaling_stat: *scaling_stat,
            energy_cost: *energy_cost,
            recovery_us: *recovery_us,
            cooldown_us: *cooldown_us,
            hit_delay_us: *hit_delay_us,
            utility: None,
        },
        _ => unreachable!(),
    }
}

struct AdmittedInvocation {
    actor_idx: usize,
    ability_id: String,
    arr_idx: usize,
    damage_effect_id: String,
    atk_rate: Ratio,
    scaling_stat: NativeScalingStat,
    energy_cost: u64,
    recovery_us: u64,
    cooldown_us: u64,
    hit_delay_us: u64,
    utility: Option<crate::runtime_api::UtilityInvocation>,
}

struct EnergySpendTrace<'a> {
    actor_idx: usize,
    command_index: usize,
    ability_id: &'a str,
    cost: u64,
    energy_before: u64,
    energy_after: u64,
    charge_before: ExactFraction,
    charge_after: ExactFraction,
    utility_id: Option<&'a str>,
}

fn resolve_action_selection(
    request: &super::GameScenarioRequest,
    command_index: usize,
    config: &super::GameActorConfig,
    actor: &super::state::ActorRuntime,
    time_us: u64,
    cmd: &mut AdmittedInvocation,
    prepared: &super::metadata::ActionPacketMetadata,
) -> Result<
    Result<Option<super::metadata::ActionPacketMetadata>, super::rejection::AdmissionFailure>,
    String,
> {
    if let Some(selected) =
        crate::registry::utility_invocation(config, actor, &cmd.ability_id, time_us)
    {
        return Ok(Ok(Some(utility::select(request, cmd, prepared, selected)?)));
    }
    let held = match crate::registry::held_normal_override(request, command_index, actor, time_us) {
        Ok(selected) => selected,
        Err(message) => {
            return Ok(Err(super::rejection::AdmissionFailure {
                reason: super::GameAdmissionReason::Other {
                    message: message.clone(),
                },
                message,
            }));
        }
    };
    let selected = normal_input::select(
        request,
        command_index,
        config,
        held.or_else(|| crate::registry::normal_action_override(config, actor)),
        &cmd.ability_id,
        cmd.arr_idx,
        prepared,
    )?;
    if let Some(selection) = &selected {
        cmd.damage_effect_id = selection.metadata.packet.damage_effect_id.clone();
        cmd.atk_rate = selection.rate;
        cmd.scaling_stat = selection.scaling;
        if let Some((recovery, delay)) = selection.timing {
            cmd.recovery_us = recovery;
            cmd.hit_delay_us = delay;
        }
    }
    Ok(Ok(selected.map(|s| s.metadata)))
}

fn apply_actor_admission_lock(
    actor: &mut super::state::ActorRuntime,
    ability_id: &str,
    primary_slot: Option<super::metadata::PrimaryActionSlot>,
    energy_cost: u64,
    timing: ntesim_core::kernel::action::TimingAdmission,
    time_us: u64,
    preserve_cooldown: bool,
) -> Result<(u64, ExactFraction), String> {
    if matches!(
        primary_slot,
        Some(
            super::metadata::PrimaryActionSlot::Skill
                | super::metadata::PrimaryActionSlot::Ultimate
        )
    ) {
        actor.aiming = false;
    }
    let energy_before = actor.energy.current();
    let charge_remainder = actor.charge_remainder;
    actor.spend_energy(energy_cost)?;
    actor.locked_until_us = timing.locked_until_us;
    actor.single_support_start_us = None;
    if !preserve_cooldown {
        actor
            .ability_cooldowns
            .insert(ability_id.to_string(), timing.ready_at_us);
    }
    actor.last_action_start_us = time_us;
    Ok((energy_before, charge_remainder))
}

fn record_admission_trace(
    trace: &mut Vec<GameEventReceipt>,
    time_us: u64,
    active_actor: usize,
    spend: EnergySpendTrace<'_>,
) {
    trace.push(GameEventReceipt {
        time_us,
        phase: "ActionAdmission".into(),
        event_kind: if spend.utility_id.is_some() {
            "utility_ability"
        } else {
            "invoke_ability"
        }
        .into(),
        active_actor,
        hit_index: None,
        from_actor: Some(spend.actor_idx),
        to_actor: None,
        energy_change: Some(super::GameEnergyReceipt::Spend(
            super::GameEnergySpendReceipt {
                source: super::GameEnergySpendSource::AbilityCost,
                source_actor_index: spend.actor_idx,
                recipient_actor_index: spend.actor_idx,
                command_index: spend.command_index,
                ability_id: spend.ability_id.to_string(),
                energy_before: spend.energy_before,
                spent: spend.cost,
                energy_after: spend.energy_after,
                charge_remainder_before: (!spend.charge_before.is_zero())
                    .then_some(spend.charge_before),
                charge_remainder_after: (!spend.charge_after.is_zero())
                    .then_some(spend.charge_after),
            },
        )),
        energy_fanout: None,
        data: spend.utility_id.map(|utility| GameEventPayload {
            utility_id: Some(utility.to_string()),
            ..Default::default()
        }),
    });
}

fn enqueue_hit_delivery(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    seq: &mut u64,
    time_us: u64,
    command_index: usize,
    cmd: &AdmittedInvocation,
    target_index: usize,
    metadata: Box<super::metadata::ActionPacketMetadata>,
) -> Result<u64, String> {
    let hit_time_us = time_us
        .checked_add(cmd.hit_delay_us)
        .ok_or("hit delivery deadline overflow")?;
    *seq = seq.checked_add(1).ok_or("event sequence overflow")?;
    queue.push(ScheduledGameEvent {
        time_us: hit_time_us,
        priority: CausalPriority::HitDelivery,
        seq: *seq,
        data: GameEventData::HitDelivery {
            command_index,
            actor_index: cmd.actor_idx,
            target_index,
            ability_id: cmd.ability_id.clone(),
            damage_effect_id: cmd.damage_effect_id.clone(),
            atk_rate: cmd.atk_rate,
            scaling_stat: cmd.scaling_stat,
            metadata,
        },
    });
    Ok(hit_time_us)
}

pub(super) fn admit_ability(
    ctx: &mut super::context::EventContext<'_>,
    command_index: usize,
    command: &GameActionCommand,
    resolved: Option<super::metadata::ActionPacketMetadata>,
    origin: AdmissionOrigin,
) -> Result<Option<super::rejection::AdmissionFailure>, String> {
    let request = ctx.request;
    let state = &mut *ctx.state;
    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;
    let trace = &mut *ctx.trace;
    let mut cmd = unpack_command(command);

    if let Some(until_us) = crate::gear::arc::Arc::startup_recovery_deadline(
        &state.actors[cmd.actor_idx],
        state.current_time_us,
    ) {
        let message = format!(
            "ARC_UNAVAILABLE: actor {}; Black Tome startup is pending until {until_us}us",
            cmd.actor_idx
        );
        return Ok(Some(super::rejection::AdmissionFailure {
            reason: super::GameAdmissionReason::Recovery { until_us },
            message,
        }));
    }

    let config = &request.actors[cmd.actor_idx];
    crate::registry::validate_ability_admission(
        request,
        config,
        &state.actors[cmd.actor_idx],
        &cmd.ability_id,
        state.current_time_us,
    )?;

    let prepared = resolved
        .as_ref()
        .or_else(|| {
            ctx.prepared_packets
                .get(command_index)
                .and_then(Option::as_ref)
        })
        .ok_or("prepared ability packet missing")?;
    let selected_override = match resolve_action_selection(
        request,
        command_index,
        config,
        &state.actors[cmd.actor_idx],
        state.current_time_us,
        &mut cmd,
        prepared,
    )? {
        Ok(opt) => opt,
        Err(failure) => return Ok(Some(failure)),
    };
    let metadata = selected_override.as_ref().unwrap_or(prepared);

    validate_packet(
        config,
        &state.actors[cmd.actor_idx],
        &cmd.ability_id,
        metadata,
        cmd.scaling_stat,
        state.current_time_us,
    )?;

    let recovery_override = crate::registry::held_recovery_override(
        request,
        &state.actors[cmd.actor_idx],
        command_index,
        state.current_time_us,
    )?
    .or(crate::registry::recovery_lock_override(
        request,
        config,
        &state.actors[cmd.actor_idx],
        &cmd.ability_id,
        state.current_time_us,
    )?);
    let timing = match state.prepare_ability_with_timing(
        cmd.actor_idx,
        &cmd.ability_id,
        cmd.energy_cost,
        cmd.recovery_us,
        cmd.cooldown_us,
        super::state::AbilityTimingOverrides {
            recovery_lock: recovery_override.map(|(deadline, _)| deadline),
            ready_at: cmd.utility.map(|action| action.ready_at_us).or_else(|| {
                crate::registry::cooldown_override(
                    config,
                    &state.actors[cmd.actor_idx],
                    &cmd.ability_id,
                    &cmd.damage_effect_id,
                )
            }),
        },
    ) {
        Ok(timing) => timing,
        Err(error) => return Ok(Some(error)),
    };

    if let Some(failure) = crate::registry::resource_admission_failure(
        request,
        command_index,
        config,
        &state.actors[cmd.actor_idx],
        &cmd.ability_id,
        state.current_time_us,
        state.combat_stamina.as_ref(),
    ) {
        return Ok(Some(failure));
    }
    let mut team_hp_buf = [(0u64, 0u64); 4];
    let team_hp_len = team_hp_snapshot(state, &mut team_hp_buf)?;
    let team_hp_snapshot = &team_hp_buf[..team_hp_len];

    let actor = &mut state.actors[cmd.actor_idx];
    let (energy_before, charge_remainder) = apply_actor_admission_lock(
        actor,
        &cmd.ability_id,
        metadata.action.primary_slot,
        cmd.energy_cost,
        timing,
        state.current_time_us,
        cmd.utility.is_some()
            || crate::registry::preserves_cooldown(config, &cmd.ability_id, &cmd.damage_effect_id),
    )?;

    record_admission_trace(
        trace,
        state.current_time_us,
        state.active_actor_index,
        EnergySpendTrace {
            actor_idx: cmd.actor_idx,
            command_index,
            ability_id: &cmd.ability_id,
            cost: cmd.energy_cost,
            energy_before,
            energy_after: actor.energy.current(),
            charge_before: charge_remainder,
            charge_after: actor.charge_remainder,
            utility_id: cmd.utility.map(|action| action.ability_id),
        },
    );

    let hit_time_us = enqueue_hit_delivery(
        queue,
        seq,
        state.current_time_us,
        command_index,
        &cmd,
        request.aim_target(command_index),
        Box::new(metadata.clone()),
    )?;

    let actor_config = &request.actors[cmd.actor_idx];
    let mut execution_assumptions = std::collections::BTreeMap::new();
    if let Some((_, model)) = recovery_override {
        crate::runtime_api::record_hypothesis_id(request, &mut execution_assumptions, model)?;
    }
    if let Some(action) = cmd.utility {
        crate::runtime_api::record_hypothesis_id(
            request,
            &mut execution_assumptions,
            action.model,
        )?;
    }
    let mut cast_ctx = crate::runtime_api::CastContext {
        origins: &mut state.origins,
        command_index,
        native_array_index: Some(cmd.arr_idx),
        execution_assumptions: &mut execution_assumptions,
        config: actor_config,
        request,
        actor_index: cmd.actor_idx,
        active_actor_index: state.active_actor_index,
        target_index: request.aim_target(command_index),
        ability_id: &cmd.ability_id,
        action_metadata: &metadata.action,
        packet_metadata: Some(&metadata.packet),
        hit_time_us,
        world_time_us: state.mode_clock.relative_us(),
        time_us: state.current_time_us,
        buffs: &mut state.buffs,
        actor,
        stamina: state.combat_stamina.as_mut(),
        team_hp_snapshot,
        targets: &state.targets,
        queue,
        sequence: seq,
    };
    if cmd.utility.is_none() {
        crate::gear::on_cast(&mut cast_ctx)?;
    }
    crate::registry::on_cast(&mut cast_ctx)?;

    finish_admission(
        ctx,
        command_index,
        &cmd,
        metadata,
        execution_assumptions,
        origin,
    )
}

fn finish_admission(
    ctx: &mut super::context::EventContext<'_>,
    command_index: usize,
    cmd: &AdmittedInvocation,
    metadata: &super::metadata::ActionPacketMetadata,
    mut execution_assumptions: std::collections::BTreeMap<String, String>,
    origin: AdmissionOrigin,
) -> Result<Option<super::rejection::AdmissionFailure>, String> {
    let request = ctx.request;
    let state = &mut *ctx.state;
    let is_ultimate_cinematic = metadata.action.primary_slot
        == Some(super::metadata::PrimaryActionSlot::Ultimate)
        && cmd.damage_effect_id != "GE_Player_Shinku_UltraSkillPre_Rage_Damage"
        && cmd.damage_effect_id != "GE_Player_Akane_UltraSkill2_Damage";
    super::pause::committed(
        state,
        request,
        ctx.queue,
        ctx.sequence,
        cmd.actor_idx,
        command_index,
        is_ultimate_cinematic.then_some(cmd.ability_id.as_str()),
    )?;
    if cmd.utility.is_none() {
        crate::characters::team::observe_admission(
            request,
            state,
            cmd.actor_idx,
            command_index,
            &cmd.ability_id,
            &metadata.action,
            &mut execution_assumptions,
            ctx.trace,
            ctx.queue,
            ctx.sequence,
        )?;
    }
    ctx.action_metadata_ledger.push(admission_receipt(
        request,
        state,
        command_index,
        cmd.actor_idx,
        &cmd.ability_id,
        metadata,
        execution_assumptions,
    )?);
    if cmd.utility.is_some() {
        return Ok(None);
    }
    crate::gear::observe_team_admission(
        request,
        state,
        cmd.actor_idx,
        command_index,
        &metadata.action,
    )?;
    crate::registry::observe_recorded_admission(
        request,
        state,
        ctx.trace,
        origin,
        crate::registry::RecordedAdmission {
            owner: cmd.actor_idx,
            command_index,
            ability_id: &cmd.ability_id,
            array_index: cmd.arr_idx,
            damage_effect_id: &cmd.damage_effect_id,
            atk_rate: cmd.atk_rate,
            scaling_stat: cmd.scaling_stat,
            hit_delay_us: cmd.hit_delay_us,
            metadata,
        },
    )?;
    Ok(None)
}

fn validate_packet(
    config: &super::GameActorConfig,
    actor: &super::state::ActorRuntime,
    ability_id: &str,
    metadata: &super::metadata::ActionPacketMetadata,
    scaling_stat: super::NativeScalingStat,
    time_us: u64,
) -> Result<(), String> {
    if scaling_stat.uses_hp()
        && !crate::build::supports_hp_packet(
            &config.character_id,
            ability_id,
            &metadata.packet.damage_effect_id,
        )
    {
        return Err("HP scaling is unsupported for this native owner/action/packet".into());
    }
    crate::registry::validate_packet_admission(
        config,
        actor,
        ability_id,
        &metadata.packet.damage_effect_id,
        time_us,
    )
}

pub(crate) fn team_hp_snapshot(
    state: &super::state::GameSimulatorState,
    out: &mut [(u64, u64); 4],
) -> Result<usize, String> {
    let count = state.actors.len().min(4);
    for (index, actor) in state.actors.iter().enumerate().take(count) {
        let max_hp = crate::simulation::state::effective_hp_max(
            &state.buffs,
            actor.hp_max_base,
            &actor.hp_max_static_bonus,
            index,
            actor.hp_max,
        )?;
        out[index] = (actor.hp, max_hp);
    }
    Ok(count)
}

fn admission_receipt(
    request: &super::GameScenarioRequest,
    state: &mut super::state::GameSimulatorState,
    command_index: usize,
    actor_index: usize,
    ability_id: &str,
    metadata: &super::metadata::ActionPacketMetadata,
    execution_model_assumptions: std::collections::BTreeMap<String, String>,
) -> Result<super::ActionMetadataOccurrence, String> {
    let execution_model_assumption_ids =
        state.record_receipt_assumptions(execution_model_assumptions)?;
    let (metadata_id, energy_group_id) = state.intern_action_packet_metadata(metadata)?;
    Ok(super::ActionMetadataOccurrence {
        switch_origin: super::GameSwitchOrigin::support(request, command_index),
        command_indices: vec![command_index],
        actor_index,
        ability_id: ability_id.into(),
        damage_effect_id: metadata.packet.damage_effect_id.clone(),
        metadata_id,
        energy_group_id,
        execution_model_assumption_ids,
    })
}
