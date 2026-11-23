use crate::build::studio_projection::project as studio_project;
use ntesim_core::formula::Ratio;
use serde_json::json;
use std::collections::BTreeMap;

const SET_ID: &str = "set:Suit8";
const TWO_PIECE_PACK: &str = "Attack_2";
const FOUR_PIECE_BUFF_CLASS: &str = "Buff_Equipment_Atk_4";

#[test]
fn two_piece_static_pack_projects_exact_atk_attribute() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for set_count in [1u32, 2] {
        let report = ntesim_data::stat_projection::compile(
            tables,
            &super::native_request(SET_ID, set_count),
        )
        .unwrap();
        let attribute = &report["attributes"]["AtkUp"];
        if set_count >= 2 {
            assert_eq!(attribute, &super::make_receipt("1", "10"));
        } else {
            assert_eq!(
                attribute["numerator"].as_str().unwrap(),
                "0",
                "no AtkUp below the two-piece threshold"
            );
        }
    }
}

#[test]
fn studio_two_piece_scales_derived_atk_by_eleven_tenths() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for set_count in 0..=4u32 {
        let report = studio_project(tables, &super::studio_request(SET_ID, set_count)).unwrap();
        // Independent oracle: base ATK override 100, no other AtkUp source.
        // Below threshold atk = 100; at two pieces Attack_2 adds AtkUp 1/10,
        // so atk = 100 * 11/10 = 110, exactly 11/10 of the below-threshold value.
        let expected = if set_count >= 2 {
            Ratio {
                numerator: 110,
                denominator: 1,
            }
        } else {
            Ratio {
                numerator: 100,
                denominator: 1,
            }
        };
        assert_eq!(report.static_stats.atk, expected);
        assert!(!report.unmodeled_attributes.contains_key("AtkUp"));
        let evidence_hit = report.evidence.as_array().unwrap().iter().any(|entry| {
            entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID
                && entry["row"] == TWO_PIECE_PACK
        });
        assert_eq!(evidence_hit, set_count >= 2);
    }
}

#[test]
fn frozen_suit8_rows_bind_static_two_piece_and_unresolved_four_piece() {
    let row = &ntesim_data::set_definitions::transport()["rows"]["Suit8"];
    assert_eq!(row["SuitTitle"]["LocalizedString"], "Shadow Creed");
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
    assert!(description.contains("Increases wearer's ATK by 25% for 20s after casting a Skill"));
    let pack = &ntesim_data::set_modifiers::transport()["rows"][TWO_PIECE_PACK];
    assert!(pack["ConditionArray"].as_array().unwrap().is_empty());
    let modifiers = pack["ModifyData"].as_array().unwrap();
    assert_eq!(modifiers.len(), 1);
    assert_eq!(
        modifiers[0]["ModifierOp"],
        "EModifyModOp::MODIFY_MODOP_ADDITIVE"
    );
    assert_eq!(modifiers[0]["PropName"], "AtkUp");
    assert_eq!(modifiers[0]["PropValue"], json!(0.1));
}

#[test]
fn native_build_applies_static_atk_two_piece() {
    let (tables, coverage, _) = super::skill_fixture();
    let report = ntesim_data::equipment::compile(
        &tables,
        &super::equipped_set_request(SET_ID, &tables, &coverage),
    )
    .unwrap();
    assert_eq!(report["set_id"], SET_ID);
    assert!(report["known_attributes"].get("AtkUp").is_some());
    let two_piece = report["contributions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|contribution| {
            contribution["category"] == "set_unconditional_pack" && contribution["threshold"] == 2
        })
        .expect("two-piece set pack contribution missing");
    assert_eq!(
        two_piece["attributes"]["AtkUp"],
        super::make_receipt("1", "10")
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
    let (tables, coverage, _) = super::skill_fixture();
    let report = ntesim_data::equipment::compile(
        &tables,
        &super::equipped_set_request(SET_ID, &tables, &coverage),
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
    assert!(description.contains("Increases wearer's ATK by 25% for 20s after casting a Skill"));
}

#[test]
fn atk_up_sums_within_single_zone_not_multiplicatively() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let mut request = super::studio_request(SET_ID, 2);
    request.additive_stats.insert(
        "atk_percent".into(),
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    );
    // Independent oracle: AtkUp totals 1/5 + 1/10 = 3/10 in one zone.
    // atk = 100 * 13/10 = 130, not 100 * 1.2 * 1.1 = 132.
    let with_pack = studio_project(tables, &request).unwrap();
    assert_eq!(
        with_pack.static_stats.atk,
        Ratio {
            numerator: 130,
            denominator: 1
        }
    );
    request.set_count = 1;
    let baseline = studio_project(tables, &request).unwrap();
    assert_eq!(
        baseline.static_stats.atk,
        Ratio {
            numerator: 120,
            denominator: 1
        }
    );
    let native = ntesim_data::stat_projection::compile(
        tables,
        &ntesim_data::stat_projection::Request {
            additive_attributes: BTreeMap::from([("AtkUp".into(), super::make_receipt("1", "5"))]),
            ..super::native_request(SET_ID, 2)
        },
    )
    .unwrap();
    assert_eq!(
        native["attributes"]["AtkUp"],
        super::make_receipt("3", "10")
    );
}

#[test]
fn invalid_studio_configurations_reject() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let over_count = studio_project(tables, &super::studio_request(SET_ID, 5));
    assert!(
        over_count
            .unwrap_err()
            .contains("set count must be in 0..=4")
    );
    let mut missing_set = super::studio_request(SET_ID, 2);
    missing_set.set_id = None;
    assert!(studio_project(tables, &missing_set).is_err());
    let mut unknown_set = super::studio_request(SET_ID, 2);
    unknown_set.set_id = Some("set:Suit999".into());
    assert!(studio_project(tables, &unknown_set).is_err());
}
