use super::*;

#[test]
fn missing_receipts_and_nonzero_unsupported_typed_bonus_fail() {
    let mut attributes = serde_json::Map::new();
    for name in [
        "DamageUpNormalBase",
        "HPMaxUp",
        "HPMaxAdd",
        "HPMaxBase",
        "DefBase",
        "DefUp",
        "DefAdd",
        "CritBase",
        "CritDamageBase",
        "DamageUpPsychicallyBase",
        "UnbalIntensityBase",
        "UnbalAccrueEfficiencyBase",
        "DamageUpGeneralBase",
        "DamageUpChaosBase",
        "DamageUpCosmosBase",
        "DamageUpIncantationBase",
        "DamageUpLakshanaBase",
        "DamageUpNatureBase",
        "DamageUpPsycheBase",
    ] {
        attributes.insert(name.into(), json!({"receipt": make_receipt("0", "1")}));
    }
    let mut report = json!({
        "base_atk": make_receipt("100", "1"),
        "known_attributes": attributes,
        "known_derived_stats": {
            "atk_receipt": make_receipt("100", "1"),
            "hp_receipt": make_receipt("1000", "1"),
            "def_receipt": make_receipt("50", "1"),
        },
    });
    report["known_attributes"]["DamageUpPsychicallyBase"]["receipt"] =
        json!({"numerator": "1", "denominator": "10"});
    assert_eq!(
        project(&report, "DamageUpNatureBase")
            .unwrap()
            .mental_damage_bonus,
        Ratio {
            numerator: 1,
            denominator: 10
        }
    );
    report["known_attributes"]["DamageUpNormalBase"]["receipt"]["numerator"] = json!("1");
    assert!(
        project(&report, "DamageUpNatureBase")
            .unwrap_err()
            .contains("unsupported typed")
    );
    report["known_attributes"]["DamageUpNormalBase"]["receipt"] = Value::Null;
    assert!(
        project(&report, "DamageUpNatureBase")
            .unwrap_err()
            .contains("missing exact")
    );
}
