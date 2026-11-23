//! Consumed level1..80 Break bases; native UnbaldamagePara, selected content model.
const BASE_BY_LEVEL: [u64; 80] = [
    91, 93, 96, 98, 101, 104, 106, 109, 111, 114, 117, 119, 122, 124, 127, 130, 132, 135, 137, 140,
    148, 159, 170, 182, 193, 204, 214, 226, 241, 257, 273, 289, 306, 325, 345, 367, 390, 413, 438,
    462, 488, 517, 548, 581, 621, 663, 707, 752, 800, 854, 897, 942, 990, 1041, 1106, 1168, 1233,
    1300, 1370, 1440, 1515, 1604, 1693, 1788, 1896, 1990, 2083, 2184, 2285, 2384, 2486, 2602, 2707,
    2821, 2984, 3135, 3250, 3367, 3484, 3603,
];

pub(super) fn base_at_level(level: u32) -> Result<u64, String> {
    level
        .checked_sub(1)
        .and_then(|index| BASE_BY_LEVEL.get(index as usize))
        .copied()
        .ok_or_else(|| "Break base requires character level 1..=80".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn break_bases_preserve_independent_band_endpoints_and_reject_outside_levels() {
        for (level, expected) in [(1, 91), (20, 140), (40, 462), (60, 1440), (80, 3603)] {
            assert_eq!(base_at_level(level).unwrap(), expected);
        }
        assert!(base_at_level(0).is_err());
        assert!(base_at_level(81).is_err());
    }
}
