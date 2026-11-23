use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn far_side_of_the_moon_binds_all_five_refinement_curves_and_ignores_cosmetic_exclusives() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_prism")
        .unwrap()
        .1;
    assert_eq!(item["ItemName"]["LocalizedString"], "Far Side of the Moon");
    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO"
    );
    assert_eq!(
        item["ElementData"]["ExclusiveCharacterIDArray"],
        json!(["1057"])
    );
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot":0,"character_id":"character:1054","character_level":80,"awakening":[],
        "purchased_skill_level":8,"initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_prism","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
          "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap();
    let mut incompatible = actor.clone();
    incompatible.character_id = "character:1003".into();
    assert!(
        crate::weapons::far_side_moon::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut incompatible,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    actor.character_id = "character:1054".into();

    let expected = [
        [(4, 25), (7, 100), (7, 25), (4, 25)],
        [(1, 5), (7, 80), (7, 20), (1, 5)],
        [(6, 25), (21, 200), (21, 50), (6, 25)],
        [(7, 25), (49, 400), (49, 100), (7, 25)],
        [(8, 25), (7, 50), (14, 25), (8, 25)],
    ];
    for (tier, values) in (1..=5).zip(expected) {
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::far_side_moon::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        for (name, value) in [
            ("crit_rate_bonus", values[0]),
            ("skill_damage_per_stack", values[1]),
            ("basic_attack_damage_bonus", values[2]),
            ("lakshana_damage_bonus", values[3]),
        ] {
            assert_eq!(proc[name], json!(format!("{}/{}", value.0, value.1)));
        }
        assert_eq!(proc["duration_us"], 15_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 5);
    }

    let mut changed = tables.clone();
    actor.weapon_tier = 1;
    let row = changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_prism_1"]["DataList"]
        .as_array_mut().unwrap();
    row.iter_mut()
        .find(|entry| entry["NameID"] == "buff_Prism_DamageUp2_CD")
        .unwrap()["bIsPercent"] = json!(true);
    assert!(
        crate::weapons::far_side_moon::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );

    let catalog_tables = weapon_parameter_tables();
    let coverage = ntesim_data::character::released_inventory();
    let catalog = crate::build::studio::catalog(&catalog_tables, &coverage).unwrap();
    let procs = catalog["weapon_procs"]["weapon:fork_prism"]
        .as_array()
        .unwrap();
    assert_eq!(procs.len(), 5);
    assert!(procs.iter().all(|proc| proc["kind"] == "far_side_moon"));
    assert!(catalog["gear_assumptions"]["far_side_moon"]["weapons.far_side_moon_v2"].is_string());
    assert!(
        catalog["gear_assumptions"]["far_side_moon"]["gear.skill_cast_buff_before_hits_v1"]
            .is_string()
    );
}
