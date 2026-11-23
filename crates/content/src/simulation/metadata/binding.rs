use super::*;

pub(crate) fn resolve_action_metadata_with_owner(
    character_id: &str,
    native: &NativeAbilityMetadata,
) -> Result<ActionMetadata, String> {
    match resolve_action_metadata(native) {
        Ok(metadata) => Ok(metadata),
        Err(native_error) => {
            let Some(owner_role) =
                crate::characters::action_roles::resolve(character_id, &native.ability_id)
            else {
                return Err(native_error);
            };
            let native_category_absent = native.row_state == RowState::MissingNativeRow
                || matches!(native.tag_state, FieldState::Missing | FieldState::Empty)
                || (!native.gameplay_tag_names.is_empty()
                    && native
                        .gameplay_tag_names
                        .iter()
                        .all(|tag| tag.value.as_deref() == Some("None")));
            if !native_category_absent {
                return Err(native_error);
            }
            let (input_role, primary_slot, input_role_provenance) = native_input_role(native)?;
            let mut tags = native
                .gameplay_tag_names
                .iter()
                .filter_map(|tag| tag.value.clone())
                .collect::<Vec<_>>();
            let tag = owner_role.category.native_tag().to_owned();
            if !tags.contains(&tag) {
                tags.push(tag);
            }
            Ok(ActionMetadata {
                categories: BTreeSet::from([owner_role.category]),
                gameplay_tags: tags,
                ability_id: native.ability_id.clone(),
                native_ability_index: native.native_ability_index,
                input_role: Some(input_role),
                input_role_provenance: Some(input_role_provenance),
                primary_slot,
                source_evidence: native.evidence.clone(),
                identity_evidence: Some(native.config_evidence.clone()),
                provenance: MetadataProvenance::AcceptedModelHypothesis {
                    id: owner_role.hypothesis_id.into(),
                    description: owner_role.description.into(),
                },
            })
        }
    }
}

pub(super) fn owner_packet_semantics(
    effect_id: &str,
    accepted_assumptions: &BTreeMap<String, String>,
) -> Result<(BTreeSet<String>, Option<MetadataProvenance>), String> {
    let Some(semantics) = crate::characters::packet_semantics::resolve(effect_id) else {
        return Ok((BTreeSet::new(), None));
    };
    if accepted_assumptions
        .get(semantics.hypothesis_id.as_str())
        .map(String::as_str)
        != Some(semantics.hypothesis_id.description())
    {
        return Err(format!(
            "packet {} requires exact accepted model assumption {}",
            effect_id, semantics.hypothesis_id
        ));
    }
    Ok((
        semantics.tags.iter().map(|tag| (*tag).to_owned()).collect(),
        Some(MetadataProvenance::AcceptedModelHypothesis {
            id: semantics.hypothesis_id.as_str().into(),
            description: semantics.hypothesis_id.description().into(),
        }),
    ))
}

pub(super) fn resolve_action_packet_binding(
    character_id: &str,
    initiating_ability_id: &str,
    damage_effect_id: &str,
    native_packet: &NativeDamageEffectMetadata,
    research: Option<(&crate::simulation::ResearchActionMetadata, &str)>,
) -> Result<ActionPacketBinding, String> {
    let native_ga_name = native_packet.ga_name.as_deref();
    let make = |kind, provenance| ActionPacketBinding {
        kind,
        energy_group_id: None,
        initiating_ability_id: initiating_ability_id.to_owned(),
        packet_ga_name: native_packet.ga_name.clone(),
        provenance,
    };

    if native_packet.row_state == RowState::MissingNativeRow {
        let Some((metadata, description)) = research else {
            return Err(format!(
                "packet relationship for {initiating_ability_id} → {damage_effect_id} has no native row or explicit research declaration"
            ));
        };
        if metadata.packet_ga_name_binding.is_some() {
            return Err(format!(
                "research packet GAName binding for {damage_effect_id} cannot be checked because its native row is missing"
            ));
        }
        return Ok(make(
            ActionPacketBindingKind::ResearchDeclared,
            Some(MetadataProvenance::AcceptedModelHypothesis {
                id: metadata.hypothesis_id.clone(),
                description: description.to_owned(),
            }),
        ));
    }

    if let Some(native_ga_name) = native_ga_name
        && native_ga_name.eq_ignore_ascii_case(initiating_ability_id)
    {
        if research.is_some_and(|(metadata, _)| metadata.packet_ga_name_binding.is_some()) {
            return Err(format!(
                "research packet GAName binding for {damage_effect_id} is unused because its native GAName directly matches the initiating action"
            ));
        }
        return Ok(make(ActionPacketBindingKind::NativeDirect, None));
    }

    if let Some(owner_binding) = crate::characters::action_packet_bindings::resolve(
        character_id,
        initiating_ability_id,
        &native_packet.effect_id,
    ) {
        let Some(native_ga_name) = native_ga_name else {
            return Err(format!(
                "owner packet binding for {initiating_ability_id} → {damage_effect_id} requires native GAName {} but the frozen row omits GAName",
                owner_binding.packet_ga_name
            ));
        };
        if !native_ga_name.eq_ignore_ascii_case(owner_binding.packet_ga_name) {
            return Err(format!(
                "owner packet binding for {initiating_ability_id} → {damage_effect_id} expects native GAName {}, found {native_ga_name}",
                owner_binding.packet_ga_name
            ));
        }
        if research.is_some_and(|(metadata, _)| metadata.packet_ga_name_binding.is_some()) {
            return Err(format!(
                "research packet GAName binding cannot override the owner-declared relationship for {damage_effect_id}"
            ));
        }
        let provenance = MetadataProvenance::AcceptedModelHypothesis {
            id: owner_binding.hypothesis_id.as_str().into(),
            description: owner_binding.hypothesis_id.description().into(),
        };
        return Ok(make(ActionPacketBindingKind::OwnerChild, Some(provenance)));
    }

    let Some((metadata, description)) = research else {
        let detail = native_ga_name.map_or_else(
            || "native row has no GAName".to_owned(),
            |name| format!("native GAName {name} does not match the initiating action"),
        );
        return Err(format!(
            "unsupported action/packet relationship for {initiating_ability_id} → {damage_effect_id}: {detail}"
        ));
    };
    let declared_ga_name = metadata.packet_ga_name_binding.as_deref().ok_or_else(|| {
        format!(
            "research packet relationship for {initiating_ability_id} → {damage_effect_id} requires packet_ga_name_binding and its exact accepted hypothesis"
        )
    })?;
    let Some(native_ga_name) = native_ga_name else {
        return Err(format!(
            "research packet GAName binding for {damage_effect_id} cannot be verified because its native row omits GAName"
        ));
    };
    if !declared_ga_name.eq_ignore_ascii_case(native_ga_name) {
        return Err(format!(
            "research packet GAName binding for {damage_effect_id} declares {declared_ga_name}, but the frozen row says {native_ga_name}"
        ));
    }
    Ok(make(
        ActionPacketBindingKind::ResearchDeclared,
        Some(MetadataProvenance::AcceptedModelHypothesis {
            id: metadata.hypothesis_id.clone(),
            description: description.to_owned(),
        }),
    ))
}
