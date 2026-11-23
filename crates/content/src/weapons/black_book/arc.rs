use super::HYPOTHESIS_ID;
use crate::execution::CausalPriority;
use crate::formula::Ratio;
use crate::runtime_api::{CallbackContext, increment_sequence, record_hypothesis_id};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{
    ActionCategory, ActionMetadata, ActionPacketBinding, ActionPacketBindingKind,
    ActionPacketMetadata, DamageElement, InputActionRole, MetadataProvenance, PacketClassification,
    PacketMetadata,
};
use crate::simulation::{GameActorConfig, GameEventPayload, GameEventReceipt, NativeScalingStat};
use ntesim_core::combat::DamageKind;
use std::collections::BTreeSet;

pub(crate) const ABILITY: &str = "weapon:fork_BlackBook:arc";
pub(crate) const ATTACK_EFFECT_ID: &str = "GE_Fork_BlackBook_damage";
pub(crate) const STARTUP_US: u64 = 200_000;
pub(crate) const CANCEL_COOLDOWN_US: u64 = 30_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PendingStartup {
    pub(crate) generation: u64,
    pub(crate) command_index: usize,
    pub(crate) completes_at_us: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct State {
    pub(crate) command_index: usize,
    pub(crate) chains_remaining: u32,
    pub(crate) startup_generation: u64,
    pub(crate) pending_startup: Option<PendingStartup>,
    pub(crate) arc_active_until_us: u64,
    pub(crate) generation: u64,
    pub(crate) designated_target: Option<usize>,
    pub(crate) chaos_damage_bonus: Ratio,
    pub(crate) break_attack_ratio: Ratio,
}

impl Default for State {
    fn default() -> Self {
        Self {
            command_index: 0,
            chains_remaining: 2,
            startup_generation: 0,
            pending_startup: None,
            arc_active_until_us: 0,
            generation: 0,
            designated_target: None,
            chaos_damage_bonus: Ratio::zero(),
            break_attack_ratio: Ratio::zero(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Parameters {
    pub(crate) duration_us: u64,
    pub(crate) designate_interval_us: u64,
    pub(crate) startup_us: u64,
    pub(crate) chaos_damage_bonus: Ratio,
    pub(crate) break_attack_ratio: Ratio,
}

pub(crate) fn parameters(actor: &GameActorConfig) -> Result<Parameters, String> {
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_BlackBook")
    {
        return Err("Arc: Black Tome requires equipped Black Tome".into());
    }
    use crate::build::{WeaponParameters, compile_weapon_values, ratio, weapon_seconds_to_us};
    crate::build::weapon_wearer_item(
        ntesim_data::stat_projection::frozen_tables(),
        &actor.character_id,
        "fork_BlackBook",
        "Black Tome",
    )?;
    let native = compile_weapon_values(
        ntesim_data::stat_projection::frozen_tables(),
        1,
        "ntesim-current",
        actor,
    )?;
    let values = WeaponParameters::new(
        &native,
        &[
            ("buff_BlackBook2_Unbal", false),
            ("buff_BlackBook2_CD", false),
            ("buff_BlackBook2_CD2", false),
            ("buff_BlackBook2_DamageUpChaosBase", true),
            ("buff_BlackBook2_SkillDamage", true),
        ],
        "Black Tome",
    )?;
    Ok(Parameters {
        duration_us: weapon_seconds_to_us(
            values.fraction("buff_BlackBook2_CD")?,
            "Black Tome duration",
        )?,
        designate_interval_us: weapon_seconds_to_us(
            values.fraction("buff_BlackBook2_CD2")?,
            "Black Tome designate interval",
        )?,
        startup_us: STARTUP_US,
        chaos_damage_bonus: ratio(values.fraction("buff_BlackBook2_DamageUpChaosBase")?)?,
        break_attack_ratio: ratio(values.fraction("buff_BlackBook2_SkillDamage")?)?,
    })
}

pub(crate) fn activate(
    ctx: &mut CallbackContext<'_>,
    command_index: usize,
    actor_index: usize,
    parameters: Parameters,
    cooldown_until_us: u64,
) -> Result<(), String> {
    record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    let now_us = ctx.state.current_time_us;
    let completes_at_us = now_us
        .checked_add(parameters.startup_us)
        .ok_or("Black Tome startup deadline overflow")?;
    completes_at_us
        .checked_add(parameters.duration_us)
        .ok_or("Black Tome duration deadline overflow")?;
    completes_at_us
        .checked_add(parameters.designate_interval_us)
        .ok_or("Black Tome designation deadline overflow")?;
    if ctx.state.actors[actor_index]
        .equipment
        .black_book
        .pending_startup
        .is_some()
    {
        return Err("Black Tome Arc startup is already pending".into());
    }
    let generation = ctx.state.actors[actor_index]
        .equipment
        .black_book
        .startup_generation
        .checked_add(1)
        .ok_or("Black Tome startup generation overflow")?;
    increment_sequence(ctx.sequence)?;
    {
        let actor = &mut ctx.state.actors[actor_index];
        actor.locked_until_us = completes_at_us;
        actor.equipment.black_book.startup_generation = generation;
        actor.equipment.black_book.pending_startup = Some(PendingStartup {
            generation,
            command_index,
            completes_at_us,
        });
    }
    ctx.queue.push(ScheduledGameEvent {
        time_us: completes_at_us,
        priority: CausalPriority::ModePause,
        seq: *ctx.sequence,
        data: GameEventData::Content(
            crate::callbacks::ContentEvent::BlackBookArcStartupComplete {
                actor_index,
                command_index,
                generation,
                cooldown_until_us,
                parameters,
            },
        ),
    });
    ctx.trace.push(GameEventReceipt {
        time_us: now_us,
        phase: "ActionAdmission".into(),
        event_kind: "black_book_arc_startup".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            completes_at_us: Some(completes_at_us),
            cooldown_until_us: Some(cooldown_until_us),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn startup_complete(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    command_index: usize,
    generation: u64,
    cooldown_until_us: u64,
    parameters: Parameters,
) -> Result<(), String> {
    let pending = ctx.state.actors[actor_index]
        .equipment
        .black_book
        .pending_startup;
    if pending
        != Some(PendingStartup {
            generation,
            command_index,
            completes_at_us: ctx.state.current_time_us,
        })
    {
        return Ok(());
    }
    ctx.state.actors[actor_index]
        .equipment
        .black_book
        .pending_startup = None;
    if ctx.state.actors[actor_index].hp == 0 {
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ActionAdmission".into(),
            event_kind: "black_book_arc_startup_cancelled".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                reason: Some("defeated".into()),
                ..Default::default()
            }),
        });
        return Ok(());
    }

    let start_us = ctx.state.current_time_us;
    let end_time_us = start_us
        .checked_add(parameters.duration_us)
        .ok_or("Black Tome duration overflow")?;
    let generation = ctx.state.actors[actor_index]
        .equipment
        .black_book
        .generation
        .checked_add(1)
        .ok_or("Black Tome generation overflow")?;
    let initial_target = ctx.state.targets.iter().position(|t| t.hp_current > 0);
    {
        let actor = &mut ctx.state.actors[actor_index];
        let state = &mut actor.equipment.black_book;
        state.command_index = command_index;
        state.arc_active_until_us = end_time_us;
        state.generation = generation;
        state.chains_remaining = 2;
        state.designated_target = initial_target;
        state.chaos_damage_bonus = parameters.chaos_damage_bonus;
        state.break_attack_ratio = parameters.break_attack_ratio;
    }
    ctx.trace.push(GameEventReceipt {
        time_us: start_us,
        phase: "ActionAdmission".into(),
        event_kind: "black_book_arc".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: initial_target,
            expires_at_us: Some(end_time_us),
            cooldown_until_us: Some(cooldown_until_us),
            ..Default::default()
        }),
    });

    let first_tick_us = start_us
        .checked_add(parameters.designate_interval_us)
        .ok_or("Black Tome designate tick overflow")?;
    if first_tick_us < end_time_us && first_tick_us <= ctx.request.absolute_limit_us {
        increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: first_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::BlackBookDesignateTick {
                actor_index,
                generation,
                end_time_us,
                interval_us: parameters.designate_interval_us,
            }),
        });
    }
    Ok(())
}

pub(crate) fn interrupt_startup(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
) -> Result<Option<usize>, String> {
    let Some(pending) = ctx.state.actors[actor_index]
        .equipment
        .black_book
        .pending_startup
    else {
        return Ok(None);
    };
    if ctx.state.current_time_us >= pending.completes_at_us {
        return Ok(None);
    }
    let cancel_ready_at_us = ctx
        .state
        .current_time_us
        .checked_add(CANCEL_COOLDOWN_US)
        .ok_or("Black Tome cancel cooldown deadline overflow")?;
    let actor = &mut ctx.state.actors[actor_index];
    actor.equipment.black_book.pending_startup = None;
    actor
        .ability_cooldowns
        .insert(ABILITY.into(), cancel_ready_at_us);
    ctx.queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(
                crate::callbacks::ContentEvent::BlackBookArcStartupComplete {
                    actor_index: queued_actor,
                    generation,
                    ..
                }
            ) if *queued_actor == actor_index && *generation == pending.generation
        )
    });
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "SettlementAndDeath".into(),
        event_kind: "black_book_arc_startup_cancelled".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            reason: Some("interrupted".into()),
            cooldown_until_us: Some(cancel_ready_at_us),
            ..Default::default()
        }),
    });
    Ok(Some(pending.command_index))
}

pub(crate) fn validate_interrupt_deadline(timestamp_us: u64) -> Result<(), String> {
    timestamp_us
        .checked_add(CANCEL_COOLDOWN_US)
        .ok_or("Black Tome cancel cooldown deadline overflow")?;
    Ok(())
}

pub(crate) fn designate_tick(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    generation: u64,
    end_time_us: u64,
    interval_us: u64,
) -> Result<(), String> {
    if ctx.state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    if ctx.state.actors[actor_index]
        .equipment
        .black_book
        .generation
        != generation
    {
        return Ok(());
    }
    if ctx.state.current_time_us >= end_time_us {
        return Ok(());
    }

    let current_designated = ctx.state.actors[actor_index]
        .equipment
        .black_book
        .designated_target;
    let new_designated = match current_designated {
        Some(idx) if idx < ctx.state.targets.len() && ctx.state.targets[idx].hp_current > 0 => {
            Some(idx)
        }
        _ => ctx.state.targets.iter().position(|t| t.hp_current > 0),
    };
    ctx.state.actors[actor_index]
        .equipment
        .black_book
        .designated_target = new_designated;

    let next_tick_us = ctx
        .state
        .current_time_us
        .checked_add(interval_us)
        .ok_or("Black Tome next designate tick overflow")?;
    if next_tick_us < end_time_us && next_tick_us <= ctx.request.absolute_limit_us {
        increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: next_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::BlackBookDesignateTick {
                actor_index,
                generation,
                end_time_us,
                interval_us,
            }),
        });
    }
    Ok(())
}

pub(crate) fn on_target_break_damage(
    ctx: &mut CallbackContext<'_>,
    target_index: usize,
) -> Result<(), String> {
    for actor_index in 0..ctx.state.actors.len() {
        let actor = &ctx.state.actors[actor_index];
        if actor.hp == 0 {
            continue;
        }
        let state = &actor.equipment.black_book;
        if ctx.state.current_time_us >= state.arc_active_until_us {
            continue;
        }
        if state.designated_target != Some(target_index) {
            continue;
        }
        let atk_rate = state.break_attack_ratio;
        increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.state.current_time_us,
            priority: CausalPriority::HitDelivery,
            seq: *ctx.sequence,
            data: GameEventData::HitDelivery {
                command_index: state.command_index,
                actor_index,
                target_index,
                ability_id: ABILITY.into(),
                damage_effect_id: ATTACK_EFFECT_ID.into(),
                atk_rate,
                scaling_stat: NativeScalingStat::Atk,
                metadata: Box::new(metadata(ATTACK_EFFECT_ID)),
            },
        });
    }
    Ok(())
}

fn metadata(effect: &str) -> ActionPacketMetadata {
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: HYPOTHESIS_ID.as_str().into(),
        description: HYPOTHESIS_ID.description().into(),
    };
    ActionPacketMetadata {
        action: ActionMetadata {
            categories: BTreeSet::from([ActionCategory::Other]),
            gameplay_tags: Vec::new(),
            ability_id: ABILITY.into(),
            native_ability_index: None,
            input_role: Some(InputActionRole::Other),
            input_role_provenance: Some(provenance.clone()),
            primary_slot: None,
            source_evidence: None,
            identity_evidence: None,
            provenance: provenance.clone(),
        },
        packet: PacketMetadata {
            damage_effect_id: effect.into(),
            native_ga_name: None,
            native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_CHAOS".into(),
            native_damage_source_category: None,
            damage_kind: DamageKind::Elemental,
            element: Some(DamageElement::Chaos),
            fixed_crit_rate: None,
            fixed_crit_rate_provenance: None,
            unbal_value: None,
            unbal_value_provenance: None,
            semantic_tags: BTreeSet::new(),
            semantic_provenance: None,
            source_evidence: None,
            provenance: provenance.clone(),
            packet_kind: PacketClassification::ArcChain,
        },
        binding: ActionPacketBinding {
            energy_group_id: None,
            kind: ActionPacketBindingKind::OwnerChild,
            initiating_ability_id: ABILITY.into(),
            packet_ga_name: None,
            provenance: Some(provenance),
        },
    }
}

#[cfg(test)]
pub(crate) fn is_packet(metadata: &ActionPacketMetadata) -> bool {
    metadata.action.ability_id == ABILITY && metadata.packet.damage_effect_id == ATTACK_EFFECT_ID
}
