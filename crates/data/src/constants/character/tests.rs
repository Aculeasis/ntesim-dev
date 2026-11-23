use super::*;
fn fixture() -> Request {
    Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        character_id: "character:1003".into(),
        level: 80,
        breakthrough: 6,
    }
}
#[test]
fn typed_stats_preserve_independent_endpoints_and_only_consumed_fields() {
    let request = fixture();
    let report = compile(&request).unwrap();
    for (stat, n, d) in [
        ("AtkBase", "620", "1"),
        ("HPMaxBase", "16483", "1"),
        ("DefBase", "909", "1"),
        ("CritBase", "1", "20"),
        ("CritDamageBase", "1", "2"),
        ("ChargeMax", "120", "1"),
    ] {
        assert_eq!(
            report["attributes"][stat],
            json!({"numerator":n,"denominator":d})
        );
    }
    for unused in [
        "MagBase",
        "SatietyMax",
        "TractionMass",
        "DamageImmuNormal",
        "DamageResistNatureBase",
    ] {
        assert!(report["attributes"].get(unused).is_none());
    }
    assert_eq!(report["steps"].as_array().unwrap().len(), 3);
    assert_eq!(report["evidence"].as_array().unwrap().len(), 2);
    assert_eq!(report["evidence"][1]["kind"], "typed_character_stats");
}
#[test]
fn caps_gate_growth_and_only_the_selected_breakthrough_applies() {
    let mut request = fixture();
    request.level = 20;
    request.breakthrough = 0;
    let report = compile(&request).unwrap();
    assert_eq!(report["attributes"]["AtkBase"]["numerator"], "152");
    assert!(report["steps"][2]["model_assumption"].is_string());
    request.level = 21;
    assert!(compile(&request).is_err());
    request.breakthrough = 1;
    // Base78 + growth78 + terminal39 =195; no previous-stage accumulation.
    assert_eq!(
        compile(&request).unwrap()["attributes"]["AtkBase"]["numerator"],
        "195"
    );
    for (level, cap, stage) in [(20, None, 0), (21, None, 1), (20, Some(30), 1)] {
        assert_eq!(
            crate::skill_levels::progression("character:1003", level, cap).unwrap()["breakthrough"],
            stage
        );
    }
    assert!(crate::skill_levels::progression("character:1003", 20, Some(25)).is_err());
}
#[test]
fn invalid_progression_identity_and_request_schema_diagnose() {
    let request = fixture();
    for (level, breakthrough) in [(0, 6), (81, 6), (80, 5), (80, 7)] {
        assert!(
            compile(&Request {
                level,
                breakthrough,
                ..request.clone()
            })
            .is_err()
        );
    }
    for (field, value) in [
        ("schema_version", json!(2)),
        ("content_id", json!("synthetic-v1")),
        ("character_id", json!("1003")),
        ("character_id", json!("character:1051")),
    ] {
        let mut bad = serde_json::to_value(&request).unwrap();
        bad[field] = value;
        assert!(compile(&serde_json::from_value(bad).unwrap()).is_err());
    }
    for (field, value) in [
        ("level", json!(true)),
        ("level", json!(-1)),
        ("extra", json!(1)),
    ] {
        let mut bad = serde_json::to_value(&request).unwrap();
        bad[field] = value;
        assert!(serde_json::from_value::<Request>(bad).is_err());
    }
}
