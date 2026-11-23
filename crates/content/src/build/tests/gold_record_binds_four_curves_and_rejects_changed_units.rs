use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn gold_record_binds_four_curves_and_rejects_changed_units() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_GoldRecord")
        .unwrap()
        .1;
    assert_eq!(item["ItemName"]["LocalizedString"], "Voice of the Voyager");

    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE"
    );
    assert_eq!(
        item["ElementData"]["ExclusiveCharacterIDArray"],
        json!(["1072"])
    );
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1072", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 0,
        "energy_capacity": 100, "weapon_id": "weapon:fork_GoldRecord", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1", "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap();

    let expected_atk = [(6, 25), (3, 10), (9, 25), (21, 50), (12, 25)];
    let expected_qte = [(33, 50), (33, 40), (99, 100), (231, 200), (33, 25)];
    let expected_ult = [(11, 50), (11, 40), (33, 100), (77, 200), (11, 25)];

    for character_id in [
        "character:1010",
        "character:1025",
        "character:1070",
        "character:1072",
    ] {
        actor.character_id = character_id.into();
        for tier in 1..=5 {
            let i = tier - 1;
            actor.weapon_tier = tier as u32;
            actor.gear_procs.clear();
            let report = crate::weapons::gold_record::projection::project(
                &tables,
                1,
                "ntesim-current",
                &mut actor,
                &[],
            )
            .unwrap();
            assert_eq!(
                report["native_description"]["localized"],
                "Increases ATK by <lv>{0}</>.\r\nIncreases the wearer's Support Skill Crit DMG by <lv>{1}</>. Dealing DMG with a triggered Support Skill increases their Ultimate Crit DMG by <lv>{2}</> for <lv>{3}</>s, up to <lv>3</> stacks. Resets the duration when triggered again."
            );
            assert!(report["native_description"].get("source").is_none());
            assert_eq!(
                report["compiled_proc"]["atk_bonus"],
                json!(format!("{}/{}", expected_atk[i].0, expected_atk[i].1))
            );
            assert_eq!(
                report["compiled_proc"]["qte_crit_damage_bonus"],
                json!(format!("{}/{}", expected_qte[i].0, expected_qte[i].1))
            );
            assert_eq!(
                report["compiled_proc"]["ult_crit_damage_per_stack"],
                json!(format!("{}/{}", expected_ult[i].0, expected_ult[i].1))
            );
            assert_eq!(report["compiled_proc"]["duration_us"], 3_000_000);
            assert_eq!(report["native_parameter_report"]["parameter_count"], 4);
        }
    }

    actor.character_id = "character:1019".into();
    assert!(
        crate::weapons::gold_record::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );

    actor.character_id = "character:1072".into();
    actor.weapon_tier = 1;
    let mut changed = tables.clone();
    let data_list = changed
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()["rows"]["upgradestar_pack_fork_GoldRecord_1"]["DataList"]
        .as_array_mut()
        .unwrap();
    data_list
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_GoldRecord_AtkUp")
        .unwrap()["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::gold_record::projection::project(
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
