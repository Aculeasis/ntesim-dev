use super::*;

pub(crate) fn native_input_role(
    native: &NativeAbilityMetadata,
) -> Result<
    (
        InputActionRole,
        Option<PrimaryActionSlot>,
        MetadataProvenance,
    ),
    String,
> {
    let role =
        ntesim_data::native_action_metadata::resolve_primary_role(native.native_ability_index)?;
    let (input_role, primary_slot) = match role {
        ntesim_data::native_action_metadata::NativePrimaryRole::Normal => {
            (InputActionRole::Normal, Some(PrimaryActionSlot::Normal))
        }
        ntesim_data::native_action_metadata::NativePrimaryRole::RedirectSkill => (
            InputActionRole::RedirectSkill,
            Some(PrimaryActionSlot::Skill),
        ),
        ntesim_data::native_action_metadata::NativePrimaryRole::Ultimate => {
            (InputActionRole::Ultimate, Some(PrimaryActionSlot::Ultimate))
        }
        ntesim_data::native_action_metadata::NativePrimaryRole::Qte => (InputActionRole::Qte, None),
    };
    Ok((
        input_role,
        primary_slot,
        MetadataProvenance::AcceptedModelHypothesis {
            id: ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
            description:
                ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_DESCRIPTION
                    .into(),
        },
    ))
}

pub fn resolve_action_metadata(native: &NativeAbilityMetadata) -> Result<ActionMetadata, String> {
    if native.row_state != RowState::Present {
        return Err(format!(
            "unsupported action metadata: {} has no DT_CharacterAbilityEffectConfig row",
            native.ability_id
        ));
    }
    if native.tag_state != FieldState::Present {
        return Err(format!(
            "unsupported action metadata: {} has no configured AbilityGameplayTag.TagName",
            native.ability_id
        ));
    }
    let mut categories = BTreeSet::new();
    let mut tags = Vec::new();
    for tag in &native.gameplay_tag_names {
        if tag.state != FieldState::Present {
            continue;
        }
        let value = tag.value.as_deref().ok_or_else(|| {
            format!(
                "unsupported action metadata: missing TagName at {}",
                tag.pointer
            )
        })?;
        tags.push(value.to_owned());
        if value != "None" {
            categories.insert(ActionCategory::from_native_tag(value)?);
        }
    }
    if categories.is_empty() {
        return Err(format!(
            "unsupported action metadata: {} has no native action category; an owner declaration is required",
            native.ability_id
        ));
    }
    let evidence = native
        .evidence
        .clone()
        .ok_or("native action metadata has no row evidence")?;
    let (input_role, primary_slot, input_role_provenance) = native_input_role(native)?;
    Ok(ActionMetadata {
        categories,
        gameplay_tags: tags,
        ability_id: native.ability_id.clone(),
        native_ability_index: native.native_ability_index,
        input_role: Some(input_role),
        input_role_provenance: Some(input_role_provenance),
        primary_slot,
        source_evidence: native.evidence.clone(),
        identity_evidence: Some(native.config_evidence.clone()),
        provenance: MetadataProvenance::Native { evidence },
    })
}

pub(crate) fn resolve_packet_metadata(
    native: &NativeDamageEffectMetadata,
    accepted_assumptions: &BTreeMap<String, String>,
) -> Result<PacketMetadata, String> {
    if native.row_state != RowState::Present {
        return Err(format!(
            "unsupported packet metadata: {} has no DT_SkillDamageData row",
            native.effect_id
        ));
    }
    let raw = native.damage_type_ex.as_deref().ok_or_else(|| {
        format!(
            "unsupported packet metadata: {} has no string DamageTypeEX",
            native.effect_id
        )
    })?;
    let (fixed_crit_rate, fixed_crit_rate_provenance) =
        native_fixed_crit_rate(native, accepted_assumptions)?;
    let (unbal_value, unbal_value_provenance) = native_unbal_value(native)?;
    let (damage_kind, element, provenance) = match raw {
        "EExecutionDamageTypeEX::DAMAGE_TYPE_COSMOS" => (
            DamageKind::Elemental,
            Some(DamageElement::Cosmos),
            native_provenance(native)?,
        ),
        "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE" => (
            DamageKind::Elemental,
            Some(DamageElement::Nature),
            native_provenance(native)?,
        ),
        "EExecutionDamageTypeEX::DAMAGE_TYPE_CHAOS" => (
            DamageKind::Elemental,
            Some(DamageElement::Chaos),
            native_provenance(native)?,
        ),
        "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION" => (
            DamageKind::Elemental,
            Some(DamageElement::Incantation),
            native_provenance(native)?,
        ),
        "EExecutionDamageTypeEX::DAMAGE_TYPE_LAKSHANA" => (
            DamageKind::Elemental,
            Some(DamageElement::Lakshana),
            native_provenance(native)?,
        ),
        "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE" => (
            DamageKind::Elemental,
            Some(DamageElement::Psyche),
            native_provenance(native)?,
        ),
        "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHICALLY" => {
            (DamageKind::Mental, None, native_provenance(native)?)
        }
        "EExecutionDamageTypeEX::DAMAGE_TYPE_TRUE" => {
            (DamageKind::True, None, native_provenance(native)?)
        }
        "EExecutionDamageTypeEX::DAMAGE_TYPE_NORMAL" => {
            let description = accepted_assumptions
                .get(NORMAL_PHYSICAL_HYPOTHESIS)
                .filter(|value| *value == NORMAL_PHYSICAL_DESCRIPTION)
                .ok_or_else(|| {
                    format!(
                        "unsupported packet metadata: {} uses DAMAGE_TYPE_NORMAL; accept model assumption {NORMAL_PHYSICAL_HYPOTHESIS} to model it as physical",
                        native.effect_id
                    )
                })?;
            (
                DamageKind::Elemental,
                Some(DamageElement::Physical),
                MetadataProvenance::AcceptedModelHypothesis {
                    id: NORMAL_PHYSICAL_HYPOTHESIS.into(),
                    description: description.clone(),
                },
            )
        }
        unsupported => {
            return Err(format!(
                "unsupported packet metadata: {} has unmodeled DamageTypeEX {unsupported}",
                native.effect_id
            ));
        }
    };
    let (semantic_tags, semantic_provenance) =
        owner_packet_semantics(&native.effect_id, accepted_assumptions)?;
    Ok(PacketMetadata {
        damage_effect_id: native.effect_id.clone(),
        native_ga_name: native.ga_name.clone(),
        native_damage_type_ex: raw.to_owned(),
        native_damage_source_category: native.damage_source_category.clone(),
        damage_kind,
        element,
        fixed_crit_rate,
        fixed_crit_rate_provenance,
        unbal_value,
        unbal_value_provenance,
        semantic_tags,
        semantic_provenance,
        source_evidence: native.evidence.clone(),
        provenance,
        packet_kind: PacketClassification::Standard,
    })
}

pub(super) fn native_unbal_value(
    native: &NativeDamageEffectMetadata,
) -> Result<(Option<ExactFraction>, Option<MetadataProvenance>), String> {
    let row = native
        .raw_row
        .as_ref()
        .ok_or_else(|| format!("native packet {} has no serialized row", native.effect_id))?;
    let Some(raw) = row.get("UnbalValue") else {
        return Ok((None, None));
    };
    let parsed = ntesim_data::native::parse_native_scalar(raw).map_err(|error| {
        format!(
            "native packet {} has invalid UnbalValue: {error}",
            native.effect_id
        )
    })?;
    let value = ExactFraction::new(parsed.numerator(), parsed.denominator())?;
    Ok((Some(value), Some(native_provenance(native)?)))
}

pub(super) fn native_fixed_crit_rate(
    native: &NativeDamageEffectMetadata,
    accepted_assumptions: &BTreeMap<String, String>,
) -> Result<(Option<ExactFraction>, Option<MetadataProvenance>), String> {
    let row = native
        .raw_row
        .as_ref()
        .ok_or_else(|| format!("native packet {} has no serialized row", native.effect_id))?;
    let raw = row.get("FixedCritRate").ok_or_else(|| {
        format!(
            "native packet {} is missing FixedCritRate",
            native.effect_id
        )
    })?;
    let parsed = ntesim_data::native::parse_native_scalar(raw).map_err(|error| {
        format!(
            "native packet {} has invalid FixedCritRate: {error}",
            native.effect_id
        )
    })?;
    let value = ExactFraction::new(parsed.numerator(), parsed.denominator())?;
    if value.numerator > value.denominator {
        return Err(format!(
            "native packet {} FixedCritRate is outside [0, 1]",
            native.effect_id
        ));
    }
    // Frozen 0.0 is the source's no-override sentinel. A fixed probability of
    // zero remains expressible by research metadata as Some(0/1).
    if value.numerator == 0 {
        return Ok((None, None));
    }
    if let Some((sentinel, rate, model)) = crate::registry::fixed_crit_encoding(&native.effect_id)
        && value == sentinel
        && accepted_assumptions.get(model.as_str()).map(String::as_str) == Some(model.description())
    {
        return Ok((
            Some(rate),
            Some(MetadataProvenance::AcceptedModelHypothesis {
                id: model.into(),
                description: model.description().into(),
            }),
        ));
    }
    if accepted_assumptions
        .get(FIXED_CRIT_PROBABILITY_HYPOTHESIS)
        .map(String::as_str)
        != Some(FIXED_CRIT_PROBABILITY_DESCRIPTION)
    {
        return Err(format!(
            "native packet {} has nonzero FixedCritRate; accept model assumption {FIXED_CRIT_PROBABILITY_HYPOTHESIS}",
            native.effect_id
        ));
    }
    Ok((
        Some(value),
        Some(MetadataProvenance::AcceptedModelHypothesis {
            id: FIXED_CRIT_PROBABILITY_HYPOTHESIS.into(),
            description: FIXED_CRIT_PROBABILITY_DESCRIPTION.into(),
        }),
    ))
}

fn native_provenance(native: &NativeDamageEffectMetadata) -> Result<MetadataProvenance, String> {
    Ok(MetadataProvenance::Native {
        evidence: native
            .evidence
            .clone()
            .ok_or("native damage metadata has no row evidence")?,
    })
}

#[cfg(test)]
pub(crate) fn resolve_native_action_packet(
    content_id: &str,
    tables: &BTreeMap<String, serde_json::Value>,
    character_id: &str,
    ability_id: &str,
    damage_effect_id: &str,
    accepted_assumptions: &BTreeMap<String, String>,
) -> Result<ActionPacketMetadata, String> {
    let ability = ntesim_data::native_action_metadata::resolve_ability(
        content_id,
        tables,
        character_id,
        ability_id,
    )?;
    let effect = ntesim_data::native_action_metadata::resolve_damage_effect(
        content_id,
        tables,
        damage_effect_id,
    )?;
    let action = resolve_action_metadata(&ability)?;
    let packet = resolve_packet_metadata(&effect, accepted_assumptions)?;
    let binding = resolve_action_packet_binding(
        character_id,
        &action.ability_id,
        damage_effect_id,
        &effect,
        None,
    )?;
    Ok(ActionPacketMetadata {
        action,
        packet,
        binding,
    })
}

pub(crate) fn resolve_packet_for_action(
    content_id: &str,
    tables: &BTreeMap<String, serde_json::Value>,
    character_id: &str,
    action: &ActionMetadata,
    damage_effect_id: &str,
    accepted_assumptions: &BTreeMap<String, String>,
) -> Result<ActionPacketMetadata, String> {
    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        content_id,
        tables,
        damage_effect_id,
    )?;
    let packet = resolve_packet_metadata(&native, accepted_assumptions)?;
    let binding = resolve_action_packet_binding(
        character_id,
        &action.ability_id,
        damage_effect_id,
        &native,
        None,
    )?;
    Ok(ActionPacketMetadata {
        action: action.clone(),
        packet,
        binding,
    })
}
