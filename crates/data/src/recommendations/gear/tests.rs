use super::*;

#[test]
fn quoted_csv_preserves_commas_quotes_and_multiline_values_without_repairing_errors() {
    assert_eq!(
        csv_records("\u{feff}a,b\r\n\"x,y\",\"a\"\"b\"\r\n\"line\nnext\",\"\"").unwrap(),
        vec![vec!["a", "b"], vec!["x,y", "a\"b"], vec!["line\nnext", ""]]
    );
    for csv in ["\"unterminated", "\"a\"junk,b", "a\"b,c", "a,b\r"] {
        assert!(csv_records(csv).is_err());
    }
    let report = bind(
        &BTreeMap::new(),
        &crate::character::released_inventory(),
        crate::research_inputs::RECOMMENDED_GEAR_CSV,
    )
    .unwrap();
    assert_eq!(report["character_count"], 23);
    assert_eq!(report["recommendation_records"], 141);
}
