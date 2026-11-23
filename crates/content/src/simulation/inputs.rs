use super::{GameActionCommand, GameEventPayload, GameEventReceipt, PlayerInput};
use ntesim_contracts::hypotheses::HypothesisId;

pub(crate) const TIMING_ID: HypothesisId = HypothesisId::InputsSpecialDodgeTimingV1;
pub(crate) const CRITICAL_DODGE_ID: HypothesisId =
    HypothesisId::InputsCriticalDodgeAssumedSuccessV1;

pub(super) fn validate_dash_directions(request: &super::GameScenarioRequest) -> Result<(), String> {
    for (&index, direction) in &request.dash_directions {
        let Some(GameActionCommand::PlayerInput {
            actor_index,
            input: PlayerInput::Dash,
            ..
        }) = request.actions.get(index)
        else {
            return Err(format!(
                "dash_directions index {index} must identify a Dash input"
            ));
        };
        if *direction == super::GameDashDirection::Back {
            crate::registry::validate_back_dash_binding(request, &request.actors[*actor_index])?;
            crate::runtime_api::record_hypothesis_id(
                request,
                &mut Default::default(),
                HypothesisId::InputsDashDirectionsV1,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn recovery(input: PlayerInput) -> u64 {
    match input {
        PlayerInput::Special => 800_000,
        PlayerInput::Dash => 500_000,
        PlayerInput::Arc => 0,
        PlayerInput::Jump => 0,
    }
}

pub(super) fn admit(
    ctx: &mut super::context::EventContext<'_>,
    command_index: usize,
    actor_index: usize,
    input: PlayerInput,
    critical_success: bool,
) -> Result<Option<super::rejection::AdmissionFailure>, String> {
    crate::registry::validate_player_input_state(
        ctx.request,
        &ctx.request.actors[actor_index],
        &ctx.state.actors[actor_index],
    )?;
    if input == PlayerInput::Arc {
        if critical_success {
            return Err("critical_success is only valid for dash inputs".into());
        }
        return admit_arc(ctx, command_index, actor_index);
    }
    if let Some(until_us) = crate::gear::arc::Arc::startup_recovery_deadline(
        &ctx.state.actors[actor_index],
        ctx.state.current_time_us,
    ) {
        let message = format!(
            "ARC_UNAVAILABLE: actor {actor_index}; Black Tome startup is pending until {until_us}us"
        );
        return Ok(Some(super::rejection::AdmissionFailure {
            reason: super::GameAdmissionReason::Recovery { until_us },
            message,
        }));
    }
    let config = &ctx.request.actors[actor_index];
    let mode_before =
        crate::characters::player_inputs::attack_mode(&ctx.state.actors[actor_index]).to_owned();
    crate::characters::player_inputs::validate(&config.character_id, input)?;
    let key = match input {
        PlayerInput::Special => "special",
        PlayerInput::Dash => "dash",
        PlayerInput::Jump => "jump",
        PlayerInput::Arc => unreachable!(),
    };
    let recovery_override = crate::registry::recovery_lock_override(
        ctx.request,
        config,
        &ctx.state.actors[actor_index],
        key,
        ctx.state.current_time_us,
    )?;
    if let Some(failure) = crate::registry::resource_admission_failure(
        ctx.request,
        command_index,
        config,
        &ctx.state.actors[actor_index],
        key,
        ctx.state.current_time_us,
        ctx.state.combat_stamina.as_ref(),
    ) {
        return Ok(Some(failure));
    }
    let timing = match ctx.state.prepare_ability_with_timing(
        actor_index,
        key,
        0,
        recovery(input),
        if input == PlayerInput::Dash && ctx.state.combat_stamina.is_some() {
            900_000
        } else {
            0
        },
        super::state::AbilityTimingOverrides::recovery(
            recovery_override.map(|(deadline, _)| deadline),
        ),
    ) {
        Ok(timing) => timing,
        Err(error) => return Ok(Some(error)),
    };
    if input == PlayerInput::Jump && recovery_override.is_none() {
        let message = "jump requires a currently active owner cancel window".to_owned();
        return Ok(Some(super::rejection::AdmissionFailure {
            reason: super::GameAdmissionReason::Other {
                message: message.clone(),
            },
            message,
        }));
    }
    let mut assumptions = std::collections::BTreeMap::new();
    if let Some(failure) = prepare_stamina(ctx, command_index, input, &mut assumptions)? {
        return Ok(Some(failure));
    }
    if input != PlayerInput::Dash && critical_success {
        return Err("critical_success is only valid for dash inputs".into());
    }
    if critical_success {
        prepare_critical_dodge(ctx, actor_index, &mut assumptions)?;
    }
    if let Some((command, metadata)) = special_action(ctx, actor_index, input)? {
        let result = super::admission::admit_ability(
            ctx,
            command_index,
            &command,
            Some(metadata),
            super::admission::AdmissionOrigin::Authored,
        )?;
        if result.is_none() {
            record(
                ctx,
                command_index,
                actor_index,
                input,
                critical_success,
                mode_before,
                assumptions,
            )?;
        }
        return Ok(result);
    }
    if config.character_id == "character:1004" {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut assumptions,
            crate::characters::lacrimosa::skill::FORM_HYPOTHESIS_ID,
        )?;
    } else if crate::characters::player_inputs::aim_packet(&config.character_id).is_some() {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut assumptions,
            crate::characters::player_inputs::AIM_HYPOTHESIS_ID,
        )?;
    }
    let attack_mode_after = commit_mode_input(
        ctx,
        command_index,
        actor_index,
        input,
        timing.locked_until_us,
    )?;
    crate::registry::on_committed_player_input(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        command_index,
        actor_index,
        input,
    )?;
    if critical_success {
        dispatch_critical_dodge(ctx, command_index, actor_index, &mut assumptions)?;
    }
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ActionAdmission".into(),
        event_kind: key.into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            attack_mode: Some(attack_mode_after),
            ..Default::default()
        }),
    });
    record(
        ctx,
        command_index,
        actor_index,
        input,
        critical_success,
        mode_before,
        assumptions,
    )?;
    Ok(None)
}

fn special_action(
    ctx: &super::context::EventContext<'_>,
    actor_index: usize,
    input: PlayerInput,
) -> Result<Option<(GameActionCommand, super::metadata::ActionPacketMetadata)>, String> {
    let Some((ability, packet)) = (input == PlayerInput::Special)
        .then(|| crate::characters::player_inputs::special_ability(&ctx.state.actors[actor_index]))
        .flatten()
    else {
        return Ok(None);
    };
    let config = &ctx.request.actors[actor_index];
    let array_index = normal_coordinate(config, ability)?;
    let (atk_rate, scaling_stat) = coefficient(ctx.request, config, ability, packet, array_index)?;
    let command = GameActionCommand::InvokeAbility {
        timestamp_us: ctx.state.current_time_us,
        actor_index,
        ability_id: ability.into(),
        array_index,
        damage_effect_id: packet.into(),
        atk_rate,
        scaling_stat,
        energy_cost: 0,
        recovery_us: recovery(input),
        cooldown_us: 0,
        hit_delay_us: 0,
    };
    let metadata = super::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        ability,
        packet,
        None,
        &ctx.request.model_assumptions,
    )?;
    Ok(Some((command, metadata)))
}

fn prepare_stamina(
    ctx: &super::context::EventContext<'_>,
    command_index: usize,
    input: PlayerInput,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<Option<super::rejection::AdmissionFailure>, String> {
    if ctx.request.dash_directions.get(&command_index) == Some(&super::GameDashDirection::Back) {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            assumptions,
            HypothesisId::InputsDashDirectionsV1,
        )?;
    }
    if let Some(model) = crate::registry::validate_player_input(
        ctx.request,
        &ctx.request.actors[ctx.state.active_actor_index],
        input,
    )? {
        crate::runtime_api::record_hypothesis_id(ctx.request, assumptions, model)?;
    }
    if input == PlayerInput::Jump {
        return Ok(None);
    }
    if input == PlayerInput::Dash
        && let Some(stamina) = &ctx.state.combat_stamina
    {
        if let Some(reason) = stamina.prepare_dodge(ctx.state.current_time_us)? {
            return Ok(Some(super::rejection::AdmissionFailure {
                reason,
                message: "insufficient combat stamina for dash".into(),
            }));
        }
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            assumptions,
            super::stamina::HYPOTHESIS_ID,
        )?;
    } else {
        crate::runtime_api::record_hypothesis_id(ctx.request, assumptions, TIMING_ID)?;
    }
    Ok(None)
}

fn prepare_critical_dodge(
    ctx: &super::context::EventContext<'_>,
    actor_index: usize,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(ctx.request, assumptions, CRITICAL_DODGE_ID)?;
    if let Some(model) =
        crate::registry::critical_dodge_model(ctx.request, &ctx.request.actors[actor_index])
    {
        if ctx.state.combat_stamina.is_none() {
            return Err("owner Critical Dodge requires an explicit combat stamina profile".into());
        }
        crate::runtime_api::record_hypothesis_id(ctx.request, assumptions, model)?;
    }
    Ok(())
}

fn commit_mode_input(
    ctx: &mut super::context::EventContext<'_>,
    command_index: usize,
    actor_index: usize,
    input: PlayerInput,
    locked_until_us: u64,
) -> Result<String, String> {
    let time = ctx.state.current_time_us;
    if input == PlayerInput::Dash
        && let Some(stamina) = &mut ctx.state.combat_stamina
    {
        stamina.spend_dodge(command_index, actor_index, time)?;
        ctx.state.actors[actor_index]
            .ability_cooldowns
            .insert("dash".into(), time + 900_000);
    }
    let actor = &mut ctx.state.actors[actor_index];
    actor.locked_until_us = locked_until_us;
    actor.last_action_start_us = time;
    crate::characters::player_inputs::apply(actor, input, ctx.request);
    crate::registry::on_player_input(ctx.request, &ctx.request.actors[actor_index], actor, input)?;
    Ok(crate::characters::player_inputs::attack_mode(actor).to_owned())
}

fn admit_arc(
    ctx: &mut super::context::EventContext<'_>,
    command_index: usize,
    actor_index: usize,
) -> Result<Option<super::rejection::AdmissionFailure>, String> {
    let config = &ctx.request.actors[actor_index];
    let arc = crate::gear::arc::Arc::project(config)?;
    let ability_id = arc.ability_id();
    let timing =
        match ctx
            .state
            .prepare_ability_admission(actor_index, ability_id, 0, 0, arc.cooldown_us())
        {
            Ok(timing) => timing,
            Err(error) => return Ok(Some(error)),
        };
    if let Some(failure) = arc.check_admission(
        actor_index,
        &ctx.state.actors[actor_index],
        ctx.state.current_time_us,
    ) {
        return Ok(Some(failure));
    }
    let mut assumptions = std::collections::BTreeMap::new();
    crate::runtime_api::record_hypothesis_id(ctx.request, &mut assumptions, arc.hypothesis())?;
    let mode =
        crate::characters::player_inputs::attack_mode(&ctx.state.actors[actor_index]).to_owned();
    let actor = &mut ctx.state.actors[actor_index];
    actor
        .ability_cooldowns
        .insert(ability_id.into(), timing.ready_at_us);
    actor.last_action_start_us = ctx.state.current_time_us;
    crate::registry::on_player_input(ctx.request, config, actor, PlayerInput::Arc)?;
    crate::registry::on_committed_player_input(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        command_index,
        actor_index,
        PlayerInput::Arc,
    )?;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ActionAdmission".into(),
        event_kind: "arc".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            ability_id: Some(ability_id.into()),
            ..Default::default()
        }),
    });
    arc.activate(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        command_index,
        actor_index,
        timing.ready_at_us,
    )?;
    let receipt_index = record(
        ctx,
        command_index,
        actor_index,
        PlayerInput::Arc,
        false,
        mode,
        assumptions,
    )?;
    ctx.state.input_receipts[receipt_index].ability_id = Some(ability_id.into());
    Ok(None)
}

fn record(
    ctx: &mut super::context::EventContext<'_>,
    command_index: usize,
    actor_index: usize,
    input: PlayerInput,
    critical_success: bool,
    mode_before: String,
    assumptions: std::collections::BTreeMap<String, String>,
) -> Result<usize, String> {
    let execution_model_assumption_ids = ctx.state.record_receipt_assumptions(assumptions)?;
    let receipt_index = ctx.state.input_receipts.len();
    ctx.state.input_receipts.push(super::PlayerInputReceipt {
        command_index,
        actor_index,
        ability_id: None,
        input,
        critical_success,
        time_us: ctx.state.current_time_us,
        attack_mode_before: mode_before,
        attack_mode_after: crate::characters::player_inputs::attack_mode(
            &ctx.state.actors[actor_index],
        )
        .into(),
        execution_model_assumption_ids,
    });
    Ok(receipt_index)
}

fn dispatch_critical_dodge(
    ctx: &mut super::context::EventContext<'_>,
    command_index: usize,
    actor_index: usize,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let current_time_us = state.current_time_us;
    let active_actor_index = state.active_actor_index;
    let mut gear_assumptions = std::collections::BTreeMap::new();
    let action_metadata = crate::simulation::metadata::ActionMetadata {
        categories: std::collections::BTreeSet::new(),
        gameplay_tags: vec![crate::weapons::thief_candy::CRITICAL_DODGE_TAG.into()],
        ability_id: "player_input_dash".into(),
        native_ability_index: None,
        input_role: None,
        input_role_provenance: None,
        primary_slot: None,
        source_evidence: None,
        identity_evidence: None,
        provenance: crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: CRITICAL_DODGE_ID.as_str().into(),
            description: CRITICAL_DODGE_ID.description().into(),
        },
    };
    let mut cast = crate::runtime_api::CastContext {
        origins: &mut state.origins,
        command_index,
        native_array_index: None,
        execution_assumptions: &mut gear_assumptions,
        config: &ctx.request.actors[actor_index],
        request: ctx.request,
        actor_index,
        active_actor_index,
        target_index: ctx.request.aim_target(command_index),
        ability_id: "player_input_dash",
        action_metadata: &action_metadata,
        packet_metadata: None,
        hit_time_us: current_time_us,
        world_time_us: state.mode_clock.relative_us(),
        time_us: current_time_us,
        buffs: &mut state.buffs,
        actor: &mut state.actors[actor_index],
        stamina: state.combat_stamina.as_mut(),
        team_hp_snapshot: &[],
        targets: &state.targets,
        queue: ctx.queue,
        sequence: ctx.sequence,
    };
    crate::registry::on_critical_dodge(&mut cast)?;
    crate::weapons::thief_candy::on_critical_dodge(&mut cast)?;
    assumptions.extend(gear_assumptions);
    Ok(())
}

pub(crate) fn normal_coordinate(
    config: &super::GameActorConfig,
    ability: &str,
) -> Result<usize, String> {
    let purchased = config
        .talent_levels
        .map(|levels| levels[0])
        .unwrap_or(config.purchased_skill_level);
    let (bonus, _) = crate::characters::resonance::bonus(
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        ability,
        &config.awakening,
    )?;
    usize::try_from(
        purchased
            .checked_add(bonus)
            .and_then(|level| level.checked_sub(1))
            .ok_or("input talent coordinate overflow")?,
    )
    .map_err(|_| "input talent coordinate overflow".into())
}

pub(crate) fn coefficient(
    request: &super::GameScenarioRequest,
    config: &super::GameActorConfig,
    ability: &str,
    packet: &str,
    index: usize,
) -> Result<(crate::formula::Ratio, crate::simulation::NativeScalingStat), String> {
    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &request.content_id,
            character_id: &config.character_id,
            ability_id: ability,
            damage_effect_id: packet,
            array_index: index.try_into().map_err(|_| "input coordinate overflow")?,
            research_metadata: None,
            model_assumptions: &request.model_assumptions,
        },
    )?;
    let rate = serde_json::from_value(projection["compiled_rate"].clone())
        .map_err(|error| error.to_string())?;
    let stat = serde_json::from_value(projection["compiled_scaling_stat"].clone())
        .map_err(|error| error.to_string())?;
    Ok((rate, stat))
}
