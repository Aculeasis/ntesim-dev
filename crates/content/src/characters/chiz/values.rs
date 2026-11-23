//! Grain capacity and the common grant consumed by the five admitted Normal stages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LevelGrains {
    pub(super) capacity: u64,
    pub(super) normal_grant: u64,
}

impl LevelGrains {
    const fn new(capacity: u64, normal_grant: u64) -> Self {
        Self {
            capacity,
            normal_grant,
        }
    }
}

const BY_LEVEL: [LevelGrains; 80] = [
    LevelGrains::new(500, 20),
    LevelGrains::new(500, 20),
    LevelGrains::new(525, 21),
    LevelGrains::new(550, 22),
    LevelGrains::new(575, 23),
    LevelGrains::new(600, 24),
    LevelGrains::new(625, 25),
    LevelGrains::new(650, 26),
    LevelGrains::new(675, 27),
    LevelGrains::new(700, 28),
    LevelGrains::new(730, 29),
    LevelGrains::new(760, 30),
    LevelGrains::new(790, 31),
    LevelGrains::new(820, 32),
    LevelGrains::new(850, 34),
    LevelGrains::new(880, 35),
    LevelGrains::new(910, 36),
    LevelGrains::new(940, 37),
    LevelGrains::new(970, 38),
    LevelGrains::new(1000, 40),
    LevelGrains::new(1140, 45),
    LevelGrains::new(1280, 51),
    LevelGrains::new(1420, 56),
    LevelGrains::new(1560, 62),
    LevelGrains::new(1700, 68),
    LevelGrains::new(1840, 73),
    LevelGrains::new(1980, 79),
    LevelGrains::new(2120, 84),
    LevelGrains::new(2260, 86),
    LevelGrains::new(2400, 90),
    LevelGrains::new(2660, 93),
    LevelGrains::new(2920, 100),
    LevelGrains::new(3180, 110),
    LevelGrains::new(3440, 120),
    LevelGrains::new(3700, 125),
    LevelGrains::new(3960, 135),
    LevelGrains::new(4220, 145),
    LevelGrains::new(4480, 155),
    LevelGrains::new(4740, 165),
    LevelGrains::new(5000, 170),
    LevelGrains::new(5500, 183),
    LevelGrains::new(6000, 196),
    LevelGrains::new(6500, 209),
    LevelGrains::new(7000, 222),
    LevelGrains::new(7500, 235),
    LevelGrains::new(8000, 248),
    LevelGrains::new(8500, 261),
    LevelGrains::new(9000, 274),
    LevelGrains::new(9500, 287),
    LevelGrains::new(10000, 300),
    LevelGrains::new(10200, 306),
    LevelGrains::new(10400, 312),
    LevelGrains::new(10600, 318),
    LevelGrains::new(10800, 324),
    LevelGrains::new(11000, 330),
    LevelGrains::new(11200, 336),
    LevelGrains::new(11400, 342),
    LevelGrains::new(11600, 348),
    LevelGrains::new(11800, 354),
    LevelGrains::new(12000, 360),
    LevelGrains::new(12200, 366),
    LevelGrains::new(12400, 372),
    LevelGrains::new(12600, 378),
    LevelGrains::new(12800, 384),
    LevelGrains::new(13000, 390),
    LevelGrains::new(13200, 396),
    LevelGrains::new(13400, 402),
    LevelGrains::new(13600, 408),
    LevelGrains::new(13800, 414),
    LevelGrains::new(14000, 420),
    LevelGrains::new(14100, 423),
    LevelGrains::new(14200, 426),
    LevelGrains::new(14300, 429),
    LevelGrains::new(14400, 432),
    LevelGrains::new(14500, 435),
    LevelGrains::new(14600, 438),
    LevelGrains::new(14700, 441),
    LevelGrains::new(14800, 444),
    LevelGrains::new(14900, 447),
    LevelGrains::new(15000, 450),
];

pub(super) fn at_level(level: u32) -> Result<&'static LevelGrains, String> {
    level
        .checked_sub(1)
        .and_then(|index| BY_LEVEL.get(index as usize))
        .ok_or_else(|| "Chiz Grain values require character level 1..=80".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grain_capacity_and_normal_grants_retain_independent_level_boundaries() {
        // Native frozen values: grants are not a fixed fraction of capacity.
        for (level, capacity, normal_grant) in [
            (1, 500, 20),
            (20, 1000, 40),
            (40, 5000, 170),
            (60, 12000, 360),
            (80, 15000, 450),
        ] {
            assert_eq!(
                *at_level(level).unwrap(),
                LevelGrains {
                    capacity,
                    normal_grant
                }
            );
        }
        assert_eq!(at_level(2).unwrap(), at_level(1).unwrap());
        assert!(at_level(0).is_err());
        assert!(at_level(81).is_err());
    }
}
