use crate::build::studio_projection::{
    BaseOverrides, Request as StudioRequest, project as project_studio,
};
use crate::build::{element_attribute, project as project_build_stats};
use crate::formula::Ratio;
use crate::simulation::ExactFraction;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn equipment_tables() -> BTreeMap<String, Value> {
    BTreeMap::new()
}

fn suit9_native_request() -> ntesim_data::equipment::Request {
    serde_json::from_value(json!({"schema_version":1,"content_id":"ntesim-current","character_id":"character:1021","weapon_id":"weapon:fork_PaperPlane","weapon_tier":1,"set_id":"set:Suit9","set_count":4,
 "additive_attributes":{"HPMaxUp":{"numerator":"2","denominator":"5"}}})).unwrap()
}

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
fn theas_tavern_two_piece_native_build_projects_hp_max_up() {
    let tables = equipment_tables();
    let report = ntesim_data::equipment::compile(&tables, &suit9_native_request())
        .expect("Suit9 native build compiles");
    let contributions = report["contributions"]
        .as_array()
        .expect("contributions present");
    let set_pack = contributions
        .iter()
        .find(|entry| {
            entry["category"] == "set_unconditional_pack" && entry["threshold"] == json!(2)
        })
        .expect("2pc set contribution present");
    assert_eq!(
        set_pack["attributes"]["HPMaxUp"],
        json!({"numerator": "1", "denominator": "10"}),
        "Heal_2 grants HPMaxUp +0.1"
    );
    let authored = contributions
        .iter()
        .find(|entry| entry["category"] == "authored_stats")
        .expect("authored stats contribution present");
    assert_eq!(
        authored["attributes"]["HPMaxUp"],
        json!({"numerator":"2","denominator":"5"})
    );
    assert_eq!(
        report["known_attributes"]["HPMaxUp"]["receipt"],
        json!({"numerator": "1", "denominator": "2"}),
        "additive HPMaxUp zone: 1/10 set pack + 4/10 authored addition"
    );
    let element = ntesim_data::character::definition("character:1021")
        .unwrap()
        .element
        .native_key();
    let stats = project_build_stats(&report, element_attribute(element).expect("element maps"))
        .expect("build stats project");
    assert_eq!(
        stats.hp_percent,
        Ratio {
            numerator: 1,
            denominator: 2
        }
    );
    let base = scalar(&report["known_attributes"]["HPMaxBase"]["receipt"]);
    let flat = scalar(&report["known_attributes"]["HPMaxAdd"]["receipt"]);
    let up = scalar(&report["known_attributes"]["HPMaxUp"]["receipt"]);
    let expected = base
        .mul(ExactFraction::new(1, 1).unwrap().add(up).unwrap())
        .unwrap()
        .add(flat)
        .unwrap();
    assert_eq!(
        scalar(&report["known_derived_stats"]["hp_receipt"]),
        expected,
        "hp = character_hp * (1 + HPMaxUp) + HPMaxAdd"
    );
    assert_eq!(stats.hp, expected.round_u64().expect("hp rounds"));
    assert_eq!(
        stats.healing_bonus,
        Ratio {
            numerator: 1,
            denominator: 5
        },
        "the equipped four-shape build also carries the static Heal_4 pack"
    );
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
        set_id: Some("set:Suit9".into()),
        set_count,
        base_overrides: BTreeMap::new(),
        additive_attributes: BTreeMap::new(),
    }
}

#[test]
fn theas_tavern_two_piece_static_projection_applies_at_threshold() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for (count, applies) in [(1u32, false), (2u32, true)] {
        let report = ntesim_data::stat_projection::compile(tables, &data_request(count))
            .expect("static projection compiles");
        let attributes = report["attributes"]
            .as_object()
            .expect("attributes present");
        let up = attributes
            .get("HPMaxUp")
            .map(scalar)
            .unwrap_or(ExactFraction::new(0, 1).unwrap());
        if applies {
            assert_eq!(
                up,
                ExactFraction::new(1, 10).unwrap(),
                "Heal_2 grants HPMaxUp +0.1"
            );
        } else {
            assert_eq!(
                up,
                ExactFraction::new(0, 1).unwrap(),
                "no HPMaxUp below the 2pc threshold; the native character base pack contributes 0"
            );
        }
        let base = scalar(&attributes["HPMaxBase"]);
        let flat = attributes
            .get("HPMaxAdd")
            .map(scalar)
            .unwrap_or(ExactFraction::new(0, 1).unwrap());
        let expected = base
            .mul(ExactFraction::new(1, 1).unwrap().add(up).unwrap())
            .unwrap()
            .add(flat)
            .unwrap();
        assert_eq!(
            scalar(&report["derived"]["hp_receipt"]),
            expected,
            "hp = character_hp * (1 + HPMaxUp) + HPMaxAdd"
        );
        assert_eq!(has_set_evidence(&report, "Heal_2"), applies);
        assert!(
            !has_set_evidence(&report, "Heal_4"),
            "the 4pc pack must not apply below 4 pieces"
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
fn theas_tavern_two_piece_studio_projection_applies_at_threshold() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for count in 0..=4 {
        let mut input = studio_request();
        input.set_id = Some("set:Suit9".into());
        input.set_count = count;
        let report = project_studio(tables, &input).expect("studio projection compiles");
        let expected = if count >= 2 { 1 } else { 0 };
        assert_eq!(report.static_stats.hp_percent.numerator, expected);
        assert_eq!(
            report.static_stats.hp_percent.denominator,
            if count >= 2 { 10 } else { 1 },
            "Heal_2 grants HPMaxUp +0.1 at 2 pieces"
        );
        assert!(!report.unmodeled_attributes.contains_key("HPMaxUp"));
        let evidence = report.evidence.as_array().expect("evidence is an array");
        let has_heal_2 = evidence.iter().any(|entry| {
            entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID && entry["row"] == "Heal_2"
        });
        assert_eq!(has_heal_2, count >= 2);
    }
}

#[test]
fn theas_tavern_four_piece_studio_healing_bonus_regression() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for count in 0..=4 {
        let mut input = studio_request();
        input.set_id = Some("set:Suit9".into());
        input.set_count = count;
        let report = project_studio(tables, &input).expect("studio projection compiles");
        let expected = if count == 4 { (1, 5) } else { (0, 1) };
        assert_eq!(
            report.static_stats.healing_bonus.numerator, expected.0,
            "set_count={count}"
        );
        assert_eq!(
            report.static_stats.healing_bonus.denominator, expected.1,
            "Heal_4 grants HealUp +0.2 only at 4 pieces; set_count={count}"
        );
        assert!(!report.unmodeled_attributes.contains_key("HealUp"));
        let evidence = report.evidence.as_array().expect("evidence is an array");
        let has_heal_4 = evidence.iter().any(|entry| {
            entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID && entry["row"] == "Heal_4"
        });
        assert_eq!(has_heal_4, count == 4, "set_count={count}");
    }
}
