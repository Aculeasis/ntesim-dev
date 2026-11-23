use super::native::native_unbal_value;
use super::*;

const CALLER_DECLARED_ACTION_SEMANTIC_TAGS: &[&str] = &[
    "ntesim.trigger.parry_attack",
    crate::weapons::thief_candy::CRITICAL_DODGE_TAG,
];

fn accepted_declaration<'a>(
    research: Option<&'a crate::simulation::ResearchActionMetadata>,
    ability_id: &str,
    damage_effect_id: &str,
    accepted_assumptions: &BTreeMap<String, String>,
) -> Result<Option<(&'a crate::simulation::ResearchActionMetadata, String)>, String> {
    Ok(match research {
        Some(metadata) => {
            if metadata.ability_id != ability_id || metadata.damage_effect_id != damage_effect_id {
                return Err(
                    "research action metadata identity does not match its indexed action".into(),
                );
            }
            let description = accepted_assumptions
                .get(&metadata.hypothesis_id)
                .filter(|description| !description.trim().is_empty())
                .ok_or_else(|| {
                    format!(
                        "research metadata hypothesis {} must be accepted in model_assumptions",
                        metadata.hypothesis_id
                    )
                })?;
            Some((metadata, description.clone()))
        }
        None => None,
    })
}

/// Resolve a caller action and packet. Research declarations are accepted only for missing
/// native metadata and only when the request repeats the exact hypothesis description.
pub(crate) fn resolve_requested_action_packet(
    content_id: &str,
    tables: &BTreeMap<String, serde_json::Value>,
    character_id: &str,
    ability_id: &str,
    damage_effect_id: &str,
    research: Option<&crate::simulation::ResearchActionMetadata>,
    accepted_assumptions: &BTreeMap<String, String>,
) -> Result<ActionPacketMetadata, String> {
    let explicit =
        accepted_declaration(research, ability_id, damage_effect_id, accepted_assumptions)?;
    let mut relationship_declaration_used = false;
    if explicit
        .as_ref()
        .is_some_and(|(metadata, _)| metadata.packet_ga_name_binding.is_some())
    {
        relationship_declaration_used = true;
    }

    let native_action = ntesim_data::native_action_metadata::resolve_ability(
        content_id,
        tables,
        character_id,
        ability_id,
    );
    let mut action_declaration_used = false;
    let action = match native_action {
        Ok(native) => match resolve_action_metadata_with_owner(character_id, &native) {
            Ok(action) => {
                if explicit.as_ref().is_some_and(|(metadata, _)| {
                    metadata.action_gameplay_tags.is_some() || metadata.input_action_role.is_some()
                }) {
                    return Err(
                        "research action fields cannot override native action metadata".into(),
                    );
                }
                action
            }
            Err(error) => {
                let Some((metadata, description)) = explicit.as_ref() else {
                    return Err(error);
                };
                let explicit_none_only = !native.gameplay_tag_names.is_empty()
                    && native
                        .gameplay_tag_names
                        .iter()
                        .all(|tag| tag.value.as_deref() == Some("None"));
                if native.row_state == RowState::Present
                    && native.tag_state == FieldState::Present
                    && !explicit_none_only
                {
                    return Err(error);
                }
                if crate::characters::action_roles::resolve(character_id, &native.ability_id)
                    .is_some()
                {
                    return Err(error);
                }
                action_declaration_used = true;
                action_from_research(
                    &native.ability_id,
                    metadata.action_gameplay_tags.as_deref().ok_or_else(|| {
                        format!("{error}; research action_gameplay_tags required")
                    })?,
                    &metadata.hypothesis_id,
                    description,
                    metadata.input_action_role,
                    native.evidence.clone(),
                    Some(native.config_evidence.clone()),
                )?
            }
        },
        Err(error) => {
            let Some((metadata, description)) = explicit.as_ref() else {
                return Err(error);
            };
            action_declaration_used = true;
            action_from_research(
                ability_id,
                metadata
                    .action_gameplay_tags
                    .as_deref()
                    .ok_or_else(|| format!("{error}; research action_gameplay_tags required"))?,
                &metadata.hypothesis_id,
                description,
                metadata.input_action_role,
                None,
                None,
            )?
        }
    };

    let native_packet = ntesim_data::native_action_metadata::resolve_damage_effect(
        content_id,
        tables,
        damage_effect_id,
    )?;
    if native_packet.row_state == RowState::Present
        && explicit
            .as_ref()
            .is_some_and(|(metadata, _)| metadata.unbal_value.is_some())
    {
        return Err("research UnbalValue cannot override a native damage row".into());
    }
    let packet_declaration_used =
        !(native_packet.row_state == RowState::Present && native_packet.damage_type_ex.is_some());
    if !packet_declaration_used
        && explicit.as_ref().is_some_and(|(metadata, _)| {
            metadata.damage_type_ex.is_some()
                || metadata.damage_source_category.is_some()
                || metadata.fixed_crit_rate.is_some()
                || metadata.unbal_value.is_some()
        })
    {
        return Err("research packet fields cannot override native damage metadata".into());
    }
    let packet = if !packet_declaration_used {
        resolve_packet_metadata(&native_packet, accepted_assumptions)?
    } else {
        let (metadata, description) = explicit.as_ref().ok_or_else(|| {
            if native_packet.row_state == RowState::MissingNativeRow {
                format!("unsupported packet metadata: {damage_effect_id} has no native damage row")
            } else {
                format!(
                    "unsupported packet metadata: {damage_effect_id} has no string DamageTypeEX"
                )
            }
        })?;
        let raw_type = metadata
            .damage_type_ex
            .as_deref()
            .ok_or("research damage_type_ex required for a packet without native type")?;
        let fixed_crit = if native_packet.row_state == RowState::Present {
            native_fixed_crit_rate(&native_packet, accepted_assumptions)?
        } else {
            research_fixed_crit_rate(
                metadata.fixed_crit_rate.as_ref(),
                damage_effect_id,
                accepted_assumptions,
            )?
        };
        packet_from_research(
            damage_effect_id,
            raw_type,
            metadata.damage_source_category.clone(),
            &metadata.hypothesis_id,
            description,
            accepted_assumptions,
            native_packet.ga_name.clone(),
            native_packet.evidence.clone(),
            fixed_crit,
            if native_packet.row_state == RowState::Present {
                native_unbal_value(&native_packet)?
            } else {
                research_unbal_value(metadata.unbal_value, &metadata.hypothesis_id, description)?
            },
        )?
    };
    let binding = resolve_action_packet_binding(
        character_id,
        &action.ability_id,
        damage_effect_id,
        &native_packet,
        explicit
            .as_ref()
            .map(|(metadata, description)| (*metadata, description.as_str())),
    )?;
    if explicit.is_some()
        && !action_declaration_used
        && !packet_declaration_used
        && !relationship_declaration_used
    {
        return Err("research action metadata declaration was not used by this action".into());
    }
    Ok(ActionPacketMetadata {
        action,
        packet,
        binding,
    })
}

pub(super) fn action_from_research(
    ability_id: &str,
    raw_tags: &[String],
    hypothesis_id: &str,
    description: &str,
    input_action_role: Option<InputActionRole>,
    source_evidence: Option<NativeEvidence>,
    identity_evidence: Option<NativeEvidence>,
) -> Result<ActionMetadata, String> {
    let mut categories = BTreeSet::new();
    for tag in raw_tags {
        if tag == "None" {
            return Err(
                "research metadata cannot declare native None as an action category".into(),
            );
        } else if tag == "Owner.QTE" {
            categories.insert(ActionCategory::Qte);
        } else if tag == "Owner.Other"
            || CALLER_DECLARED_ACTION_SEMANTIC_TAGS.contains(&tag.as_str())
        {
            categories.insert(ActionCategory::Other);
        } else {
            categories.insert(ActionCategory::from_native_tag(tag)?);
        }
    }
    if categories.is_empty() {
        return Err(
            "research action metadata requires at least one recognized gameplay tag".into(),
        );
    }
    Ok(ActionMetadata {
        categories,
        gameplay_tags: raw_tags.to_vec(),
        ability_id: ability_id.to_owned(),
        native_ability_index: None,
        input_role: input_action_role,
        input_role_provenance: input_action_role.map(|_| {
            MetadataProvenance::AcceptedModelHypothesis {
                id: hypothesis_id.to_owned(),
                description: description.to_owned(),
            }
        }),
        primary_slot: input_action_role.and_then(PrimaryActionSlot::from_input_role),
        source_evidence,
        identity_evidence,
        provenance: MetadataProvenance::AcceptedModelHypothesis {
            id: hypothesis_id.to_owned(),
            description: description.to_owned(),
        },
    })
}

fn packet_from_research(
    effect_id: &str,
    raw_type: &str,
    source_category: Option<String>,
    hypothesis_id: &str,
    description: &str,
    assumptions: &BTreeMap<String, String>,
    native_ga_name: Option<String>,
    source_evidence: Option<NativeEvidence>,
    (fixed_crit_rate, fixed_crit_rate_provenance): (
        Option<ExactFraction>,
        Option<MetadataProvenance>,
    ),
    (unbal_value, unbal_value_provenance): (Option<ExactFraction>, Option<MetadataProvenance>),
) -> Result<PacketMetadata, String> {
    let (damage_kind, element) = match raw_type {
        "EExecutionDamageTypeEX::DAMAGE_TYPE_COSMOS" => {
            (DamageKind::Elemental, Some(DamageElement::Cosmos))
        }
        "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE" => {
            (DamageKind::Elemental, Some(DamageElement::Nature))
        }
        "EExecutionDamageTypeEX::DAMAGE_TYPE_CHAOS" => {
            (DamageKind::Elemental, Some(DamageElement::Chaos))
        }
        "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION" => {
            (DamageKind::Elemental, Some(DamageElement::Incantation))
        }
        "EExecutionDamageTypeEX::DAMAGE_TYPE_LAKSHANA" => {
            (DamageKind::Elemental, Some(DamageElement::Lakshana))
        }
        "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE" => {
            (DamageKind::Elemental, Some(DamageElement::Psyche))
        }
        "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHICALLY" => (DamageKind::Mental, None),
        "EExecutionDamageTypeEX::DAMAGE_TYPE_TRUE" => (DamageKind::True, None),
        "EExecutionDamageTypeEX::DAMAGE_TYPE_NORMAL" => {
            if assumptions
                .get(NORMAL_PHYSICAL_HYPOTHESIS)
                .map(String::as_str)
                != Some(NORMAL_PHYSICAL_DESCRIPTION)
            {
                return Err(format!(
                    "research NORMAL packet requires accepted model assumption {NORMAL_PHYSICAL_HYPOTHESIS}"
                ));
            }
            (DamageKind::Elemental, Some(DamageElement::Physical))
        }
        _ => {
            return Err(format!(
                "unsupported research packet DamageTypeEX: {raw_type}"
            ));
        }
    };
    if assumptions.get(hypothesis_id).map(String::as_str) != Some(description) {
        return Err(format!(
            "research metadata hypothesis {hypothesis_id} is not accepted with its exact description"
        ));
    }
    let (semantic_tags, semantic_provenance) = owner_packet_semantics(effect_id, assumptions)?;
    Ok(PacketMetadata {
        damage_effect_id: effect_id.to_owned(),
        native_ga_name,
        native_damage_type_ex: raw_type.to_owned(),
        native_damage_source_category: source_category,
        damage_kind,
        element,
        fixed_crit_rate,
        fixed_crit_rate_provenance,
        unbal_value,
        unbal_value_provenance,
        semantic_tags,
        semantic_provenance,
        source_evidence,
        provenance: MetadataProvenance::AcceptedModelHypothesis {
            id: hypothesis_id.to_owned(),
            description: description.to_owned(),
        },
        packet_kind: PacketClassification::Standard,
    })
}

fn research_unbal_value(
    declaration: Option<ntesim_core::formula::Ratio>,
    hypothesis_id: &str,
    description: &str,
) -> Result<(Option<ExactFraction>, Option<MetadataProvenance>), String> {
    let Some(value) = declaration else {
        return Ok((None, None));
    };
    Ok((
        Some(ExactFraction::from_ratio(value)?),
        Some(MetadataProvenance::AcceptedModelHypothesis {
            id: hypothesis_id.to_owned(),
            description: description.to_owned(),
        }),
    ))
}

fn research_fixed_crit_rate(
    declaration: Option<&crate::simulation::ResearchFixedCritRate>,
    effect_id: &str,
    accepted_assumptions: &BTreeMap<String, String>,
) -> Result<(Option<ExactFraction>, Option<MetadataProvenance>), String> {
    let declaration = declaration.ok_or_else(|| {
        format!("research packet {effect_id} requires an explicit fixed_crit_rate declaration")
    })?;
    let rate = match declaration {
        crate::simulation::ResearchFixedCritRate::NoOverride => return Ok((None, None)),
        crate::simulation::ResearchFixedCritRate::Probability { rate } => {
            ExactFraction::from_ratio(*rate)?
        }
    };
    if rate.numerator > rate.denominator {
        return Err(format!(
            "research packet {effect_id} fixed crit probability is outside [0, 1]"
        ));
    }
    if accepted_assumptions
        .get(FIXED_CRIT_PROBABILITY_HYPOTHESIS)
        .map(String::as_str)
        != Some(FIXED_CRIT_PROBABILITY_DESCRIPTION)
    {
        return Err(format!(
            "research packet {effect_id} fixed crit declaration requires exact accepted model assumption {FIXED_CRIT_PROBABILITY_HYPOTHESIS}"
        ));
    }
    Ok((
        Some(rate),
        Some(MetadataProvenance::AcceptedModelHypothesis {
            id: FIXED_CRIT_PROBABILITY_HYPOTHESIS.into(),
            description: FIXED_CRIT_PROBABILITY_DESCRIPTION.into(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_unbal_value_distinguishes_absent_from_declared_zero() {
        let absent = research_unbal_value(None, "research.packet", "test").unwrap();
        assert_eq!(absent, (None, None));

        let zero = research_unbal_value(
            Some(ntesim_core::formula::Ratio::zero()),
            "research.packet",
            "test",
        )
        .unwrap();
        assert_eq!(zero.0, Some(ExactFraction::new(0, 1).unwrap()));
        assert!(matches!(
            zero.1,
            Some(MetadataProvenance::AcceptedModelHypothesis { id, .. })
                if id == "research.packet"
        ));
    }
}
