use crate::simulation::{ExactFraction, metadata::*};
use std::collections::BTreeSet;
pub(super) fn packet(element: &str) -> Result<ActionPacketMetadata, String> {
    let element_value = match element {
        "CHAOS" => DamageElement::Chaos,
        "COSMOS" => DamageElement::Cosmos,
        "INCANTATION" => DamageElement::Incantation,
        "NATURE" => DamageElement::Nature,
        "LAKSHANA" => DamageElement::Lakshana,
        "PSYCHE" => DamageElement::Psyche,
        _ => return Err("unsupported Break contribution element".into()),
    };
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: super::HYPOTHESIS_ID.as_str().into(),
        description: super::HYPOTHESIS_ID.description().into(),
    };
    Ok(ActionPacketMetadata {
        action: ActionMetadata {
            categories: BTreeSet::from([ActionCategory::Other]),
            gameplay_tags: vec!["State.Damage.Unbalance".into()],
            ability_id: "ntesim.break".into(),
            native_ability_index: None,
            input_role: Some(InputActionRole::Other),
            input_role_provenance: Some(provenance.clone()),
            primary_slot: None,
            source_evidence: None,
            identity_evidence: None,
            provenance: provenance.clone(),
        },
        packet: PacketMetadata {
            damage_effect_id: "ntesim.break.contribution".into(),
            native_ga_name: None,
            native_damage_type_ex: format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}"),
            native_damage_source_category: None,
            damage_kind: ntesim_core::combat::DamageKind::Elemental,
            element: Some(element_value),
            fixed_crit_rate: Some(ExactFraction::new(0, 1)?),
            fixed_crit_rate_provenance: Some(provenance.clone()),
            unbal_value: Some(ExactFraction::new(0, 1)?),
            unbal_value_provenance: Some(provenance.clone()),
            semantic_tags: BTreeSet::from(["State.Damage.Unbalance".into()]),
            semantic_provenance: Some(provenance.clone()),
            source_evidence: None,
            provenance: provenance.clone(),
            packet_kind: crate::simulation::metadata::PacketClassification::Standard,
        },
        binding: ActionPacketBinding {
            energy_group_id: None,
            kind: ActionPacketBindingKind::OwnerChild,
            initiating_ability_id: "ntesim.break".into(),
            packet_ga_name: None,
            provenance: Some(provenance),
        },
    })
}
