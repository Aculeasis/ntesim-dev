use crate::build::studio_projection::{BaseOverrides, Request, project as studio_project};
use ntesim_core::formula::Ratio;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn receipt(numerator: &str, denominator: &str) -> Value {
    json!({"numerator": numerator, "denominator": denominator})
}

fn studio_request(set_count: u32) -> Request {
    Request {
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
        set_id: Some("set:Suit7".into()),
        set_count,
        base_overrides: BaseOverrides {
            atk: Some(100),
            hp: Some(200),
            def: Some(50),
        },
        additive_stats: BTreeMap::from([
            (
                "atk_percent".into(),
                Ratio {
                    numerator: 1,
                    denominator: 3,
                },
            ),
            (
                "atk".into(),
                Ratio {
                    numerator: 1,
                    denominator: 8,
                },
            ),
            (
                "hp".into(),
                Ratio {
                    numerator: 1,
                    denominator: 2,
                },
            ),
            (
                "def".into(),
                Ratio {
                    numerator: 1,
                    denominator: 3,
                },
            ),
        ]),
    }
}

fn native_request(set_count: u32) -> ntesim_data::stat_projection::Request {
    ntesim_data::stat_projection::Request {
        character_id: "character:1003".into(),
        level: 80,
        max_level: Some(80),
        weapon_id: None,
        weapon_level: None,
        weapon_max_level: None,
        refinement: 1,
        set_id: Some("set:Suit7".into()),
        set_count,
        base_overrides: BTreeMap::from([
            ("AtkBase".into(), receipt("100", "1")),
            ("HPMaxBase".into(), receipt("200", "1")),
            ("DefBase".into(), receipt("50", "1")),
        ]),
        additive_attributes: BTreeMap::from([
            ("AtkUp".into(), receipt("1", "3")),
            ("AtkAdd".into(), receipt("1", "8")),
            ("HPMaxUp".into(), receipt("1", "4")),
            ("HPMaxAdd".into(), receipt("1", "2")),
            ("DefAdd".into(), receipt("1", "3")),
        ]),
    }
}

fn equipped_native_request(
    tables: &BTreeMap<String, serde_json::Value>,
    coverage: &serde_json::Value,
) -> ntesim_data::equipment::Request {
    super::legal_native_build(
        tables,
        coverage,
        "character:1020",
        "weapon:fork_Nakupeda",
        "set:Suit7",
    )
}

#[test]
fn native_suit7_packs_bind_exact_static_values() {
    let structs = ntesim_data::set_definitions::transport()["rows"]["Suit7"]["SuitStructList"]
        .as_array()
        .unwrap();
    assert_eq!(structs.len(), 2);
    assert_eq!(structs[0]["SuitCondition"], 2);
    assert_eq!(structs[1]["SuitCondition"], 4);
    assert!(structs[0]["SuitBuff"].is_null() && structs[1]["SuitBuff"].is_null());
    assert_eq!(structs[0]["SuitModifyPackID"], "Shield_2");
    assert_eq!(structs[1]["SuitModifyPackID"], "Shield_4");
    let packs = &ntesim_data::set_modifiers::transport()["rows"];
    let two = &packs["Shield_2"];
    assert!(two["ConditionArray"].as_array().unwrap().is_empty());
    let two_entries = two["ModifyData"].as_array().unwrap();
    assert_eq!(two_entries.len(), 1);
    assert_eq!(two_entries[0]["PropName"], "DefUp");
    assert_eq!(two_entries[0]["PropValue"], 0.15);
    assert_eq!(
        two_entries[0]["ModifierOp"],
        "EModifyModOp::MODIFY_MODOP_ADDITIVE"
    );
    let four = &packs["Shield_4"];
    assert!(four["ConditionArray"].as_array().unwrap().is_empty());
    let four_entries = four["ModifyData"].as_array().unwrap();
    assert_eq!(four_entries.len(), 1);
    assert_eq!(four_entries[0]["PropName"], "ShieldEfficiency");
    assert_eq!(four_entries[0]["PropValue"], 0.2);
    assert_eq!(
        four_entries[0]["ModifierOp"],
        "EModifyModOp::MODIFY_MODOP_ADDITIVE"
    );
}

#[test]
fn two_piece_def_bonus_projects_through_shared_additive_interpreter() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    // Independent oracle: DefBase 50 (override), DefAdd 1/3 (additive fixture).
    // Below threshold DefUp is 0: def = 50 + 1/3 = 151/3.
    for count in [0, 1] {
        let report = ntesim_data::stat_projection::compile(tables, &native_request(count)).unwrap();
        assert_eq!(report["attributes"]["DefUp"], receipt("0", "1"));
        assert_eq!(report["derived"]["def_receipt"], receipt("151", "3"));
    }
    // At two pieces Shield_2 adds DefUp 3/20: def = 50 * 23/20 + 1/3 = 347/6.
    for count in [2, 3] {
        let report = ntesim_data::stat_projection::compile(tables, &native_request(count)).unwrap();
        assert_eq!(report["attributes"]["DefUp"], receipt("3", "20"));
        assert_eq!(report["derived"]["def_receipt"], receipt("347", "6"));
    }
}

#[test]
fn studio_two_piece_def_bonus_consumes_native_attribute_with_evidence() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for count in 0..=3 {
        let report = studio_project(tables, &studio_request(count)).unwrap();
        // Independent oracle: Shield_2 grants DefUp 3/20 at two pieces.
        let (numerator, denominator) = if count >= 2 { (3, 20) } else { (0, 1) };
        assert_eq!(report.static_stats.def_percent.numerator, numerator);
        assert_eq!(report.static_stats.def_percent.denominator, denominator);
        assert!(!report.unmodeled_attributes.contains_key("DefUp"));
        if count >= 2 {
            // Hand calculation: nearest(50 * 23/20 + 1/3) = nearest(347/6) = 58.
            assert_eq!(report.static_stats.def, 58);
            let evidence = serde_json::to_value(&report.evidence).unwrap();
            assert!(evidence.as_array().unwrap().iter().any(|entry| {
                entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID
                    && entry["row"] == "Shield_2"
            }));
        } else {
            // Hand calculation: nearest(50 + 1/3) = nearest(151/3) = 50.
            assert_eq!(report.static_stats.def, 50);
        }
    }
}

#[test]
fn studio_quantizes_completed_hp_and_def_once_after_additions() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    // Independent product oracle: 100*(1+1/4)+2.49 = 127.49 -> 127;
    // replacing 2.49 with 2.5 gives an exact 127.5 -> 128 half-up tie.
    for (flat_numerator, flat_denominator, expected) in [(249, 100, 127), (5, 2, 128)] {
        let mut request = studio_request(0);
        request.base_overrides.hp = Some(100);
        request.base_overrides.def = Some(100);
        request.additive_stats.clear();
        request.additive_stats.extend([
            (
                "hp_percent".into(),
                Ratio {
                    numerator: 1,
                    denominator: 4,
                },
            ),
            (
                "hp".into(),
                Ratio {
                    numerator: flat_numerator,
                    denominator: flat_denominator,
                },
            ),
            (
                "def_percent".into(),
                Ratio {
                    numerator: 1,
                    denominator: 4,
                },
            ),
            (
                "def".into(),
                Ratio {
                    numerator: flat_numerator,
                    denominator: flat_denominator,
                },
            ),
        ]);

        let raw_request = ntesim_data::stat_projection::Request {
            character_id: "character:1003".into(),
            level: 80,
            max_level: Some(80),
            weapon_id: None,
            weapon_level: None,
            weapon_max_level: None,
            refinement: 1,
            set_id: None,
            set_count: 0,
            base_overrides: BTreeMap::from([
                ("HPMaxBase".into(), receipt("100", "1")),
                ("DefBase".into(), receipt("100", "1")),
            ]),
            additive_attributes: BTreeMap::from([
                ("HPMaxUp".into(), receipt("1", "4")),
                (
                    "HPMaxAdd".into(),
                    receipt(&flat_numerator.to_string(), &flat_denominator.to_string()),
                ),
                ("DefUp".into(), receipt("1", "4")),
                (
                    "DefAdd".into(),
                    receipt(&flat_numerator.to_string(), &flat_denominator.to_string()),
                ),
            ]),
        };
        let raw = ntesim_data::stat_projection::compile(tables, &raw_request).unwrap();
        let expected_receipt = if expected == 127 {
            receipt("12749", "100")
        } else {
            receipt("255", "2")
        };
        assert_eq!(raw["derived"]["hp_receipt"], expected_receipt);
        assert_eq!(raw["derived"]["def_receipt"], expected_receipt);

        let report = studio_project(tables, &request).unwrap();
        assert_eq!(report.static_stats.hp, expected);
        assert_eq!(report.static_stats.def, expected);
        assert_eq!(
            report.model_assumptions
                [crate::build::assumptions::STAT_QUANTIZATION_HYPOTHESIS_ID.as_str()],
            crate::build::assumptions::STAT_QUANTIZATION_HYPOTHESIS_ID.description()
        );
    }
}

#[test]
fn four_piece_shield_efficiency_projects_as_static_granter_bonus() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let report = studio_project(tables, &studio_request(4)).unwrap();
    assert_eq!(
        report.static_stats.shield_bonus,
        Ratio {
            numerator: 1,
            denominator: 5
        }
    );
    assert!(!report.unmodeled_attributes.contains_key("ShieldEfficiency"));
    let native = ntesim_data::stat_projection::compile(tables, &native_request(4)).unwrap();
    assert_eq!(native["attributes"]["ShieldEfficiency"], receipt("1", "5"));
}

#[test]
fn native_equipped_four_piece_projects_typed_shield_bonus() {
    let (tables, coverage, _) = super::skill_fixture();
    let request = equipped_native_request(&tables, &coverage);
    let mut report = ntesim_data::equipment::compile(&tables, &request).unwrap();
    assert_eq!(
        report["known_attributes"]["ShieldEfficiency"]["receipt"],
        receipt("1", "5")
    );
    assert_eq!(
        crate::build::stats::project_shield_bonus(&report).unwrap(),
        Ratio {
            numerator: 1,
            denominator: 5
        }
    );
    assert_eq!(
        report["known_derived_stats"]["def_receipt"],
        receipt("32542", "25")
    );
    let projected = crate::build::project(&report, "DamageUpNatureBase").unwrap();
    assert_eq!(projected.def, 1302);
    assert_eq!(
        projected.hp,
        crate::build::receipt(&report["known_derived_stats"]["hp_receipt"])
            .unwrap()
            .round_u64()
            .unwrap()
    );

    // Native equipped projection keeps HP/DEF fractional until the same one-time
    // product settlement: 201/2 -> 101 and 2009/20 -> 100. Native behavior remains open.
    report["known_derived_stats"]["hp_receipt"] = receipt("201", "2");
    report["known_derived_stats"]["def_receipt"] = receipt("201", "2");
    let half_ties = crate::build::project(&report, "DamageUpNatureBase").unwrap();
    assert_eq!(half_ties.hp, 101);
    assert_eq!(half_ties.def, 101);
    report["known_derived_stats"]["hp_receipt"] = receipt("2009", "20");
    report["known_derived_stats"]["def_receipt"] = receipt("2009", "20");
    let below_ties = crate::build::project(&report, "DamageUpNatureBase").unwrap();
    assert_eq!(below_ties.hp, 100);
    assert_eq!(below_ties.def, 100);
}

#[test]
fn native_equipped_build_without_shield_attribute_defaults_to_zero() {
    let (tables, coverage, _) = super::skill_fixture();
    let request = super::legal_native_build(
        &tables,
        &coverage,
        "character:1020",
        "weapon:fork_Nakupeda",
        "set:Suit1",
    );
    let mut report = ntesim_data::equipment::compile(&tables, &request).unwrap();
    report["known_attributes"]
        .as_object_mut()
        .unwrap()
        .remove("ShieldEfficiency");
    assert_eq!(
        crate::build::stats::project_shield_bonus(&report).unwrap(),
        Ratio::zero()
    );
}

#[test]
fn nacupeda_accepts_native_four_piece_shield_strength() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let mut request = studio_request(4);
    request.character_id = "character:1020".into();
    request.weapon_id = Some("weapon:fork_Nakupeda".into());
    request.weapon_level = Some(80);
    request.weapon_max_level = Some(80);
    let four_piece = studio_project(tables, &request).unwrap();
    assert_eq!(
        four_piece.static_stats.shield_bonus,
        Ratio {
            numerator: 1,
            denominator: 5
        }
    );
    request.set_count = 2;
    assert!(studio_project(tables, &request).is_ok());
}
