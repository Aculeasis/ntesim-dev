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

pub(crate) const ABILITY: &str = "weapon:fork_PoliceRat:arc";
pub(crate) const DAMAGE_EFFECT_ID: &str = "weapon:fork_PoliceRat:whisker_attack";
pub(crate) const TICK_INTERVAL_US: u64 = 1_000_000;
pub(crate) const DURATION_US: u64 = 30_000_000;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) command_index: usize,
    pub(crate) arc_active_until_us: u64,
    pub(crate) generation: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Parameters {
    pub(crate) cooldown_us: u64,
    pub(crate) damage_ratio: Ratio,
}

pub(crate) fn parameters(actor: &GameActorConfig) -> Result<Parameters, String> {
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_PoliceRat")
    {
        return Err("Arc: Officer Whisker requires equipped Justice Executioner".into());
    }
    use crate::build::{WeaponParameters, compile_weapon_values, ratio, weapon_seconds_to_us};
    crate::build::weapon_wearer_item(
        ntesim_data::stat_projection::frozen_tables(),
        &actor.character_id,
        "fork_PoliceRat",
        "Justice Executioner",
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
            ("buff_PoliceRat_AtkUp", true),
            ("buff_PoliceRat_Up", true),
            ("buff_PoliceRat_SkillDamage", true),
            ("buff_Rat_CD", false),
        ],
        "Justice Executioner",
    )?;
    Ok(Parameters {
        cooldown_us: weapon_seconds_to_us(
            values.fraction("buff_Rat_CD")?,
            "Justice Executioner cooldown",
        )?,
        damage_ratio: ratio(values.fraction("buff_PoliceRat_SkillDamage")?)?,
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
    let actor = &mut ctx.state.actors[actor_index];
    actor.equipment.police_rat.command_index = command_index;
    actor.equipment.police_rat.arc_active_until_us = ctx
        .state
        .current_time_us
        .checked_add(DURATION_US)
        .ok_or("Officer Whisker duration overflow")?;
    actor.equipment.police_rat.generation = actor
        .equipment
        .police_rat
        .generation
        .checked_add(1)
        .ok_or("Officer Whisker generation overflow")?;
    let generation = actor.equipment.police_rat.generation;
    let end_time_us = actor.equipment.police_rat.arc_active_until_us;

    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ActionAdmission".into(),
        event_kind: "police_rat_arc".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            cooldown_until_us: Some(cooldown_until_us),
            ..Default::default()
        }),
    });

    let first_tick_us = ctx
        .state
        .current_time_us
        .checked_add(TICK_INTERVAL_US)
        .ok_or("Officer Whisker tick overflow")?;
    if first_tick_us <= end_time_us {
        increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: first_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::PoliceRatWhiskerTick {
                actor_index,
                target_index: 0,
                generation,
                end_time_us,
                interval_us: TICK_INTERVAL_US,
                damage_ratio: parameters.damage_ratio,
            }),
        });
    }
    Ok(())
}

pub(crate) fn whisker_tick(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    generation: u64,
    end_time_us: u64,
    interval_us: u64,
    damage_ratio: Ratio,
) -> Result<(), String> {
    if ctx.state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    if ctx.state.actors[actor_index]
        .equipment
        .police_rat
        .generation
        != generation
    {
        return Ok(());
    }
    if ctx.state.current_time_us > end_time_us {
        return Ok(());
    }

    // One recipient per attack; all declared dummies are in range. Retain the
    // preferred living target, otherwise select the lowest living index.
    let target_index = if ctx
        .state
        .targets
        .get(target_index)
        .is_some_and(|target| target.hp_current > 0)
    {
        target_index
    } else if let Some(living) = ctx
        .state
        .targets
        .iter()
        .position(|target| target.hp_current > 0)
    {
        living
    } else {
        return Ok(());
    };

    increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.state.current_time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: ctx.state.actors[actor_index]
                .equipment
                .police_rat
                .command_index,
            actor_index,
            target_index,
            ability_id: ABILITY.into(),
            damage_effect_id: DAMAGE_EFFECT_ID.into(),
            atk_rate: damage_ratio,
            scaling_stat: NativeScalingStat::Atk,
            metadata: Box::new(metadata(DAMAGE_EFFECT_ID)),
        },
    });

    let next_tick_us = ctx
        .state
        .current_time_us
        .checked_add(interval_us)
        .ok_or("Officer Whisker next tick overflow")?;
    if next_tick_us <= end_time_us {
        increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: next_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::PoliceRatWhiskerTick {
                actor_index,
                target_index,
                generation,
                end_time_us,
                interval_us,
                damage_ratio,
            }),
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
            native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_PHYSICAL".into(),
            native_damage_source_category: None,
            damage_kind: DamageKind::Elemental,
            element: Some(DamageElement::Physical),
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
pub(crate) fn is_packet(
    metadata: &crate::simulation::metadata::NormalizedActionPacketMetadata,
) -> bool {
    metadata.action.ability_id == ABILITY && metadata.packet.damage_effect_id == DAMAGE_EFFECT_ID
}
