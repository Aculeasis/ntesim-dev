use crate::build::studio_projection::project as studio_project;
use ntesim_core::formula::Ratio;
use serde_json::json;
use std::collections::BTreeMap;

const SET_ID: &str = "set:Suit12";
const TWO_PIECE_PACK: &str = "Psychically_2";
const FOUR_PIECE_BUFF_CLASS: &str = "Buff_Equipment_Psychically_4";

#[test]
fn frozen_suit12_rows_bind_static_two_piece_and_buff_four_piece() {
    let row = &ntesim_data::set_definitions::transport()["rows"]["Suit12"];
    assert_eq!(row["SuitTitle"]["LocalizedString"], "Quiet Manor");
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
    assert!(
        description.contains(
            "12% Mental DMG Bonus per Basic Attack, up to 3 stacks. Each stack lasts 6s."
        )
    );
    let pack = &ntesim_data::set_modifiers::transport()["rows"][TWO_PIECE_PACK];
    assert!(pack["ConditionArray"].as_array().unwrap().is_empty());
    let modifiers = pack["ModifyData"].as_array().unwrap();
    assert_eq!(modifiers.len(), 1);
    assert_eq!(
        modifiers[0]["ModifierOp"],
        "EModifyModOp::MODIFY_MODOP_ADDITIVE"
    );
    assert_eq!(modifiers[0]["PropName"], "DamageUpPsychicallyBase");
    assert_eq!(modifiers[0]["PropValue"], json!(0.1));
}

#[test]
fn studio_two_piece_scales_mental_bonus_by_exact_tenth() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for set_count in 0..=4u32 {
        let report = studio_project(tables, &super::studio_request(SET_ID, set_count)).unwrap();
        // Independent oracle: Psychically_2 is an unconditional additive pack of
        // DamageUpPsychicallyBase 0.1, so the compiled Mental zone bonus is 1/10
        // exactly at and above the two-piece threshold, zero below it.
        let expected = if set_count >= 2 {
            Ratio {
                numerator: 1,
                denominator: 10,
            }
        } else {
            Ratio::zero()
        };
        assert_eq!(report.static_stats.mental_damage_bonus, expected);
        assert!(
            !report
                .unmodeled_attributes
                .contains_key("DamageUpPsychicallyBase")
        );
        let evidence_hit = report.evidence.as_array().unwrap().iter().any(|entry| {
            entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID
                && entry["row"] == TWO_PIECE_PACK
        });
        assert_eq!(evidence_hit, set_count >= 2);
    }
}

#[test]
fn native_stat_projection_applies_static_mental_two_piece() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for set_count in [1u32, 2] {
        let report = ntesim_data::stat_projection::compile(
            tables,
            &super::native_request(SET_ID, set_count),
        )
        .unwrap();
        let attribute = &report["attributes"]["DamageUpPsychicallyBase"];
        if set_count >= 2 {
            assert_eq!(attribute, &super::make_receipt("1", "10"));
        } else {
            assert_eq!(
                attribute["numerator"].as_str().unwrap(),
                "0",
                "no Mental attribute below the two-piece threshold"
            );
        }
    }
}

#[test]
fn native_build_applies_static_mental_two_piece() {
    let (tables, coverage, _) = super::skill_fixture();
    let report = ntesim_data::equipment::compile(
        &tables,
        &super::equipped_set_request(SET_ID, &tables, &coverage),
    )
    .unwrap();
    assert_eq!(report["set_id"], SET_ID);
    assert!(
        report["known_attributes"]
            .get("DamageUpPsychicallyBase")
            .is_some()
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
        two_piece["attributes"]["DamageUpPsychicallyBase"],
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
fn mental_bonus_sums_within_single_zone_not_multiplicatively() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let mut request = super::studio_request(SET_ID, 2);
    request.additive_stats.insert(
        "mental_damage_bonus".into(),
        Ratio {
            numerator: 1,
            denominator: 4,
        },
    );
    // Independent oracle: the Mental zone totals 1/10 + 1/4 = 7/20 by addition.
    // A multiplicative reading would give 1.1 * 1.25 = 1.375 instead of 1.35.
    let with_pack = studio_project(tables, &request).unwrap();
    assert_eq!(
        with_pack.static_stats.mental_damage_bonus,
        Ratio {
            numerator: 7,
            denominator: 20
        }
    );
    request.set_count = 1;
    let baseline = studio_project(tables, &request).unwrap();
    assert_eq!(
        baseline.static_stats.mental_damage_bonus,
        Ratio {
            numerator: 1,
            denominator: 4
        }
    );
    let native = ntesim_data::stat_projection::compile(
        tables,
        &ntesim_data::stat_projection::Request {
            additive_attributes: BTreeMap::from([(
                "DamageUpPsychicallyBase".into(),
                super::make_receipt("1", "4"),
            )]),
            ..super::native_request(SET_ID, 2)
        },
    )
    .unwrap();
    assert_eq!(
        native["attributes"]["DamageUpPsychicallyBase"],
        super::make_receipt("7", "20")
    );
}

#[test]
fn mental_and_psyche_zones_stay_isolated() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    // Direction one: the Quiet Manor two-piece feeds the Mental zone and never
    // the Psyche elemental zone.
    for set_count in [2u32, 4] {
        let report = studio_project(tables, &super::studio_request(SET_ID, set_count)).unwrap();
        assert_eq!(
            report.static_stats.mental_damage_bonus,
            Ratio {
                numerator: 1,
                denominator: 10
            }
        );
        assert_eq!(
            report.static_stats.elemental_bonuses["PSYCHE"],
            Ratio::zero(),
            "Quiet Manor must not write the Psyche elemental zone"
        );
    }
    // Direction two: a Psyche-zone bonus never leaks into the Mental zone.
    for set_count in [1u32, 2] {
        let mut request = super::studio_request(SET_ID, set_count);
        request.additive_stats.insert(
            "psyche_bonus".into(),
            Ratio {
                numerator: 1,
                denominator: 4,
            },
        );
        let report = studio_project(tables, &request).unwrap();
        assert_eq!(
            report.static_stats.elemental_bonuses["PSYCHE"],
            Ratio {
                numerator: 1,
                denominator: 4
            }
        );
        let expected_mental = if set_count >= 2 {
            Ratio {
                numerator: 1,
                denominator: 10,
            }
        } else {
            Ratio::zero()
        };
        assert_eq!(
            report.static_stats.mental_damage_bonus, expected_mental,
            "Psyche-zone bonus must not leak into the Mental zone"
        );
        assert!(
            !report
                .unmodeled_attributes
                .contains_key("DamageUpPsycheBase")
        );
    }
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

#[test]
fn native_build_marks_four_piece_dynamic() {
    let (tables, coverage, _) = super::skill_fixture();
    let report = ntesim_data::equipment::compile(
        &tables,
        &super::equipped_set_request(SET_ID, &tables, &coverage),
    )
    .unwrap();
    // The static compiler keeps the four-piece in unresolved_set_buffs: it is a
    // dynamic per-Normal-input proc, not a static pack. The live behavior binds
    // separately through GearProcConfig::QuietManorMental plus the native set
    // projection path (covered in quiet_manor_native_build.rs); this test pins
    // the split so the four-piece is never misread as "unmodeled".
    let four_piece = report["unresolved_set_buffs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["threshold"] == 4)
        .expect("four-piece unresolved set buff missing");
    let buff = four_piece["buff"]["ObjectPath"].as_str().unwrap();
    assert!(buff.contains(FOUR_PIECE_BUFF_CLASS));
    let description = four_piece["description"].as_str().unwrap();
    assert!(description.contains("12% Mental DMG Bonus per Basic Attack"));
}
