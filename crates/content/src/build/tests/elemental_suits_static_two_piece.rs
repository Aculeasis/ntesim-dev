use crate::formula::Ratio;
use crate::simulation::GameScenarioRequest;
use serde_json::{Value, json};
use std::collections::BTreeMap;

struct ElementalSuitCase {
    suit_key: &'static str,
    set_id: &'static str,
    two_piece_pack: &'static str,
    suit_title: &'static str,
    element: &'static str,
    attribute_name: &'static str,
    damage_type_ex: &'static str,
    four_piece_desc: &'static str,
}

const ELEMENTAL_SUITS: &[ElementalSuitCase] = &[
    ElementalSuitCase {
        suit_key: "Suit1",
        set_id: "set:Suit1",
        two_piece_pack: "Chaos_2",
        suit_title: "Diabolos",
        element: "CHAOS",
        attribute_name: "DamageUpChaosBase",
        damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_CHAOS",
        four_piece_desc: "Chaos RES",
    },
    ElementalSuitCase {
        suit_key: "Suit2",
        set_id: "set:Suit2",
        two_piece_pack: "Nature_2",
        suit_title: "Fireflies and the Forest",
        element: "NATURE",
        attribute_name: "DamageUpNatureBase",
        damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE",
        four_piece_desc: "CRIT DMG by 8%",
    },
    ElementalSuitCase {
        suit_key: "Suit3",
        set_id: "set:Suit3",
        two_piece_pack: "Psyche_2",
        suit_title: "Devil's Blood: Curse",
        element: "PSYCHE",
        attribute_name: "DamageUpPsycheBase",
        damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE",
        four_piece_desc: "Increases wearer's DMG by 18%",
    },
    ElementalSuitCase {
        suit_key: "Suit4",
        set_id: "set:Suit4",
        two_piece_pack: "Incantation_2",
        suit_title: "Crimson: Twin Butterflies",
        element: "INCANTATION",
        attribute_name: "DamageUpIncantationBase",
        damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION",
        four_piece_desc: "Increases wearer's ATK by 6%",
    },
    ElementalSuitCase {
        suit_key: "Suit5",
        set_id: "set:Suit5",
        two_piece_pack: "Lakshana_2",
        suit_title: "Street Boxer",
        element: "LAKSHANA",
        attribute_name: "DamageUpLakshanaBase",
        damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_LAKSHANA",
        four_piece_desc: "Crit Chance",
    },
    ElementalSuitCase {
        suit_key: "Suit6",
        set_id: "set:Suit6",
        two_piece_pack: "Cosmos_2",
        suit_title: "Lost Radiance",
        element: "COSMOS",
        attribute_name: "DamageUpCosmosBase",
        damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_COSMOS",
        four_piece_desc: "Ignores 25% of enemies' DEF",
    },
];

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
        base_overrides: Default::default(),
        additive_stats: BTreeMap::new(),
    }
}

fn native_tables() -> (BTreeMap<String, Value>, Value) {
    let (tables, coverage, _) = super::skill_fixture();
    (tables, coverage)
}

fn elemental_native_request(
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
    set_id: &str,
) -> ntesim_data::equipment::Request {
    super::legal_native_build(
        tables,
        coverage,
        "character:1003",
        "weapon:fork_mofeikesi",
        set_id,
    )
}

fn damage_fixture_request(damage_type_ex: &str) -> GameScenarioRequest {
    let mut request = crate::simulation::tests::research_request();
    request.stochastic_crit = None;
    request.actions.truncate(1);
    request.action_metadata.retain(|index, _| *index == 0);
    request.action_metadata.get_mut(&0).unwrap().damage_type_ex = Some(damage_type_ex.into());
    request.actors[0].static_stats.crit_rate = Ratio::zero();
    request.targets[0].defense = 0;
    request.targets[0].resistance = Ratio::zero();
    request
}

#[test]
fn two_piece_static_pack_projects_exact_elemental_attributes() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    for suit in ELEMENTAL_SUITS {
        for set_count in [1u32, 2] {
            let report = ntesim_data::stat_projection::compile(
                tables,
                &ntesim_data::stat_projection::Request {
                    character_id: "character:1003".into(),
                    level: 80,
                    max_level: Some(80),
                    weapon_id: None,
                    weapon_level: None,
                    weapon_max_level: None,
                    refinement: 1,
                    set_id: Some(suit.set_id.into()),
                    set_count,
                    base_overrides: BTreeMap::new(),
                    additive_attributes: BTreeMap::new(),
                },
            )
            .unwrap();
            let attribute = report["attributes"].get(suit.attribute_name);
            if set_count >= 2 {
                let attribute = attribute.unwrap_or_else(|| {
                    panic!("{} must project at two pieces", suit.two_piece_pack)
                });
                assert_eq!(attribute["numerator"], "1", "{}: numerator", suit.set_id);
                assert_eq!(
                    attribute["denominator"], "10",
                    "{}: denominator",
                    suit.set_id
                );
            } else {
                let numerator = attribute
                    .map(|value| value["numerator"].as_str().unwrap())
                    .unwrap_or("0");
                assert_eq!(
                    numerator, "0",
                    "no {} bonus below two-piece threshold",
                    suit.element
                );
            }
        }
    }
}

#[test]
fn studio_two_piece_binds_elemental_bonuses_with_native_evidence() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let all_elements = [
        "CHAOS",
        "COSMOS",
        "INCANTATION",
        "LAKSHANA",
        "NATURE",
        "PSYCHE",
    ];
    for suit in ELEMENTAL_SUITS {
        for set_count in 0..=4u32 {
            let report = crate::build::studio_projection::project(
                tables,
                &studio_request(suit.set_id, set_count),
            )
            .unwrap();
            let expected = if set_count >= 2 {
                Ratio {
                    numerator: 1,
                    denominator: 10,
                }
            } else {
                Ratio::zero()
            };
            assert_eq!(
                report.static_stats.elemental_bonuses[suit.element], expected,
                "{}: {} bonus",
                suit.set_id, suit.element
            );
            for other_element in all_elements {
                if other_element != suit.element {
                    assert_eq!(
                        report.static_stats.elemental_bonuses[other_element],
                        Ratio::zero(),
                        "{}: other element {}",
                        suit.set_id,
                        other_element
                    );
                }
            }
            assert!(
                !report
                    .unmodeled_attributes
                    .contains_key(suit.attribute_name),
                "{}: attribute {} should be modeled",
                suit.set_id,
                suit.attribute_name
            );
            let evidence_hit = report.evidence.as_array().unwrap().iter().any(|entry| {
                entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID
                    && entry["row"] == suit.two_piece_pack
            });
            assert_eq!(
                evidence_hit,
                set_count >= 2,
                "{}: evidence for {}",
                suit.set_id,
                suit.two_piece_pack
            );
        }
    }
}

#[test]
fn frozen_elemental_suit_rows_bind_static_two_piece_and_four_piece() {
    for suit in ELEMENTAL_SUITS {
        let row = &ntesim_data::set_definitions::transport()["rows"][suit.suit_key];
        assert_eq!(
            row["SuitTitle"]["LocalizedString"], suit.suit_title,
            "{}",
            suit.set_id
        );
        let effects = row["SuitStructList"].as_array().unwrap();
        assert_eq!(effects.len(), 2, "{}", suit.set_id);
        assert_eq!(effects[0]["SuitCondition"], 2, "{}", suit.set_id);
        assert_eq!(
            effects[0]["SuitModifyPackID"], suit.two_piece_pack,
            "{}",
            suit.set_id
        );
        assert!(effects[0]["SuitBuff"].is_null(), "{}", suit.set_id);
        assert_eq!(effects[1]["SuitCondition"], 4, "{}", suit.set_id);
        assert_eq!(effects[1]["SuitModifyPackID"], "None", "{}", suit.set_id);
        let description = effects[1]["SuitBuffDescription"]["LocalizedString"]
            .as_str()
            .unwrap();
        assert!(
            description.contains(suit.four_piece_desc),
            "{}: description '{}' must contain '{}'",
            suit.set_id,
            description,
            suit.four_piece_desc
        );

        let pack = &ntesim_data::set_modifiers::transport()["rows"][suit.two_piece_pack];
        assert!(
            pack["ConditionArray"].as_array().unwrap().is_empty(),
            "{}",
            suit.two_piece_pack
        );
        let modifiers = pack["ModifyData"].as_array().unwrap();
        assert_eq!(modifiers.len(), 1, "{}", suit.two_piece_pack);
        assert_eq!(
            modifiers[0]["ModifierOp"], "EModifyModOp::MODIFY_MODOP_ADDITIVE",
            "{}",
            suit.two_piece_pack
        );
        assert_eq!(
            modifiers[0]["PropName"], suit.attribute_name,
            "{}",
            suit.two_piece_pack
        );
        assert_eq!(
            modifiers[0]["PropValue"],
            json!(0.1),
            "{}",
            suit.two_piece_pack
        );
    }
}

#[test]
fn native_build_applies_static_elemental_two_piece() {
    let (tables, coverage) = native_tables();
    for suit in ELEMENTAL_SUITS {
        let report = ntesim_data::equipment::compile(
            &tables,
            &elemental_native_request(&tables, &coverage, suit.set_id),
        )
        .unwrap();
        assert_eq!(report["set_id"], suit.set_id);
        assert_eq!(
            report["known_attributes"][suit.attribute_name]["receipt"],
            json!({"numerator": "1", "denominator": "10"}),
            "{}",
            suit.set_id
        );
        let two_piece = report["contributions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|contribution| {
                contribution["category"] == "set_unconditional_pack"
                    && contribution["threshold"] == 2
            })
            .unwrap_or_else(|| panic!("{}: two-piece set pack contribution missing", suit.set_id));
        assert_eq!(
            two_piece["attributes"][suit.attribute_name],
            json!({"numerator": "1", "denominator": "10"}),
            "{}",
            suit.set_id
        );
        assert!(
            two_piece["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["row"] == suit.two_piece_pack),
            "{}",
            suit.set_id
        );
    }
}

#[test]
fn elemental_packet_damage_applies_elemental_bonus_additively() {
    for suit in ELEMENTAL_SUITS {
        let request = damage_fixture_request(suit.damage_type_ex);
        if suit.element == "NATURE" {
            super::verify_additive_element_damage_off_element(
                request,
                suit.element,
                "EExecutionDamageTypeEX::DAMAGE_TYPE_CHAOS",
            );
        } else {
            super::verify_additive_element_damage(request, suit.element);
        }
    }
}

#[test]
fn invalid_studio_configurations_reject() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let over_count =
        crate::build::studio_projection::project(tables, &studio_request("set:Suit1", 5));
    assert!(
        over_count
            .unwrap_err()
            .contains("set count must be in 0..=4")
    );
    let mut missing_set = studio_request("set:Suit1", 2);
    missing_set.set_id = None;
    assert!(crate::build::studio_projection::project(tables, &missing_set).is_err());
    let mut unknown_set = studio_request("set:Suit1", 2);
    unknown_set.set_id = Some("set:Suit999".into());
    assert!(crate::build::studio_projection::project(tables, &unknown_set).is_err());
}

#[test]
fn street_boxer_static_baseline_rejects_invalid_native_units() {
    for value in [json!(0), json!(1.000000000000001)] {
        let mut tables = ntesim_data::stat_projection::frozen_tables().clone();
        tables.insert(
            "CT_Equipmentbuff".into(),
            ntesim_data::set_parameters::overlay(),
        );
        tables.get_mut("CT_Equipmentbuff").unwrap()["rows"]["Lakshana_4_1"]["Keys"][0]["Value"] =
            value;
        assert!(
            crate::build::studio_projection::project(&tables, &studio_request("set:Suit5", 4))
                .unwrap_err()
                .contains("positive fraction <=1")
        );
    }
}

#[test]
fn devils_blood_native_equipped_build_projects_four_piece_and_keeps_two_piece_static() {
    let (tables, coverage, mut request) = super::fireflies_fixture();
    request.scenario.actors.truncate(1);
    request.builds.truncate(1);
    request.scenario.actions.clear();
    request.scenario.action_metadata.clear();
    request.scenario.stochastic_crit = None;
    request.action_projection = None;
    request.gear_projection = None;
    request.scenario.actors[0].set_id = Some("set:Suit3".into());
    request.scenario.actors[0].weapon_tier = 1;
    request.scenario.actors[0].gear_procs.clear();
    let actor = &request.scenario.actors[0];
    request.builds[0].build = super::legal_native_build(
        &tables,
        &coverage,
        &actor.character_id,
        &actor.weapon_id,
        "set:Suit3",
    );
    request.builds[0].build.additive_attributes.insert(
        "HPMaxAdd".into(),
        serde_json::json!({"numerator":"5600","denominator":"1"}),
    );
    let compiled = crate::build::compile(
        &tables,
        &coverage,
        &request,
        &ntesim_core::execution::ExecutionLimits::default(),
    )
    .unwrap();
    assert!(
        compiled.receipt["actor_build_receipts"][0]["native_devils_blood_projection"].is_object()
    );
    assert_eq!(
        compiled.scenario.actors[0].static_stats.elemental_bonuses["PSYCHE"],
        Ratio {
            numerator: 1,
            denominator: 10
        }
    );
    assert!(
        compiled.scenario.actors[0]
            .gear_procs
            .iter()
            .any(|proc| matches!(
                proc,
                crate::simulation::GearProcConfig::DevilsBloodDamage { .. }
            ))
    );
}
