use super::parameters::PARAMETERS;
use crate::runtime_api::{CallbackContext, DamageContext, HitModifiers};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{
    ActionCategory, ActionMetadata, InputActionRole, MetadataProvenance, PrimaryActionSlot,
};
use crate::simulation::{ExactFraction, GameScenarioRequest, state::GameSimulatorState};
use std::collections::BTreeSet;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::AkanePerfectPitchV1;
pub(crate) const ABILITY: &str = "GA_Akane_Passive_1";
pub(crate) const PACKET: &str = "GE_Reaction_3_new_1057_Damage";

pub(crate) fn validate_inputs(request: &GameScenarioRequest) -> Result<(), String> {
    if request.actions.iter().any(|action| {
        matches!(
            action,
            crate::simulation::GameActionCommand::InvokeAbility {
                damage_effect_id, ..
            } if damage_effect_id == PACKET
        )
    }) {
        return Err(
            "Akane Perfect Pitch packet requires a real Remora reapplication, not an authored ability input"
                .into(),
        );
    }
    Ok(())
}

pub(crate) fn schedule_tick(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    owner: usize,
    generation: u64,
    time_us: u64,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us,
        priority: crate::execution::CausalPriority::BuffTickOrExpiry,
        seq: *sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Akane(
            super::events::Event::PerfectPitchTick { owner, generation },
        )),
    });
    Ok(())
}

pub(crate) fn on_remora_applied(
    ctx: &mut CallbackContext<'_>,
    _applier: usize,
    target: usize,
    existing_remaining_us: Option<u64>,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    for owner in 0..ctx.request.actors.len() {
        if ctx.request.actors[owner].character_id != "character:1057"
            || ctx.state.actors[owner].hp == 0
        {
            continue;
        }
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            MODEL,
        )?;
        let (mode_gen, need_schedule) = {
            let state = ctx.state.actors[owner].character.akane_mut()?;
            state.perfect_pitch_unlocked = true;
            let active = state.mode_active(now);
            let schedule = active && state.perfect_pitch_next_tick_us.is_none();
            if schedule {
                let next_tick = now
                    .checked_add(PARAMETERS.perfect_pitch_tick_us)
                    .ok_or("Akane Perfect Pitch tick deadline overflow")?;
                if next_tick < state.mode_until_us(now) {
                    state.perfect_pitch_next_tick_us = Some(next_tick);
                    (state.mode_generation, Some(next_tick))
                } else {
                    (state.mode_generation, None)
                }
            } else {
                (state.mode_generation, None)
            }
        };
        if let Some(next_tick) = need_schedule {
            schedule_tick(ctx.queue, ctx.sequence, owner, mode_gen, next_tick)?;
        }

        let Some(rem_us) = existing_remaining_us.filter(|&rem| rem > 0) else {
            continue;
        };
        if ctx.state.targets[target].hp_current == 0 {
            continue;
        }
        let icd_until = ctx.state.actors[owner]
            .character
            .akane()?
            .perfect_pitch_icd_until_us
            .get(&target)
            .copied()
            .unwrap_or(0);
        if now < icd_until {
            continue;
        }
        let next_icd = now
            .checked_add(PARAMETERS.perfect_pitch_icd_us)
            .ok_or("Akane Perfect Pitch ICD overflow")?;
        ctx.state.actors[owner]
            .character
            .akane_mut()?
            .perfect_pitch_icd_until_us
            .insert(target, next_icd);

        let whole_secs = (rem_us / 1_000_000).min(8);
        let final_bonus = ExactFraction::new(u128::from(whole_secs), 5)?;
        let command = crate::simulation::reactions::remora_command(ctx.state, target)
            .or_else(|| {
                ctx.state.actors[owner]
                    .character
                    .akane()
                    .ok()
                    .and_then(|s| s.mode_command_index())
            })
            .unwrap_or(0);
        emit_proc(ctx, owner, target, command, final_bonus)?;
    }
    Ok(())
}

pub(crate) fn on_tick(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let (mode, mode_cmd, next_tick) = {
        let state = ctx.state.actors[owner].character.akane_mut()?;
        if !state.perfect_pitch_unlocked
            || state.mode_generation != generation
            || !state.mode_active(now)
        {
            return Ok(());
        }
        state.perfect_pitch_next_tick_us = None;
        let next = now
            .checked_add(PARAMETERS.perfect_pitch_tick_us)
            .ok_or("Akane Perfect Pitch tick deadline overflow")?;
        let next_tick = if next < state.mode_until_us(now) {
            state.perfect_pitch_next_tick_us = Some(next);
            Some(next)
        } else {
            None
        };
        let cmd = state.mode_command_index().unwrap_or(0);
        (state.mode.clone(), cmd, next_tick)
    };
    if let Some(next) = next_tick {
        schedule_tick(ctx.queue, ctx.sequence, owner, generation, next)?;
    }
    let mut targets = Vec::new();
    match mode {
        super::skill::Mode::Overtone { target, .. } => {
            if target < ctx.request.targets.len() && ctx.state.targets[target].hp_current > 0 {
                targets.push(target);
            }
        }
        super::skill::Mode::ReverbField { .. } => {
            for target in 0..ctx.request.targets.len() {
                if ctx.state.targets[target].hp_current > 0 {
                    targets.push(target);
                }
            }
        }
        super::skill::Mode::None => return Ok(()),
    }
    for target in targets {
        crate::simulation::reactions::apply_remora_from_owner(ctx, owner, target, mode_cmd)?;
    }
    Ok(())
}

fn emit_proc(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    command: usize,
    final_bonus: ExactFraction,
) -> Result<(), String> {
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: MODEL.into(),
        description: MODEL.description().into(),
    };
    let action = ActionMetadata {
        categories: BTreeSet::from([ActionCategory::Skill]),
        gameplay_tags: Vec::new(),
        ability_id: ABILITY.into(),
        native_ability_index: None,
        input_role: Some(InputActionRole::RedirectSkill),
        input_role_provenance: Some(provenance.clone()),
        primary_slot: Some(PrimaryActionSlot::Skill),
        source_evidence: None,
        identity_evidence: None,
        provenance,
    };
    let config = &ctx.request.actors[owner];
    let mut metadata = crate::simulation::metadata::resolve_packet_for_action(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        &action,
        PACKET,
        &ctx.request.model_assumptions,
    )?;
    let (rate, scaling_stat) =
        crate::simulation::inputs::coefficient(ctx.request, config, "GA_Akane_Skill", PACKET, 0)?;
    let now = ctx.state.current_time_us;
    let trigger = match ctx.state.origins.identify(ctx.request, command)? {
        (None, Some(autonomous_event_index)) => {
            crate::simulation::GameCausalTrigger::OwnerDeadline {
                autonomous_event_index,
            }
        }
        (Some(cmd), None) => {
            if matches!(
                ctx.request.actions.get(cmd),
                Some(crate::simulation::GameActionCommand::SwitchActor { .. })
            ) {
                crate::simulation::GameCausalTrigger::FieldEntry { command_index: cmd }
            } else {
                crate::simulation::GameCausalTrigger::Action { command_index: cmd }
            }
        }
        _ => return Err("unexpected causal identity for Akane Perfect Pitch".into()),
    };
    let key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: ABILITY.into(),
            time_us: now,
            trigger,
            recipient_healing: None,
        },
    )?;
    metadata.binding.energy_group_id = Some(crate::runtime_api::next_packet_group(
        owner,
        &mut ctx.state.actors[owner],
    )?);
    ctx.state.actors[owner]
        .character
        .akane_mut()?
        .perfect_pitch_final_bonuses
        .insert(key, final_bonus);
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: now,
        priority: crate::execution::CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: key,
            actor_index: owner,
            target_index: target,
            ability_id: ABILITY.into(),
            damage_effect_id: PACKET.into(),
            atk_rate: rate,
            scaling_stat,
            metadata: Box::new(metadata),
        },
    });
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: now,
        priority: crate::execution::CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Akane(
            super::events::Event::PerfectPitchCleanup { owner, key },
        )),
    });
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if ctx.damage_effect_id != PACKET {
        return Ok(modifiers);
    }
    let state = ctx.actor.character.akane()?;
    if let Some(&add) = state.perfect_pitch_final_bonuses.get(&ctx.command_index) {
        modifiers.add_final(add)
    } else {
        Ok(modifiers)
    }
}

pub(crate) fn cleanup(
    state: &mut GameSimulatorState,
    owner: usize,
    key: usize,
) -> Result<(), String> {
    state.actors[owner]
        .character
        .akane_mut()?
        .perfect_pitch_final_bonuses
        .remove(&key);
    Ok(())
}
