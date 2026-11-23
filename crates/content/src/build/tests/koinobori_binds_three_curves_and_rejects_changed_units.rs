use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn koinobori_binds_three_curves_and_rejects_changed_units() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_koinobori")
        .unwrap()
        .1;
    assert_eq!(item["ItemName"]["LocalizedString"], "A Time Will Come");

    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE"
    );
    assert_eq!(item["ElementData"]["ExclusiveCharacterIDArray"], json!([]));
    assert_eq!(
        item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"],
        Value::Null
    );
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1033", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 0,
        "energy_capacity": 100, "weapon_id": "weapon:fork_koinobori", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1", "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap();

    let expected_ratios = [(1, 10), (3, 25), (7, 50), (4, 25), (9, 50)];
    for character_id in [
        "character:1023",
        "character:1033",
        "character:1039",
        "character:1071",
        "character:1076",
    ] {
        actor.character_id = character_id.into();
        for (tier, expected) in (1..=5).zip(expected_ratios) {
            actor.weapon_tier = tier;
            actor.gear_procs.clear();
            let report = crate::weapons::koinobori::projection::project(
                &tables,
                1,
                "ntesim-current",
                &mut actor,
                &[],
            )
            .unwrap();
            assert_eq!(
                report["native_description"]["localized"],
                "Grants the wearer <lv>{0}</> ATK, <lv>{1}</> DEF, and <lv>{2}</> HP when the team has at least <lv>3</> different Esper Types."
            );
            assert!(report["native_description"].get("source").is_none());
            assert_eq!(
                report["compiled_proc"]["atk_bonus"],
                json!(format!("{}/{}", expected.0, expected.1))
            );
            assert_eq!(
                report["compiled_proc"]["def_bonus"],
                json!(format!("{}/{}", expected.0, expected.1))
            );
            assert_eq!(
                report["compiled_proc"]["hp_bonus"],
                json!(format!("{}/{}", expected.0, expected.1))
            );
            assert_eq!(report["native_parameter_report"]["parameter_count"], 3);
        }
    }

    actor.character_id = "character:1003".into();
    assert!(
        crate::weapons::koinobori::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );

    actor.character_id = "character:1033".into();
    actor.weapon_tier = 1;
    let mut changed = tables.clone();
    let data_list = changed
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()["rows"]["upgradestar_pack_fork_koinobori_1"]["DataList"]
        .as_array_mut()
        .unwrap();
    data_list
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_koinobori_AtkUp")
        .unwrap()["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::koinobori::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}
