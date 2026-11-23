use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AcceptanceScenarioKind {
    #[serde(rename = "dps_1_real120")]
    SingleTargetReal120,
    #[serde(rename = "dps_4_real120")]
    FourTargetReal120,
    #[serde(rename = "dps_1_mode120")]
    SingleTargetMode120,
    #[serde(rename = "dps_4_mode120")]
    FourTargetMode120,
    #[serde(rename = "dps_no_break")]
    NoBreakReal120,
}

impl AcceptanceScenarioKind {
    pub const ALL: [Self; 5] = [
        Self::SingleTargetReal120,
        Self::FourTargetReal120,
        Self::SingleTargetMode120,
        Self::FourTargetMode120,
        Self::NoBreakReal120,
    ];

    pub fn code(&self) -> &'static str {
        match self {
            Self::SingleTargetReal120 => "dps_1_real120",
            Self::FourTargetReal120 => "dps_4_real120",
            Self::SingleTargetMode120 => "dps_1_mode120",
            Self::FourTargetMode120 => "dps_4_mode120",
            Self::NoBreakReal120 => "dps_no_break",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SingleTargetReal120 => "1 target, 120s real elapsed combat time, break modeled",
            Self::FourTargetReal120 => {
                "4 in-range dummies, 120s real elapsed combat time, break modeled"
            }
            Self::SingleTargetMode120 => {
                "1 target, 120s game-mode timer clock, including mode pauses"
            }
            Self::FourTargetMode120 => {
                "4 in-range dummies, 120s game-mode timer clock, including mode pauses"
            }
            Self::NoBreakReal120 => "1 target, 120s real elapsed combat time, without break",
        }
    }

    pub fn target_count(&self) -> usize {
        match self {
            Self::SingleTargetReal120 | Self::SingleTargetMode120 | Self::NoBreakReal120 => 1,
            Self::FourTargetReal120 | Self::FourTargetMode120 => 4,
        }
    }

    pub fn horizon_us(&self) -> u64 {
        120_000_000
    }

    pub fn is_mode_clock(&self) -> bool {
        matches!(self, Self::SingleTargetMode120 | Self::FourTargetMode120)
    }

    pub fn is_break_enabled(&self) -> bool {
        !matches!(self, Self::NoBreakReal120)
    }

    pub fn create_targets(&self) -> Vec<GameTargetConfig> {
        let count = self.target_count();

        (0..count)
            .map(|idx| GameTargetConfig {
                original_atk: None,
                base_defense: None,
                control_immune: false,
                target_index: idx,
                is_boss: false,
                level: 80,
                hp_max: 10_000_000,
                hp_current: 10_000_000,
                defense: 1080,
                resistance: Ratio {
                    numerator: 1,
                    denominator: 5,
                },
                elemental_resistances: Default::default(),
                airborne_until_us: 0,
                break_profile: None,
            })
            .collect()
    }
}
