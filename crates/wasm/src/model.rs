use ntesim_contracts::sha256;
use ntesim_research::tabular_verification::TabularTestCase;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_VARIANTS_CSV: &str = ntesim_data::research_inputs::VARIANTS_CSV;
pub const DEFAULT_CONDITIONS_JSON: &str = ntesim_data::research_inputs::CONDITIONS_JSON;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WasmRotationRequest {
    pub fixture: Value,
    pub rotation: String,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WasmGameBatchRequest {
    pub scenario: ntesim_content::simulation::GameScenarioRequest,
    pub start_seed: u64,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WasmGameProgramBatchRequest {
    pub program: ntesim_content::simulation::GameScenarioProgramRequest,
    pub start_seed: u64,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WasmBatchRequest {
    pub scenario: Value,
    pub request: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WasmTabularRequest {
    #[serde(default)]
    pub cases: Option<Vec<TabularTestCase>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WasmCompositionRequest {
    #[serde(default)]
    pub variants_csv: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WasmInvokeEnvelope {
    pub action: String,
    pub payload: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WasmEnvironmentMetadata {
    pub adapter: String,
    pub target_arch: String,
    pub runtime_model: String,
    pub content_cutoff: String,
    pub parity_verified: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WasmInvokeSuccessResponse {
    pub status: String,
    pub action: String,
    pub environment: WasmEnvironmentMetadata,
    pub data: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WasmErrorResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    pub error: String,
}

pub const WASM_BUILD_STARTED_AT: &str = env!("NTESIM_WASM_BUILD_TIME");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WasmVersionInfo {
    pub version: String,
    pub engine_source_sha256: String,
    pub content_id: String,
    pub content_cutoff: String,
    pub status: String,
    pub game_dps_computed: bool,
    pub adapter: String,
    pub target_arch: String,
    pub coverage_sha256: String,
    pub variants_sha256: String,
    pub build_started_at: String,
}

pub fn version_info() -> WasmVersionInfo {
    WasmVersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        engine_source_sha256: ntesim_contracts::ENGINE_SOURCE_SHA256.to_string(),
        content_id: "ntesim-current".to_string(),
        content_cutoff: "2026-10-04".to_string(),
        status: "INCOMPLETE".to_string(),
        game_dps_computed: false,
        adapter: "ntesim-wasm".to_string(),
        target_arch: if cfg!(target_arch = "wasm32") {
            "wasm32".to_string()
        } else {
            "native".to_string()
        },
        coverage_sha256: sha256(
            ntesim_data::character::released_inventory()
                .to_string()
                .as_bytes(),
        ),
        variants_sha256: sha256(DEFAULT_VARIANTS_CSV.as_bytes()),
        build_started_at: WASM_BUILD_STARTED_AT.to_string(),
    }
}

pub fn default_environment_metadata() -> WasmEnvironmentMetadata {
    WasmEnvironmentMetadata {
        adapter: "ntesim-wasm".to_string(),
        target_arch: if cfg!(target_arch = "wasm32") {
            "wasm32".to_string()
        } else {
            "native".to_string()
        },
        runtime_model: "browser-worker-or-node".to_string(),
        content_cutoff: "2026-10-04".to_string(),
        // Parity is established by an external receipt bound to the built bytes;
        // executing this adapter cannot certify its own artifact.
        parity_verified: false,
    }
}
