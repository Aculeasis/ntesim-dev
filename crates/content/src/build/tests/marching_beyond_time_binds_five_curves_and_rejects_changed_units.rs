use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn marching_beyond_time_binds_t1_to_t5_curves_and_rejects_unit_changes() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_Time")
        .unwrap()
        .1;
    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE"
    );
    let upgrade =
        &tables["DT_ForkUpgradeStarDataTable"]["rows"]["upgradestar_pack_fork_Time_1"]["DataList"];
    assert_eq!(
        upgrade
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["NameID"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "buff_Time_AtkUp",
            "buff_Time_stateCritDamageUp",
            "buff_Time_CritDamageUp",
            "buff_Time_DefIgnore",
            "buff_Time_DefIgnore_Dur",
        ]
    );
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot":0,"character_id":"character:1052","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_Time","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    })).unwrap();
    actor.character_id = "character:1019".into();
    assert!(
        crate::weapons::marching_beyond_time::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    actor.character_id = "character:1052".into();
    let expected = [
        [(4, 25), (6, 25), (2, 25), (3, 25)],
        [(1, 5), (3, 10), (1, 10), (3, 20)],
        [(6, 25), (9, 25), (3, 25), (9, 50)],
        [(7, 25), (21, 50), (7, 50), (21, 100)],
        [(8, 25), (12, 25), (4, 25), (6, 25)],
    ];
    for tier in 1..=5 {
        actor.weapon_tier = tier as u32;
        actor.gear_procs.clear();
        let report = crate::weapons::marching_beyond_time::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        for (name, pair) in [
            "atk_bonus",
            "state_crit_damage_bonus",
            "crit_damage_per_stack",
            "def_ignore",
        ]
        .into_iter()
        .zip(expected[tier - 1])
        {
            assert_eq!(proc[name], json!(format!("{}/{}", pair.0, pair.1)));
        }
        assert_eq!(proc["def_ignore_duration_us"], 70_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 5);
    }
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_Time_1"]["DataList"].as_array_mut().unwrap()
        .iter_mut().find(|entry| entry["NameID"] == "buff_Time_DefIgnore").unwrap()["bIsPercent"] = json!(false);
    actor.weapon_tier = 1;
    assert!(
        crate::weapons::marching_beyond_time::projection::project(
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
