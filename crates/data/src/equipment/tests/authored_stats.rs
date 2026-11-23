use super::*;

fn request() -> Request {
    serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/build-stats.json"
    )))
    .unwrap()
}
fn tables() -> BTreeMap<String, Value> {
    BTreeMap::new()
}

#[test]
fn console_stats_are_explicit_and_never_inferred_from_character_or_set() {
    let tables = tables();
    let mut req = request();
    req.set_count = 0;
    req.additive_attributes.clear();
    let bare = compile(&tables, &req).unwrap();
    assert_eq!(
        bare["base_atk"],
        json!({"numerator":"1094","denominator":"1"})
    );
    assert_eq!(
        bare["known_derived_stats"]["hp_receipt"]["numerator"],
        "16483"
    );
    for (name, n, d) in [
        ("AtkUp", "1", "2"),
        ("AtkAdd", "100", "1"),
        ("HPMaxAdd", "100", "1"),
    ] {
        req.additive_attributes
            .insert(name.into(), json!({"numerator":n,"denominator":d}));
    }
    let report = compile(&tables, &req).unwrap();
    // The equipped weapon already supplies ATK +45%: 1094*(1+.45+.5)+100.
    assert_eq!(
        report["known_derived_stats"]["atk_receipt"],
        json!({"numerator":"22333","denominator":"10"})
    );
    assert_eq!(
        report["known_derived_stats"]["hp_receipt"]["numerator"],
        "16583"
    );
    assert!(report.get("occupied_cells").is_none());
    assert!(report.get("specialization_modules").is_none());
    let mut old = serde_json::to_value(&req).unwrap();
    old["packing"] = json!({"modules":[]});
    assert!(serde_json::from_value::<Request>(old).is_err());
}

#[test]
fn set_static_bonuses_activate_once_at_declared_two_and_four_piece_thresholds() {
    let tables = tables();
    let mut req = request();
    req.additive_attributes.clear();
    req.set_id = "set:Suit9".into();
    for (count, hp_up, heal) in [
        (0, "0", "0"),
        (1, "0", "0"),
        (2, "1", "0"),
        (3, "1", "0"),
        (4, "1", "1"),
    ] {
        req.set_count = count;
        let report = compile(&tables, &req).unwrap();
        assert_eq!(
            report["known_attributes"]["HPMaxUp"]["receipt"]["numerator"],
            hp_up
        );
        if count >= 2 {
            assert_eq!(
                report["known_attributes"]["HPMaxUp"]["receipt"]["denominator"],
                "10"
            );
        }
        if count == 4 {
            assert_eq!(
                report["known_attributes"]["HealUp"]["receipt"],
                json!({"numerator":heal,"denominator":"5"})
            );
        } else {
            assert!(report["known_attributes"].get("HealUp").is_none());
        }
    }
    req.set_count = 5;
    assert!(compile(&tables, &req).is_err());
}
