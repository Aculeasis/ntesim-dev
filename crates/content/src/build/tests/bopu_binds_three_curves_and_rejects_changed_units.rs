use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn bopu_binds_three_curves_and_rejects_changed_units() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_bopu")
        .unwrap()
        .1;
    assert_eq!(
        item["ItemName"]["LocalizedString"],
        "Cosmos Daze, Wild Reverie"
    );

    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR"
    );
    assert_eq!(item["ElementData"]["ExclusiveCharacterIDArray"], json!([]));
    assert_eq!(
        item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"],
        Value::Null
    );
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1003", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 0,
        "energy_capacity": 100, "weapon_id": "weapon:fork_bopu", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1", "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap();

    let expected_ratios = [(9, 50), (21, 100), (6, 25), (27, 100), (3, 10)];
    for character_id in [
        "character:1003",
        "character:1008",
        "character:1036",
        "character:1042",
        "character:1073",
    ] {
        actor.character_id = character_id.into();
        for (tier, expected) in (1..=5).zip(expected_ratios) {
            actor.weapon_tier = tier;
            actor.gear_procs.clear();
            let report = crate::weapons::bopu::projection::project(
                &tables,
                1,
                "ntesim-current",
                &mut actor,
                &[],
            )
            .unwrap();
            assert_eq!(
                report["native_description"]["localized"],
                "Increases wearer's damage dealt by <lv>{1}</> for <lv>{0}s</> after casting Support Skill. Triggers at most once every <lv>{2}s</>."
            );
            assert!(report["native_description"].get("source").is_none());
            assert_eq!(
                report["compiled_proc"]["damage_bonus"],
                json!(format!("{}/{}", expected.0, expected.1))
            );
            assert_eq!(report["compiled_proc"]["duration_us"], 10_000_000);
            assert_eq!(report["compiled_proc"]["cooldown_us"], 20_000_000);
            assert_eq!(report["native_parameter_report"]["parameter_count"], 3);
        }
    }

    actor.character_id = "character:1010".into();
    assert!(
        crate::weapons::bopu::projection::project(&tables, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("group is incompatible")
    );

    actor.character_id = "character:1003".into();
    actor.weapon_tier = 1;
    let mut changed = tables.clone();
    let data_list = changed
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()["rows"]["upgradestar_pack_fork_bopu_1"]["DataList"]
        .as_array_mut()
        .unwrap();
    data_list
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_haokewu_Up")
        .unwrap()["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::bopu::projection::project(&changed, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("unit hint")
    );
}
