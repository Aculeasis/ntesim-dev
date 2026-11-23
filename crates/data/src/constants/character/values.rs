//! Released character primary stats and energy capacity only.
//! Source: frozen Global DT_PlayerPackData and DT_PlayerModifyPackData.
//! Growth and terminal bonuses match all 5,934 retained source stat values;
//! exact integer rounding happens separately before the terms are added.
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) struct Primary {
    pub(super) hp: u32,
    pub(super) atk: u32,
    pub(super) def: u32,
}
impl Primary {
    const fn new(hp: u32, atk: u32, def: u32) -> Self {
        Self { hp, atk, def }
    }
    fn scaled(
        self,
        hp_def_numerator: u32,
        atk_numerator: u32,
        denominator: u32,
    ) -> Result<Self, String> {
        let round = |base: u32, numerator: u32| {
            let n = u64::from(base) * u64::from(numerator);
            u32::try_from((n + u64::from(denominator) / 2) / u64::from(denominator))
                .map_err(|_| "character stat growth overflow".to_owned())
        };
        Ok(Self::new(
            round(self.hp, hp_def_numerator)?,
            round(self.atk, atk_numerator)?,
            round(self.def, hp_def_numerator)?,
        ))
    }
    fn add(self, rhs: Self) -> Result<Self, String> {
        let add = |a: u32, b: u32| {
            a.checked_add(b)
                .ok_or_else(|| "character stat total overflow".to_owned())
        };
        Ok(Self::new(
            add(self.hp, rhs.hp)?,
            add(self.atk, rhs.atk)?,
            add(self.def, rhs.def)?,
        ))
    }
}

pub(super) struct Base {
    pub(super) primary: Primary,
    pub(super) energy: u32,
}
impl Base {
    pub(super) const fn new(hp: u32, atk: u32, def: u32, energy: u32) -> Self {
        Self {
            primary: Primary::new(hp, atk, def),
            energy,
        }
    }
    pub(super) fn compile(&self, level: u32, stage: u32) -> Result<Compiled, String> {
        let max_level = super::progression::max_level(stage)?;
        if level == 0 || level > max_level {
            return Err("character level exceeds selected breakthrough limit".into());
        }
        // Growth HP/DEF: 8% per level after level one. Growth ATK: 5%.
        let growth = self.primary.scaled(8 * (level - 1), 5 * (level - 1), 100)?;
        // Terminal bonus only: HP/DEF 80%, ATK 50% per selected stage.
        let terminal = self.primary.scaled(8 * stage, 5 * stage, 10)?;
        let total = self.primary.add(growth)?.add(terminal)?;
        Ok(Compiled {
            growth,
            terminal,
            total,
            max_level,
        })
    }
}
pub(super) struct Compiled {
    pub(super) growth: Primary,
    pub(super) terminal: Primary,
    pub(super) total: Primary,
    pub(super) max_level: u32,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_released_level_eighty_endpoints_match_independent_published_values() {
        // Historical published CN endpoints, distinct from the Global growth exports.
        for (id, expected) in [
            ("1003", Primary::new(16483, 620, 909)),
            ("1004", Primary::new(15998, 636, 909)),
            ("1008", Primary::new(13332, 557, 667)),
            ("1010", Primary::new(15998, 636, 909)),
            ("1019", Primary::new(12120, 596, 727)),
            ("1020", Primary::new(13696, 493, 848)),
            ("1021", Primary::new(16362, 421, 824)),
            ("1023", Primary::new(16604, 644, 800)),
            ("1025", Primary::new(15998, 644, 884)),
            ("1033", Primary::new(14302, 437, 994)),
            ("1036", Primary::new(15514, 660, 909)),
            ("1039", Primary::new(17089, 596, 909)),
            ("1042", Primary::new(15514, 668, 884)),
            ("1046", Primary::new(15150, 676, 909)),
            ("1052", Primary::new(16120, 652, 848)),
            ("1054", Primary::new(16180, 644, 873)),
            ("1055", Primary::new(15635, 652, 909)),
            ("1070", Primary::new(11878, 604, 727)),
            ("1071", Primary::new(15756, 636, 921)),
            ("1072", Primary::new(15514, 652, 921)),
            ("1073", Primary::new(15514, 660, 909)),
            ("1075", Primary::new(17210, 596, 909)),
            ("1076", Primary::new(16120, 636, 897)),
        ] {
            assert_eq!(
                super::super::definition(&format!("character:{id}"))
                    .unwrap()
                    .base
                    .compile(80, 6)
                    .unwrap()
                    .total,
                expected,
                "{id}"
            );
        }
        assert!(super::super::definition("character:1051").is_err());
    }
    #[test]
    fn independent_half_terms_round_before_accumulation_and_caps_gate_growth() {
        let base = Base::new(100, 81, 75, 100);
        let result = base.compile(11, 1).unwrap();
        // Growth40.5 ->41, terminal40.5 ->41, then base81: 163, not162.
        assert_eq!(result.growth.atk, 41);
        assert_eq!(result.terminal.atk, 41);
        assert_eq!(result.total.atk, 163);
        assert_eq!(base.compile(20, 0).unwrap().total.atk, 158);
        assert!(base.compile(21, 0).is_err());
        assert_eq!(base.compile(21, 1).unwrap().total.atk, 203);
        assert!(base.compile(0, 0).is_err());
        assert!(base.compile(81, 6).is_err());
        assert!(base.compile(1, 7).is_err());
    }
}
