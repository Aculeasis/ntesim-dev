use super::*;
use std::{fs, path::Path};

fn load_fixture_data() -> (BTreeMap<String, Value>, Value) {
    (
        BTreeMap::new(),
        ntesim_data::character::released_inventory(),
    )
}

const SAMPLE_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/mint-variant.csv"
));

fn sample_mint_build() -> equipment::Request {
    let mut request: equipment::Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../reviews/s1-mint-a0/build.json"
    )))
    .unwrap();
    request.weapon_tier = 5;
    request
}

#[test]
fn run_rejects_invalid_schema_or_empty_hypothesis() {
    let (tables, coverage) = load_fixture_data();
    let mut request = Request {
        schema_version: 2,
        content_id: "ntesim-current".into(),
        hypothesis: "Test hypothesis".into(),
        variants_csv: Some(SAMPLE_CSV.into()),
        builds: BTreeMap::new(),
    };
    assert!(
        run(
            &tables,
            &coverage,
            SAMPLE_CSV,
            &request,
            &ExecutionLimits::default()
        )
        .is_err()
    );

    request.schema_version = 1;
    request.hypothesis = "   ".into();
    assert!(
        run(
            &tables,
            &coverage,
            SAMPLE_CSV,
            &request,
            &ExecutionLimits::default()
        )
        .is_err()
    );

    request.hypothesis = "Valid hypothesis".into();
    request.builds.clear();
    assert!(
        run(
            &tables,
            &coverage,
            SAMPLE_CSV,
            &request,
            &ExecutionLimits::default()
        )
        .is_err()
    );
}

#[test]
fn run_rejects_missing_variant_character_override() {
    let (tables, coverage) = load_fixture_data();
    let mut builds = BTreeMap::new();
    builds.insert("mint".to_string(), sample_mint_build());

    let request = Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        hypothesis: "Test hypothesis".into(),
        variants_csv: Some(SAMPLE_CSV.into()),
        builds,
    };

    let err = run(
        &tables,
        &coverage,
        SAMPLE_CSV,
        &request,
        &ExecutionLimits::default(),
    )
    .unwrap_err();
    assert!(err.contains("without an audited build override"));
}

#[test]
fn run_rejects_character_id_mismatch() {
    let (tables, coverage) = load_fixture_data();
    let mut bad_mint = sample_mint_build();
    bad_mint.character_id = "character:1003".into();

    let mut builds = BTreeMap::new();
    builds.insert("mint".to_string(), bad_mint);

    let request = Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        hypothesis: "Test hypothesis".into(),
        variants_csv: Some(SAMPLE_CSV.into()),
        builds,
    };

    let err = run(
        &tables,
        &coverage,
        SAMPLE_CSV,
        &request,
        &ExecutionLimits::default(),
    )
    .unwrap_err();
    assert!(err.contains("disagrees with canonical profile character_id"));
}

#[test]
fn run_executes_variant_with_audited_builds_and_retains_receipts() {
    let (tables, coverage) = load_fixture_data();
    let team_loadout_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/team-loadout.json");
    let team_loadout: Value =
        serde_json::from_slice(&fs::read(team_loadout_path).unwrap()).unwrap();
    let actors = team_loadout["actors"].as_array().unwrap();

    let mut builds = BTreeMap::new();
    let slugs = ["sakiri", "lacrimosa", "skia", "nanally"];
    for (i, slug) in slugs.iter().enumerate() {
        let build_req: equipment::Request =
            serde_json::from_value(actors[i]["build"].clone()).unwrap();
        builds.insert(slug.to_string(), build_req);
    }

    let custom_csv = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/built-roster.csv"
    ));

    let request = Request {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        hypothesis: "Audited static build projection hypothesis for team loadout variant".into(),
        variants_csv: Some(custom_csv.into()),
        builds,
    };

    let result = run(
        &tables,
        &coverage,
        custom_csv,
        &request,
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(result["kind"], "published-built-variants-v1");
    assert_eq!(result["status"], "INCOMPLETE");
    assert_eq!(result["quality"], "incomplete");
    assert_eq!(result["game_dps_computed"], false);
    assert_eq!(result["acceptance_ready"], false);
    assert_eq!(result["audited_characters_count"], 4);

    let sakiri_receipt = &result["character_build_receipts"]["sakiri"];
    assert_eq!(sakiri_receipt["character_id"], "character:1003");
    assert_eq!(sakiri_receipt["weapon_id"], "weapon:fork_mofeikesi");
    assert_eq!(
        sakiri_receipt["element_bonus_attribute"],
        "DamageUpIncantationBase"
    );
    assert_eq!(sakiri_receipt["compiled_static_stats"]["base_atk"], 1094);

    let batch = &result["batch"];
    assert_eq!(batch["total_variants"], 1);
    assert_eq!(batch["passed_variants"], 1);
    assert_eq!(batch["failed_variants"], 0);
    assert_eq!(batch["all_invariants_passed"], true);
    assert_eq!(batch["all_energy_conserved"], true);

    let variant_summary = &batch["variants"][0];
    assert_eq!(variant_summary["success"], true);
    assert_eq!(variant_summary["energy_conserved"], true);
    assert_eq!(variant_summary["invariants_passed"], true);
    assert!(variant_summary["hits_settled"].as_u64().unwrap() >= 4);
    assert!(variant_summary["total_damage_dealt"].as_f64().unwrap() > 0.0);
}
