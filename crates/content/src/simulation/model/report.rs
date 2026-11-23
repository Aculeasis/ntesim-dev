use super::*;
use crate::rng::Sample;
use ntesim_contracts::Quality;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameActorDamageEvent {
    pub timestamp_us: u64,
    pub actor_index: usize,
    pub delivered_damage: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GameActorDamageReceipt {
    /// The optional explicit DEF stage precedes redistribution and shields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defense_mitigation: Option<GameIncomingDefenseReceipt>,
    pub event_index: usize,
    pub time_us: u64,
    pub actor_index: usize,
    pub delivered_damage: u64,
    /// Nonrecursive child shares; the root retains the authored delivered total.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redirected: Vec<GameActorDamageReceipt>,
    /// HP damage prevented by an explicit immunity/interception after ordinary mitigation.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub prevented_hp_loss: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub absorbed_by_shields: Vec<GameShieldAbsorptionReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub absorbed_by_effects: Vec<GameIncomingEffectAbsorptionReceipt>,
    pub hp_before: u64,
    pub actual_hp_loss: u64,
    pub overflow: u64,
    pub hp_after: u64,
    pub defeated: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GameActorHpCostReceipt {
    pub event_index: usize,
    pub time_us: u64,
    pub actor_index: usize,
    pub source: String,
    pub requested_hp: u64,
    pub hp_before: u64,
    pub actual_hp_loss: u64,
    pub hp_after: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameShieldAbsorptionReceipt {
    pub shield: String,
    pub amount: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameIncomingEffectAbsorptionReceipt {
    pub effect_id: String,
    pub source_actor_index: usize,
    pub broken_shield: String,
    pub amount: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameActorShieldSummary {
    pub clock_basis: ntesim_core::kernel::timing::ClockDomain,
    pub id: String,
    pub source: String,
    pub remaining_hp: u64,
    pub max_hp: u64,
    pub expires_at_us: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameProcSelectionReceipt {
    pub time_us: u64,
    pub actor_index: usize,
    pub stream: String,
    pub sample: crate::rng::Sample,
    pub outcome_index: u64,
    pub cooldown_until_us: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GameAdmissionReason {
    Stamina {
        required: ExactFraction,
        available: ExactFraction,
    },
    ActorNotActive {
        active_actor_index: usize,
    },
    Recovery {
        until_us: u64,
    },
    Cooldown {
        until_us: u64,
    },
    Energy {
        required: u64,
        available: u64,
    },
    TimeReversal {
        last_us: u64,
    },
    TimingOverflow,
    Other {
        message: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameRejectedAction {
    pub command_index: usize,
    pub time_us: u64,
    pub actor_index: usize,
    pub reason: GameAdmissionReason,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GameExecutionLimit {
    EventCount { limit: u64 },
    AbsoluteTime { limit_us: u64 },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GameEventPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utility_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buff_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shield_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lamb_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dummy_event: Option<GameDummyCombatEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_target_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_target_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_hit_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordinal: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stacks: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stacks_before: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_stacks: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consumed: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discord_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scorch_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nova_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bud_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub word_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_pct: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipients: Option<Vec<usize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_deposits: Option<BTreeMap<String, GameResourceDepositReceipt>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completes_at_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown_until_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_before_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_after_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leave_grant_due_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_origin_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admitted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leaving: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enhanced: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copied: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defeated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launchable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_crit: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub critical: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_cancelled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_alive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gross: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_before: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_after: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_max: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_max_before: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivered_damage: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absorbed_by_shields: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absorbed_by_effects: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pool: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atk_add: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atk_percent: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub def_percent: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub def_ignore: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crit_rate_bonus: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crit_damage_bonus: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_bonus: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mental_damage_bonus: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atk_ratio: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_amount: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_before: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_after: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_delta: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_raw: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_scaled: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_carry: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_efficiency: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_capacity: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_overflow: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wide_basis: Option<ntesim_core::kernel::state::WideFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wide_amount: Option<ntesim_core::kernel::state::WideFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wide_overflow: Option<ntesim_core::kernel::state::WideFraction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameEventReceipt {
    pub time_us: u64,
    pub phase: String,
    pub event_kind: String,
    pub active_actor: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_actor: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_actor: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_change: Option<GameEnergyReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_fanout: Option<GameEnergyFanoutReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<GameEventPayload>,
}

impl GameEventReceipt {
    pub fn energy_grants(&self) -> Vec<GameEnergyGrantReceipt> {
        match (&self.energy_change, &self.energy_fanout) {
            (Some(GameEnergyReceipt::Grant(grant)), None) => vec![grant.clone()],
            (None, Some(fanout)) => fanout.grants().collect(),
            (Some(GameEnergyReceipt::Grant(grant)), Some(fanout)) => std::iter::once(grant.clone())
                .chain(fanout.grants())
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn energy_spend(&self) -> Option<&GameEnergySpendReceipt> {
        match &self.energy_change {
            Some(GameEnergyReceipt::Spend(spend)) => Some(spend),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionMetadataOccurrence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_origin: Option<GameSwitchOrigin>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution_model_assumption_ids: Vec<String>,
    pub command_indices: Vec<usize>,
    pub actor_index: usize,
    pub ability_id: String,
    pub damage_effect_id: String,
    pub metadata_id: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_group_id: Option<String>,
}

/// Engine-owned consequences of a switch retain the command and participants;
/// consumers validate this relationship without reproducing combat admission.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GameSwitchOrigin {
    Support {
        incoming_actor_index: usize,
    },
    EsperCycle {
        outgoing_actor_index: usize,
        incoming_actor_index: usize,
    },
    EsperCycleReplacement {
        outgoing_actor_index: usize,
        incoming_actor_index: usize,
        owner_actor_index: usize,
        trigger_hit_index: usize,
    },
}

impl GameSwitchOrigin {
    pub(crate) fn support(
        request: &super::request::GameScenarioRequest,
        command_index: usize,
    ) -> Option<Self> {
        match request.actions.get(command_index) {
            Some(super::request::GameActionCommand::SwitchActor {
                target_actor_index, ..
            }) => Some(Self::Support {
                incoming_actor_index: *target_actor_index,
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GameFirefliesStack {
    pub expires_at_us: u64,
    pub crit_damage_bonus: ExactFraction,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameActorStateSummary {
    pub slot: usize,
    pub character_id: String,
    pub attack_mode: String,
    pub hp: u64,
    pub hp_max: u64,
    pub energy: u64,
    pub energy_capacity: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_shields: Vec<GameActorShieldSummary>,
    pub fireflies_stacks: u32,
    pub active_fireflies_stacks: Vec<GameFirefliesStack>,
    pub energy_conserved: bool,
    pub initial_energy: u64,
    pub granted_energy: u64,
    pub spent_energy: u64,
    pub overflow_energy: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_remainder: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle: Option<GameCycleLedgerReport>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub resources: BTreeMap<String, ntesim_core::kernel::state::Pool>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fractional_resources: BTreeMap<String, ntesim_core::kernel::state::ResourcePoolSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameCycleLedgerReport {
    pub initial: u64,
    pub current: u64,
    pub capacity: u64,
    pub granted: u64,
    pub spent: u64,
    pub overflow: u64,
    pub carry: ExactFraction,
    pub conserved: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameTargetStateSummary {
    pub target_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack: Option<ExactFraction>,
    #[serde(default)]
    pub control_immune: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_defense: Option<u64>,
    /// Scenario-supplied model metadata; does not assert native enemy classification.
    #[serde(default)]
    pub is_boss: bool,
    pub hp_current: u64,
    pub hp_max: u64,
    pub defense: u64,
    #[serde(default)]
    pub airborne_until_us: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<GameTargetStatusSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_state: Option<GameTargetBreakStateSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameTargetBreakStateSummary {
    pub capacity: ExactFraction,
    pub gauge: ExactFraction,
    pub broken_duration_us: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broken_until_us: Option<u64>,
    pub generation: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameTargetStatusSummary {
    pub clock_basis: ntesim_core::kernel::timing::ClockDomain,
    pub source_actor_index: usize,
    pub id: String,
    pub expires_at_us: u64,
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub semantic_tags: std::collections::BTreeSet<String>,
    pub stacks: u32,
    pub resistance_reductions: BTreeMap<String, ExactFraction>,
}

/// Shared lifecycle snapshot. WhileAlive effects retain their declared source
/// lifetime; final actor HP determines whether their modifiers still apply.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameBuffStateSummary {
    pub time_us: u64,
    pub active_actor_index: usize,
    pub instances: Vec<super::ActiveBuffRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameHitReceipt {
    /// Captured at settlement, independently of damage ownership or cast time.
    pub active_actor_index: usize,
    /// Earlier settled hit causing this same-owner/target damage follow-up.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_hit_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_base_override: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autonomous_event_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_applications: Vec<GameControlApplication>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_origin: Option<GameSwitchOrigin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_contribution: Option<GameBreakContribution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution_model_assumption_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_index: Option<usize>,
    pub time_us: u64,
    pub actor_index: usize,
    pub target_index: usize,
    pub ability_id: String,
    pub damage_effect_id: String,
    #[serde(default)]
    pub scaling_stat: NativeScalingStat,
    #[serde(default = "Ratio::zero")]
    pub scaling_rate: Ratio,
    pub metadata_id: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_group_id: Option<String>,
    pub static_general_damage_bonus: ExactFraction,
    pub static_element_damage_bonus: ExactFraction,
    pub static_mental_damage_bonus: ExactFraction,
    pub non_crit_damage: ExactFraction,
    pub crit_damage: ExactFraction,
    pub crit_rate: Ratio,
    pub critical: bool,
    pub damage: ExactFraction,
    pub rounded_damage: u64,
    pub applied_hp_loss: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crit_sample: Option<Sample>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameControlKind {
    Suppress,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameControlApplication {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autonomous_event_index: Option<usize>,
    pub kind: GameControlKind,
    pub source_actor_index: usize,
    pub target_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_index: Option<usize>,
    pub duration_us: u64,
    pub expires_at_us: u64,
}

impl GameControlApplication {
    pub(crate) fn causal_key(&self) -> Result<usize, String> {
        match (self.command_index, self.autonomous_event_index) {
            (Some(command), None) => Ok(command),
            (None, Some(event)) => super::super::origin::Origins::event_key(event),
            _ => Err("control receipt must have exactly one causal source".into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameBreakContribution {
    pub generation: u64,
    pub character_level: u32,
    pub base_break: ExactFraction,
    pub original_capacity: ExactFraction,
    pub intensity: ExactFraction,
    pub bonus: ExactFraction,
    pub defense_ignore: ExactFraction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameScenarioReport {
    pub dps_unavailable_reason: Option<GameDpsUnavailableReason>,
    pub clock_contract: ntesim_contracts::ClockContract,
    pub timer_policy: ntesim_contracts::TimerPolicy,
    pub absolute_limit_us: u64,
    pub relative_elapsed_us: u64,
    /// Detailed sample observations; compact batches carry only elapsed scalars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timer_pauses: Option<Vec<ntesim_core::kernel::timing::OwnedModePause>>,
    pub timer_bindings: BTreeMap<usize, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timer_catalog: Option<Vec<super::GameTimerDefinition>>,
    pub state_timeline: Vec<super::GameStateFrame>,
    pub healing_receipts: Vec<super::GameHealingReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_reached: Option<GameExecutionLimit>,
    pub input_time: GameTimeLedger,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub autonomous_events: Vec<super::GameAutonomousEventReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interrupt_receipts: Vec<GameActorInterruptReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat_stamina: Option<super::super::CombatStaminaReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub player_input_receipts: Vec<PlayerInputReceipt>,
    pub schema_version: u32,
    pub content_id: String,
    pub status: String,
    pub quality: Quality,
    pub game_dps_computed: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub model_assumptions: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub resolved_model_assumptions: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub callback_model_assumptions: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_catalog: Vec<ntesim_data::native_action_metadata::NativeEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata_catalog: Vec<crate::simulation::metadata::NormalizedActionPacketMetadata>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buff_catalog: Vec<super::GameBuffCatalogEntry>,
    pub success: bool,
    pub execution_completed: bool,
    pub pending_events: usize,
    pub relative_duration_us: u64,
    /// Actual absolute combat time, used by playback and support metrics.
    pub absolute_elapsed_us: u64,
    pub seed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stochastic_crit: Option<ResearchCritPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rng: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaction_rng: Option<Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reaction_counts: BTreeMap<String, u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proc_rng: Option<Value>,
    pub invariants_passed: bool,
    pub total_events_processed: usize,
    pub hits_settled: usize,
    pub max_proc_depth_reached: usize,
    /// Presentation conversion of the checked sum of rounded hit damage.
    pub total_damage_dealt: f64,
    /// Primary damage, including overkill, represented exactly as N/1.
    pub total_damage_exact: ExactFraction,
    pub hit_receipts: Vec<GameHitReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub live_branch_receipts: Vec<GameLiveBranchReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub live_energy_wait_receipts: Vec<GameLiveEnergyWaitReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incoming_damage_receipts: Vec<GameActorDamageReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hp_cost_receipts: Vec<GameActorHpCostReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parry_receipts: Vec<GameActorParryReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub combat_transition_receipts: Vec<GameActorCombatTransitionReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proc_selection_receipts: Vec<GameProcSelectionReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_metadata_ledger: Vec<ActionMetadataOccurrence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub talent_projection_receipts: Vec<Value>,
    pub final_actor_states: Vec<GameActorStateSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structural_hp_changes: Vec<GameStructuralHpChange>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub atk_transfers: Vec<super::GameAtkTransferReceipt>,
    pub final_target_states: Vec<GameTargetStateSummary>,
    pub final_buff_state: GameBuffStateSummary,
    pub trace: Vec<GameEventReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub halt_diagnostic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejected_action: Option<GameRejectedAction>,
    pub available_shield_hp: ntesim_contracts::TimeBasisPair<f64>,
    pub field_times: Vec<ntesim_contracts::TimeBasisPair<u64>>,
    pub engine_version: String,
    pub engine_source_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameDpsUnavailableReason {
    ZeroRelativeDuration,
    IncompleteExecution,
    UnassessedModel,
}

pub(crate) fn validate_elapsed(
    relative_us: u64,
    absolute_us: u64,
    relative_duration_us: u64,
    absolute_limit_us: u64,
    actors: usize,
    field_times: &[ntesim_contracts::TimeBasisPair<u64>],
) -> Result<(), String> {
    if relative_us > absolute_us
        || absolute_us > absolute_limit_us
        || absolute_limit_us == 0
        || relative_duration_us == 0
        || field_times.len() != actors
    {
        return Err("invalid relative/absolute elapsed time or execution bounds".into());
    }
    let totals = field_times
        .iter()
        .try_fold((0_u64, 0_u64), |(relative, absolute), time| {
            let (r, a) = (time.relative?, time.absolute?);
            (r <= a).then_some((relative.checked_add(r)?, absolute.checked_add(a)?))
        });
    if totals != Some((relative_us, absolute_us)) {
        return Err("invalid relative/absolute elapsed time or execution bounds".into());
    }
    Ok(())
}

impl GameScenarioReport {
    pub fn validate_clocks(&self) -> Result<(), String> {
        validate_elapsed(
            self.relative_elapsed_us,
            self.absolute_elapsed_us,
            self.relative_duration_us,
            self.absolute_limit_us,
            self.final_actor_states.len(),
            &self.field_times,
        )?;
        if self.game_dps_computed != self.dps_unavailable_reason.is_none()
            || (self.relative_elapsed_us == 0
                && self.dps_unavailable_reason
                    != Some(GameDpsUnavailableReason::ZeroRelativeDuration))
            || (self.relative_elapsed_us > 0
                && self.dps_unavailable_reason
                    == Some(GameDpsUnavailableReason::ZeroRelativeDuration))
        {
            return Err("invalid relative/absolute elapsed time or execution bounds".into());
        }
        if let Some(pauses) = &self.timer_pauses {
            let catalog = self
                .timer_catalog
                .as_ref()
                .ok_or("missing native timer catalog")?;
            for (index, timer) in catalog.iter().enumerate() {
                if timer
                    .owner
                    .is_some_and(|owner| owner >= self.final_actor_states.len())
                    || timer.key.is_empty()
                    || !matches!(
                        timer.clock_basis,
                        ntesim_core::kernel::timing::ClockDomain::World
                            | ntesim_core::kernel::timing::ClockDomain::ObjectLifetime
                    )
                    || catalog[..index]
                        .iter()
                        .any(|entry| entry.owner == timer.owner && entry.key == timer.key)
                {
                    return Err("invalid or duplicate native timer definition".into());
                }
            }
            let mut intervals = Vec::with_capacity(pauses.len());
            let mut owners = BTreeMap::new();
            let mut previous_start = 0;
            for pause in pauses {
                if pause.owner.owner >= self.final_actor_states.len()
                    || !self
                        .timer_bindings
                        .get(&pause.owner.owner)
                        .is_some_and(|binding| !binding.is_empty())
                    || pause.start_absolute_us < previous_start
                    || pause.start_absolute_us >= pause.end_absolute_us
                    || pause.end_absolute_us > self.absolute_elapsed_us
                    || owners
                        .insert(pause.owner, pause.end_absolute_us)
                        .is_some_and(|end| end > pause.start_absolute_us)
                {
                    return Err("invalid mode-timer pause bounds or generation ownership".into());
                }
                previous_start = pause.start_absolute_us;
                intervals.push([pause.start_absolute_us, pause.end_absolute_us]);
            }
            let paused: u64 = ntesim_core::kernel::timing::merge_intervals(intervals)
                .iter()
                .map(|[start, end]| end - start)
                .sum();
            if self.absolute_elapsed_us - paused != self.relative_elapsed_us {
                return Err(
                    "relative elapsed time plus union pause must equal absolute elapsed time"
                        .into(),
                );
            }
            if self
                .state_timeline
                .first()
                .is_none_or(|frame| frame.timers.is_none())
                || self
                    .state_timeline
                    .last()
                    .is_none_or(|frame| frame.timers.is_none())
            {
                return Err("missing initial or final native timer state".into());
            }
            for frame in &self.state_timeline {
                if frame.time_us > self.absolute_elapsed_us {
                    return Err("timer observation exceeds elapsed time".into());
                }
                if let Some(timers) = &frame.timers {
                    let mut previous_id = None;
                    for timer in timers {
                        let definition = catalog
                            .get(timer.timer_def_id)
                            .ok_or("invalid native timer reference")?;
                        let intervals = pauses
                            .iter()
                            .filter(|pause| {
                                definition.clock_basis
                                    == ntesim_core::kernel::timing::ClockDomain::World
                                    || pause.kind
                                        == ntesim_core::kernel::timing::ModePauseKind::Cinematic
                            })
                            .filter_map(|pause| {
                                let end = pause.end_absolute_us.min(frame.time_us);
                                (pause.start_absolute_us < end)
                                    .then_some([pause.start_absolute_us, end])
                            });
                        let stopped: u64 =
                            ntesim_core::kernel::timing::merge_intervals(intervals.collect())
                                .iter()
                                .map(|[start, end]| end - start)
                                .sum();
                        if previous_id.is_some_and(|id| id >= timer.timer_def_id)
                            || timer.expires_at_us <= frame.time_us - stopped
                        {
                            return Err(
                                "invalid native timer remainder or reference ordering".into()
                            );
                        }
                        previous_id = Some(timer.timer_def_id);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn hit_metadata(
        &self,
        hit: &GameHitReceipt,
    ) -> Option<&crate::simulation::metadata::NormalizedActionPacketMetadata> {
        self.metadata_catalog.get(hit.metadata_id)
    }

    pub fn action_metadata(
        &self,
        occurrence: &ActionMetadataOccurrence,
    ) -> Option<&crate::simulation::metadata::NormalizedActionPacketMetadata> {
        self.metadata_catalog.get(occurrence.metadata_id)
    }

    pub fn evidence(
        &self,
        evidence_id: usize,
    ) -> Option<&ntesim_data::native_action_metadata::NativeEvidence> {
        self.evidence_catalog.get(evidence_id)
    }

    pub fn buff_definition(
        &self,
        buff: &super::ActiveBuffRef,
    ) -> Option<&super::GameBuffCatalogEntry> {
        self.buff_catalog.get(buff.buff_def_id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameActorInterruptReceipt {
    pub event_index: usize,
    pub actor_index: usize,
    pub time_us: u64,
    pub interrupted_command_index: Option<usize>,
    pub cancelled_hit_deliveries: usize,
    pub recovery_before_us: u64,
    pub recovery_after_us: u64,
    pub skill_cooldown_before_us: Option<u64>,
    pub skill_cooldown_after_us: Option<u64>,
    pub field_cancelled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameActorParryReceipt {
    pub event_index: usize,
    pub actor_index: usize,
    pub time_us: u64,
    pub malice_granted: ExactFraction,
    pub skill_permission_until_us: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameActorCombatTransitionReceipt {
    pub event_index: usize,
    pub actor_index: usize,
    pub time_us: u64,
    pub leaving: bool,
    pub malice_granted: ExactFraction,
    pub leave_grant_due_us: Option<u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub resource_deposits: BTreeMap<String, GameResourceDepositReceipt>,
}

/// An exact owner deposit before repayment/cap settlement; decimal components
/// retain the wide financial ledger's precision through every transport.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GameResourceDepositReceipt {
    pub input: ntesim_core::kernel::state::WideFraction,
    pub repaid: ntesim_core::kernel::state::WideFraction,
    pub credited: ntesim_core::kernel::state::WideFraction,
    pub overflow: ntesim_core::kernel::state::WideFraction,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameLiveBranchReceipt {
    pub time_us: u64,
    pub condition: super::GameLiveBranchCondition,
    pub observer_actor_index: usize,
    pub observed_value: u64,
    pub selected_wait_us: u64,
    pub baseline_wait_us: u64,
    pub after_action_index: usize,
    pub source_line: u32,
    pub source_column: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GameLiveEnergyWaitReceipt {
    pub started_time_us: u64,
    pub actor_index: usize,
    pub energy_at_start: u64,
    pub energy_at_least: u64,
    pub resumed_time_us: Option<u64>,
    pub energy_at_resume: Option<u64>,
    pub after_action_index: usize,
    pub source_line: u32,
    pub source_column: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GameIncomingDefenseReceipt {
    pub damage_before_defense: u64,
    pub source_target_index: usize,
    pub source_level: u32,
    pub recipient_defense: ExactFraction,
}
