use crate::simulation::ExactFraction;

pub(super) struct HealingParameters {
    pub(super) base_duration_us: u64,
    pub(super) per_key_duration_us: u64,
    pub(super) interval_us: u64,
    pub(super) flat: ExactFraction,
    pub(super) max_hp_ratio: ExactFraction,
    pub(super) a6_def_ratio: ExactFraction,
}

struct Healing {
    flat: u16,
    hp_ten_thousandths: u16,
}

// Skill_Treat_MaxHPPer / Skill_Treat_Value; consumed by each live Skill heal.
const SKILL_HP: [u16; 13] = [
    140, 151, 163, 176, 190, 205, 222, 239, 259, 279, 302, 326, 352,
];
const SKILL_FLAT: [u16; 13] = [
    300, 324, 350, 378, 408, 441, 476, 514, 555, 600, 648, 699, 755,
];

pub(super) fn skill_heal(coordinate: usize) -> Result<(ExactFraction, ExactFraction), String> {
    let index = coordinate
        .checked_sub(1)
        .filter(|index| *index < 13)
        .ok_or("Edgar Skill heal coordinate must be in1..=13")?;
    Ok((
        ExactFraction::new(SKILL_FLAT[index].into(), 1)?,
        ExactFraction::new(SKILL_HP[index].into(), 10000)?,
    ))
}

// Global 2026-10-04 GA_UltraSkill_Treat_Value / GA_UltraSkill_Treat_MaxHPPer.
// These are exact serialized talent values, without interpolation or extrapolation.
const HEALING: [Healing; 13] = [
    Healing {
        flat: 350,
        hp_ten_thousandths: 110,
    },
    Healing {
        flat: 378,
        hp_ten_thousandths: 118,
    },
    Healing {
        flat: 408,
        hp_ten_thousandths: 128,
    },
    Healing {
        flat: 441,
        hp_ten_thousandths: 138,
    },
    Healing {
        flat: 476,
        hp_ten_thousandths: 149,
    },
    Healing {
        flat: 514,
        hp_ten_thousandths: 161,
    },
    Healing {
        flat: 555,
        hp_ten_thousandths: 174,
    },
    Healing {
        flat: 600,
        hp_ten_thousandths: 188,
    },
    Healing {
        flat: 648,
        hp_ten_thousandths: 203,
    },
    Healing {
        flat: 700,
        hp_ten_thousandths: 219,
    },
    Healing {
        flat: 756,
        hp_ten_thousandths: 237,
    },
    Healing {
        flat: 816,
        hp_ten_thousandths: 256,
    },
    Healing {
        flat: 881,
        hp_ten_thousandths: 276,
    },
];

pub(super) fn healing(coordinate: u32) -> Result<HealingParameters, String> {
    let row = coordinate
        .checked_sub(1)
        .and_then(|index| HEALING.get(index as usize))
        .ok_or("Edgar healing talent coordinate must be in 1..=13")?;
    Ok(HealingParameters {
        // GA_UltraSkill_Duration, GA_UltraSkill_PerKeyAddDuration,
        // GA_UltraSkill_Treat_Period and Buff_Edgar_Star6_DefUP are constant on 1..=13.
        base_duration_us: 10_000_000,
        per_key_duration_us: 1_000_000,
        interval_us: 1_000_000,
        flat: ExactFraction::new(u128::from(row.flat), 1)?,
        max_hp_ratio: ExactFraction::new(u128::from(row.hp_ten_thousandths), 10_000)?,
        a6_def_ratio: ExactFraction::new(3, 20)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healing_parameters_preserve_native_units_and_supported_levels() {
        for (level, flat, hp_num) in [(1, 350, 110), (8, 600, 188), (9, 648, 203), (13, 881, 276)] {
            let params = healing(level).unwrap();
            assert_eq!(params.flat, ExactFraction::new(flat, 1).unwrap());
            assert_eq!(
                params.max_hp_ratio,
                ExactFraction::new(hp_num, 10_000).unwrap()
            );
            assert_eq!(params.a6_def_ratio, ExactFraction::new(3, 20).unwrap());
            assert_eq!(params.base_duration_us, 10_000_000);
            assert_eq!(params.per_key_duration_us, 1_000_000);
            assert_eq!(params.interval_us, 1_000_000);
        }
        for level in [0, 14, u32::MAX] {
            assert!(healing(level).is_err());
        }
    }
}
