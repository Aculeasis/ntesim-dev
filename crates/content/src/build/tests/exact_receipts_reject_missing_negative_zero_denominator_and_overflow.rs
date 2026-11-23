use super::*;

#[test]
fn exact_receipts_reject_missing_negative_zero_denominator_and_overflow() {
    for value in [
        json!({}),
        json!({"numerator":"-1","denominator":"1"}),
        json!({"numerator":"1","denominator":"0"}),
    ] {
        assert!(receipt(&value).is_err());
    }
    assert!(ratio(ExactFraction::new(u128::from(u64::MAX) + 1, 1).unwrap()).is_err());
    assert!(integer(&json!({"numerator":"3","denominator":"2"}), "HP").is_err());
    assert_eq!(
        receipt(&json!({"numerator":"6","denominator":"4"})).unwrap(),
        ExactFraction::new(3, 2).unwrap()
    );
}
