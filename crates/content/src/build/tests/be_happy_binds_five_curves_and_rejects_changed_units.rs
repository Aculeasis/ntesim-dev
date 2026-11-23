use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn be_happy_binds_five_curves_and_rejects_changed_units() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let data =
        &tables["DT_ForkUpgradeStarDataTable"]["rows"]["upgradestar_pack_fork_vine_1"]["DataList"];
    assert_eq!(
        data.as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["NameID"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["buff_vine_Hp", "buff_vine_CD"]
    );
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot":0,"character_id":"character:1008","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_vine","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    })).unwrap();
    let expected = [(3, 25), (7, 50), (4, 25), (9, 50), (1, 5)];
    for tier in 1..=5 {
        actor.weapon_tier = tier as u32;
        actor.gear_procs.clear();
        let report = crate::weapons::be_happy::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(
            proc["heal_ratio"],
            json!(format!("{}/{}", expected[tier - 1].0, expected[tier - 1].1))
        );
        assert_eq!(proc["cooldown_us"], 20_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 2);
    }
    actor.character_id = "character:1019".into();
    assert!(
        crate::weapons::be_happy::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    actor.character_id = "character:1008".into();
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_vine_1"]["DataList"].as_array_mut().unwrap()
        .iter_mut().find(|entry| entry["NameID"] == "buff_vine_Hp").unwrap()["bIsPercent"] = json!(false);
    actor.weapon_tier = 1;
    assert!(
        crate::weapons::be_happy::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("unit hint")
    );
}
