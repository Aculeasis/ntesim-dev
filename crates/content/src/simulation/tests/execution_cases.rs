use super::*;

#[test]
pub(crate) fn exact_rounding_preserves_half_and_large_integer_boundaries() {
    for (n, d, expected) in [
        (1, 2, 1),
        (1, 3, 0),
        (5, 2, 3),
        (u64::MAX as u128, 1, u64::MAX),
        ((1u128 << 54) + 1, 1, (1u64 << 54) + 1),
    ] {
        assert_eq!(
            ExactFraction::new(n, d).unwrap().round_u64().unwrap(),
            expected
        );
    }
    assert!(
        ExactFraction::new(u64::MAX as u128 + 1, 1)
            .unwrap()
            .round_u64()
            .is_err()
    );
}
