use super::*;
use std::collections::BTreeMap;

#[test]
fn exact_integer_json_survives_typed_value_and_envelope_boundaries() {
    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
    struct Fraction {
        numerator: u128,
        denominator: u128,
    }
    for numerator in [u64::MAX as u128 + 1, (1u128 << 100) + 37, u128::MAX] {
        let original = Fraction {
            numerator,
            denominator: u128::MAX - 1,
        };
        let raw = serde_json::to_string(&original).unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["numerator"].to_string(), numerator.to_string());
        let envelope = serde_json::json!({ "payload": value });
        let reparsed: serde_json::Value =
            serde_json::from_slice(&serde_json::to_vec(&envelope).unwrap()).unwrap();
        assert_eq!(
            serde_json::from_value::<Fraction>(reparsed["payload"].clone()).unwrap(),
            original
        );
        assert_eq!(
            serde_json::from_value::<Fraction>(serde_json::to_value(&original).unwrap()).unwrap(),
            original
        );
    }
}

fn scenario() -> Scenario {
    Scenario {
        schema_version: 1,
        content_id: "fixture".into(),
        duration_us: 10,
        seed: 0,
        dependencies: vec!["timing".into()],
        assumptions: BTreeMap::new(),
    }
}

fn deps(confirmed: bool, implemented: bool) -> BTreeMap<String, Dependency> {
    BTreeMap::from([(
        "timing".into(),
        Dependency {
            confirmed,
            implemented,
            gap: Some("clock unknown".into()),
        },
    )])
}

#[test]
fn unconfirmed_dependency_without_assumption_fails() {
    let v = scenario().validate("fixture", &deps(false, true));
    assert!(!v.valid);
    assert_eq!(v.diagnostics[0].code, "MISSING_EVIDENCE");
}

#[test]
fn only_essential_dependencies_block() {
    let mut d = deps(true, true);
    d.insert(
        "unused".into(),
        Dependency {
            confirmed: false,
            implemented: false,
            gap: None,
        },
    );
    assert!(scenario().validate("fixture", &d).valid);
}

#[test]
fn explicit_unbounded_hypothesis() {
    let mut s = scenario();
    s.assumptions.insert(
        "timing".into(),
        Assumption {
            value: 1.0,
            unit: "s".into(),
            justification: "explicit synthetic test".into(),
            interval: None,
        },
    );
    assert_eq!(
        s.validate("fixture", &deps(false, true)).quality,
        Quality::UnboundedHypothesis
    );
    assert!(!s.validate("fixture", &deps(false, false)).valid);
}

#[test]
fn invalid_intervals_and_confirmed_overrides_fail() {
    let mut s = scenario();
    s.assumptions.insert(
        "timing".into(),
        Assumption {
            value: 2.0,
            unit: "s".into(),
            justification: "test".into(),
            interval: Some([0.0, 1.0]),
        },
    );
    assert!(!s.validate("fixture", &deps(false, true)).valid);
    assert!(!s.validate("fixture", &deps(true, true)).valid);
}

#[test]
fn schema_typo_is_not_ignored() {
    assert!(serde_json::from_str::<Scenario>(
        r#"{"schema_version":1,"content_id":"fixture","duration_us":10,"seed":0,"dependencies":[],"assumptions":{},"duraton_us":20}"#
    )
    .is_err());
}

#[test]
fn hash_matches_independent_known_vector() {
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn strongly_typed_ids_interop_with_strings_and_serde() {
    let char_id = CharacterId::new("character:1046");
    assert_eq!(char_id.as_str(), "character:1046");
    assert_eq!(&*char_id, "character:1046");
    assert_eq!(char_id, "character:1046");
    assert_eq!(char_id.to_string(), "character:1046");

    let serialized = serde_json::to_string(&char_id).unwrap();
    assert_eq!(serialized, "\"character:1046\"");
    let deserialized: CharacterId = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, char_id);

    let ability_id = AbilityId::new("GA_Female046_Melee");
    assert_eq!(ability_id.as_str(), "GA_Female046_Melee");

    let effect_id = DamageEffectId::new("GE_Player_Female046_Melee1_Damage");
    assert_eq!(effect_id.as_str(), "GE_Player_Female046_Melee1_Damage");

    let pkg_id = PackageId::new("ntesim-current");
    assert_eq!(pkg_id.as_str(), "ntesim-current");
}

#[test]
fn contract_error_display_and_conversion() {
    let err = ContractError::ZeroDuration;
    assert_eq!(err.to_string(), "duration must be positive");

    let err2: ContractError = "custom error".into();
    assert_eq!(err2.to_string(), "custom error");

    let err3 = ContractError::PackageMismatch {
        expected: "pkg-a".into(),
        actual: "pkg-b".into(),
    };
    assert!(err3.to_string().contains("expected pkg-a, got pkg-b"));
}
