//! Only the shield scaling and periodic Skill clocks consumed by Adler.
//! Frozen Adler_SkillShield_base/cofficient and SkillDam_Period/Time inputs.
use crate::formula::Ratio;

pub(super) const SKILL_TICK_INTERVAL_US: u64 = 1_000_000;
pub(super) const SKILL_TRAIN_DURATION_US: u64 = 10_000_000;

pub(super) struct BlessingScaling {
    pub(super) flat: u64,
    pub(super) defense_coefficient: Ratio,
}

impl BlessingScaling {
    const fn new(flat: u64, numerator: u64, denominator: u64) -> Self {
        Self {
            flat,
            defense_coefficient: Ratio {
                numerator,
                denominator,
            },
        }
    }
}

const BLESSING_BY_TALENT: [BlessingScaling; 13] = [
    BlessingScaling::new(1000, 4, 5),
    BlessingScaling::new(1080, 43, 50),
    BlessingScaling::new(1166, 93, 100),
    BlessingScaling::new(1260, 1, 1),
    BlessingScaling::new(1361, 27, 25),
    BlessingScaling::new(1470, 117, 100),
    BlessingScaling::new(1587, 63, 50),
    BlessingScaling::new(1714, 137, 100),
    BlessingScaling::new(1851, 37, 25),
    BlessingScaling::new(2000, 159, 100),
    BlessingScaling::new(2160, 43, 25),
    BlessingScaling::new(2332, 93, 50),
    BlessingScaling::new(2519, 201, 100),
];

pub(super) fn blessing(level: u32) -> Result<&'static BlessingScaling, String> {
    level
        .checked_sub(1)
        .and_then(|index| BLESSING_BY_TALENT.get(index as usize))
        .ok_or_else(|| "Adler Blessing requires effective Skill level 1..=13".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::ExactFraction as F;

    #[test]
    fn shield_scaling_preserves_independent_flat_and_defense_terms_at_talent_boundaries() {
        // DEF1000, no outgoing shield bonus: 1000+.8*1000=1800;
        // 1714+1.37*1000=3084; 2519+2.01*1000=4529.
        for (level, flat, numerator, denominator, expected) in [
            (1, 1000, 4, 5, 1800),
            (8, 1714, 137, 100, 3084),
            (13, 2519, 201, 100, 4529),
        ] {
            let input = blessing(level).unwrap();
            assert_eq!(input.flat, flat);
            assert_eq!(
                input.defense_coefficient,
                Ratio {
                    numerator,
                    denominator
                }
            );
            let amount = F::new(input.flat.into(), 1)
                .unwrap()
                .add(
                    F::from_ratio(input.defense_coefficient)
                        .unwrap()
                        .mul(F::new(1000, 1).unwrap())
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(amount, F::new(expected, 1).unwrap());
        }
        assert!(blessing(0).is_err());
        assert!(blessing(14).is_err());
    }
}
