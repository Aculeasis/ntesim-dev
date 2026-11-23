use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn bit_game_binds_five_curves_and_rejects_changed_units() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_BitGame")
        .unwrap()
        .1;
    assert_eq!(item["ItemName"]["LocalizedString"], "Blow up the Crowd");

    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE"
    );
    assert_eq!(item["ElementData"]["ExclusiveCharacterIDArray"], json!([]));
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1055", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 0,
        "energy_capacity": 100, "weapon_id": "weapon:fork_BitGame", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1", "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap();

    let expected_off_field_atk_base = [(1, 10), (11, 100), (3, 25), (13, 100), (7, 50)];
    let expected_off_field_atk_per_stack = [(1, 50), (23, 1000), (13, 500), (29, 1000), (4, 125)];
    let expected_on_field_psyche_base = [(3, 25), (7, 50), (4, 25), (9, 50), (1, 5)];
    let expected_on_field_psyche_per_stack = [(1, 50), (23, 1000), (13, 500), (29, 1000), (4, 125)];

    for character_id in [
        "character:1020",
        "character:1046",
        "character:1052",
        "character:1055",
    ] {
        actor.character_id = character_id.into();
        for tier in 1..=5 {
            let i = tier - 1;
            actor.weapon_tier = tier as u32;
            actor.gear_procs.clear();
            let report = crate::weapons::bit_game::projection::project(
                &tables,
                1,
                "ntesim-current",
                &mut actor,
                &[],
            )
            .unwrap();
            assert_eq!(
                report["native_description"]["localized"],
                "Increases the active character's ATK by <lv>{0}</> while the wearer is off-field, and increases the active character's ATK by <lv>{1}</> when the wearer deals damage, up to <lv>4</> stacks and triggering at most <lv>once</> every <lv>{2}</>s. Resets the effect when the wearer becomes the active character.\r\nIncreases Psyche DMG by <lv>{3}</> while the wearer is the active character, and increases Psyche DMG by <lv>{4}</> when dealing Psyche DMG with a Basic Attack, up to <lv>10</> stacks. Triggers at most once every <lv>0.3</>s. Resets the effect when switching characters."
            );
            assert!(report["native_description"].get("source").is_none());
            assert_eq!(
                report["compiled_proc"]["off_field_atk_base"],
                json!(format!(
                    "{}/{}",
                    expected_off_field_atk_base[i].0, expected_off_field_atk_base[i].1
                ))
            );
            assert_eq!(
                report["compiled_proc"]["off_field_atk_per_stack"],
                json!(format!(
                    "{}/{}",
                    expected_off_field_atk_per_stack[i].0, expected_off_field_atk_per_stack[i].1
                ))
            );
            assert_eq!(report["compiled_proc"]["off_field_icd_us"], 2_000_000);
            assert_eq!(
                report["compiled_proc"]["on_field_psyche_base"],
                json!(format!(
                    "{}/{}",
                    expected_on_field_psyche_base[i].0, expected_on_field_psyche_base[i].1
                ))
            );
            assert_eq!(
                report["compiled_proc"]["on_field_psyche_per_stack"],
                json!(format!(
                    "{}/{}",
                    expected_on_field_psyche_per_stack[i].0,
                    expected_on_field_psyche_per_stack[i].1
                ))
            );
            assert_eq!(report["native_parameter_report"]["parameter_count"], 5);
        }
    }

    actor.character_id = "character:1003".into();
    assert!(
        crate::weapons::bit_game::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );

    actor.character_id = "character:1055".into();
    actor.weapon_tier = 1;
    let mut changed = tables.clone();
    let data_list = changed
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()["rows"]["upgradestar_pack_fork_BitGame_1"]["DataList"]
        .as_array_mut()
        .unwrap();
    data_list
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_BitGame2_AtkUp1")
        .unwrap()["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::bit_game::projection::project(
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
