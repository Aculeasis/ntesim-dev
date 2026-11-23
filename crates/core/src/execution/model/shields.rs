use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShieldSpec {
    pub id: String,
    pub source: String,
    pub amount_hp: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ShieldState {
    pub id: String,
    pub source: String,
    pub remaining_hp: u64,
    pub max_hp: u64,
}
