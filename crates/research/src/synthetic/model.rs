use super::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClockInput {
    pub id: ClockId,
    pub rate_ppm: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetInput {
    pub hp: u64,
    pub max_hp: u64,
    pub shield_order: ShieldOrder,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoolInput {
    pub current: u64,
    pub capacity: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorActionInput {
    pub rule: Rule,
    pub ready_at_us: u64,
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorInput {
    pub clock: ClockId,
    pub initial_time_us: u64,
    pub locked_until_us: u64,
    pub actions: BTreeMap<String, ActorActionInput>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeightedDamage {
    pub weight: u64,
    pub amount: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    ObserveState {
        label: String,
    },
    InvokeAction {
        actor: String,
        action: String,
    },
    Damage {
        target: String,
        amount: u64,
        hit_delay_us: u64,
    },
    FormulaDamage {
        target: String,
        formula: Box<Formula>,
        stats: BTreeMap<String, Ratio>,
        hit_delay_us: u64,
    },
    RandomDamage {
        target: String,
        stream: String,
        outcomes: Vec<WeightedDamage>,
        hit_delay_us: u64,
    },
    Heal {
        target: String,
        amount: u64,
    },
    Shield {
        target: String,
        id: String,
        source: String,
        amount: u64,
    },
    Gain {
        pool: String,
        amount: u64,
    },
    Spend {
        pool: String,
        amount: u64,
    },
    SetRate {
        clock: ClockId,
        rate_ppm: u32,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub clock: ClockId,
    pub delay_us: u64,
    pub effect: Effect,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub schema_version: u32,
    pub content_id: String,
    pub data_status: String,
    pub seed: u64,
    #[serde(default)]
    pub rng_streams: BTreeMap<String, u64>,
    pub duration_us: u64,
    pub mode_clock: ClockId,
    pub clocks: Vec<ClockInput>,
    pub targets: BTreeMap<String, TargetInput>,
    pub resources: BTreeMap<String, PoolInput>,
    #[serde(default)]
    pub actors: BTreeMap<String, ActorInput>,
    pub commands: Vec<Command>,
}
