use super::calculation::{CalculatedHit, RolledHit};
use super::eligibility::PreparedHit;
use super::*;

pub(super) struct HitOutcome {
    pub hit_index: usize,
    pub applied_hp_loss: u64,
    pub connected: bool,
    pub critical: bool,
    pub target_defeated: bool,
}

pub(super) fn apply_damage(
    ctx: &mut EventContext<'_>,
    delivery: &HitDelivery,
    prepared: PreparedHit,
    calculated: &CalculatedHit,
    rolled: RolledHit,
) -> Result<HitOutcome, String> {
    let HitIdentity {
        command_index,
        actor_index,
        target_index,
        ..
    } = delivery.identity;
    let ability_id = &delivery.ability_id;
    let damage_effect_id = &delivery.damage_effect_id;
    let atk_rate = delivery.atk_rate;
    let scaling_stat = delivery.scaling_stat;
    let metadata = &delivery.metadata;
    let request = ctx.request;
    let state = &mut *ctx.state;
    let (report_command_index, autonomous_event_index) =
        state.origins.identify(request, command_index)?;
    let trace = &mut *ctx.trace;
    let actor_config = &request.actors[actor_index];
    let target = &state.targets[target_index];
    let speculative_buffs = prepared.buffs;
    let non_crit = calculated.non_crit;
    let crit_damage = calculated.crit_damage;
    let rate = calculated.crit_rate;
    let critical = rolled.critical;
    let damage_exact = rolled.damage;
    let dmg_u64 = damage_exact.round_u64()?;
    let mut target_vitals = ntesim_core::state::VitalState::new(
        target.hp_current,
        target.hp_max,
        ntesim_core::state::ShieldOrder::OldestFirst,
    )
    .map_err(|error| format!("target vital state failed: {error:?}"))?;
    let settlement = target_vitals
        .damage(dmg_u64)
        .map_err(|error| format!("target damage settlement failed: {error:?}"))?;
    let applied_hp_loss = settlement.hp_lost;
    let target_defeated = settlement.died;
    for (source_config, source_actor) in request.actors.iter().zip(&state.actors) {
        crate::registry::validate_packet_delivery(
            source_config,
            source_actor,
            &metadata.packet,
            prepared.target_has_scorch,
            state
                .reactions
                .supports_dot_replacement(target_index, state.current_time_us),
            target_defeated,
        )?;
    }
    if !damage_exact.is_zero()
        && let Some(speculative) = speculative_buffs
    {
        state.buffs = speculative;
    }
    state.targets[target_index].hp_current = target_vitals.hp();
    crate::registry::validate_hit_target(
        request,
        actor_config,
        ability_id,
        &metadata.packet,
        &state.targets[target_index],
    )?;
    state.hits_settled = state
        .hits_settled
        .checked_add(1)
        .ok_or("settled hit count overflow")?;
    state.add_rounded_damage(dmg_u64)?;
    let hit_index = ctx.hits.len();
    let switch_origin = if let Some(parent) = delivery.identity.parent_hit_index {
        ctx.hits
            .get(parent)
            .ok_or("Follow-up damage parent missing")?
            .switch_origin
            .clone()
    } else {
        super::super::GameSwitchOrigin::support(request, command_index)
    };
    let (metadata_id, energy_group_id) = state.intern_action_packet_metadata(metadata)?;
    ctx.hits.push(GameHitReceipt {
        active_actor_index: state.active_actor_index,
        parent_hit_index: delivery.identity.parent_hit_index,
        damage_base_override: calculated.damage_base_override,
        autonomous_event_index,
        control_applications: Vec::new(),
        switch_origin,
        break_contribution: None,
        execution_model_assumption_ids: Vec::new(),
        command_index: report_command_index,
        static_general_damage_bonus: ExactFraction::from_ratio(
            actor_config.static_stats.damage_bonus,
        )?,
        static_element_damage_bonus: calculated.static_element_bonus,
        static_mental_damage_bonus: calculated.static_mental_bonus,
        time_us: state.current_time_us,
        actor_index,
        target_index,
        ability_id: ability_id.clone(),
        damage_effect_id: damage_effect_id.clone(),
        scaling_stat,
        scaling_rate: atk_rate,
        metadata_id,
        energy_group_id,
        non_crit_damage: non_crit,
        crit_damage,
        crit_rate: rate,
        critical,
        damage: damage_exact,
        rounded_damage: dmg_u64,
        applied_hp_loss,
        crit_sample: rolled.crit_sample,
    });

    if state.record_detail {
        let stacks = crate::gear::active_stacks(&state.buffs, actor_index);
        trace.push(GameEventReceipt {
            time_us: state.current_time_us,
            phase: "HitDelivery".into(),
            event_kind: "hit_damage".into(),
            active_actor: state.active_actor_index,
            hit_index: Some(hit_index),
            from_actor: None,
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: (stacks > 0).then(|| GameEventPayload {
                stacks: Some(stacks),
                ..Default::default()
            }),
        });
    }

    Ok(HitOutcome {
        hit_index,
        applied_hp_loss,
        target_defeated,
        critical,
        connected: !damage_exact.is_zero(),
    })
}

pub(super) fn after_damage(
    ctx: &mut EventContext<'_>,
    delivery: &HitDelivery,
    outcome: &HitOutcome,
    calculated: &CalculatedHit,
) -> Result<(), String> {
    let HitIdentity {
        command_index,
        actor_index,
        target_index,
        ..
    } = delivery.identity;
    let ability_id = &delivery.ability_id;
    let metadata = &delivery.metadata;
    let request = ctx.request;
    let actor_config = &request.actors[actor_index];
    let applied_hp_loss = outcome.applied_hp_loss;
    let target_defeated = outcome.target_defeated;
    let connected_hit = outcome.connected;
    let separate_parry_packet = metadata.packet.is_separate_parry();
    let arc_packet = metadata.packet.is_arc_chain();
    let mut execution_assumptions = calculated.assumptions.clone();
    execution_assumptions.extend(packet_assumptions(request, metadata)?);
    ctx.state
        .execution_model_assumptions
        .extend(execution_assumptions.clone());
    observe_positive_direct(ctx, delivery, outcome, separate_parry_packet, arc_packet)?;
    let state = &mut *ctx.state;
    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;
    let trace = &mut *ctx.trace;
    if !separate_parry_packet && !arc_packet {
        let active_gained = crate::simulation::energy::ordinary_hit_grants(
            request,
            &request.actors,
            &mut state.actors,
            state.active_actor_index,
            command_index,
            outcome.hit_index,
            actor_index,
            target_index,
            applied_hp_loss,
            metadata,
            &mut execution_assumptions,
            trace,
            state.current_time_us,
        )?;
        let active_actor = state.active_actor_index;
        crate::simulation::energy::notify_active_actor_energy_gain(
            &mut crate::runtime_api::CallbackContext {
                state,
                request,
                queue,
                sequence: seq,
                trace,
            },
            active_actor,
            active_gained,
        )?;
    }
    let mut control_applications = Vec::new();
    let (authored_command_index, autonomous_event_index) =
        state.origins.identify(request, command_index)?;
    let active_granted_before = state.actors[state.active_actor_index].granted_energy;
    if connected_hit && !separate_parry_packet && !arc_packet {
        crate::registry::on_hit(&mut crate::runtime_api::ActorHitContext {
            hit_index: outcome.hit_index,
            packet_group_id: metadata.binding.energy_group_id.as_deref(),
            authored_command_index,
            autonomous_event_index,
            target_alive: !target_defeated,
            control_applications: &mut control_applications,
            applied_hp_loss,
            active_actor_index: state.active_actor_index,
            command_index,
            request,
            execution_assumptions: &mut state.execution_model_assumptions,
            action_metadata: &metadata.action,
            target_level: request.targets[target_index].level,
            queue,
            sequence: seq,
            config: actor_config,
            actor_index,
            ability_id,
            damage_effect_id: &delivery.damage_effect_id,
            world_time_us: state.mode_clock.relative_us(),
            time_us: state.current_time_us,
            target_index,
            target_airborne: state.targets[target_index].airborne_until_us > state.current_time_us,
            buffs: &mut state.buffs,
            actor: &mut state.actors[actor_index],
            trace,
        })?;
        ctx.hits
            .get_mut(outcome.hit_index)
            .ok_or("control callback has no parent hit receipt")?
            .control_applications = control_applications.clone();
        let active_actor = ctx.state.active_actor_index;
        let active_gained = ctx.state.actors[active_actor].granted_energy > active_granted_before;
        crate::simulation::energy::notify_active_actor_energy_gain(
            &mut crate::runtime_api::CallbackContext {
                state: ctx.state,
                request: ctx.request,
                queue: ctx.queue,
                sequence: ctx.sequence,
                trace: ctx.trace,
            },
            active_actor,
            active_gained,
        )?;
    }
    if connected_hit && !separate_parry_packet {
        trigger_equipment_hit(
            ctx,
            delivery,
            outcome,
            &mut execution_assumptions,
            &control_applications,
        )?;
    }
    if target_defeated {
        observe_party_defeat(ctx, actor_index, target_index)?;
    }
    let state = &mut *ctx.state;
    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;
    let trace = &mut *ctx.trace;
    if target_defeated {
        crate::characters::team::on_target_defeated(request, state, target_index, trace)?;
        if !separate_parry_packet {
            for (config, actor) in request.actors.iter().zip(&mut state.actors) {
                crate::registry::on_target_defeated(config, actor, state.current_time_us)?;
            }
            crate::gear::on_target_defeated(
                &mut crate::runtime_api::CallbackContext {
                    state,
                    request,
                    queue,
                    sequence: seq,
                    trace,
                },
                actor_index,
                target_index,
            )?;
        }
        super::super::reactions::clear_target_statuses(&mut state.buffs, target_index);
    }
    if !separate_parry_packet {
        super::super::break_state::schedule_unbalance(
            state,
            queue,
            seq,
            command_index,
            actor_index,
            target_index,
            super::super::break_state::GaugeChange::for_packet(
                actor_config,
                &state.actors[actor_index],
                command_index,
                ability_id,
                metadata.packet.unbal_value,
            )?,
            connected_hit,
            target_defeated,
        )?;
    }
    if connected_hit && !target_defeated && !separate_parry_packet {
        crate::gear::on_broken_target_hit(
            &mut crate::runtime_api::CallbackContext {
                state,
                request,
                queue,
                sequence: seq,
                trace,
            },
            actor_index,
            target_index,
            &mut execution_assumptions,
        )?;
    }
    if connected_hit && !separate_parry_packet && !arc_packet {
        crate::registry::on_connected_packet(
            &mut crate::runtime_api::CallbackContext {
                state,
                request,
                queue,
                sequence: seq,
                trace,
            },
            actor_index,
            command_index,
            metadata.binding.energy_group_id.as_deref(),
            target_index,
            &delivery.damage_effect_id,
        )?;
    }
    let assumption_ids = ctx
        .state
        .record_receipt_assumptions(execution_assumptions)?;
    ctx.hits
        .get_mut(outcome.hit_index)
        .expect("delivered hit was recorded")
        .execution_model_assumption_ids = assumption_ids;
    Ok(())
}

fn observe_party_defeat(
    ctx: &mut EventContext<'_>,
    owner: usize,
    target: usize,
) -> Result<(), String> {
    crate::registry::on_party_target_defeated(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        owner,
        target,
    )
}

fn packet_assumptions(
    request: &crate::simulation::GameScenarioRequest,
    metadata: &crate::simulation::metadata::ActionPacketMetadata,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut assumptions = std::collections::BTreeMap::new();
    if metadata.packet.packet_kind != crate::simulation::metadata::PacketClassification::Standard
        && let crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id,
            description,
        } = &metadata.packet.provenance
    {
        crate::runtime_api::record_hypothesis(request, &mut assumptions, id, description)?;
    }
    Ok(assumptions)
}

fn observe_positive_direct(
    ctx: &mut EventContext<'_>,
    delivery: &HitDelivery,
    outcome: &HitOutcome,
    parry: bool,
    arc: bool,
) -> Result<(), String> {
    if outcome.applied_hp_loss == 0 || parry || arc {
        return Ok(());
    }
    crate::registry::observe_team_direct_hit(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        delivery.identity.actor_index,
        delivery.identity.target_index,
        &delivery.metadata,
    )
}

fn trigger_equipment_hit(
    ctx: &mut EventContext<'_>,
    delivery: &HitDelivery,
    outcome: &HitOutcome,
    execution_assumptions: &mut std::collections::BTreeMap<String, String>,
    controls: &[crate::simulation::GameControlApplication],
) -> Result<(), String> {
    let HitIdentity {
        command_index,
        actor_index,
        target_index,
        ..
    } = delivery.identity;
    let request = ctx.request;
    let state = &mut *ctx.state;
    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;
    let trace = &mut *ctx.trace;
    let actor_config = &request.actors[actor_index];
    let ability_id = &delivery.ability_id;
    let metadata = &delivery.metadata;
    let applied_hp_loss = outcome.applied_hp_loss;
    let mut hit_context = crate::runtime_api::CastContext {
        origins: &mut state.origins,
        command_index,
        native_array_index: None,
        execution_assumptions,
        config: actor_config,
        request,
        actor_index,
        active_actor_index: state.active_actor_index,
        target_index,
        ability_id,
        action_metadata: &metadata.action,
        packet_metadata: Some(&metadata.packet),
        hit_time_us: state.current_time_us,
        world_time_us: state.mode_clock.relative_us(),
        time_us: state.current_time_us,
        buffs: &mut state.buffs,
        actor: &mut state.actors[actor_index],
        stamina: state.combat_stamina.as_mut(),
        team_hp_snapshot: &[],
        targets: &state.targets,
        queue,
        sequence: seq,
    };
    if applied_hp_loss > 0 {
        crate::registry::on_hp_loss_hit(&mut hit_context)?;
        crate::gear::on_hp_loss_hit(
            &mut hit_context,
            metadata.binding.energy_group_id.as_deref(),
        )?;
    }
    for control in controls {
        crate::gear::on_control(&mut hit_context, control)?;
    }
    crate::gear::on_hit(&mut hit_context, outcome.critical, target_index)?;
    let callback_time_us = state.current_time_us;
    match &metadata.binding.kind {
        crate::simulation::metadata::ActionPacketBindingKind::ResearchDeclared => {
            crate::gear::on_research_hit(&mut hit_context)?;
        }
        crate::simulation::metadata::ActionPacketBindingKind::NativeDirect
        | crate::simulation::metadata::ActionPacketBindingKind::OwnerChild => {
            crate::gear::on_team_hit(
                &mut crate::runtime_api::CallbackContext {
                    state,
                    request,
                    queue,
                    sequence: seq,
                    trace,
                },
                actor_index,
                &metadata.packet,
                callback_time_us,
                execution_assumptions,
            )?;
        }
    }
    Ok(())
}
