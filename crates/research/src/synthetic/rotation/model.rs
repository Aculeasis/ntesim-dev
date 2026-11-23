use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Fixture {
    pub(crate) schema_version: u32,
    pub(crate) content_id: String,
    pub(crate) data_status: String,
    pub(crate) seed: u64,
    #[serde(default)]
    pub(crate) rng_streams: BTreeMap<String, u64>,
    pub(crate) duration_us: u64,
    pub(crate) mode_clock: ClockId,
    pub(crate) clocks: Vec<ClockInput>,
    pub(crate) targets: BTreeMap<String, TargetInput>,
    pub(crate) resources: BTreeMap<String, PoolInput>,
    #[serde(default)]
    pub(crate) actors: BTreeMap<String, ActorInput>,
    pub(crate) actions: BTreeMap<String, Vec<Effect>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Metadata {
    pub language_version: String,
    pub fixture_sha256: String,
    pub source_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_override: Option<u64>,
    pub origins: Vec<Origin>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    #[serde(flatten)]
    pub simulation: synthetic::Result,
    pub rotation: Metadata,
}
