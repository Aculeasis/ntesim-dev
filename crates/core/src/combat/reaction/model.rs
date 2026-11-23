use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum BreakState {
    Unbroken,
    Broken { broken_at_us: u64, duration_us: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TargetProfile {
    pub id: String,
    pub position: Point3D,
    pub defense: u64,
    pub resistances: BTreeMap<String, Ratio>,
    pub break_cap: u64,
    pub current_break_gauge: u64,
    pub break_state: BreakState,
    pub auras: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ActorCombatProfile {
    pub actor_index: usize,
    pub character_id: String,
    pub level: u32,
    pub base_break: u64,
    pub break_intensity: Ratio,
    pub energy_pool: Pool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactionKind {
    Blossom,
    Scorch,
    Nova,
    Remora,
}

impl ReactionKind {
    pub fn base_damage(&self) -> u64 {
        match self {
            Self::Blossom => BLOSSOM_BASE_DAMAGE,
            Self::Scorch => SCORCH_BASE_DAMAGE,
            Self::Nova => NOVA_BASE_DAMAGE,
            Self::Remora => 10_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ScorchInstance {
    pub source_actor: usize,
    pub stacks: u32,
    pub next_tick_us: u64,
    pub expires_at_us: u64,
}
