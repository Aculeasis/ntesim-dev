use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActorSpec {
    pub actor_index: usize,
    pub character_id: String,
    pub level: u32,
    pub current_hp: u64,
    pub max_hp: u64,
    pub defense: u64,
    #[serde(default)]
    pub initial_shields: Vec<ShieldSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ActorState {
    pub actor_index: usize,
    pub character_id: String,
    pub level: u32,
    pub current_hp: u64,
    pub max_hp: u64,
    pub defense: u64,
    pub is_alive: bool,
    pub locked_until_us: u64,
    pub cooldown_until_us: u64,
    pub total_damage_taken: u64,
    pub total_absorbed: u64,
    pub total_hp_lost: u64,
    pub total_healed: u64,
    pub total_overheal: u64,
    pub active_shields: Vec<ShieldState>,
}
