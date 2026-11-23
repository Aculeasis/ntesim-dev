use super::*;

#[test]
fn invoke_commands_default_to_atk_scaling() {
    let action: GameActionCommand = serde_json::from_value(serde_json::json!({
        "kind": "invoke_ability",
        "timestamp_us": 0,
        "actor_index": 0,
        "ability_id": "GA_Adler_Melee",
        "damage_effect_id": "GE_Player_Adler_Melee1_Damage",
        "array_index": 0,
        "atk_rate": "1/1",
        "energy_cost": 0,
        "recovery_us": 0,
        "cooldown_us": 0,
        "hit_delay_us": 0
    }))
    .unwrap();
    let GameActionCommand::InvokeAbility { scaling_stat, .. } = action else {
        panic!("action did not deserialize as an ability");
    };
    assert_eq!(scaling_stat, NativeScalingStat::Atk);
}

#[test]
fn mixed_scaling_transport_requires_exact_named_terms_and_rejects_extra_fields() {
    let value = serde_json::json!({"mixed":{
        "atk":"1/3",
        "def":"2/7",
        "hp":"0/1"
    }});
    let stat: NativeScalingStat = serde_json::from_value(value.clone()).unwrap();
    stat.validate().unwrap();
    assert_eq!(serde_json::to_value(stat).unwrap(), value);
    let mut extra = value.clone();
    extra["mixed"]["flat"] = serde_json::json!("1/1");
    assert!(serde_json::from_value::<NativeScalingStat>(extra).is_err());
    let mut absent = value;
    absent["mixed"].as_object_mut().unwrap().remove("hp");
    assert!(serde_json::from_value::<NativeScalingStat>(absent).is_err());
}

#[test]
fn duplicate_target_resistance_keys_are_rejected() {
    let result = serde_json::from_str::<GameTargetConfig>(
        r#"{"target_index":0,"level":80,"hp_max":10,"hp_current":10,"defense":0,"resistance":"0/1","elemental_resistances":{"NATURE":"1/1","NATURE":"2/1"}}"#,
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("duplicate elemental resistance key")
    );
}

#[test]
fn target_break_profile_is_optional_and_round_trips_exact_ratios() {
    let target: GameTargetConfig = serde_json::from_value(serde_json::json!({
        "target_index": 0,
        "hp_max": 10,
        "hp_current": 10,
        "defense": 0,
        "resistance": "0/1"
    }))
    .unwrap();
    assert!(target.break_profile.is_none());
    assert!(
        serde_json::to_value(target)
            .unwrap()
            .get("break_profile")
            .is_none()
    );

    let target: GameTargetConfig = serde_json::from_value(serde_json::json!({
        "target_index": 0,
        "hp_max": 10,
        "hp_current": 10,
        "defense": 0,
        "resistance": "0/1",
        "break_profile": {
            "capacity": "7/3",
            "current_gauge": "5/6",
            "broken_duration_us": 2500000
        }
    }))
    .unwrap();
    assert_eq!(target.break_profile.unwrap().current_gauge.numerator, 5);
}

#[test]
fn target_boss_metadata_defaults_false_and_preserves_explicit_true() {
    let mut target: GameTargetConfig = serde_json::from_value(serde_json::json!({
        "target_index": 0,
        "hp_max": 10,
        "hp_current": 10,
        "defense": 0,
        "resistance": "0/1"
    }))
    .unwrap();
    assert!(!target.is_boss);
    assert_eq!(serde_json::to_value(&target).unwrap()["is_boss"], false);

    target.is_boss = true;
    let value = serde_json::to_value(&target).unwrap();
    assert_eq!(value["is_boss"], true);
    assert!(
        serde_json::from_value::<GameTargetConfig>(value)
            .unwrap()
            .is_boss
    );
}
