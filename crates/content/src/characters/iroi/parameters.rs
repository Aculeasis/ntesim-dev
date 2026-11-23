use crate::simulation::ExactFraction;

#[allow(dead_code)]
pub(crate) enum HealKind {
    Skill,
    Burst,
    BurstPeriodic,
    Qte,
    A2,
}

struct Healing {
    skill_milli: u16,
    burst_milli: u16,
    burst_periodic_milli: u16,
    qte_milli: u16,
}

// Global 2026-10-04 Oneiroi_Skill_Treat / Oneiroi_UltraSkill_Treat /
// Oneiroi_UltraSkill_TreatPeriod / Oneiroi_QTE_Treat,
// exact authored coordinates 1..=13. Keep native numeric units; settlement owns scaling.
const HEALING: [Healing; 13] = [
    Healing {
        skill_milli: 943,
        burst_milli: 1413,
        burst_periodic_milli: 314,
        qte_milli: 600,
    },
    Healing {
        skill_milli: 1018,
        burst_milli: 1526,
        burst_periodic_milli: 339,
        qte_milli: 648,
    },
    Healing {
        skill_milli: 1100,
        burst_milli: 1648,
        burst_periodic_milli: 366,
        qte_milli: 700,
    },
    Healing {
        skill_milli: 1188,
        burst_milli: 1780,
        burst_periodic_milli: 396,
        qte_milli: 756,
    },
    Healing {
        skill_milli: 1283,
        burst_milli: 1923,
        burst_periodic_milli: 427,
        qte_milli: 817,
    },
    Healing {
        skill_milli: 1386,
        burst_milli: 2076,
        burst_periodic_milli: 462,
        qte_milli: 882,
    },
    Healing {
        skill_milli: 1496,
        burst_milli: 2243,
        burst_periodic_milli: 499,
        qte_milli: 953,
    },
    Healing {
        skill_milli: 1616,
        burst_milli: 2422,
        burst_periodic_milli: 538,
        qte_milli: 1029,
    },
    Healing {
        skill_milli: 1745,
        burst_milli: 2616,
        burst_periodic_milli: 581,
        qte_milli: 1111,
    },
    Healing {
        skill_milli: 1885,
        burst_milli: 2825,
        burst_periodic_milli: 628,
        qte_milli: 1200,
    },
    Healing {
        skill_milli: 2036,
        burst_milli: 3051,
        burst_periodic_milli: 678,
        qte_milli: 1296,
    },
    Healing {
        skill_milli: 2199,
        burst_milli: 3295,
        burst_periodic_milli: 732,
        qte_milli: 1400,
    },
    Healing {
        skill_milli: 2375,
        burst_milli: 3559,
        burst_periodic_milli: 791,
        qte_milli: 1512,
    },
];

pub(crate) fn heal_ratio(kind: HealKind, coordinate: u32) -> Result<ExactFraction, String> {
    if matches!(kind, HealKind::A2) {
        return ExactFraction::new(1200, 1000);
    }
    let row = coordinate
        .checked_sub(1)
        .and_then(|index| HEALING.get(index as usize))
        .ok_or("Iroi healing talent coordinate must be in 1..=13")?;
    let milli = match kind {
        HealKind::Skill => row.skill_milli,
        HealKind::Burst => row.burst_milli,
        HealKind::BurstPeriodic => row.burst_periodic_milli,
        HealKind::Qte => row.qte_milli,
        HealKind::A2 => unreachable!(),
    };
    ExactFraction::new(u128::from(milli), 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healing_parameters_preserve_native_units_and_supported_levels() {
        for (level, skill_num, burst_num, period_num, qte_num) in [
            (1, 943, 1413, 314, 600),
            (8, 1616, 2422, 538, 1029),
            (9, 1745, 2616, 581, 1111),
            (13, 2375, 3559, 791, 1512),
        ] {
            assert_eq!(
                heal_ratio(HealKind::Skill, level).unwrap(),
                ExactFraction::new(skill_num, 1000).unwrap()
            );
            assert_eq!(
                heal_ratio(HealKind::Burst, level).unwrap(),
                ExactFraction::new(burst_num, 1000).unwrap()
            );
            assert_eq!(
                heal_ratio(HealKind::BurstPeriodic, level).unwrap(),
                ExactFraction::new(period_num, 1000).unwrap()
            );
            assert_eq!(
                heal_ratio(HealKind::Qte, level).unwrap(),
                ExactFraction::new(qte_num, 1000).unwrap()
            );
            assert_eq!(
                heal_ratio(HealKind::A2, level).unwrap(),
                ExactFraction::new(1200, 1000).unwrap()
            );
        }
        for level in [0, 14, u32::MAX] {
            assert!(heal_ratio(HealKind::Skill, level).is_err());
            assert!(heal_ratio(HealKind::Burst, level).is_err());
            assert!(heal_ratio(HealKind::BurstPeriodic, level).is_err());
            assert!(heal_ratio(HealKind::Qte, level).is_err());
        }
    }
}
