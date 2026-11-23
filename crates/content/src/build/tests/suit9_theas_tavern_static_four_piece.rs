use crate::build::studio_projection::{
    BaseOverrides, Request as StudioRequest, project as project_studio,
};
use crate::formula::Ratio;
use crate::simulation::ExactFraction;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SET_ID: &str = "set:Suit9";
const FOUR_PIECE_PACK: &str = "Heal_4";
const TWO_PIECE_PACK: &str = "Heal_2";

fn scalar(receipt: &Value) -> ExactFraction {
    ExactFraction::new(
        receipt["numerator"]
            .as_str()
            .expect("receipt numerator")
            .parse()
            .expect("numerator parses"),
        receipt["denominator"]
            .as_str()
            .expect("receipt denominator")
            .parse()
            .expect("denominator parses"),
    )
    .expect("valid fraction")
}

fn has_set_evidence(report: &Value, row: &str) -> bool {
    report["evidence"]
        .as_array()
        .expect("evidence is an array")
        .iter()
        .any(|entry| {
            entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID && entry["row"] == row
        })
}

#[test]
fn frozen_suit9_rows_bind_static_four_piece() {
    let row = &ntesim_data::set_definitions::transport()["rows"]["Suit9"];
    assert_eq!(row["SuitTitle"]["LocalizedString"], "Thea's Night Tavern");
    let effects = row["SuitStructList"]
        .as_array()
        .expect("suit effects present");
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[1]["SuitCondition"], 4);
    assert_eq!(effects[1]["SuitModifyPackID"], FOUR_PIECE_PACK);
    assert!(
        effects[1]["SuitBuff"].is_null(),
        "the 4pc effect is a static pack, not a live buff"
    );
    let description = effects[1]["SuitBuffDescription"]["LocalizedString"]
        .as_str()
        .expect("4pc description present");
    assert!(description.contains("Increases wearer's Healing Bonus by 20%"));
    let pack = &ntesim_data::set_modifiers::transport()["rows"][FOUR_PIECE_PACK];
    assert!(
        pack["ConditionArray"]
            .as_array()
            .expect("conditions present")
            .is_empty(),
        "empty ConditionArray makes the pack unconditional"
    );
    let modifiers = pack["ModifyData"].as_array().expect("modifiers present");
    assert_eq!(modifiers.len(), 1);
    assert_eq!(
        modifiers[0]["ModifierOp"],
        "EModifyModOp::MODIFY_MODOP_ADDITIVE"
    );
    assert_eq!(modifiers[0]["PropName"], "HealUp");
    assert_eq!(modifiers[0]["PropValue"], json!(0.2));
}

#[test]
fn native_build_applies_static_heal_four_piece() {
    let (tables, coverage, _) = super::skill_fixture();
    let request = super::legal_native_build(
        &tables,
        &coverage,
        "character:1003",
        "weapon:fork_mofeikesi",
        SET_ID,
    );
    let report =
        ntesim_data::equipment::compile(&tables, &request).expect("Suit9 native build compiles");
    assert_eq!(report["set_id"], SET_ID);
    let four_piece = report["contributions"]
        .as_array()
        .expect("contributions present")
        .iter()
        .find(|entry| {
            entry["category"] == "set_unconditional_pack" && entry["threshold"] == json!(4)
        })
        .expect("Suit9 4pc set_unconditional_pack contribution missing: the declared set count must reach four");
    assert_eq!(
        four_piece["attributes"]["HealUp"],
        json!({"numerator": "1", "denominator": "5"}),
        "frozen PropValue 0.2 reads as the exact fraction 1/5"
    );
    assert!(
        four_piece["evidence"]
            .as_array()
            .expect("contribution evidence present")
            .iter()
            .any(|entry| entry["row"] == FOUR_PIECE_PACK),
        "contribution carries Heal_4 evidence"
    );
    let two_piece = report["contributions"]
        .as_array()
        .expect("contributions present")
        .iter()
        .find(|entry| {
            entry["category"] == "set_unconditional_pack" && entry["threshold"] == json!(2)
        })
        .expect("2pc set contribution present in the same build");
    assert_eq!(
        two_piece["attributes"]["HPMaxUp"],
        json!({"numerator": "1", "denominator": "10"}),
        "both thresholds coexist in one equipped build"
    );
    // Honest boundary: build-stats projection is not applied to this equipped
    // Sakiri build because her fractional HP (HPMaxUp 1/10 over a base HP not
    // divisible by 10) requires an evidenced quantization policy, which the
    // projector does not have. That limitation predates Heal_4 and is
    // unrelated to the static pack. The healing_bonus projection of the Heal_4
    // pack is pinned at the Studio layer (test 4) and for an equipped Edgar
    // build in the 2pc file's native test.
}

fn data_request(set_count: u32) -> ntesim_data::stat_projection::Request {
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

#[test]
fn four_piece_static_projection_applies_at_threshold() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let zero = ExactFraction::new(0, 1).unwrap();
    for count in [1u32, 2, 3, 4] {
        let report = ntesim_data::stat_projection::compile(tables, &data_request(count))
            .expect("static projection compiles");
        let attributes = report["attributes"]
            .as_object()
            .expect("attributes present");
        let heal_up = attributes.get("HealUp").map(scalar);
        if count == 4 {
            assert_eq!(
                heal_up,
                Some(ExactFraction::new(1, 5).unwrap()),
                "Heal_4 grants HealUp +0.2 at exactly 4 pieces; set_count={count}"
            );
        } else {
            assert!(
                heal_up.is_none_or(|value| value == zero),
                "no HealUp below the 4pc threshold; set_count={count}"
            );
        }
        assert_eq!(
            has_set_evidence(&report, FOUR_PIECE_PACK),
            count == 4,
            "Heal_4 evidence appears only at 4 pieces; set_count={count}"
        );
        let base = scalar(&attributes["HPMaxBase"]);
        let up = attributes.get("HPMaxUp").map(scalar).unwrap_or(zero);
        let flat = attributes.get("HPMaxAdd").map(scalar).unwrap_or(zero);
        let expected = base
            .mul(ExactFraction::new(1, 1).unwrap().add(up).unwrap())
            .unwrap()
            .add(flat)
            .unwrap();
        assert_eq!(
            scalar(&report["derived"]["hp_receipt"]),
            expected,
            "hp = character_hp * (1 + HPMaxUp) + HPMaxAdd pins HealUp out of the hp derivation; set_count={count}"
        );
    }
}

fn studio_request() -> StudioRequest {
    StudioRequest {
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
        set_id: None,
        set_count: 0,
        base_overrides: BaseOverrides::default(),
        additive_stats: BTreeMap::new(),
    }
}

#[test]
fn studio_four_piece_healing_bonus_applies_at_four_pieces() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    assert_eq!(
        ntesim_data::set_modifiers::transport()["rows"][FOUR_PIECE_PACK]["ModifyData"][0]["PropValue"],
        json!(0.2),
        "frozen anchor: Heal_4 grants HealUp +0.2, which the interpreter reads as 1/5"
    );
    for count in 0..=4u32 {
        let mut input = studio_request();
        input.set_id = Some(SET_ID.into());
        input.set_count = count;
        let report = project_studio(tables, &input).expect("studio projection compiles");
        let expected = if count == 4 {
            Ratio {
                numerator: 1,
                denominator: 5,
            }
        } else {
            Ratio {
                numerator: 0,
                denominator: 1,
            }
        };
        assert_eq!(
            report.static_stats.healing_bonus, expected,
            "set_count={count}"
        );
        assert!(
            !report.unmodeled_attributes.contains_key("HealUp"),
            "HealUp is modeled in the healing_bonus domain; set_count={count}"
        );
        let evidence = report.evidence.as_array().expect("evidence is an array");
        let has = |row: &str| {
            evidence.iter().any(|entry| {
                entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID && entry["row"] == row
            })
        };
        assert_eq!(has(FOUR_PIECE_PACK), count == 4, "set_count={count}");
        assert_eq!(has(TWO_PIECE_PACK), count >= 2, "set_count={count}");
    }
}

#[test]
fn heal_up_sums_within_single_zone_not_multiplicatively() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let mut input = studio_request();
    input.set_id = Some(SET_ID.into());
    input.set_count = 4;
    input.additive_stats.insert(
        "healing_bonus".into(),
        Ratio {
            numerator: 1,
            denominator: 4,
        },
    );
    let report = project_studio(tables, &input).expect("studio projection compiles");
    // Independent oracle: 1/4 manual + 1/5 pack = 9/20 in one additive zone.
    // A multiplicative model would give (5/4) * (6/5) = 30/20 = 3/2; the exact
    // equality below rejects it.
    assert_eq!(
        report.static_stats.healing_bonus,
        Ratio {
            numerator: 9,
            denominator: 20
        }
    );
    assert!(!report.unmodeled_attributes.contains_key("HealUp"));
    input.set_count = 3;
    let below = project_studio(tables, &input).expect("studio projection compiles");
    assert_eq!(
        below.static_stats.healing_bonus,
        Ratio {
            numerator: 1,
            denominator: 4
        },
        "the manual additive stands alone below the 4pc threshold"
    );
    let mut native = data_request(4);
    native.additive_attributes.insert(
        "HealUp".into(),
        json!({"numerator": "1", "denominator": "4"}),
    );
    let compiled =
        ntesim_data::stat_projection::compile(tables, &native).expect("static projection compiles");
    assert_eq!(
        compiled["attributes"]["HealUp"],
        json!({"numerator": "9", "denominator": "20"}),
        "the data layer composes HealUp additively too"
    );
    let mut native_below = data_request(3);
    native_below.additive_attributes.insert(
        "HealUp".into(),
        json!({"numerator": "1", "denominator": "4"}),
    );
    let compiled_below = ntesim_data::stat_projection::compile(tables, &native_below)
        .expect("static projection compiles");
    assert_eq!(
        compiled_below["attributes"]["HealUp"],
        json!({"numerator": "1", "denominator": "4"})
    );
}

#[test]
fn invalid_studio_configurations_reject() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let mut over_count = studio_request();
    over_count.set_id = Some(SET_ID.into());
    over_count.set_count = 5;
    assert!(
        project_studio(tables, &over_count)
            .unwrap_err()
            .contains("set count must be in 0..=4"),
        "five pieces cannot satisfy a four-shape native set"
    );
    let mut missing_set = studio_request();
    missing_set.set_count = 4;
    assert!(
        project_studio(tables, &missing_set).is_err(),
        "a set count without a set id must reject"
    );
    let mut unknown_set = studio_request();
    unknown_set.set_id = Some("set:Suit999".into());
    unknown_set.set_count = 4;
    assert!(
        project_studio(tables, &unknown_set).is_err(),
        "an unknown set id must reject"
    );
}
