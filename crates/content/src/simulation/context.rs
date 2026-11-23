use super::state::GameSimulatorState;
use super::{ActionMetadataOccurrence, GameEventReceipt, GameHitReceipt, GameScenarioRequest};
use crate::rng::Rng;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;

pub(super) struct EventContext<'a> {
    pub request: &'a GameScenarioRequest,
    pub prepared_packets: &'a [Option<super::metadata::ActionPacketMetadata>],
    pub state: &'a mut GameSimulatorState,
    pub queue: &'a mut BinaryHeap,
    pub sequence: &'a mut u64,
    pub trace: &'a mut Vec<GameEventReceipt>,
    pub rng: &'a mut Option<Rng>,
    pub hits: &'a mut Vec<GameHitReceipt>,
    pub action_metadata_ledger: &'a mut Vec<ActionMetadataOccurrence>,
}
