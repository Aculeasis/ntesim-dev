use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Required on current native/source/report transports; obsolete clock data is
/// rejected by serde rather than reconstructed from action names or timestamps.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockContract {
    #[default]
    #[serde(rename = "relative-combat-v1")]
    RelativeCombatV1,
}

impl ClockContract {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RelativeCombatV1 => "relative-combat-v1",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerPolicy {
    #[default]
    #[serde(rename = "paused-world-v1")]
    PausedWorldV1,
}

impl TimerPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PausedWorldV1 => "paused-world-v1",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Confirmed,
    BoundedHypothesis,
    UnboundedHypothesis,
    Incomplete,
    Synthetic,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TimeBasisPair<T> {
    pub relative: Option<T>,
    pub absolute: Option<T>,
}

impl<T: Eq> Eq for TimeBasisPair<T> {}

impl<T> TimeBasisPair<T> {
    pub const fn new(relative: Option<T>, absolute: Option<T>) -> Self {
        Self { relative, absolute }
    }

    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> TimeBasisPair<U> {
        TimeBasisPair {
            relative: self.relative.map(&mut f),
            absolute: self.absolute.map(f),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assumption {
    pub value: f64,
    pub unit: String,
    pub justification: String,
    pub interval: Option<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub schema_version: u32,
    pub content_id: String,
    pub duration_us: u64,
    pub seed: u64,
    pub dependencies: Vec<String>,
    pub assumptions: BTreeMap<String, Assumption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub confirmed: bool,
    pub implemented: bool,
    pub gap: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub valid: bool,
    pub quality: Quality,
    pub diagnostics: Vec<Diagnostic>,
}
