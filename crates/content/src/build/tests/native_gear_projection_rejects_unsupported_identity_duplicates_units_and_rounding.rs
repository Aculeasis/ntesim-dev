use super::*;

#[test]
fn native_gear_projection_rejects_unsupported_identity_duplicates_units_and_rounding() {
    let tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/native-gear-built-game-research.json"
    )))
    .unwrap();
    let projection = request.gear_projection.as_ref().unwrap();
    let mut wrong_weapon = request.scenario.clone();
    wrong_weapon.actors[0].weapon_id = "weapon:fork_Rose".into();
    let mut wrong_character = request.scenario.clone();
    wrong_character.actors[0].character_id = "character:1004".into();
    let mut duplicate = request.scenario.clone();
    let extra_proc = duplicate.actors[0].gear_procs[0].clone();
    duplicate.actors[0].gear_procs.push(extra_proc);
    for mut scenario in [wrong_weapon, wrong_character, duplicate] {
        assert!(project_gear(&tables, &mut scenario, projection).is_err());
    }
    let mut no_hypothesis = projection.clone();
    no_hypothesis.hypothesis.clear();
    assert!(project_gear(&tables, &mut request.scenario.clone(), &no_hypothesis).is_err());
    for (table, pointer, value) in [
        (
            "CT_ForkBuff",
            "/rows/buff_xiangdao_energy/Keys/4/Value",
            json!(18.5),
        ),
        (
            "CT_ForkBuff",
            "/rows/buff_xiangdao_CD/Keys/1/Value",
            json!(0.0000001),
        ),
        (
            "CT_ForkBuff",
            "/rows/buff_xiangdao_CD/Keys/1/Value",
            json!(0),
        ),
        (
            "CT_ForkBuff",
            "/rows/buff_xiangdao_CD/Keys/1/Value",
            json!(1e20),
        ),
        (
            "DT_ForkUpgradeStarDataTable",
            "/rows/upgradestar_pack_fork_lingganzhongjiezhe_5/DataList/0/bIsPercent",
            json!(true),
        ),
    ] {
        let mut modified = tables.clone();
        *modified
            .get_mut(table)
            .unwrap()
            .pointer_mut(pointer)
            .unwrap() = value;
        assert!(
            project_gear(&modified, &mut request.scenario.clone(), projection).is_err(),
            "{pointer}"
        );
    }
}
