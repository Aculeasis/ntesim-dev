use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPolicy {
    KnownStaticAdditiveV1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActorBuild {
    pub slot: usize,
    pub build: equipment::Request,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purchased_skill_levels: Option<BTreeMap<String, u32>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionProjectionPolicy {
    NativeAtkCoordinateV1,
    NativeMintSkillLevelV1,
    NativeSkillLevelV1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActionProjection {
    pub policy: ActionProjectionPolicy,
    /// Explicit native coordinate, exact stat scaling and packet-delivery hypothesis.
    pub hypothesis: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearProjectionPolicy {
    NativeMindRoyaleCurveV1,
    NativeMindRoyaleFirefliesSoloV1,
    NativeWeaponCurveV1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GearProjection {
    pub policy: GearProjectionPolicy,
    /// Explicit T-coordinate, integer-energy, seconds and existing Mint-trigger hypothesis.
    pub hypothesis: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub projection_policy: ProjectionPolicy,
    /// Explicit hypothesis that these partial static attributes apply to all hits.
    pub hypothesis: String,
    pub scenario: GameScenarioRequest,
    pub builds: Vec<ActorBuild>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_projection: Option<ActionProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_projection: Option<GearProjection>,
}

#[derive(Debug)]
pub struct Compiled {
    pub scenario: GameScenarioRequest,
    pub receipt: Value,
}
