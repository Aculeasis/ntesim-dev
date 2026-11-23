use crate::HypothesisId;
use crate::simulation::state::ActorRuntime;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{GameScenarioRequest, events::GameEventData};

pub(crate) fn next_packet_group(
    actor_index: usize,
    actor: &mut ActorRuntime,
) -> Result<String, String> {
    actor.packet_group_ordinal = actor
        .packet_group_ordinal
        .checked_add(1)
        .ok_or("native packet group ordinal overflow")?;
    // Owner-local identity is independent of future authored commands.
    let id = ((actor_index as u128) << 64) | u128::from(actor.packet_group_ordinal);
    Ok(format!("packet:{id}"))
}

#[derive(Clone, Copy)]
pub(crate) struct NormalActionOverride {
    pub(crate) packet: &'static str,
    pub(crate) role: Option<crate::simulation::metadata::InputActionRole>,
    pub(crate) model: HypothesisId,
    /// Conservative recovery and primary impact delay in real microseconds.
    pub(crate) timing: Option<(u64, u64)>,
}

/// Captured by a newly created Blossom generation; later party changes do not
/// rewrite already scheduled emissions.
pub(crate) struct BlossomEmissionProfile {
    pub(crate) count: u32,
    pub(crate) interval_us: u64,
    pub(crate) model: HypothesisId,
}

/// A non-damaging replacement of an authored ability input. Its own gate
/// replaces the ordinary cooldown query without overwriting that cooldown.
#[derive(Clone, Copy)]
pub(crate) struct UtilityInvocation {
    pub(crate) ability_id: &'static str,
    pub(crate) ready_at_us: u64,
    pub(crate) recovery_us: u64,
    pub(crate) model: HypothesisId,
}

impl UtilityInvocation {
    pub(crate) fn validate(self, request: &GameScenarioRequest) -> Result<(), String> {
        super::record_hypothesis_id(request, &mut Default::default(), self.model)
    }
}

pub(crate) struct PlungeInputContext<'a> {
    pub(crate) request: &'a GameScenarioRequest,
    pub(crate) metadata: &'a crate::simulation::metadata::ActionPacketMetadata,
    pub(crate) input: &'a crate::simulation::GamePlungeInput,
    pub(crate) timestamp_us: u64,
}

/// A reached owner decision needs settled equal-time HP before extending
/// recovery or spending resources. Its two-phase callback uses this barrier
/// once, then commits in ProcCallback before these ordinary inputs resume.
pub(crate) fn defer_current_admissions(queue: &mut BinaryHeap, now: u64) {
    let mut events = queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if event.time_us == now
            && matches!(
                event.data,
                GameEventData::ActionAdmission { .. } | GameEventData::SwitchActor { .. }
            )
        {
            event.priority = ntesim_core::execution::CausalPriority::Observation;
        }
    }
    queue.replace(events);
}

pub(crate) enum OwnerEntryAction {
    Attack(OwnerEntryAttack),
    Support(&'static SupportEntry),
    Effect {
        source_actor_index: usize,
        support: Option<&'static SupportEntry>,
        apply:
            fn(&mut super::context::CallbackContext<'_>, usize, usize, usize) -> Result<(), String>,
    },
}

pub(crate) struct SupportEntry {
    pub ability_id: &'static str,
    pub damage_effect_id: &'static str,
    pub hit_delay_us: u64,
    pub recovery_us: u64,
    pub dispatch_model: HypothesisId,
    pub packet_model: Option<HypothesisId>,
    pub extra_damage_effects: &'static [&'static str],
}

pub(crate) struct SinglePacketSupport {
    pub(crate) entry: SupportEntry,
    pub(crate) switch_gate_us: u64,
    pub(crate) connected_pause_us: u64,
}

impl SinglePacketSupport {
    pub(crate) const fn new(
        ability: &'static str,
        effect: &'static str,
        impact_us: u64,
        recovery_us: u64,
        switch_gate_us: u64,
        connected_pause_us: u64,
    ) -> Self {
        Self {
            entry: SupportEntry {
                ability_id: ability,
                damage_effect_id: effect,
                hit_delay_us: impact_us,
                recovery_us,
                dispatch_model: HypothesisId::CombatSinglePacketSupportV1,
                packet_model: None,
                extra_damage_effects: &[],
            },
            switch_gate_us,
            connected_pause_us,
        }
    }
}

pub(crate) struct OwnerEntryAttack {
    pub(crate) ability: &'static str,
    pub(crate) effect: &'static str,
    pub(crate) coordinate: usize,
    pub(crate) hit_delay_us: u64,
    pub(crate) recovery_us: u64,
    pub(crate) model: HypothesisId,
}

pub(crate) fn increment_sequence(sequence: &mut u64) -> Result<(), String> {
    *sequence = sequence.checked_add(1).ok_or("event sequence overflow")?;
    Ok(())
}
