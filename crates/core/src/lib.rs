//! Deterministic shared engine services. No content, source-specific presets or ambient time.
pub mod actors;
pub mod buffs;
pub mod combat;
pub mod execution;
pub mod fast_hash;
pub use fast_hash::{FastHashMap, FastHashSet};
pub mod kernel;
#[cfg(feature = "performance-diagnostics")]
pub mod performance;
pub use actors::lifecycle;
pub use buffs::dispatch;
pub use combat::reaction;
pub use kernel::{action, formula, rng, scheduler, state, timing};
