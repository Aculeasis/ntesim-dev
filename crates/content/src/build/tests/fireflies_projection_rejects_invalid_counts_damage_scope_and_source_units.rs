use super::*;

#[test]
fn fireflies_projection_preserves_solo_damage_scope_and_source_units() {
    let (mut tables, coverage, request) = fireflies_fixture();
    tables.insert(
        "DT_EquipmentSuitData".into(),
        ntesim_data::set_definitions::overlay(),
    );
    tables.insert(
        "CT_Equipmentbuff".into(),
        ntesim_data::set_parameters::overlay(),
    );
    let mut no_action_projection = request.clone();
    no_action_projection.action_projection = None;
    assert!(
        compile(
            &tables,
            &coverage,
            &no_action_projection,
            &ExecutionLimits::default()
        )
        .unwrap_err()
        .contains("action projection")
    );
    let mut team = request.scenario.clone();
    team.actors.push(team.actors[0].clone());
    let mut wrong_set = request.scenario.clone();
    wrong_set.actors[0].set_id = Some("set:Suit3".into());
    let mut duplicate = request.scenario.clone();
    let extra_proc = duplicate.actors[0].gear_procs[1].clone();
    duplicate.actors[0].gear_procs.push(extra_proc);
    for mut scenario in [team, wrong_set, duplicate] {
        assert!(
            crate::sets::fireflies::projection::project(&tables, &mut scenario, &request.builds)
                .is_err()
        );
    }
    let mut wrong_shapes = request.builds.clone();
    wrong_shapes[0].build.set_count = 3;
    assert!(
        crate::sets::fireflies::projection::project(
            &tables,
            &mut request.scenario.clone(),
            &wrong_shapes
        )
        .unwrap_err()
        .contains("set count 4")
    );
    for (table, pointer, value) in [
        (
            "DT_SkillDamageData",
            "/rows/GE_Player_Mint_Melee1_Damage_New/DamageTypeEX",
            json!("EExecutionDamageTypeEX::DAMAGE_TYPE_LIGHT"),
        ),
        (
            "DT_EquipmentSuitData",
            "/rows/Suit2/SuitStructList/1/SuitBuff/ObjectPath",
            json!("unsupported"),
        ),
        (
            "CT_Equipmentbuff",
            "/rows/Nature2_4_Stack/Keys/0/Value",
            json!(7.5),
        ),
        (
            "CT_Equipmentbuff",
            "/rows/Nature2_4_Stack/Keys/0/Value",
            json!(0),
        ),
        (
            "CT_Equipmentbuff",
            "/rows/Nature2_4_Stack/Keys/0/Value",
            json!(65),
        ),
        (
            "CT_Equipmentbuff",
            "/rows/Nature2_4_CritDamage/Keys/0/Value",
            json!(-0.08),
        ),
        (
            "CT_Equipmentbuff",
            "/rows/Nature2_4_CD/Keys/0/Value",
            json!(0),
        ),
        (
            "CT_Equipmentbuff",
            "/rows/Nature2_4_CD/Keys/0/Value",
            json!(0.0000001),
        ),
        (
            "CT_Equipmentbuff",
            "/rows/Nature2_4_CD/Keys/0/Value",
            json!(1e20),
        ),
        (
            "CT_Equipmentbuff",
            "/rows/Nature2_4_CD/Keys/0/Time",
            json!(2),
        ),
        (
            "CT_Equipmentbuff",
            "/rows/Nature2_4_CD/InterpMode",
            json!("unknown"),
        ),
        ("CT_Equipmentbuff", "/rows/Nature2_4_CD/Keys", json!([])),
    ] {
        let mut modified = tables.clone();
        *modified
            .get_mut(table)
            .unwrap()
            .pointer_mut(pointer)
            .unwrap() = value;
        assert!(
            crate::sets::fireflies::projection::project(
                &modified,
                &mut request.scenario.clone(),
                &request.builds
            )
            .is_err(),
            "{pointer}"
        );
    }
    let mut missing = tables.clone();
    missing.get_mut("CT_Equipmentbuff").unwrap()["rows"] = json!({});
    assert!(
        crate::sets::fireflies::projection::project(
            &missing,
            &mut request.scenario.clone(),
            &request.builds
        )
        .is_err()
    );
    let mut overflow = request.scenario.clone();
    overflow.absolute_limit_us = u64::MAX;
    assert!(
        crate::sets::fireflies::projection::project(&tables, &mut overflow, &request.builds)
            .unwrap_err()
            .contains("deadline overflow")
    );
}
