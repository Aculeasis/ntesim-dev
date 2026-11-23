use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn spider_binds_three_curves_and_rejects_changed_units() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_spider")
        .unwrap()
        .1;
    assert_eq!(
        item["ItemName"]["LocalizedString"],
        "Failing You, Heavy in My Heart"
    );

    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR"
    );
    assert_eq!(item["ElementData"]["ExclusiveCharacterIDArray"], json!([]));
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot": 0, "character_id": "character:1003", "character_level": 80, "awakening": [],
        "purchased_skill_level": 8, "initial_hp": 1, "initial_hp_max": 1, "initial_energy": 0,
        "energy_capacity": 100, "weapon_id": "weapon:fork_spider", "weapon_tier": 1,
        "static_stats": {
            "base_atk": 1, "atk": "1/1", "hp": 1, "def": 1,
            "crit_rate": "0/1", "crit_damage_bonus": "0/1"
        }
    }))
    .unwrap();

    let expected_atk_per_stack = [(1, 100), (23, 2000), (13, 1000), (29, 2000), (2, 125)];
    let expected_atk_max_stack = [(1, 50), (23, 1000), (13, 500), (29, 1000), (4, 125)];

    for character_id in [
        "character:1003",
        "character:1008",
        "character:1036",
        "character:1042",
        "character:1073",
    ] {
        actor.character_id = character_id.into();
        for tier in 1..=5 {
            let i = tier - 1;
            actor.weapon_tier = tier as u32;
            actor.gear_procs.clear();
            let report = crate::weapons::spider::projection::project(
                &tables,
                1,
                "ntesim-current",
                &mut actor,
                &[],
            )
            .unwrap();
            assert_eq!(
                report["native_description"]["localized"],
                "Grants the wearer <lv>1</> stack of Spider Knowledge when dealing damage with Basic Attacks, up to <lv>8</> stacks (up to <lv>1</> stack every <lv>0.5</>s). All stacks are consumed when casting Ultimate. Increases the entire team's ATK by <lv>{1}</> for <lv>{0}</>s <lv>per</> stack consumed. Increases ATK by an additional <lv>{2}</> when <lv>8</> stacks are consumed."
            );
            assert!(report["native_description"].get("source").is_none());
            assert_eq!(
                report["compiled_proc"]["atk_bonus_per_stack"],
                json!(format!(
                    "{}/{}",
                    expected_atk_per_stack[i].0, expected_atk_per_stack[i].1
                ))
            );
            assert_eq!(
                report["compiled_proc"]["atk_bonus_max_stack"],
                json!(format!(
                    "{}/{}",
                    expected_atk_max_stack[i].0, expected_atk_max_stack[i].1
                ))
            );
            assert_eq!(report["compiled_proc"]["duration_us"], 10_000_000);
            assert_eq!(report["native_parameter_report"]["parameter_count"], 3);
        }
    }

    actor.character_id = "character:1019".into();
    assert!(
        crate::weapons::spider::projection::project(&tables, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("group is incompatible")
    );

    actor.character_id = "character:1003".into();
    actor.weapon_tier = 1;
    let mut changed = tables.clone();
    let data_list = changed
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()["rows"]["upgradestar_pack_fork_spider_1"]["DataList"]
        .as_array_mut()
        .unwrap();
    data_list
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_spider_AtkUp")
        .unwrap()["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::spider::projection::project(&changed, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("unit hint")
    );
}
