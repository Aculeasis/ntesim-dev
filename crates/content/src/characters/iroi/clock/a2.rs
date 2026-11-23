//! Awakening 2 assist attack and team heal scheduling.
use crate::runtime_api::CallbackContext;
use crate::simulation::{
    ExactFraction, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    metadata::InputActionRole,
    state::GameSimulatorState,
};
use ntesim_core::execution::CausalPriority;

pub(crate) fn observe_team_admission(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    actor_index: usize,
    command_index: usize,
    metadata: &crate::simulation::metadata::ActionMetadata,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
) -> Result<(), String> {
    let Some(iroi_idx) = request
        .actors
        .iter()
        .position(|a| a.character_id == "character:1075")
    else {
        return Ok(());
    };
    if !super::accepted(request) || !request.actors[iroi_idx].awakening.has_effect(2) {
        return Ok(());
    }
    if state.actors[iroi_idx].hp == 0 || state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    let is_eligible = matches!(
        metadata.input_role,
        Some(InputActionRole::RedirectSkill | InputActionRole::Ultimate | InputActionRole::Qte)
    ) || matches!(
        metadata.primary_slot,
        Some(
            crate::simulation::metadata::PrimaryActionSlot::Skill
                | crate::simulation::metadata::PrimaryActionSlot::Ultimate
        )
    );
    if !is_eligible {
        return Ok(());
    }

    let now = state.current_time_us;
    let iroi_state = state.actors[iroi_idx].character.iroi_mut()?;
    if now < iroi_state.a2_cooldown_until_us {
        return Ok(());
    }
    iroi_state.a2_cooldown_until_us = now
        .checked_add(5_000_000)
        .ok_or("Iroi A2 cooldown overflow")?;

    let stream = format!("character:1075:{iroi_idx}:a2");
    if iroi_state.a2_rng.is_none() {
        iroi_state.a2_rng = Some(ntesim_core::kernel::rng::Rng::new(
            request.seed,
            std::collections::BTreeMap::from([(stream.clone(), 1_075_002)]),
        )?);
    }
    let choice = iroi_state
        .a2_rng
        .as_mut()
        .unwrap()
        .uniform(&stream, 3)?
        .value as usize;

    let delay = match choice {
        0 => 1_200_000,
        1 => 300_000,
        _ => 550_000,
    };

    let target_time = now
        .checked_add(delay)
        .ok_or("Iroi A2 actor spawn deadline overflow")?;
    super::push(
        queue,
        sequence,
        target_time,
        super::Event::A2Creation {
            iroi_idx,
            candidate: choice,
            command_index,
        },
    )?;

    Ok(())
}

pub(super) fn creation(
    ctx: &mut CallbackContext<'_>,
    iroi_idx: usize,
    candidate: usize,
    command_index: usize,
) -> Result<(), String> {
    if ctx.state.actors[iroi_idx].hp == 0 {
        return Ok(());
    }

    let trigger = match ctx.request.actions.get(command_index) {
        Some(crate::simulation::GameActionCommand::SwitchActor { .. }) => {
            crate::simulation::GameCausalTrigger::FieldEntry { command_index }
        }
        _ => crate::simulation::GameCausalTrigger::Action { command_index },
    };
    let key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: iroi_idx,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: "GA_Oneiroi_Skill".into(),
            time_us: ctx.state.current_time_us,
            trigger,
            recipient_healing: None,
        },
    )?;
    let (_, Some(autonomous_index)) = ctx.state.origins.identify(ctx.request, key)? else {
        return Err("Iroi A2 autonomous identity missing".into());
    };

    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "iroi_a2_assist".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(iroi_idx),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            slot: Some(candidate),
            event_index: Some(autonomous_index),
            ..Default::default()
        }),
    });

    // 1. Team heal: 1.20 * live Iroi final ATK per living actual party member
    let atk_ratio = ExactFraction::new(6, 5)?;
    let mut recipients: Vec<usize> = (0..ctx.request.actors.len()).collect();
    recipients.sort_by_key(|&idx| (ctx.request.actors[idx].slot, idx));
    for recipient in recipients {
        let is_regressed = ctx.state.actors[iroi_idx]
            .character
            .iroi()?
            .regressed_allies
            .contains_key(&recipient);
        if is_regressed || ctx.state.actors[recipient].hp == 0 {
            continue;
        }
        super::super::callback::apply_team_heal(
            ctx,
            iroi_idx,
            recipient,
            atk_ratio,
            "iroi_a2_heal",
            "Skill",
        )?;
    }

    // 2. Schedule damage packets
    let now = ctx.state.current_time_us;
    let (effect, count, interval) = match candidate {
        0 => ("GE_Damage_sheep_gun_awaken2", 1, 0),
        1 => ("GE_Damage_sheep_syringe_awaken2", 7, 100_000),
        _ => ("GE_Damage_sheep_trumpet_awaken2", 1, 0),
    };

    let metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &ctx.request.actors[iroi_idx].character_id,
        "GA_Oneiroi_Skill",
        effect,
        None,
        &ctx.state.execution_model_assumptions,
    )?;

    for i in 0..count {
        let hit_time = now
            .checked_add(i as u64 * interval)
            .ok_or("Iroi A2 hit timestamp overflow")?;
        for target_index in 0..ctx.request.targets.len() {
            if ctx.state.targets[target_index].hp_current == 0 {
                continue;
            }
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: hit_time,
                priority: CausalPriority::HitDelivery,
                seq: *ctx.sequence,
                data: GameEventData::HitDelivery {
                    command_index: key,
                    actor_index: iroi_idx,
                    target_index,
                    ability_id: "GA_Oneiroi_Skill".into(),
                    damage_effect_id: effect.into(),
                    atk_rate: ntesim_core::formula::Ratio::one(),
                    scaling_stat: crate::simulation::NativeScalingStat::Atk,
                    metadata: Box::new(metadata.clone()),
                },
            });
        }
    }

    Ok(())
}
