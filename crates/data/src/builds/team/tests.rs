use super::*;

fn sample_build(cid: &str, wid: &str) -> equipment::Request {
    let mut build: equipment::Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/build-stats.json"
    )))
    .unwrap();
    build.character_id = format!("character:{cid}");
    build.weapon_id = format!("weapon:{wid}");
    build
}

fn load_test_tables() -> BTreeMap<String, Value> {
    BTreeMap::new()
}

#[test]
fn team_compiles_four_distinct_actors_and_validates_scenario() {
    let tables = load_test_tables();
    let request = Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        actors: vec![
            ActorLoadout {
                slot: 0,
                build: sample_build("1003", "fork_mofeikesi"),
                awakening: 0,
                initial_energy: Some(100.0),
                initial_hp_fraction: Some(1.0),
            },
            ActorLoadout {
                slot: 1,
                build: sample_build("1004", "fork_Rose"),
                awakening: 1,
                initial_energy: Some(50.0),
                initial_hp_fraction: Some(1.0),
            },
            ActorLoadout {
                slot: 2,
                build: sample_build("1008", "fork_Kite"),
                awakening: 0,
                initial_energy: None,
                initial_hp_fraction: None,
            },
            ActorLoadout {
                slot: 3,
                build: sample_build("1010", "fork_GoldRecord"),
                awakening: 2,
                initial_energy: None,
                initial_hp_fraction: None,
            },
        ],
        targets: vec![TargetProfile {
            id: "target_boss".into(),
            level: 90,
            base_hp: 5_000_000,
            base_def: 800,
            resistances: BTreeMap::from([
                ("INCANTATION".into(), 0.1),
                ("CHAOS".into(), 0.1),
                ("LAKSHANA".into(), 0.1),
                ("NATURE".into(), 0.1),
            ]),
            weakness: Some("INCANTATION".into()),
            break_gauge: Some(1000),
        }],
        duration_us: 120_000_000,
        active_actor_slot: 0,
    };

    let report = compile(&tables, &request).unwrap();
    assert_eq!(report["kind"], "static-team-scenario-v1");
    assert_eq!(report["team_size"], 4);
    assert_eq!(report["active_actor_slot"], 0);
    assert_eq!(report["active_character_id"], "character:1003");
    assert_eq!(report["quality"], "incomplete");
    assert_eq!(report["total_target_hp"], "5000000");

    let elements = report["team_elements"].as_array().unwrap();
    assert_eq!(elements.len(), 4);

    let actors = report["actors"].as_array().unwrap();
    assert_eq!(actors.len(), 4);
    assert_eq!(actors[0]["character_id"], "character:1003");
    assert_eq!(actors[1]["character_id"], "character:1004");
    assert_eq!(actors[2]["character_id"], "character:1008");
    assert_eq!(actors[3]["character_id"], "character:1010");
    assert_eq!(actors[0]["known_derived_stats"]["atk"], 2416.55);
    assert_eq!(actors[1]["known_derived_stats"]["atk"], 2078.25);
}

#[test]
fn team_rejects_duplicate_characters() {
    let tables = load_test_tables();
    let request = Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        actors: vec![
            ActorLoadout {
                slot: 0,
                build: sample_build("1003", "fork_mofeikesi"),
                awakening: 0,
                initial_energy: None,
                initial_hp_fraction: None,
            },
            ActorLoadout {
                slot: 1,
                build: sample_build("1003", "fork_mofeikesi"),
                awakening: 0,
                initial_energy: None,
                initial_hp_fraction: None,
            },
        ],
        targets: vec![TargetProfile {
            id: "boss".into(),
            level: 80,
            base_hp: 1_000_000,
            base_def: 500,
            resistances: BTreeMap::new(),
            weakness: None,
            break_gauge: None,
        }],
        duration_us: 60_000_000,
        active_actor_slot: 0,
    };

    let err = compile(&tables, &request).unwrap_err();
    assert!(err.contains("duplicate character in team: character:1003"));
}

#[test]
fn team_rejects_empty_targets_and_out_of_bounds_slots() {
    let tables = load_test_tables();
    let mut request = Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        actors: vec![ActorLoadout {
            slot: 0,
            build: sample_build("1003", "fork_mofeikesi"),
            awakening: 0,
            initial_energy: None,
            initial_hp_fraction: None,
        }],
        targets: vec![],
        duration_us: 60_000_000,
        active_actor_slot: 0,
    };
    assert!(compile(&tables, &request).is_err());

    request.targets = vec![TargetProfile {
        id: "boss".into(),
        level: 80,
        base_hp: 1_000_000,
        base_def: 500,
        resistances: BTreeMap::new(),
        weakness: None,
        break_gauge: None,
    }];
    request.active_actor_slot = 1;
    assert!(compile(&tables, &request).is_err());
}

#[test]
fn team_uses_compiler_identities_and_rejects_unknown_state_instead_of_defaults() {
    let tables = load_test_tables();
    let request = Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        actors: vec![ActorLoadout {
            slot: 0,
            build: sample_build("1003", "FORK_MOFEIKESI"),
            awakening: 0,
            initial_energy: Some(0.0),
            initial_hp_fraction: Some(0.0),
        }],
        targets: vec![TargetProfile {
            id: "target".into(),
            level: 80,
            base_hp: 100,
            base_def: 500,
            resistances: BTreeMap::new(),
            weakness: None,
            break_gauge: None,
        }],
        duration_us: 1_000_000,
        active_actor_slot: 0,
    };
    let report = compile(&tables, &request).unwrap();
    let standalone = equipment::compile(&tables, &request.actors[0].build).unwrap();
    for field in ["character_id", "weapon_id", "set_id"] {
        assert_eq!(report["actors"][0][field], standalone[field]);
    }
    assert_eq!(report["actors"][0]["initial_energy"].as_f64(), Some(0.0));
    assert_eq!(
        report["actors"][0]["initial_hp_fraction"].as_f64(),
        Some(0.0)
    );
    for energy in [-1.0, f64::NAN, f64::INFINITY] {
        let mut changed = request.clone();
        changed.actors[0].initial_energy = Some(energy);
        assert!(
            compile(&tables, &changed)
                .unwrap_err()
                .contains("initial_energy")
        );
    }
    let mut val = serde_json::to_value(&request).unwrap();
    val["mode"] = serde_json::json!("research");
    assert!(serde_json::from_value::<Request>(val).is_err());
    let mut changed = request.clone();
    changed.actors[0].build.character_id = "character:1051".into();
    assert!(compile(&tables, &changed).is_err());
}
