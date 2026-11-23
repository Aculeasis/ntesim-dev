use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEvent {
    ActionInvoked {
        time_us: u64,
        actor_index: usize,
        action_name: String,
        cost_spent: u64,
        locked_until_us: u64,
        cooldown_until_us: u64,
    },
    ResourceGranted {
        time_us: u64,
        pool_id: String,
        amount_requested: u64,
        amount_granted: u64,
        amount_overflow: u64,
        new_balance: u64,
    },
    ResourceSpent {
        time_us: u64,
        pool_id: String,
        amount: u64,
        new_balance: u64,
    },
    DamageResolved {
        time_us: u64,
        target_id: String,
        raw_damage: u64,
        effective_damage: u64,
        absorbed_by_shields: u64,
        hp_lost: u64,
        overkill: u64,
        target_died: bool,
    },
    HealResolved {
        time_us: u64,
        actor_index: usize,
        requested_heal: u64,
        effective_heal: u64,
        overheal: u64,
        new_hp: u64,
    },
    ShieldApplied {
        time_us: u64,
        actor_index: usize,
        shield_id: String,
        source: String,
        amount_hp: u64,
    },
    ActorSwitched {
        time_us: u64,
        from_actor: usize,
        to_actor: usize,
    },
    ProcTriggered {
        time_us: u64,
        proc_id: String,
        actor_index: usize,
        depth: u32,
    },
    StateObserved {
        time_us: u64,
        label: String,
        active_actor: usize,
        total_hp: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct HaltDiagnostic {
    pub failed_step_index: usize,
    pub time_us: u64,
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExecutionScenarioReport {
    pub status: String,
    pub content_id: String,
    pub success: bool,
    pub execution_completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub halt_diagnostic: Option<HaltDiagnostic>,
    pub horizon_us: u64,
    pub final_time_us: u64,
    pub final_active_actor: usize,
    pub party_wiped: bool,
    pub total_events_processed: u64,
    pub max_proc_depth_reached: u32,
    pub invariants_checked: usize,
    pub invariants_passed: bool,
    pub state_sha256: String,
    pub trace_sha256: String,
    pub actors: Vec<ActorState>,
    pub targets: Vec<TargetState>,
    pub resource_pools: BTreeMap<String, ResourcePoolReport>,
    pub events: Vec<AuditEvent>,
    pub quality: String,
    pub combat_adapter_complete: bool,
    pub game_dps_computed: bool,
}
