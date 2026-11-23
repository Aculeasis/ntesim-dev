use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::GearProcConfig;
use crate::simulation::NativeScalingStat;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{
    ActionCategory, ActionMetadata, ActionPacketBinding, ActionPacketBindingKind,
    ActionPacketMetadata, DamageElement, InputActionRole, MetadataProvenance, PacketClassification,
    PacketMetadata,
};
use crate::{execution::CausalPriority, formula::Ratio};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsYaodaoV1;

pub(crate) const PACKET_HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsYaodaoSeparatePacketV1;

pub(crate) const PARRY_ATTACK_TAG: &str = "ntesim.trigger.parry_attack";
const PARRY_PACKET_EFFECT_ID: &str = "weapon:fork_yaodao:parry_incantation";

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if packet.semantic_tags.contains(PARRY_ATTACK_TAG)
        && ctx
            .config
            .gear_procs
            .iter()
            .any(|proc| matches!(proc, GearProcConfig::YaodaoParry { .. }))
    {
        return Err(
            "Crimson Hexblade Parry must be declared on action metadata, not packet semantic tags"
                .into(),
        );
    }
    Ok(modifiers)
}

#[cfg(test)]
pub(crate) fn is_separate_packet(metadata: &ActionPacketMetadata) -> bool {
    metadata.packet.is_separate_parry()
        && matches!(metadata.binding.kind, ActionPacketBindingKind::OwnerChild)
}

pub(crate) fn validate_trigger_assumption(
    request: &crate::simulation::GameScenarioRequest,
    config: &crate::simulation::GameActorConfig,
    parent: &ActionPacketMetadata,
) -> Result<(), String> {
    if requires_separate_packet(config, parent) {
        crate::runtime_api::record_hypothesis(
            request,
            &mut BTreeMap::new(),
            PACKET_HYPOTHESIS_ID.as_str(),
            PACKET_HYPOTHESIS_ID.description(),
        )?;
    }
    Ok(())
}

fn requires_separate_packet_action(
    config: &crate::simulation::GameActorConfig,
    action: &ActionMetadata,
) -> bool {
    (action
        .gameplay_tags
        .iter()
        .any(|tag| tag == PARRY_ATTACK_TAG)
        || action.input_role == Some(InputActionRole::ParryAttack))
        && config.weapon_id.eq_ignore_ascii_case("weapon:fork_yaodao")
        && config
            .gear_procs
            .iter()
            .any(|proc| matches!(proc, GearProcConfig::YaodaoParry { .. }))
}

fn requires_separate_packet(
    config: &crate::simulation::GameActorConfig,
    parent: &ActionPacketMetadata,
) -> bool {
    requires_separate_packet_action(config, &parent.action)
}

pub(crate) fn on_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !requires_separate_packet_action(ctx.config, ctx.action_metadata) {
        return Ok(());
    }
    let Some(parry_damage_ratio) = ctx.config.gear_procs.iter().find_map(|proc| match proc {
        GearProcConfig::YaodaoParry { parry_damage_ratio } => Some(*parry_damage_ratio),
        _ => None,
    }) else {
        return Ok(());
    };
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: PACKET_HYPOTHESIS_ID.as_str().into(),
        description: PACKET_HYPOTHESIS_ID.description().into(),
    };
    let metadata = ActionPacketMetadata {
        action: ActionMetadata {
            categories: BTreeSet::from([ActionCategory::Other]),
            gameplay_tags: Vec::new(),
            ability_id: PARRY_PACKET_EFFECT_ID.into(),
            native_ability_index: None,
            input_role: Some(InputActionRole::Other),
            input_role_provenance: None,
            primary_slot: None,
            source_evidence: None,
            identity_evidence: None,
            provenance: provenance.clone(),
        },
        packet: PacketMetadata {
            damage_effect_id: PARRY_PACKET_EFFECT_ID.into(),
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
            packet_kind: PacketClassification::SeparateParry,
        },
        binding: ActionPacketBinding {
            energy_group_id: None,
            kind: ActionPacketBindingKind::OwnerChild,
            initiating_ability_id: ctx.action_metadata.ability_id.clone(),
            packet_ga_name: None,
            provenance: Some(provenance),
        },
    };
    *ctx.sequence = ctx
        .sequence
        .checked_add(1)
        .ok_or("event sequence overflow while scheduling Crimson Hexblade packet")?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: ctx.command_index,
            actor_index: ctx.actor_index,
            target_index: ctx.target_index,
            ability_id: PARRY_PACKET_EFFECT_ID.into(),
            damage_effect_id: PARRY_PACKET_EFFECT_ID.into(),
            atk_rate: Ratio {
                numerator: parry_damage_ratio.numerator,
                denominator: parry_damage_ratio.denominator,
            },
            scaling_stat: NativeScalingStat::Atk,
            metadata: Box::new(metadata),
        },
    });
    Ok(())
}
