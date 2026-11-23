use super::*;

fn weapon_parameter_tables() -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "DT_ForkUpgradeStarDataTable".into(),
            ntesim_data::weapon_refinements::overlay(),
        ),
        ("CT_ForkBuff".into(), ntesim_data::weapon_curves::overlay()),
    ])
}
mod reviewed_weapon_hp;

fn legal_native_build(
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
    character: &str,
    weapon: &str,
    set: &str,
) -> equipment::Request {
    let _ = (tables, coverage);
    serde_json::from_value(json!({
        "schema_version":1,"content_id":"ntesim-current","character_id":character,
        "weapon_id":weapon,"weapon_tier":1,"set_id":set,
        "set_count":4,"additive_attributes":{}
    }))
    .unwrap()
}

fn make_receipt(numerator: &str, denominator: &str) -> Value {
    json!({"numerator": numerator, "denominator": denominator})
}

fn studio_request(set_id: &str, set_count: u32) -> crate::build::studio_projection::Request {
    crate::build::studio_projection::Request {
        slot: 0,
        content_id: "ntesim-current".into(),
        character_id: "character:1003".into(),
        level: 80,
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        max_level: Some(80),
        weapon_id: None,
        weapon_level: None,
        weapon_max_level: None,
        refinement: 1,
        set_id: Some(set_id.into()),
        set_count,
        base_overrides: crate::build::studio_projection::BaseOverrides {
            atk: Some(100),
            hp: None,
            def: None,
        },
        additive_stats: BTreeMap::new(),
    }
}

fn native_request(set_id: &str, set_count: u32) -> ntesim_data::stat_projection::Request {
    ntesim_data::stat_projection::Request {
        character_id: "character:1003".into(),
        level: 80,
        max_level: Some(80),
        weapon_id: None,
        weapon_level: None,
        weapon_max_level: None,
        refinement: 1,
        set_id: Some(set_id.into()),
        set_count,
        base_overrides: BTreeMap::new(),
        additive_attributes: BTreeMap::new(),
    }
}

fn equipped_set_request(
    set_id: &str,
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
) -> ntesim_data::equipment::Request {
    legal_native_build(
        tables,
        coverage,
        "character:1003",
        "weapon:fork_mofeikesi",
        set_id,
    )
}

fn fireflies_fixture() -> (BTreeMap<String, Value>, Value, Request) {
    let (mut tables, coverage, _) = skill_fixture();
    tables.extend(weapon_parameter_tables());
    let request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/native-fireflies-built-game-research.json"
    )))
    .unwrap();
    (tables, coverage, request)
}

fn skill_fixture() -> (BTreeMap<String, Value>, Value, Request) {
    let tables = ntesim_data::skill_levels::frozen::overlay_tables();
    let coverage = ntesim_data::character::released_inventory();
    let request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/skill-level-built-game-research.json"
    )))
    .unwrap();
    (tables, coverage, request)
}

fn action_fixture() -> (
    BTreeMap<String, Value>,
    GameScenarioRequest,
    ActionProjection,
) {
    let tables = ntesim_data::skill_levels::frozen::overlay_tables();
    let request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/native-built-game-research.json"
    )))
    .unwrap();
    (tables, request.scenario, request.action_projection.unwrap())
}

mod adler_native_def_coefficients;
mod appliance_binds_exact_refinements_and_rejects_unknown_parameters_and_units;
mod appliance_native_build_projects_a_legal_mint_skill;
mod be_happy_binds_five_curves_and_rejects_changed_units;
mod bit_game_binds_five_curves_and_rejects_changed_units;
mod bittercake_binds_native_curves;
mod blast_candy_native_refinements_bind_exact_ratios_and_reject_changed_units;
mod bopu_binds_three_curves_and_rejects_changed_units;
mod boxing_candy_binds_both_exact_refinement_curves_and_checks_units;
mod boxing_candy_native_build_projects_a_legal_nanally_skill;
mod castle_binds_five_curves_and_rejects_changed_units;
mod castle_studio_projection_rounds_hp_and_preserves_native_provenance;
mod charge_efficiency_native_build;
mod compiled_assumptions_survive_scenario_export_and_reject_conflicting_overrides;
mod duplicate_characters_fail_before_build_compilation;
mod edgar_a5_native_build_hp;
mod elemental_suits_static_two_piece;
mod eternal_waltz_native_build_adds_hp_to_the_base_percent_domain;
mod exact_receipts_reject_missing_negative_zero_denominator_and_overflow;
mod fadia_native_hp_coefficients;
mod far_side_moon_binds_five_curves_and_rejects_changed_units;
mod fireflies_equipped_projection_overwrites_or_inserts_source_bound_parameters;
mod fireflies_projection_rejects_invalid_counts_damage_scope_and_source_units;
mod forgotten_binds_five_curves_and_rejects_changed_units;
mod gear_assumptions_expose_batch_two_four_weapon_hypotheses;
mod gold_record_binds_four_curves_and_rejects_changed_units;
mod gold_wool_binds_three_curves_and_rejects_changed_units;
mod gold_wool_native_build_projects_a_legal_chaos_skill;
mod knight_candy_binds_exact_refinements_and_checks_both_units;
mod koinobori_binds_three_curves_and_rejects_changed_units;
mod marching_beyond_time_binds_five_curves_and_rejects_changed_units;
mod missing_receipts_and_nonzero_unsupported_typed_bonus_fail;
mod motor_candy_binds_both_native_curves_and_rejects_changed_units;
mod motor_candy_native_build_projects_a_legal_mint_skill;
mod native_action_projection_atomicity;
mod native_action_projection_replaces_rates_and_preserves_exact_coordinates;
mod native_gear_projection_overwrites_t_parameters_or_inserts_the_supported_proc;
mod native_gear_projection_rejects_unsupported_identity_duplicates_units_and_rounding;
mod native_mixed_scaling;
mod native_projection_rejects_scaling_and_crit_gaps_but_accepts_other_packet_types;
mod native_weapon_curve_projection_supports_mint_and_sakiri_multi_weapon_team;
mod noros_binds_atk_duration_and_separate_cooldown_curves;
mod noros_native_build_projects_a_legal_jiuyuan_skill;
mod oraora_binds_five_curves_to_legal_group_three_wearers;
mod owner_packet_coefficients;
mod paper_plane_binds_five_exact_refinements_and_rejects_extra_parameters;
mod paper_plane_native_build_compiler_preserves_equipped_set_and_executes_its_passive;
mod prokaryon_binds_exact_refinements_and_rejects_unknown_parameters_and_units;
mod prokaryon_native_build_projects_a_legal_nanally_attack;
mod purchased_levels_replace_forged_coordinates_and_resonance_has_an_exact_boundary;
mod quiet_manor_native_build;
mod raging_flames_binds_four_curves_and_rejects_changed_units;
mod registered_weapon_adapters;
mod skill_level_source_gaps_are_rejected_instead_of_defaulted;
mod skill_purchase_validation_checks_unused_selections_and_progression;
mod spider_binds_three_curves_and_rejects_changed_units;
mod suit10_hp_suit_static_two_piece;
mod suit11_speedy_hedgehog_static_two_piece;
mod suit12_quiet_manor_static_two_piece;
mod suit7_kingdoms_guard_static_two_piece;
mod suit8_shadow_creed_static_two_piece;
mod suit9_theas_tavern_static_four_piece;
mod suit9_theas_tavern_static_two_piece;
mod universal_skill_level_all_characters;
mod universal_skill_level_projection_supports_sakiri_and_mint_team;
mod unknown_element_never_defaults_to_zero_bonus;
mod unverified_character_fails_before_build_compilation;

fn verify_additive_element_damage(request: GameScenarioRequest, element: &str) {
    verify_additive_element_damage_off_element(
        request,
        element,
        "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE",
    )
}

fn verify_additive_element_damage_off_element(
    mut request: GameScenarioRequest,
    element: &str,
    off_element_damage_type: &str,
) {
    let plain = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let hit = &plain["hit_receipts"].as_array().unwrap()[0];
    assert_eq!(hit["non_crit_damage"], json!("1000/1"));
    request.actors[0].static_stats.elemental_bonuses.insert(
        element.into(),
        Ratio {
            numerator: 1,
            denominator: 10,
        },
    );
    let buffed = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let hit = &buffed["hit_receipts"].as_array().unwrap()[0];
    assert_eq!(hit["non_crit_damage"], json!("1100/1"));
    assert_eq!(hit["static_element_damage_bonus"], json!("1/10"));
    // Zone additivity: general +10% and Chaos +10% sum in one zone
    // (1 + 1/10 + 1/10 = 6/5 -> 1200), not 1000 x 1.1 x 1.1 = 1210.
    request.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 10,
    };
    let combined = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let hit = &combined["hit_receipts"].as_array().unwrap()[0];
    assert_eq!(hit["non_crit_damage"], json!("1200/1"));
    request.action_metadata.get_mut(&0).unwrap().damage_type_ex =
        Some(off_element_damage_type.into());
    let off_element = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let hit = &off_element["hit_receipts"].as_array().unwrap()[0];
    assert_eq!(hit["non_crit_damage"], json!("1100/1"));
    assert_eq!(hit["static_element_damage_bonus"], json!("0/1"));
}
