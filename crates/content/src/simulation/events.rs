use super::ExactFraction;
use crate::HypothesisId;
use crate::{execution::CausalPriority, formula::Ratio};

pub(crate) const INCOMING_DAMAGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::CombatIncomingDamageFixtureV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScheduledGameEvent {
    pub(crate) time_us: u64,
    pub(crate) priority: CausalPriority,
    pub(crate) seq: u64,
    pub(crate) data: GameEventData,
}

impl Ord for ScheduledGameEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .time_us
            .cmp(&self.time_us)
            .then_with(|| other.priority.cmp(&self.priority))
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

impl PartialOrd for ScheduledGameEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HitPacket {
    pub(crate) command_index: usize,
    pub(crate) actor_index: usize,
    pub(crate) target_index: usize,
    pub(crate) ability_id: String,
    pub(crate) damage_effect_id: String,
    pub(crate) atk_rate: Ratio,
    pub(crate) scaling_stat: super::NativeScalingStat,
    pub(crate) metadata: Box<super::metadata::ActionPacketMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GameEventData {
    CinematicRelease {
        context: ntesim_core::kernel::timing::PauseOwner,
    },
    FollowupHit {
        parent_hit_index: usize,
        packet: Box<HitPacket>,
    },
    DummyCombat {
        event_index: usize,
    },
    ActionAdmission {
        command_index: usize,
    },
    HitDelivery {
        command_index: usize,
        actor_index: usize,
        target_index: usize,
        ability_id: String,
        damage_effect_id: String,
        atk_rate: Ratio,
        scaling_stat: super::NativeScalingStat,
        metadata: Box<super::metadata::ActionPacketMetadata>,
    },
    TargetUnbalance {
        command_index: usize,
        actor_index: usize,
        target_index: usize,
        unbal_value: ExactFraction,
        owner_efficiency_bonus: ExactFraction,
    },
    TargetBreakDamage(Box<super::break_damage::BreakPacket>),
    TargetBreakRecovery {
        target_index: usize,
        generation: u64,
    },
    ReactionPistil(Box<super::reactions::PistilEvent>),
    ReactionScorchTick(Box<super::reactions::ScorchTickEvent>),
    ReactionScorchRefresh {
        source_actor_index: usize,
        target_index: usize,
        word_generation: u64,
    },
    ReactionNovaDetonate(Box<super::reactions::NovaEvent>),
    ReactionNovaEnd(Box<super::reactions::NovaEvent>),
    ReactionRemoraEnd {
        target_index: usize,
        generation: u64,
    },
    Content(crate::callbacks::ContentEvent),
    SwitchActor {
        command_index: usize,
        target_actor_index: usize,
        cooldown_us: u64,
    },
    DaffodilStepAfterSupport {
        owner: usize,
        command: usize,
    },
    IncomingDamage {
        event_index: usize,
        actor_index: usize,
        delivered_damage: u64,
    },
    HpCost {
        event_index: usize,
    },
    ActorInterrupt {
        event_index: usize,
    },
    ActorParry {
        event_index: usize,
    },
    CombatTransition {
        event_index: usize,
    },
    LiveBranch {
        branch_index: usize,
    },
    LiveEnergyWait {
        wait_index: usize,
    },
    ResumeLiveEnergyWait,
}
