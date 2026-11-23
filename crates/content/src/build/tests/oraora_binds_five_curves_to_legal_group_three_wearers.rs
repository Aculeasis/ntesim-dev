use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn oraora_binds_five_curves_to_legal_group_three_wearers() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_oulaquantao")
        .unwrap()
        .1;
    assert_eq!(item["ItemName"]["LocalizedString"], "Oraora!");

    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE"
    );
    assert_eq!(item["ElementData"]["ExclusiveCharacterIDArray"], json!([]));
    assert_eq!(
        item["ElementData"]["ExclusiveDesc"]["CultureInvariantString"],
        Value::Null
    );
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot":0,"character_id":"character:1010","character_level":80,"awakening":[],
        "purchased_skill_level":8,"initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_oulaquantao","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
          "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap();
    for character_id in ["character:1010", "character:1025"] {
        actor.character_id = character_id.into();
        for (tier, expected) in (1..=5).zip([(1, 50), (3, 125), (7, 250), (4, 125), (9, 250)]) {
            actor.weapon_tier = tier;
            actor.gear_procs.clear();
            let report = crate::weapons::oraora::projection::project(
                &tables,
                1,
                "ntesim-current",
                &mut actor,
                &[],
            )
            .unwrap();
            assert_eq!(
                report["native_description"]["localized"],
                "Increases the wearer's Basic Attack DMG by <lv>{1}</> for <lv>{0}</>s after each Basic Attack, up to <lv>10</> stacks. Each stack lasts independently."
            );
            assert!(report["native_description"].get("source").is_none());
            assert_eq!(
                report["compiled_proc"]["damage_bonus_per_stack"],
                json!(format!("{}/{}", expected.0, expected.1))
            );
            assert_eq!(report["compiled_proc"]["duration_us"], 10_000_000);
            assert_eq!(report["native_parameter_report"]["parameter_count"], 2);
        }
    }
    actor.character_id = "character:1003".into();
    assert!(
        crate::weapons::oraora::projection::project(&tables, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("group is incompatible")
    );
    actor.character_id = "character:1010".into();
    actor.weapon_tier = 1;
    let mut changed = tables.clone();
    let data_list = changed
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()["rows"]["upgradestar_pack_fork_oulaquantao_1"]["DataList"]
        .as_array_mut()
        .unwrap();
    data_list
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_wujinjieti_Up")
        .unwrap()["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::oraora::projection::project(&changed, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("unit hint")
    );
}
