use super::*;
use serde_json::json;

fn load_fixture_data() -> (Value, Value, String) {
    let coverage = ntesim_data::character::released_inventory();
    let conditions = json!({});
    let variants_csv = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/mint-variant.csv"
    ))
    .to_string();
    (coverage, conditions, variants_csv)
}

#[test]
fn acceptance_scenarios_have_correct_targets_and_horizons() {
    for kind in AcceptanceScenarioKind::ALL {
        let targets = kind.create_targets();
        assert_eq!(targets.len(), kind.target_count());
        assert_eq!(kind.horizon_us(), 120_000_000);
        for t in &targets {
            assert_eq!(t.level, 80);
            assert_eq!(t.hp_max, 10_000_000);
            assert_eq!(t.defense, 1080);
            assert_eq!(t.resistance.numerator, 1);
            assert_eq!(t.resistance.denominator, 5);
        }
    }
}

#[test]
fn acceptance_deliverables_match_reviewed_csv_fingerprints() {
    let (coverage, conditions, variants_csv) = load_fixture_data();
    let (deliverables, all_comp_csv, _published_csv, _arch_csv, manifest) =
        compile_acceptance_deliverables(&coverage, &conditions, &variants_csv).unwrap();

    assert_eq!(deliverables.total_compositions, 8855);
    assert_eq!(deliverables.total_variants, 1);
    assert_eq!(deliverables.total_archetypes, 1);
    assert_eq!(deliverables.scenarios.len(), 5);
    assert_eq!(deliverables.status, "INCOMPLETE");
    assert!(!deliverables.game_dps_computed);
    assert!(!deliverables.acceptance_ready);

    assert_eq!(deliverables.all_compositions_csv_lines, 8856);
    assert_eq!(deliverables.published_variants_csv_lines, 2);
    assert_eq!(deliverables.archetypes_csv_lines, 2);

    assert_eq!(all_comp_csv.lines().count(), 8856);
    let manifest: Value = serde_json::from_str(&manifest).unwrap();
    assert_eq!(manifest["status"], "INCOMPLETE");
    assert_eq!(manifest["composition_enumeration_complete"], true);
    assert_eq!(manifest["compositions"], 8855);
    assert_eq!(
        manifest["published_archetypes"],
        deliverables.total_archetypes
    );
    assert_eq!(manifest["published_variants"], deliverables.total_variants);
    assert_eq!(manifest["conditions"], conditions);
}
