pub(crate) mod admission;
pub mod analysis;
mod availability;
mod break_damage;
pub(crate) mod break_state;
mod combat;
mod context;
mod cycle;
mod dummy;
pub(crate) mod energy;
pub(crate) mod events;
pub mod export;
mod fraction;
mod hit;
pub(crate) use hit::effective_def;
mod hp_cost;
mod incoming;
pub(crate) mod incoming_defense;
pub(crate) mod inputs;
mod interrupt;
pub mod metadata;
mod model;
mod origin;
pub(crate) use origin::Origins;
mod parry;
pub(crate) mod pause;
pub use crate::gear::GearProcConfig;
mod queue;
pub(crate) mod reactions;
pub use reactions::ReactionKind;
pub(crate) mod rejection;
mod runtime;
mod stamina;
pub(crate) mod timer_policy;
pub(crate) mod timer_queue;
pub(crate) mod timer_state;
pub use stamina::{CombatStaminaProfile, CombatStaminaReport, CombatStaminaSpend};
pub(crate) mod state;
mod support;
mod talents;
mod timeline;
pub use model::{
    ActiveBuffRef, BuffTimelineDiff, GameActorStateDelta, GameBuffCatalogEntry, GameHealingReceipt,
    GameStaminaState, GameStateBoundary, GameStateFrame, GameTimerDefinition, GameTimerRef,
};
pub(crate) use talents::project as project_talent_inputs;
pub(crate) use talents::qte_coordinate;
mod validation;
pub(crate) mod vitals;

pub use fraction::ExactFraction;
pub use model::GameInitialResources;
pub use model::GameStopPolicy;
pub use model::{
    ActionMetadata, ActionMetadataOccurrence, GameActionAdmissionOrder, GameActionCommand,
    GameActorConfig, GameActorDamageEvent, GameActorDamageReceipt, GameActorHpCostEvent,
    GameActorHpCostReceipt, GameActorShieldSummary, GameActorStateSummary, GameActorStaticStats,
    GameAdmissionReason, GameBreakContribution, GameBuffStateSummary, GameControlApplication,
    GameControlKind, GameCycleLedgerReport, GameEnergyFanoutReceipt, GameEnergyFanoutRecipient,
    GameEnergyGrantReceipt, GameEnergyReceipt, GameEnergySource, GameEnergySpendReceipt,
    GameEnergySpendSource, GameEventPayload, GameEventReceipt, GameFirefliesStack, GameHitReceipt,
    GameIncomingEffectAbsorptionReceipt, GameLiveBranch, GameLiveBranchCondition,
    GameLiveBranchReceipt, GameLiveEnergyWait, GameLiveEnergyWaitReceipt, GameProcSelectionReceipt,
    GameRejectedAction, GameScenarioProgramRequest, GameScenarioReport, GameScenarioRequest,
    GameShieldAbsorptionReceipt, GameSwitchOrigin, GameTargetBreakProfileConfig,
    GameTargetBreakStateSummary, GameTargetConfig, GameTargetStateSummary, NativeScalingStat,
    OrdinaryHitEnergyBasis, PlayerInput, PlayerInputReceipt, ResearchActionMetadata,
    ResearchCritPolicy, ResearchCritRollPolicy, ResearchFixedCritRate, StochasticCritPolicy,
};
pub use model::{
    GameActorCombatTransitionEvent, GameActorCombatTransitionReceipt, GameActorInterruptEvent,
    GameActorInterruptReceipt, GameActorParryEvent, GameActorParryReceipt, GameActorTenacityEvent,
    GameBranchInput, GameDashDirection, GameDummyCombatEvent, GameHoldInput, GamePlungeInput,
    GameResourceDepositReceipt,
};
pub use model::{GameAtkTransferCause, GameAtkTransferReceipt};
pub use model::{GameAutonomousEventReceipt, GameAutonomousHealing, GameCausalTrigger};
pub use model::{GameExecutionLimit, SourceFailureContext};
pub use model::{GameHpStatCause, GameHpStatRecipient, GameStructuralHpChange};
pub use model::{GameSourceHalt, GameSourceHaltKind};
pub use model::{
    GameTimeCause, GameTimeContext, GameTimeContextKind, GameTimeEnergy, GameTimeInterval,
    GameTimeLedger, GameTimeSource, GameTimeTotal, GameTimeWarning,
};
pub use model::{
    SourceActionWindow, SourceOccurrence, SourceOccurrenceKind, SourceRunResponse,
    SourceScenarioReport,
};
pub use model::{
    SourceFixedDamageAssumption, SourcePeriodicDamageAssumption, SourceScenarioAssumptions,
};
pub use runtime::{
    BorrowedGameScenarioReport, CompletedSourceTrial, PreparedGameProgram, PreparedGameScenario,
    PreparedSourceScenario, SourceBoundary, SourceCheckpoint, SourceControlReport,
    SourceTrialOutcome, run_compiled_source, run_game_program_report, run_game_scenario,
    run_game_scenario_report, run_source,
};
pub(crate) use validation::validate_target_break_profile;
pub use validation::{validate_game_request, validate_model_assumptions};
pub(crate) use validation::{validate_party_size, validate_target_count};

#[cfg(test)]
pub(crate) mod tests;

pub use model::{GameDpsUnavailableReason, GameIncomingDamageBasis, GameIncomingDefenseReceipt};
