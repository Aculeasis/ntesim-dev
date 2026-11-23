//! Partial fixed-timeline previews of the 45 frozen published variants.
//! Manual profiles and selected effects do not establish native dispatch, legal
//! builds, complete adapters or game DPS. Quality remains incomplete throughout.

use crate::{
    execution::ExecutionLimits,
    formula::Ratio,
    simulation::{
        GameActionCommand, GameActorStaticStats, GameScenarioReport, GameScenarioRequest,
        run_game_scenario,
    },
};
pub use ntesim_content::registry::{PreviewCharacterProfile, get_character_profile};
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality};
use serde::{Deserialize, Serialize};

mod quality;
pub use quality::{ScenarioReadiness, preview_readiness};
mod csv;
pub use csv::{PublishedVariant, parse_published_variants_csv};
mod scenario;
pub use scenario::{
    ActorOverride, compile_variant_scenario, compile_variant_scenario_with_actor_overrides,
    compile_variant_scenario_with_overrides,
};
mod batch;
pub use batch::{
    VariantBatchReport, VariantExecutionSummary, run_published_batch,
    run_published_batch_with_actor_overrides, run_published_batch_with_overrides,
};

pub mod built_variants;
pub mod composition;
pub mod profile_audit;
#[cfg(test)]
mod tests;
