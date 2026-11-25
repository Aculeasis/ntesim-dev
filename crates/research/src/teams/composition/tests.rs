use super::*;

fn load_data() -> (Value, String) {
    let coverage = ntesim_data::character::released_inventory();
    let csv = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/mint-variant.csv"
    ))
    .to_string();
    (coverage, csv)
}

#[test]
fn enumeration_yields_exact_8855_compositions_and_published_variants() {
    let (coverage, csv) = load_data();
    let report = enumerate_compositions(&coverage, &csv).unwrap();

    assert_eq!(report.total_characters, 24);
    assert_eq!(report.total_compositions, 10626);
    assert_eq!(report.published_variants_count, 1);
    assert_eq!(report.matched_published_compositions, 1);
    assert_eq!(report.unconventional_compositions, 10625);
    assert_eq!(report.distinct_archetypes_count, 1);
    assert_eq!(report.status, "INCOMPLETE");
    assert_eq!(report.quality, Quality::Incomplete);
    assert!(!report.game_dps_computed);
    assert!(!report.acceptance_ready);

    assert_eq!(
        report.compositions[0].composition,
        "adler+akane+aurelia+baicang"
    );
    assert_eq!(report.compositions[0].index, 1);
    assert_eq!(
        report.compositions[10625].composition,
        "shinku+skia+zankou+zero"
    );
    assert_eq!(report.compositions[10625].index, 10626);

    let csv_text = generate_csv(&report);
    let lines: Vec<&str> = csv_text.lines().collect();
    assert_eq!(lines.len(), 10627);
    assert_eq!(
        lines[0],
        "composition,status,dps_1_real120,dps_4_real120,dps_1_mode120,dps_4_mode120,dps_no_break,reason,rotation_search_performed"
    );
    assert_eq!(
        lines[1],
        "adler+akane+aurelia+baicang,INCOMPLETE,,,,,,timings; energy; hit geometry; kit adapters; Global applicability,False"
    );
}
