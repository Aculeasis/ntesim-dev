use super::*;

#[test]
fn native_action_and_packet_metadata_are_resolved_independently() {
    let tables = ntesim_data::skill_levels::frozen::tables();
    let resolved = resolve_native_action_packet(
        "ntesim-current",
        tables,
        "character:1019",
        "GA_Mint019_Skill",
        "GE_Player_Mint_Skill1_Damage_New",
        &BTreeMap::new(),
    )
    .unwrap();
    assert!(resolved.action.categories.contains(&ActionCategory::Skill));
    assert!(matches!(
        &resolved.action.provenance,
        MetadataProvenance::Native { evidence }
            if evidence.table == ntesim_data::ability_tags::EVIDENCE_TABLE
    ));
    assert_eq!(resolved.action.native_ability_index, Some(2));
    assert_eq!(resolved.action.primary_slot, Some(PrimaryActionSlot::Skill));
    assert_eq!(resolved.packet.damage_kind, DamageKind::Elemental);
    assert_eq!(resolved.packet.element, Some(DamageElement::Nature));
    assert_eq!(
        resolved.packet.unbal_value,
        Some(ExactFraction::new(1, 2).unwrap())
    );
    assert!(matches!(
        resolved.packet.unbal_value_provenance,
        Some(MetadataProvenance::Native { .. })
    ));
    assert_eq!(resolved.binding.kind, ActionPacketBindingKind::NativeDirect);
    assert_eq!(
        resolved.binding.packet_ga_name.as_deref(),
        Some("GA_Mint019_Skill")
    );
    assert!(resolved.binding.provenance.is_none());
}

#[test]
fn native_unbal_value_keeps_exact_decimal_and_missing_zero_distinction() {
    let mut overlay = ntesim_data::skill_levels::frozen::overlay_tables();
    let nonzero_id = "GE_Player_Mint_Skill1_Damage_New";
    overlay.get_mut("DT_SkillDamageData").unwrap()["rows"][nonzero_id]["UnbalValue"] =
        serde_json::json!(1.2);
    let tables = &overlay;
    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        "ntesim-current",
        tables,
        nonzero_id,
    )
    .unwrap();
    let (value, provenance) = native::native_unbal_value(&native).unwrap();
    assert_eq!(value, Some(ExactFraction::new(6, 5).unwrap()));
    assert!(matches!(
        provenance,
        Some(MetadataProvenance::Native { .. })
    ));

    let mut missing_row = native;
    missing_row
        .raw_row
        .as_mut()
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("UnbalValue");
    assert_eq!(
        native::native_unbal_value(&missing_row).unwrap(),
        (None, None)
    );
}

#[test]
fn native_ga_name_binding_is_case_insensitive_and_preserves_source_identity() {
    let mut tables = ntesim_data::skill_levels::frozen::overlay_tables();
    tables.get_mut("DT_SkillDamageData").unwrap()["rows"]["GE_Player_Mint_Skill1_Damage_New"]["GAName"] =
        serde_json::json!("ga_mint019_skill");
    let resolved = resolve_native_action_packet(
        "ntesim-current",
        &tables,
        "character:1019",
        "GA_Mint019_Skill",
        "GE_Player_Mint_Skill1_Damage_New",
        &BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(resolved.action.ability_id, "GA_Mint019_Skill");
    assert_eq!(resolved.binding.kind, ActionPacketBindingKind::NativeDirect);
    assert_eq!(
        resolved.binding.packet_ga_name.as_deref(),
        Some("ga_mint019_skill")
    );
    assert_eq!(
        resolved.packet.source_evidence.as_ref().unwrap().table,
        "DT_SkillDamageData"
    );
}

#[test]
fn zero_packets_keep_their_native_owner_instead_of_a_foreign_alias() {
    let tables = ntesim_data::skill_levels::frozen::tables();
    for suffix in [
        "Skill4_Damage",
        "Skill_Kill_Damage_lv1",
        "Skill_Kill_Damage_lv2",
    ] {
        let own = format!("GE_Player_Female046_{suffix}");
        let resolved = resolve_requested_action_packet(
            "ntesim-current",
            tables,
            "character:1046",
            "GA_Female046_Skill",
            &own,
            None,
            &BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(resolved.binding.kind, ActionPacketBindingKind::NativeDirect);
        assert_eq!(
            resolved.packet.native_ga_name.as_deref(),
            Some("GA_Female046_Skill")
        );
        let foreign = format!("GE_Player_Female051_{suffix}");
        assert!(
            resolve_requested_action_packet(
                "ntesim-current",
                tables,
                "character:1046",
                "GA_Female046_Skill",
                &foreign,
                None,
                &BTreeMap::new()
            )
            .unwrap_err()
            .contains("has no native damage row")
        );
    }
}

#[test]
fn exact_owner_child_edges_keep_initiator_packet_evidence() {
    let tables = ntesim_data::skill_levels::frozen::tables();
    let (character_id, ability_id, effect_id, expected_ga_name) = (
        "character:1010",
        "GA_Nanally_Skill",
        "GE_Nanally010_Lv3_Damage",
        "GA_Nanally_Melee",
    );
    let native_action = ntesim_data::native_action_metadata::resolve_ability(
        "ntesim-current",
        tables,
        character_id,
        ability_id,
    )
    .unwrap();
    let action = resolve_action_metadata_with_owner(character_id, &native_action).unwrap();
    let binding =
        crate::characters::action_packet_bindings::resolve(character_id, ability_id, effect_id)
            .unwrap();
    let assumptions = BTreeMap::from([(
        binding.hypothesis_id.as_str().to_owned(),
        binding.hypothesis_id.description().to_owned(),
    )]);
    let resolved = resolve_packet_for_action(
        "ntesim-current",
        tables,
        character_id,
        &action,
        effect_id,
        &assumptions,
    )
    .unwrap();
    assert_eq!(resolved.action.ability_id, ability_id);
    assert_eq!(resolved.binding.kind, ActionPacketBindingKind::OwnerChild);
    assert_eq!(resolved.binding.initiating_ability_id, ability_id);
    assert_eq!(
        resolved.binding.packet_ga_name.as_deref(),
        Some(expected_ga_name)
    );
    assert!(matches!(
        resolved.binding.provenance.as_ref(),
        Some(MetadataProvenance::AcceptedModelHypothesis { id, description })
            if id == binding.hypothesis_id.as_str() && description == binding.hypothesis_id.description()
    ));
    assert_eq!(
        resolved.packet.source_evidence.as_ref().unwrap().table,
        ntesim_data::native_damage::TABLE_NAME
    );
}

#[test]
fn manual_native_ga_name_mismatch_requires_an_exact_accepted_binding() {
    let mut tables = ntesim_data::skill_levels::frozen::overlay_tables();
    tables.get_mut("DT_SkillDamageData").unwrap()["rows"]["GE_Player_Mint_Skill1_Damage_New"]["GAName"] =
        serde_json::json!("GA_Mint019_UltraSkill");
    let hypothesis_id = "test.native_action_packet_relationship";
    let description = "Test an explicit action-to-packet binding for a mismatched native GAName.";
    let assumptions = BTreeMap::from([(hypothesis_id.into(), description.into())]);
    let resolve = |research: Option<&crate::simulation::ResearchActionMetadata>| {
        resolve_requested_action_packet(
            "ntesim-current",
            &tables,
            "character:1019",
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
            research,
            &assumptions,
        )
    };
    let missing_declaration = resolve(None).unwrap_err();
    assert!(missing_declaration.contains("unsupported action/packet relationship"));

    let mut declaration = crate::simulation::ResearchActionMetadata {
        hypothesis_id: hypothesis_id.into(),
        ability_id: "GA_Mint019_Skill".into(),
        damage_effect_id: "GE_Player_Mint_Skill1_Damage_New".into(),
        action_gameplay_tags: None,
        input_action_role: None,
        damage_type_ex: None,
        damage_source_category: None,
        fixed_crit_rate: None,
        unbal_value: None,
        packet_ga_name_binding: Some("GA_Mint019_Melee".into()),
    };
    assert!(
        resolve(Some(&declaration))
            .unwrap_err()
            .contains("frozen row says GA_Mint019_UltraSkill")
    );
    declaration.packet_ga_name_binding = Some("ga_mint019_ultraskill".into());
    let resolved = resolve(Some(&declaration)).unwrap();
    assert_eq!(resolved.action.ability_id, "GA_Mint019_Skill");
    assert_eq!(
        resolved.binding.kind,
        ActionPacketBindingKind::ResearchDeclared
    );
    assert_eq!(
        resolved.binding.packet_ga_name.as_deref(),
        Some("GA_Mint019_UltraSkill")
    );
    assert!(matches!(
        resolved.binding.provenance.as_ref(),
        Some(MetadataProvenance::AcceptedModelHypothesis { id, description: actual })
            if id == hypothesis_id && actual == description
    ));
    assert_eq!(
        resolved.packet.source_evidence.as_ref().unwrap().table,
        "DT_SkillDamageData"
    );
}

#[test]
fn native_input_slot_and_combat_category_are_independent() {
    let tables = ntesim_data::skill_levels::frozen::tables();
    let action = ntesim_data::native_action_metadata::resolve_ability(
        "ntesim-current",
        tables,
        "character:1071",
        "GA_Chaos071_Skill",
    )
    .unwrap();
    let metadata = resolve_action_metadata_with_owner("character:1071", &action).unwrap();
    assert_eq!(metadata.native_ability_index, Some(2));
    assert_eq!(metadata.primary_slot, Some(PrimaryActionSlot::Skill));
    assert!(metadata.categories.contains(&ActionCategory::Melee));

    let ultimate = ntesim_data::native_action_metadata::resolve_ability(
        "ntesim-current",
        tables,
        "character:1071",
        "GA_Chaos071_UltraSkill",
    )
    .unwrap();
    let metadata = resolve_action_metadata(&ultimate).unwrap();
    assert_eq!(metadata.primary_slot, Some(PrimaryActionSlot::Ultimate));
    assert!(metadata.categories.contains(&ActionCategory::Melee));
}

#[test]
fn exact_zankou_dot_packet_tags_and_fixed_crit_require_visible_interpretations() {
    let tables = ntesim_data::skill_levels::frozen::tables();
    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        "ntesim-current",
        tables,
        "GE_Player_Zankou_DotDamage",
    )
    .unwrap();
    assert!(
        resolve_packet_metadata(&native, &BTreeMap::new())
            .unwrap_err()
            .contains(FIXED_CRIT_PROBABILITY_HYPOTHESIS)
    );

    let assumptions = BTreeMap::from([
        (
            FIXED_CRIT_PROBABILITY_HYPOTHESIS.into(),
            FIXED_CRIT_PROBABILITY_DESCRIPTION.into(),
        ),
        (
            crate::characters::packet_semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID.into(),
            crate::characters::packet_semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID
                .description()
                .into(),
        ),
    ]);
    let packet = resolve_packet_metadata(&native, &assumptions).unwrap();
    assert_eq!(
        packet.fixed_crit_rate,
        Some(ExactFraction::new(1, 2).unwrap())
    );
    assert_eq!(
        packet.semantic_tags,
        BTreeSet::from([
            "State.Damage".into(),
            "State.Damage.Dot".into(),
            "State.Damage.Melee".into(),
        ])
    );
    assert_eq!(
        packet.source_evidence.as_ref().unwrap().table,
        ntesim_data::native_damage::TABLE_NAME
    );
    assert!(packet.fixed_crit_rate_provenance.is_some());
    assert!(packet.semantic_provenance.is_some());
}

#[test]
fn packet_semantics_use_owner_declarations_not_identifier_suffixes() {
    let (tags, provenance) =
        owner_packet_semantics("GE_Synthetic_DotButNotDeclared", &BTreeMap::new()).unwrap();
    assert!(tags.is_empty());
    assert!(provenance.is_none());
}

#[test]
fn missing_unknown_and_unaccepted_packet_metadata_stay_errors() {
    let mut overlay = ntesim_data::skill_levels::frozen::overlay_tables();
    let normal_id = "GE_Player_Mint_Skill1_Damage_New";
    overlay.get_mut("DT_SkillDamageData").unwrap()["rows"][normal_id]["DamageTypeEX"] =
        serde_json::json!("EExecutionDamageTypeEX::DAMAGE_TYPE_NORMAL");
    let tables = &overlay;
    let normal = ntesim_data::native_action_metadata::resolve_damage_effect(
        "ntesim-current",
        tables,
        normal_id,
    )
    .unwrap();
    assert!(resolve_packet_metadata(&normal, &BTreeMap::new()).is_err());
    let accepted = BTreeMap::from([(
        NORMAL_PHYSICAL_HYPOTHESIS.into(),
        NORMAL_PHYSICAL_DESCRIPTION.into(),
    )]);
    let packet = resolve_packet_metadata(&normal, &accepted).unwrap();
    assert_eq!(packet.damage_kind, DamageKind::Elemental);
    assert_eq!(packet.element, Some(DamageElement::Physical));

    let mut unknown = ntesim_data::native_action_metadata::resolve_damage_effect(
        "ntesim-current",
        tables,
        "GE_Player_Mint_Skill1_Damage_New",
    )
    .unwrap();
    unknown.damage_type_ex = Some("EExecutionDamageTypeEX::DAMAGE_TYPE_DarkFlame".into());
    assert!(resolve_packet_metadata(&unknown, &accepted).is_err());
}

#[test]
fn research_action_accepts_only_the_explicit_parry_semantic_tag() {
    let hypothesis_id = "test.caller_declared_parry";
    let description = "Caller declares a Parry trigger for the bounded Hexblade model.";
    let action = research::action_from_research(
        "test-parry",
        &["Owner.Other".into(), "ntesim.trigger.parry_attack".into()],
        hypothesis_id,
        description,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(action.categories, BTreeSet::from([ActionCategory::Other]));
    assert_eq!(
        action.gameplay_tags,
        ["Owner.Other", "ntesim.trigger.parry_attack"]
    );
    assert!(matches!(
        action.provenance,
        MetadataProvenance::AcceptedModelHypothesis { id, description: actual }
            if id == hypothesis_id && actual == description
    ));

    let error = research::action_from_research(
        "test-parry",
        &["Owner.Other".into(), "ntesim.trigger.unknown".into()],
        hypothesis_id,
        description,
        None,
        None,
        None,
    )
    .unwrap_err();
    assert!(error.contains("unsupported native action gameplay tag"));
}
