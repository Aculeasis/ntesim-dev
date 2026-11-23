use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionOrigin {
    pub actor: String,
    pub action: String,
    pub effect_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Trace {
    pub event_id: u64,
    pub cause: Option<u64>,
    pub command_index: usize,
    pub real_time_us: u64,
    pub local_time_us: u64,
    pub clock: ClockId,
    pub phase: String,
    pub changes: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_origin: Option<ActionOrigin>,
}

#[derive(Debug, Serialize)]
pub struct Result {
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub content_id: String,
    pub input_sha256: String,
    pub seed: u64,
    pub rng_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rng: Option<Rng>,
    pub quality: Quality,
    pub success: bool,
    pub requested_duration_us: u64,
    pub real_time_us: u64,
    pub mode_time_us: u64,
    pub pending_events: usize,
    pub diagnostics: Vec<Diagnostic>,
    pub targets: BTreeMap<String, VitalState>,
    pub resources: BTreeMap<String, Pool>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub actors: BTreeMap<String, Gate>,
    pub trace: Vec<Trace>,
}

#[derive(Serialize)]
pub(crate) struct Observation<'a> {
    pub(crate) version: &'static str,
    pub(crate) label: &'a str,
    pub(crate) real_time_us: u64,
    pub(crate) clocks: BTreeMap<ClockId, ClockReading>,
    pub(crate) pending_events: usize,
    pub(crate) targets: &'a BTreeMap<String, VitalState>,
    pub(crate) resources: &'a BTreeMap<String, Pool>,
    pub(crate) actors: &'a BTreeMap<String, Gate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rng: Option<&'a Rng>,
}
