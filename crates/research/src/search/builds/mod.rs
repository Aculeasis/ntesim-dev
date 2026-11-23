//! Fixed-preview baselines; legal build search remains pending equipment-compiler integration.
use crate::execution::ExecutionLimits;
use crate::formula::Ratio;
use crate::published_teams::{
    ScenarioReadiness, compile_variant_scenario_with_overrides, parse_published_variants_csv,
    preview_readiness,
};
use crate::simulation::{GameActorStaticStats, GameScenarioReport, run_game_scenario};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod roles;
pub use roles::{CharacterRole, get_character_element, get_character_role};
mod model;
pub use model::{
    ArchetypeRefinement, BuildSearchResult, CharacterBuildProfile, SubstatLine,
    VariantComparisonResult,
};
mod optimizer;
pub use optimizer::{optimize_character_build, search_and_optimize_all_characters};
mod projection;
pub use projection::builds_to_static_stats;
mod refinement;
pub use refinement::run_published_search;

#[cfg(test)]
mod tests;
