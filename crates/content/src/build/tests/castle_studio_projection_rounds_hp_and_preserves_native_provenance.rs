use super::*;

#[test]
fn castle_studio_projection_rounds_hp_and_preserves_native_provenance() {
    let request = studio_projection::Request {
        slot: 0,
        content_id: "ntesim-current".into(),
        character_id: "character:1021".into(),
        level: 80,
        awakening: ntesim_data::AwakeningSelection::prefix(4),
        max_level: Some(80),
        weapon_id: Some("weapon:fork_Castle".into()),
        weapon_level: Some(80),
        weapon_max_level: Some(80),
        refinement: 5,
        set_id: None,
        set_count: 0,
        base_overrides: studio_projection::BaseOverrides::default(),
        additive_stats: BTreeMap::new(),
    };
    let report =
        studio_projection::project(ntesim_data::stat_projection::frozen_tables(), &request)
            .unwrap();
    assert_eq!(report.static_stats.hp, 22_498);
    assert!(
        report
            .model_assumptions
            .contains_key("build.static_additive_v1")
    );
    assert!(report.evidence.as_array().unwrap().iter().any(|item| {
        item["source_id"] == "table:typed_weapon_definitions"
            && item["pointer"] == "/rows/fork_Castle"
    }));
}
