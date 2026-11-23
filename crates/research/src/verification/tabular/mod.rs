use crate::combat::{
    self, CritRule, DamageKind, DirectDamageEvaluationRequest, DirectHitSpec, DynamicStats,
    EvaluationCritMode, NegativeResistanceModel,
};
use crate::formula::Rounding;
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality};
use serde::{Deserialize, Serialize};

mod model;
pub use model::{
    DocumentedDiscrepancy, ReferenceSourceKind, TabularCaseResult, TabularTestCase,
    TabularVerificationReport, VerificationCaseStatus,
};
mod reference_cases;
pub use reference_cases::default_tabular_cases;
mod discrepancies;
pub use discrepancies::documented_discrepancies;
mod evaluation;
pub use evaluation::{evaluate_case, run_tabular_verification};

#[cfg(test)]
mod tests;
