use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionScenarioRequest {
    pub schema_version: u32,
    pub content_id: String,
    pub team_size: usize,
    pub active_actor: usize,
    pub horizon_us: u64,
    #[serde(default)]
    pub limits: Option<ExecutionLimits>,
    pub actors: Vec<ActorSpec>,
    #[serde(default)]
    pub targets: Vec<TargetSpec>,
    #[serde(default)]
    pub resource_pools: Vec<ResourcePoolSpec>,
    pub steps: Vec<ExecutionStepSpec>,
}
