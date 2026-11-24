use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn echoes_on_hold_binds_t1_to_t5_curves_and_rejects_unit_changes() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_elyms")
        .unwrap()
        .1;
    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE"
    );
    let upgrade =
        &tables["DT_ForkUpgradeStarDataTable"]["rows"]["upgradestar_pack_fork_elyms_1"]["DataList"];
    assert_eq!(
        upgrade
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["NameID"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "ntesim.echoes_on_hold.atk",
            "ntesim.echoes_on_hold.team_def_ignore",
            "ntesim.echoes_on_hold.wearer_def_ignore",
            "ntesim.echoes_on_hold.cosmos_penetration",
            "ntesim.echoes_on_hold.duration",
        ]
    );
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot":0,"character_id":"character:1020","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_elyms","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    })).unwrap();
    actor.character_id = "character:1019".into();
    assert!(
        crate::weapons::echoes_on_hold::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    actor.character_id = "character:1020".into();
    let expected = [
        [(3, 20), (3, 25), (1, 10), (2, 25)],
        [(3, 16), (7, 50), (1, 8), (11, 100)],
        [(9, 40), (4, 25), (3, 20), (7, 50)],
        [(21, 80), (9, 50), (7, 40), (17, 100)],
        [(3, 10), (1, 5), (1, 5), (1, 5)],
    ];
    for tier in 1..=5 {
        actor.weapon_tier = tier as u32;
        actor.gear_procs.clear();
        let report = crate::weapons::echoes_on_hold::projection::project(
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
            "team_def_ignore",
            "wearer_def_ignore",
            "cosmos_penetration",
        ]
        .into_iter()
        .zip(expected[tier - 1])
        {
            assert_eq!(proc[name], json!(format!("{}/{}", pair.0, pair.1)));
        }
        assert_eq!(proc["duration_us"], 20_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 5);
    }
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_elyms_1"]["DataList"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["NameID"] == "ntesim.echoes_on_hold.team_def_ignore")
        .unwrap()["bIsPercent"] = json!(false);
    actor.weapon_tier = 1;
    assert!(
        crate::weapons::echoes_on_hold::projection::project(
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

#[test]
fn echoes_on_hold_maxed_stats_contribute_570_base_atk_and_24_crit_rate() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let compiled = ntesim_data::weapon::compile(
        &tables,
        &ntesim_data::weapon::Request {
            schema_version: 1,
            content_id: "ntesim-current".into(),
            weapon_id: "weapon:fork_elyms".into(),
            level: 80,
            breakthrough: 6,
            refinement: 1,
        },
    )
    .unwrap();
    assert_eq!(
        compiled["attributes"]["AtkBase"],
        json!({"numerator":"570","denominator":"1"})
    );
    assert_eq!(
        compiled["attributes"]["CritBase"],
        json!({"numerator":"6","denominator":"25"})
    );
}
