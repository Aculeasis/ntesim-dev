//! Explicit research workflows, trials, search and profiling; never strict readiness.
pub mod acceptance;
pub mod profiling;
pub mod search;
pub mod synthetic;
pub mod teams;
pub mod trials;
pub mod verification;
pub use acceptance::{finalists, runner as acceptance_runner, specification};
use ntesim_content::{build, simulation};
use ntesim_core::{action, combat, execution, formula, rng, scheduler, state};
use ntesim_data::{character, damage_values, equipment, weapon};
pub use profiling as pilot;
pub use search::{builds as build_search, rotation as rotation_search};
pub use specification as acceptance_spec;
pub use synthetic::rotation;
pub use teams as published_teams;
pub use teams::{built_variants, composition, profile_audit};
pub use trials::{batch, compare, cycle, game_batch};
pub use verification::tabular as tabular_verification;

pub use verification::timing as timing_verification;
