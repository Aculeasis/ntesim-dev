use super::*;

#[derive(Clone)]
pub(crate) struct SimState {
    pub(crate) active_actor: usize,
    pub(crate) actors: Vec<ActorState>,
    pub(crate) targets: Vec<TargetState>,
    pub(crate) pools: BTreeMap<String, ResourcePoolState>,
    pub(crate) events: Vec<AuditEvent>,
    pub(crate) last_event_time_us: u64,
    pub(crate) max_proc_depth_reached: u32,
}
