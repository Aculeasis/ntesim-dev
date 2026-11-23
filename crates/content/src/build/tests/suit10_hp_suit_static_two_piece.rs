use crate::build::studio_projection::{BaseOverrides, Request, project as studio_project};
use ntesim_core::formula::Ratio;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SET_ID: &str = "set:Suit10";
const TWO_PIECE_PACK: &str = "Mag_2";
const FOUR_PIECE_BUFF_CLASS: &str = "Buff_Equipment_HPsuit_4";

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
        set_id: Some(SET_ID.into()),
        set_count,
        base_overrides: BaseOverrides {
            atk: None,
            hp: Some(1000),
            def: None,
        },
        additive_stats: BTreeMap::new(),
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
        set_id: Some(SET_ID.into()),
        set_count,
        base_overrides: BTreeMap::new(),
        additive_attributes: BTreeMap::new(),
    }
}

fn equipped_hp_suit_request(
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
) -> ntesim_data::equipment::Request {
    super::legal_native_build(
        tables,
        coverage,
        "character:1003",
        "weapon:fork_mofeikesi",
        SET_ID,
    )
}

#[test]
fn frozen_suit10_rows_bind_static_two_piece_and_conditional_four_piece() {
    let row = &ntesim_data::set_definitions::transport()["rows"]["Suit10"];
    assert_eq!(row["SuitTitle"]["LocalizedString"], "Tiny Big Adventure");
    let effects = row["SuitStructList"].as_array().unwrap();
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[0]["SuitCondition"], 2);
    assert_eq!(effects[0]["SuitModifyPackID"], TWO_PIECE_PACK);
    assert!(effects[0]["SuitBuff"].is_null());
    assert_eq!(effects[1]["SuitCondition"], 4);
    assert_eq!(effects[1]["SuitModifyPackID"], "None");
    let buff = effects[1]["SuitBuff"]["ObjectPath"].as_str().unwrap();
    assert!(buff.contains(FOUR_PIECE_BUFF_CLASS));
    let description = effects[1]["SuitBuffDescription"]["LocalizedString"]
        .as_str()
        .unwrap();
    assert!(description.contains(
        "Max HP by 4% when taking damage, up to 10 stacks. Each stack lasts 10s. \
         Casting Ultimate instantly grants 10 stacks."
    ));
    let pack = &ntesim_data::set_modifiers::transport()["rows"][TWO_PIECE_PACK];
    assert!(pack["ConditionArray"].as_array().unwrap().is_empty());
    let modifiers = pack["ModifyData"].as_array().unwrap();
    assert_eq!(modifiers.len(), 1);
    assert_eq!(
        modifiers[0]["ModifierOp"],
        "EModifyModOp::MODIFY_MODOP_ADDITIVE"
    );
    assert_eq!(modifiers[0]["PropName"], "HPMaxUp");
    assert_eq!(modifiers[0]["PropValue"], json!(0.1));
}

#[test]
fn two_piece_static_pack_projects_exact_hp_attribute() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for set_count in [1u32, 2] {
        let report =
            ntesim_data::stat_projection::compile(tables, &native_request(set_count)).unwrap();
        let attribute = &report["attributes"]["HPMaxUp"];
        if set_count >= 2 {
            assert_eq!(attribute, &receipt("1", "10"));
        } else {
            assert_eq!(
                attribute["numerator"].as_str().unwrap(),
                "0",
                "no HPMaxUp below the two-piece threshold"
            );
        }
    }
}

#[test]
fn studio_two_piece_scales_derived_hp_by_eleven_tenths() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for set_count in 0..=4u32 {
        let report = studio_project(tables, &studio_request(set_count)).unwrap();
        // Independent oracle: base HP override 1000, no other HPMaxUp source.
        // Below threshold hp = 1000; at two pieces Mag_2 adds HPMaxUp 1/10,
        // so hp = 1000 * 11/10 = 1100, exactly 11/10 of the below-threshold value.
        let expected = if set_count >= 2 { 1100 } else { 1000 };
        assert_eq!(report.static_stats.hp, expected);
        assert!(!report.unmodeled_attributes.contains_key("HPMaxUp"));
        let evidence_hit = report.evidence.as_array().unwrap().iter().any(|entry| {
            entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID
                && entry["row"] == TWO_PIECE_PACK
        });
        assert_eq!(evidence_hit, set_count >= 2);
    }
}

#[test]
fn hp_up_sums_within_single_zone_not_multiplicatively() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let mut request = studio_request(2);
    request.additive_stats.insert(
        "hp_percent".into(),
        Ratio {
            numerator: 1,
            denominator: 4,
        },
    );
    // Independent oracle: HPMaxUp totals 1/4 + 1/10 = 7/20 in one zone.
    // hp = 1000 * 27/20 = 1350, not 1000 * 1.25 * 1.1 = 1375.
    let with_pack = studio_project(tables, &request).unwrap();
    assert_eq!(with_pack.static_stats.hp, 1350);
    request.set_count = 1;
    let baseline = studio_project(tables, &request).unwrap();
    assert_eq!(baseline.static_stats.hp, 1250);
    let native = ntesim_data::stat_projection::compile(
        tables,
        &ntesim_data::stat_projection::Request {
            additive_attributes: BTreeMap::from([("HPMaxUp".into(), receipt("1", "5"))]),
            ..native_request(2)
        },
    )
    .unwrap();
    assert_eq!(native["attributes"]["HPMaxUp"], receipt("3", "10"));
}

#[test]
fn native_build_applies_static_hp_two_piece() {
    let (tables, coverage, _) = super::skill_fixture();
    let report =
        ntesim_data::equipment::compile(&tables, &equipped_hp_suit_request(&tables, &coverage))
            .unwrap();
    assert_eq!(report["set_id"], SET_ID);
    assert!(report["known_attributes"].get("HPMaxUp").is_some());
    let two_piece = report["contributions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|contribution| {
            contribution["category"] == "set_unconditional_pack" && contribution["threshold"] == 2
        })
        .expect("two-piece set pack contribution missing");
    assert_eq!(two_piece["attributes"]["HPMaxUp"], receipt("1", "10"));
    assert!(
        two_piece["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["row"] == TWO_PIECE_PACK)
    );
    // Four-piece native binding behind the equipped HpSuitMaxHp proc:
    // frozen single-coordinate curves give 1/25 per stack and 10 seconds,
    // which sets/hp_suit/projection.rs turns into the 10_000_000us gear proc.
    // The baseline parameters come from consumed typed constants.
    let native = ntesim_data::set_passive_values::hp_suit(&tables).unwrap();
    assert_eq!(
        native["parameters"]["Hpsuit_4_HpMaxUp"]["selected"],
        receipt("1", "25")
    );
    assert_eq!(
        native["parameters"]["Hpsuit_4_CD"]["selected"],
        receipt("10", "1")
    );
}

#[test]
fn invalid_studio_configurations_reject() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let over_count = studio_project(tables, &studio_request(5));
    assert!(
        over_count
            .unwrap_err()
            .contains("set count must be in 0..=4")
    );
    let mut missing_set = studio_request(2);
    missing_set.set_id = None;
    assert!(studio_project(tables, &missing_set).is_err());
    let mut unknown_set = studio_request(2);
    unknown_set.set_id = Some("set:Suit999".into());
    assert!(studio_project(tables, &unknown_set).is_err());
}
