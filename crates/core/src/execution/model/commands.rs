use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StepAction {
    InvokeAction {
        actor_index: usize,
        action_name: String,
        cost_pool: Option<String>,
        cost_amount: u64,
        recovery_us: u64,
        cooldown_us: u64,
    },
    GrantResource {
        pool_id: String,
        amount: u64,
    },
    SpendResource {
        pool_id: String,
        amount: u64,
    },
    DeliverDamage {
        actor_index: Option<usize>,
        target_id: String,
        base_damage: u64,
        damage_kind: DamageKind,
    },
    DeliverHeal {
        actor_index: usize,
        amount_hp: u64,
    },
    ApplyShield {
        actor_index: usize,
        shield_id: String,
        source: String,
        amount_hp: u64,
    },
    SwitchActor {
        to_actor_index: usize,
    },
    TriggerProc {
        proc_id: String,
        actor_index: usize,
        depth: u32,
        grant_pool: Option<String>,
        grant_amount: u64,
    },
    ObserveState {
        label: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionStepSpec {
    pub time_us: u64,
    pub priority: CausalPriority,
    pub action: StepAction,
}
