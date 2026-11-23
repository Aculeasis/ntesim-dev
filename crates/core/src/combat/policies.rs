use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageKind {
    Elemental,
    Mental,
    True,
}

impl DamageKind {
    pub fn uses_defense(self) -> bool {
        matches!(self, Self::Elemental)
    }
    pub fn uses_standard_modifiers(self) -> bool {
        !matches!(self, Self::True)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CritRule {
    Normal,
    None,
    Fixed50,
    Fixed100,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NegativeResistanceModel {
    /// 1.0 - R / (1.0 - R) = (1.0 - 2R) / (1.0 - R).
    /// Selected research hypothesis; the native negative-resistance rule is unresolved.
    #[default]
    StandardRational,
    /// 1.0 - R / 1.10.
    /// Documented conflict from Drive Calculator; see docs/DRIVE_CALCULATOR_RESEARCH.md.
    /// Source: https://github.com/hxwd94666/NTE-Drive-Calculator/tree/765d78a6631718db15bfbecfa615dd5c99b3547a
    LinearScaled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotPolicy {
    AtCast,
    AtApplication,
    Dynamic,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackingPolicy {
    ReplaceBySource,
    ReplaceByTarget,
    AddStacks { max_stacks: u32 },
}
