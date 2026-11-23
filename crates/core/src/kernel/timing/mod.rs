use serde::{Deserialize, Serialize};

mod model;
pub use model::{PPM_SCALE, TimingError};
mod mode_clock;
pub use mode_clock::{
    ClockDomain, ClockProgression, ModeClockManager, ModePauseInterval, ModePauseKind,
    OwnedModePause, PauseOwner, RuntimeModeClock, merge_intervals,
};
mod hitlag;
pub use hitlag::{HitlagTracker, HitlagWindow};
mod hitlag_curve;
pub use hitlag_curve::{HitlagCurveWindow, HitlagRatePoint};
mod deadlines;
pub use deadlines::{
    ActionTimingSpec, ProjectedDeadlines, ResolvedActionDeadlines, resolve_action_deadlines,
};

#[cfg(test)]
mod curve_tests;
#[cfg(test)]
mod tests;
