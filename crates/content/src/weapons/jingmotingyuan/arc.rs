use super::{HYPOTHESIS_ID, SOURCE};
use crate::runtime_api::{CallbackContext, increment_sequence};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{
    ActionCategory, ActionMetadata, ActionPacketBinding, ActionPacketBindingKind,
    ActionPacketMetadata, DamageElement, MetadataProvenance, PacketClassification, PacketMetadata,
};
use crate::simulation::{ExactFraction, GameActorConfig, GameEventPayload, GameEventReceipt};
use crate::{execution::CausalPriority, formula::Ratio};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const ABILITY: &str = "weapon:fork_jingmotingyuan:arc";
pub(crate) const HARSH: &str = "GE_Fork_serenity_Skill1_Damage1";
pub(crate) const NAGGING: &str = "GE_Fork_serenity_Skill2_Damage";
pub(crate) const FINAL: &str = "GE_Fork_serenity_Skill2_Damage2";

#[derive(Clone, Debug)]
pub(crate) struct State {
    pub(crate) rng: crate::rng::Rng,
    pub(crate) receipts: Vec<crate::simulation::GameProcSelectionReceipt>,
}

pub(crate) fn selector(index: usize) -> u64 {
    (1_u64 << 61) + index as u64
}

impl State {
    pub(crate) fn new(seed: u64, actor_index: usize) -> Result<Self, String> {
        Ok(Self {
            rng: crate::rng::Rng::new(
                seed,
                BTreeMap::from([(
                    format!("weapon:jingmotingyuan:{actor_index}"),
                    selector(actor_index),
                )]),
            )?,
            receipts: Vec::new(),
        })
    }
}

pub(crate) struct Parameters {
    pub(crate) cooldown_us: u64,
    hp_ratio: ExactFraction,
    harsh: Ratio,
    nagging: Ratio,
}

pub(crate) fn parameters(actor: &GameActorConfig) -> Result<Parameters, String> {
    if !actor.weapon_id.eq_ignore_ascii_case(SOURCE) {
        return Err("Arc: Silent Garden requires equipped Silent Garden".into());
    }
    use crate::build::{WeaponParameters, compile_weapon_values, ratio, weapon_seconds_to_us};
    crate::build::weapon_wearer_item(
        ntesim_data::stat_projection::frozen_tables(),
        &actor.character_id,
        "fork_jingmotingyuan",
        "Silent Garden",
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
            ("buff_tingyuan_CD", false),
            ("buff_tingyuan_CritDamageUp", true),
            ("buff_tingyuan_HPreduce", true),
            ("buff_tingyuan_CD2", false),
            (HARSH, true),
            (NAGGING, true),
        ],
        "Silent Garden",
    )?;
    Ok(Parameters {
        cooldown_us: weapon_seconds_to_us(
            values.fraction("buff_tingyuan_CD2")?,
            "Silent Garden Arc cooldown",
        )?,
        hp_ratio: values.fraction("buff_tingyuan_HPreduce")?,
        harsh: ratio(values.fraction(HARSH)?)?,
        nagging: ratio(values.fraction(NAGGING)?)?,
    })
}

pub(crate) fn activate(
    ctx: &mut CallbackContext<'_>,
    command_index: usize,
    actor_index: usize,
    parameters: Parameters,
    cooldown_until_us: u64,
) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    let actor = &mut ctx.state.actors[actor_index];
    let before = actor.hp;
    let requested = ExactFraction::new(before.into(), 1)?
        .mul(parameters.hp_ratio)?
        .round_u64()?;
    let consumed = requested.min(before.saturating_sub(1));
    actor.hp -= consumed;
    let runtime = actor
        .equipment
        .jingmotingyuan
        .as_mut()
        .ok_or("Silent Garden RNG missing")?;
    let stream = format!("weapon:jingmotingyuan:{actor_index}");
    let sample = runtime.rng.uniform(&stream, 2)?;
    let outcome = sample.value;
    runtime
        .receipts
        .push(crate::simulation::GameProcSelectionReceipt {
            time_us: ctx.state.current_time_us,
            actor_index,
            stream,
            outcome_index: outcome,
            sample,
            cooldown_until_us,
        });
    let hp_after = actor.hp;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ActionAdmission".into(),
        event_kind: "jingmotingyuan_arc".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            outcome: Some(outcome.to_string()),
            hp_before: Some(before),
            hp_after: Some(hp_after),
            consumed: Some(u32::try_from(consumed).unwrap_or(u32::MAX)),
            cooldown_until_us: Some(cooldown_until_us),
            ..Default::default()
        }),
    });
    super::on_hp_consumed(ctx, actor_index, consumed)?;
    schedule_packets(ctx, command_index, actor_index, parameters, outcome)
}

fn schedule_packets(
    ctx: &mut CallbackContext<'_>,
    command_index: usize,
    actor_index: usize,
    parameters: Parameters,
    outcome: u64,
) -> Result<(), String> {
    let count = if outcome == 0 { 5 } else { 6 };
    for hit in 0..count {
        let final_hit = outcome == 1 && hit == 5;
        let effect = if outcome == 0 {
            HARSH
        } else if final_hit {
            FINAL
        } else {
            NAGGING
        };
        let mut rate = if outcome == 0 {
            parameters.harsh
        } else {
            parameters.nagging
        };
        if final_hit {
            rate.numerator = rate
                .numerator
                .checked_mul(2)
                .ok_or("Silent Garden final coefficient overflow")?;
        }
        for target_index in 0..ctx.request.targets.len() {
            increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.state.current_time_us,
                priority: CausalPriority::HitDelivery,
                seq: *ctx.sequence,
                data: GameEventData::HitDelivery {
                    command_index,
                    actor_index,
                    target_index,
                    ability_id: ABILITY.into(),
                    damage_effect_id: effect.into(),
                    atk_rate: rate,
                    scaling_stat: crate::simulation::NativeScalingStat::Atk,
                    metadata: Box::new(metadata(effect)),
                },
            });
        }
    }
    Ok(())
}

fn metadata(effect: &str) -> ActionPacketMetadata {
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: HYPOTHESIS_ID.into(),
        description: HYPOTHESIS_ID.description().into(),
    };
    ActionPacketMetadata {
        action: ActionMetadata {
            categories: BTreeSet::from([ActionCategory::Other]),
            gameplay_tags: vec!["Ability.ForkSkill".into()],
            ability_id: ABILITY.into(),
            native_ability_index: None,
            input_role: None,
            input_role_provenance: None,
            primary_slot: None,
            source_evidence: None,
            identity_evidence: None,
            provenance: provenance.clone(),
        },
        packet: PacketMetadata {
            damage_effect_id: effect.into(),
            native_ga_name: None,
            native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION".into(),
            native_damage_source_category: None,
            damage_kind: ntesim_core::combat::DamageKind::Elemental,
            element: Some(DamageElement::Incantation),
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
    use crate::simulation::metadata::NormalizedMetadataProvenance;
    let provenance = NormalizedMetadataProvenance::AcceptedModelHypothesis {
        hypothesis_id: HYPOTHESIS_ID.as_str().into(),
    };
    let expected = crate::simulation::metadata::NormalizedActionPacketMetadata {
        action: crate::simulation::metadata::NormalizedActionMetadata {
            categories: BTreeSet::from([ActionCategory::Other]),
            gameplay_tags: vec!["Ability.ForkSkill".into()],
            ability_id: ABILITY.into(),
            native_ability_index: None,
            input_role: None,
            input_role_provenance: None,
            primary_slot: None,
            source_evidence_id: None,
            identity_evidence_id: None,
            provenance: provenance.clone(),
        },
        packet: crate::simulation::metadata::NormalizedPacketMetadata {
            damage_effect_id: metadata.packet.damage_effect_id.clone(),
            native_ga_name: None,
            native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION".into(),
            native_damage_source_category: None,
            damage_kind: ntesim_core::combat::DamageKind::Elemental,
            element: Some(DamageElement::Incantation),
            fixed_crit_rate: None,
            fixed_crit_rate_provenance: None,
            unbal_value: None,
            unbal_value_provenance: None,
            semantic_tags: BTreeSet::new(),
            semantic_provenance: None,
            source_evidence_id: None,
            provenance: provenance.clone(),
            packet_kind: PacketClassification::ArcChain,
        },
        binding: crate::simulation::metadata::NormalizedActionPacketBinding {
            kind: ActionPacketBindingKind::OwnerChild,
            initiating_ability_id: ABILITY.into(),
            packet_ga_name: None,
            provenance: Some(provenance),
        },
    };
    metadata.action.ability_id == ABILITY
        && [HARSH, NAGGING, FINAL].contains(&metadata.packet.damage_effect_id.as_str())
        && metadata == &expected
}
