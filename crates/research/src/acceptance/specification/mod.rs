use crate::composition::{self, CompositionReport};
use crate::formula::Ratio;
use crate::published_teams::{PublishedVariant, parse_published_variants_csv};
use crate::simulation::GameTargetConfig;
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality, sha256};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

mod scenarios;
pub use scenarios::AcceptanceScenarioKind;
mod conditions;
pub use conditions::AcceptanceConditionsSummary;
mod csv;
pub use csv::{
    ArchetypeSummaryRow, PublishedVariantRow, generate_archetypes_csv,
    generate_published_variants_csv,
};
mod manifest;
pub use manifest::{AcceptanceRunState, generate_run_manifest};
mod deliverables;
pub use deliverables::{AcceptanceDeliverables, ScenarioInfo, compile_acceptance_deliverables};

#[cfg(test)]
mod tests;
