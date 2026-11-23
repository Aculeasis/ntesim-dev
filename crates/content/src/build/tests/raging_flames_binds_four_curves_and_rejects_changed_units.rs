use super::*;
use crate::simulation::GameActorConfig;

#[test]
fn raging_flames_binds_four_curves_and_rejects_changed_units() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let item = ntesim_data::weapon_definitions::lookup(&tables, "fork_wushoutieyu")
        .unwrap()
        .1;
    assert_eq!(item["ItemName"]["LocalizedString"], "Raging Flames");
    assert_eq!(
        item["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE"
    );
    assert_eq!(item["ElementData"]["ExclusiveCharacterIDArray"], json!([]));
    let mut actor: GameActorConfig = serde_json::from_value(json!({
        "slot":0,"character_id":"character:1010","character_level":80,"awakening":[],
        "purchased_skill_level":8,"initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_wushoutieyu","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
          "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap();
    let mut incompatible = actor.clone();
    incompatible.character_id = "character:1003".into();
    assert!(
        crate::weapons::raging_flames::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut incompatible,
            &[]
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    for (refinement, base, skill, lakshana) in [
        (1, (1, 10), (1, 20), (3, 20)),
        (2, (1, 8), (3, 50), (7, 40)),
        (3, (3, 20), (7, 100), (1, 5)),
        (4, (7, 40), (2, 25), (9, 40)),
        (5, (1, 5), (9, 100), (1, 4)),
    ] {
        actor.weapon_tier = refinement;
        actor.gear_procs.clear();
        let report = crate::weapons::raging_flames::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        assert_eq!(report["compiled_proc"]["duration_us"], 10_000_000);
        assert_eq!(
            report["compiled_proc"]["redirect_ultimate_bonus"],
            json!(format!("{}/{}", base.0, base.1))
        );
        assert_eq!(
            report["compiled_proc"]["redirect_skill_stack_bonus"],
            json!(format!("{}/{}", skill.0, skill.1))
        );
        assert_eq!(
            report["compiled_proc"]["lakshana_bonus"],
            json!(format!("{}/{}", lakshana.0, lakshana.1))
        );
        assert_eq!(report["native_parameter_report"]["parameter_count"], 4);
        assert_eq!(
            report["native_parameter_report"]["description"]["LocalizedString"],
            "Increases Lakshana DMG by <lv>{3}</>.\r\nIncreases the wearer's Redirect Skill and Ultimate DMG by <lv>{0}</> for <lv>{1}</>s after casting Ultimate. Increases the wearer’s Redirect Skill DMG by <lv>{2}</> per cast while active, up to 2 stacks."
        );
    }

    let mut changed = tables.clone();
    actor.weapon_tier = 1;
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_Fork_wushoutieyu_1"]
        ["DataList"][0]["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::raging_flames::projection::project(
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

#[test]
fn raging_flames_catalog_projects_all_refinements_for_nanally() {
    let tables = weapon_parameter_tables();
    let coverage = ntesim_data::character::released_inventory();
    let catalog = crate::build::studio::catalog(&tables, &coverage).unwrap();
    let procs = catalog["weapon_procs"]["weapon:fork_wushoutieyu"]
        .as_array()
        .unwrap();
    assert_eq!(procs.len(), 5);
    assert!(procs.iter().all(|proc| proc["kind"] == "raging_flames"));
}
