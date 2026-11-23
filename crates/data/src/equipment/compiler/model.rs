use super::*;
pub const TABLES: [&str; 0] = [];
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub content_id: String,
    pub character_id: String,
    pub weapon_id: String,
    pub weapon_tier: u32,
    pub set_id: String,
    pub set_count: u32,
    #[serde(default)]
    pub additive_attributes: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_level: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_breakthrough: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_level: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_breakthrough: Option<u32>,
}
