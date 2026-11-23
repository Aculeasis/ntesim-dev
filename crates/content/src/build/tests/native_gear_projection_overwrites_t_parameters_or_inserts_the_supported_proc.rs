use super::*;

#[test]
fn native_gear_projection_overwrites_t_parameters_or_inserts_the_supported_proc() {
    let tables: BTreeMap<String, Value> =
        BTreeMap::from([("CT_ForkBuff".into(), ntesim_data::weapon_curves::overlay())]);
    let request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/native-gear-built-game-research.json"
    )))
    .unwrap();
    for tier in 1..=5 {
        let mut scenario = request.scenario.clone();
        scenario.actors[0].weapon_tier = tier;
        let receipts = project_gear(
            &tables,
            &mut scenario,
            request.gear_projection.as_ref().unwrap(),
        )
        .unwrap();
        assert_eq!(receipts[0]["compiled_proc"]["energy_grant"], 8 + 2 * tier);
        assert_eq!(receipts[0]["compiled_proc"]["cooldown_us"], 20_000_000);
        assert_eq!(receipts[0]["replaced_proc"]["energy_grant"], 1);
        assert_eq!(receipts[0]["runtime_execution_validated"], true);
        assert_eq!(
            serde_json::to_value(&scenario.actors[0].gear_procs[1]).unwrap(),
            serde_json::to_value(&request.scenario.actors[0].gear_procs[1]).unwrap()
        );
    }
    let mut scenario = request.scenario.clone();
    scenario.actors[0].gear_procs.remove(0);
    let receipts = project_gear(
        &tables,
        &mut scenario,
        request.gear_projection.as_ref().unwrap(),
    )
    .unwrap();
    assert_eq!(receipts[0]["replaced_proc"], Value::Null);
    assert_eq!(scenario.actors[0].gear_procs.len(), 2);
}
