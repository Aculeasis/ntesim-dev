use super::*;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct QueuedEvent {
    pub(crate) time_us: u64,
    pub(crate) priority: CausalPriority,
    pub(crate) sequence_id: u64,
    pub(crate) action: StepAction,
    pub(crate) source_step: usize,
}

impl Ord for QueuedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse for min-heap
        other
            .time_us
            .cmp(&self.time_us)
            .then_with(|| other.priority.cmp(&self.priority))
            .then_with(|| other.sequence_id.cmp(&self.sequence_id))
    }
}

impl PartialOrd for QueuedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
