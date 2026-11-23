//! Native live-program compilation. Studio's scheduled export remains separate
//! until the Phase 4 consumer migration.
use super::{compiler::SourceRequest, inspection::PartyInspection, validation::Diagnostic};
use crate::simulation::{GameScenarioRequest, PreparedSourceScenario};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledSourceProgram {
    pub schema_version: u32,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub input: SourceRequest,
    /// Canonical initial scenario; combat commands are generated per trial.
    pub request: GameScenarioRequest,
    pub trials: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceCompileResponse {
    pub compiled: Option<CompiledSourceProgram>,
    pub party: Option<PartyInspection>,
    /// Effective configuration, retained even when rotation preflight fails.
    pub options: Option<super::options::CompiledOptions>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn compile_response(request: &SourceRequest) -> SourceCompileResponse {
    prepare_response(request).0
}

/// Produce the durable artifact and the validated program that generated it.
/// A local executor can start from the latter without an import round trip.
pub fn prepare_response(
    request: &SourceRequest,
) -> (SourceCompileResponse, Option<PreparedSourceScenario>) {
    let (script, config) = match super::inspection::prepare(&request.source) {
        Ok(prepared) => prepared,
        Err(diagnostics) => {
            return (
                SourceCompileResponse {
                    compiled: None,
                    party: None,
                    options: None,
                    diagnostics,
                },
                None,
            );
        }
    };
    let party = Some(PartyInspection::from_configuration(&config));
    let options = Some(config.options.clone());
    match PreparedSourceScenario::from_configuration(
        &request.source,
        config,
        request.limits,
        &Default::default(),
        &script.options[0].loc,
    ) {
        Ok(program) => (
            SourceCompileResponse {
                compiled: Some(program.export()),
                party,
                options,
                diagnostics: Vec::new(),
            },
            Some(program),
        ),
        Err(diagnostic) => (
            SourceCompileResponse {
                compiled: None,
                party,
                options,
                diagnostics: vec![diagnostic],
            },
            None,
        ),
    }
}

pub(crate) fn validate_rotation(
    config: &super::configuration::CompiledConfiguration,
) -> Result<(), Diagnostic> {
    super::source_validation::validate(&config.rotation, &config.constants, &config.targets)
}
