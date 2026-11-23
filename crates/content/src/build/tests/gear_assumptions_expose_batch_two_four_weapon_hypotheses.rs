use super::*;

#[test]
fn studio_catalog_exposes_batch_two_four_weapon_gear_assumptions() {
    let catalog_tables: BTreeMap<String, Value> = weapon_parameter_tables();
    let coverage = ntesim_data::character::released_inventory();
    let catalog = crate::build::studio::catalog(&catalog_tables, &coverage).unwrap();
    for &(kind, id) in crate::build::studio::BATCH_TWO_FOUR_WEAPON_ASSUMPTIONS {
        let entry = &catalog["gear_assumptions"][kind];
        assert!(
            entry[id.as_str()].is_string(),
            "gear_assumptions[{kind}] misses hypothesis {}",
            id.as_str()
        );
        assert!(
            catalog["weapon_procs"]
                .as_object()
                .unwrap()
                .values()
                .any(|tiers| {
                    tiers
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|proc| proc["kind"] == kind)
                }),
            "hypothesis key {kind} has no serialized weapon proc"
        );
    }
}
