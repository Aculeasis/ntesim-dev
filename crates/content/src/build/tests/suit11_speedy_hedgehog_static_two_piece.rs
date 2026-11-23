use crate::formula::Ratio;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SET_ID: &str = "set:Suit11";
const TWO_PIECE_PACK: &str = "GetEfficiency_2";
const FOUR_PIECE_BUFF_CLASS: &str = "Buff_Equipment_GetEfficiency2_4";

fn studio_request(set_count: u32) -> crate::build::studio_projection::Request {
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
        set_id: Some(SET_ID.into()),
        set_count,
        base_overrides: Default::default(),
        additive_stats: BTreeMap::new(),
    }
}

fn native_tables() -> (BTreeMap<String, Value>, Value) {
    let (tables, coverage, _) = super::skill_fixture();
    (tables, coverage)
}

fn speedy_hedgehog_native_request(
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
fn two_piece_static_pack_projects_exact_charge_efficiency() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let compile = |set_count: u32| {
        ntesim_data::stat_projection::compile(
            tables,
            &ntesim_data::stat_projection::Request {
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
            },
        )
        .unwrap()["attributes"]["ChargeGetEfficiencyBase"]
            .clone()
    };
    let below = compile(1);
    assert_eq!(below["numerator"], "1");
    assert_eq!(below["denominator"], "1");
    let at_threshold = compile(2);
    assert_eq!(at_threshold["numerator"], "28");
    assert_eq!(at_threshold["denominator"], "25");
}

#[test]
fn studio_two_piece_binds_charge_efficiency_with_native_evidence() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for set_count in 0..=4u32 {
        let report =
            crate::build::studio_projection::project(tables, &studio_request(set_count)).unwrap();
        let expected = if set_count >= 2 {
            Ratio {
                numerator: 28,
                denominator: 25,
            }
        } else {
            Ratio::one()
        };
        assert_eq!(
            report.charge_efficiency, expected,
            "charge efficiency at set_count {set_count}"
        );
        assert!(
            !report
                .unmodeled_attributes
                .contains_key("ChargeGetEfficiencyBase")
        );
        let evidence_hit = report.evidence.as_array().unwrap().iter().any(|entry| {
            entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID
                && entry["row"] == TWO_PIECE_PACK
        });
        assert_eq!(evidence_hit, set_count >= 2);
    }
}

#[test]
fn frozen_suit11_rows_bind_static_two_piece_and_unresolved_four_piece() {
    let row = &ntesim_data::set_definitions::transport()["rows"]["Suit11"];
    assert_eq!(row["SuitTitle"]["LocalizedString"], "Speedy Hedgehog");
    let effects = row["SuitStructList"].as_array().unwrap();
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[0]["SuitCondition"], 2);
    assert_eq!(effects[0]["SuitModifyPackID"], TWO_PIECE_PACK);
    assert!(effects[0]["SuitBuff"].is_null());
    let two_piece_description = effects[0]["SuitBuffDescription"]["LocalizedString"]
        .as_str()
        .unwrap();
    assert!(two_piece_description.contains("Charge Efficiency +12%"));
    assert_eq!(effects[1]["SuitCondition"], 4);
    assert_eq!(effects[1]["SuitModifyPackID"], "None");
    let buff = effects[1]["SuitBuff"]["ObjectPath"].as_str().unwrap();
    assert!(buff.contains(FOUR_PIECE_BUFF_CLASS));
    let description = effects[1]["SuitBuffDescription"]["LocalizedString"]
        .as_str()
        .unwrap();
    assert!(description.contains("Increases all allies' ATK by 15%"));
    let pack = &ntesim_data::set_modifiers::transport()["rows"][TWO_PIECE_PACK];
    assert!(pack["ConditionArray"].as_array().unwrap().is_empty());
    let modifiers = pack["ModifyData"].as_array().unwrap();
    assert_eq!(modifiers.len(), 1);
    assert_eq!(
        modifiers[0]["ModifierOp"],
        "EModifyModOp::MODIFY_MODOP_ADDITIVE"
    );
    assert_eq!(modifiers[0]["PropName"], "ChargeGetEfficiencyBase");
    assert_eq!(modifiers[0]["PropValue"], json!(0.12));
}

#[test]
fn native_build_applies_static_charge_efficiency_two_piece() {
    let (tables, coverage) = native_tables();
    let report = ntesim_data::equipment::compile(
        &tables,
        &speedy_hedgehog_native_request(&tables, &coverage),
    )
    .unwrap();
    assert_eq!(report["set_id"], SET_ID);
    assert_eq!(
        report["known_attributes"]["ChargeGetEfficiencyBase"]["receipt"],
        json!({"numerator": "28", "denominator": "25"})
    );
    let two_piece = report["contributions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|contribution| {
            contribution["category"] == "set_unconditional_pack" && contribution["threshold"] == 2
        })
        .expect("two-piece set pack contribution missing");
    assert_eq!(
        two_piece["attributes"]["ChargeGetEfficiencyBase"],
        json!({"numerator": "3", "denominator": "25"}),
        "the pack contributes exactly +0.12; the native 1/1 base stays outside the contribution"
    );
    assert!(
        two_piece["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["row"] == TWO_PIECE_PACK)
    );
}

#[test]
fn native_build_marks_four_piece_unresolved_with_threshold() {
    let (tables, coverage) = native_tables();
    let report = ntesim_data::equipment::compile(
        &tables,
        &speedy_hedgehog_native_request(&tables, &coverage),
    )
    .unwrap();
    let four_piece = report["unresolved_set_buffs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["threshold"] == 4)
        .expect("four-piece unresolved set buff missing");
    let buff = four_piece["buff"]["ObjectPath"].as_str().unwrap();
    assert!(buff.contains(FOUR_PIECE_BUFF_CLASS));
    let description = four_piece["description"].as_str().unwrap();
    assert!(description.contains("Increases all allies' ATK by 15%"));
}

#[test]
fn charge_efficiency_two_piece_stacks_additively_with_studio_addition() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let mut request = studio_request(2);
    request.additive_stats.insert(
        "charge_efficiency".into(),
        Ratio {
            numerator: 1,
            denominator: 10,
        },
    );
    let report = crate::build::studio_projection::project(tables, &request).unwrap();
    assert_eq!(
        report.charge_efficiency,
        Ratio {
            numerator: 61,
            denominator: 50,
        }
    );
    assert_ne!(
        report.charge_efficiency,
        Ratio {
            numerator: 154,
            denominator: 125,
        }
    );
}

#[test]
fn invalid_studio_configurations_reject() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let over_count = crate::build::studio_projection::project(tables, &studio_request(5));
    assert!(
        over_count
            .unwrap_err()
            .contains("set count must be in 0..=4")
    );
    let mut missing_set = studio_request(2);
    missing_set.set_id = None;
    assert!(crate::build::studio_projection::project(tables, &missing_set).is_err());
    let mut unknown_set = studio_request(2);
    unknown_set.set_id = Some("set:Suit999".into());
    assert!(crate::build::studio_projection::project(tables, &unknown_set).is_err());
}
