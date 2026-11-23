use super::*;
use crate::formula::Ratio;
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

fn default_level() -> u32 {
    80
}

fn default_skill_level() -> u32 {
    8
}

fn default_tier() -> u32 {
    5
}

fn default_switch_cooldown() -> u64 {
    4_000_000
}

fn deserialize_unique_elemental_resistances<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, Ratio>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct UniqueResistanceMap;

    impl<'de> Visitor<'de> for UniqueResistanceMap {
        type Value = BTreeMap<String, Ratio>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a map of unique native element resistance keys")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut resistances = BTreeMap::new();
            while let Some((element, resistance)) = map.next_entry::<String, Ratio>()? {
                if resistances.insert(element.clone(), resistance).is_some() {
                    return Err(A::Error::custom(format!(
                        "duplicate elemental resistance key '{element}'"
                    )));
                }
            }
            Ok(resistances)
        }
    }

    deserializer.deserialize_map(UniqueResistanceMap)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameActorStaticStats {
    pub base_atk: u64,
    pub atk: Ratio,
    pub hp: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_hp: Option<u64>,
    #[serde(default = "Ratio::zero")]
    pub hp_percent: Ratio,
    #[serde(default = "Ratio::zero")]
    pub hp_flat: Ratio,
    pub def: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_def: Option<u64>,
    #[serde(default = "Ratio::zero")]
    pub def_percent: Ratio,
    #[serde(default = "Ratio::zero")]
    pub def_flat: Ratio,
    pub crit_rate: Ratio,
    pub crit_damage_bonus: Ratio,
    #[serde(default = "Ratio::zero", skip_serializing_if = "ratio_is_zero")]
    pub healing_bonus: Ratio,
    #[serde(default = "Ratio::zero", skip_serializing_if = "ratio_is_zero")]
    pub shield_bonus: Ratio,
    #[serde(default = "Ratio::zero")]
    pub break_intensity: Ratio,
    /// Authored Esper Cycle Intensity; no evidenced native attribute binds it.
    #[serde(default = "Ratio::zero", skip_serializing_if = "ratio_is_zero")]
    pub cycle_intensity: Ratio,
    /// Full outgoing gauge multiplier; native baseline is one, not a bonus.
    #[serde(default = "Ratio::one")]
    pub break_efficiency_multiplier: Ratio,
    #[serde(default = "Ratio::zero", skip_serializing_if = "ratio_is_zero")]
    pub mental_damage_bonus: Ratio,
    #[serde(default = "Ratio::zero")]
    pub damage_bonus: Ratio,
    #[serde(default)]
    pub elemental_bonuses: BTreeMap<String, Ratio>,
    /// Legacy research scalar: applies only to the actor's own element.
    /// Native build projectors emit zero here and preserve separate identities above.
    #[serde(default = "Ratio::zero")]
    pub elemental_bonus: Ratio,
}

fn ratio_is_zero(value: &Ratio) -> bool {
    value.numerator == 0 && value.denominator != 0
}

fn cycle_is_zero(value: &u64) -> bool {
    *value == 0
}

fn ratio_is_one(value: &Ratio) -> bool {
    value.denominator != 0 && value.numerator == value.denominator
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeScalingStat {
    #[default]
    Atk,
    BaseAtk,
    Def,
    Hp,
    /// Exact weights within one packet base; `atk_rate` scales the whole sum.
    Mixed {
        atk: Ratio,
        def: Ratio,
        hp: Ratio,
    },
}

impl NativeScalingStat {
    pub(crate) fn weights(self) -> [Ratio; 3] {
        match self {
            Self::Atk | Self::BaseAtk => [Ratio::one(), Ratio::zero(), Ratio::zero()],
            Self::Def => [Ratio::zero(), Ratio::one(), Ratio::zero()],
            Self::Hp => [Ratio::zero(), Ratio::zero(), Ratio::one()],
            Self::Mixed { atk, def, hp } => [atk, def, hp],
        }
    }

    pub(crate) fn validate(self) -> Result<(), String> {
        let weights = self.weights();
        for weight in weights {
            ExactFraction::from_ratio(weight)?;
        }
        if matches!(self, Self::Mixed { .. })
            && weights.iter().filter(|rate| rate.numerator != 0).count() < 2
        {
            return Err("mixed packet scaling requires at least two nonzero terms".into());
        }
        Ok(())
    }

    pub(crate) fn uses_atk(self) -> bool {
        self.weights()[0].numerator != 0
    }
    pub(crate) fn uses_def(self) -> bool {
        self.weights()[1].numerator != 0
    }
    pub(crate) fn uses_hp(self) -> bool {
        self.weights()[2].numerator != 0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameActorConfig {
    pub slot: usize,
    pub character_id: String,
    #[serde(default = "default_level")]
    pub character_level: u32,
    #[serde(default)]
    pub awakening: ntesim_data::AwakeningSelection,
    #[serde(default = "default_skill_level")]
    pub purchased_skill_level: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purchased_qte_level: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub talent_levels: Option<[u32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_max_level: Option<u32>,
    pub initial_hp: u64,
    pub initial_hp_max: u64,
    pub initial_energy: u64,
    pub energy_capacity: u64,
    /// Declared starting Esper Cycle pool units (nominal cap 100); spent by
    /// qualified adjacent-element switches. Accrual requires an owner packet binding.
    #[serde(default, skip_serializing_if = "cycle_is_zero")]
    pub initial_cycle: u64,
    #[serde(default, skip_serializing_if = "GameInitialResources::is_empty")]
    pub initial_resources: GameInitialResources,
    /// Declared Cycle gain multiplier, separate from Ultimate Charge Efficiency.
    #[serde(default = "Ratio::one", skip_serializing_if = "ratio_is_one")]
    pub cycle_gain_efficiency: Ratio,
    pub weapon_id: String,
    #[serde(default = "default_tier")]
    pub weapon_tier: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_id: Option<String>,
    pub static_stats: GameActorStaticStats,
    #[serde(default)]
    pub gear_procs: Vec<GearProcConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameTargetConfig {
    pub target_index: usize,
    /// Explicit initial dummy ATK, required only by reached ATK extraction models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_atk: Option<u64>,
    /// Authored dummy control assumption, independent of the Boss assumption.
    #[serde(default)]
    pub control_immune: bool,
    /// Authored base DEF for base-scaled debuffs; omission treats total DEF as base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_defense: Option<u64>,
    /// Explicit scenario model input; false is the runner default, not native classification.
    #[serde(default)]
    pub is_boss: bool,
    #[serde(default = "default_level")]
    pub level: u32,
    pub hp_max: u64,
    pub hp_current: u64,
    pub defense: u64,
    pub resistance: Ratio,
    #[serde(
        default,
        deserialize_with = "deserialize_unique_elemental_resistances",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub elemental_resistances: BTreeMap<String, Ratio>,
    #[serde(default)]
    pub airborne_until_us: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_profile: Option<GameTargetBreakProfileConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GameTargetBreakProfileConfig {
    pub capacity: Ratio,
    pub current_gauge: Ratio,
    pub broken_duration_us: u64,
    /// Explicit fraction of original gauge capacity removed by each qualified Discord.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discord_fraction: Option<Ratio>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameActionCommand {
    TargetGauge {
        timestamp_us: u64,
        target_index: usize,
        gauge: Ratio,
    },
    PlayerInput {
        timestamp_us: u64,
        actor_index: usize,
        input: PlayerInput,
        #[serde(default)]
        critical_success: bool,
    },
    InvokeAbility {
        timestamp_us: u64,
        actor_index: usize,
        ability_id: String,
        array_index: usize,
        damage_effect_id: String,
        atk_rate: Ratio,
        #[serde(default)]
        scaling_stat: NativeScalingStat,
        energy_cost: u64,
        #[serde(default)]
        recovery_us: u64,
        #[serde(default)]
        cooldown_us: u64,
        #[serde(default)]
        hit_delay_us: u64,
    },
    SwitchActor {
        timestamp_us: u64,
        target_actor_index: usize,
        #[serde(default = "default_switch_cooldown")]
        cooldown_us: u64,
    },
    Wait {
        timestamp_us: u64,
        duration_us: u64,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PlayerInput {
    Special,
    Dash,
    Arc,
    Jump,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlayerInputReceipt {
    pub command_index: usize,
    pub actor_index: usize,
    /// Resolved Arc identity, independent of the owner's ordinary abilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability_id: Option<String>,
    pub time_us: u64,
    pub input: PlayerInput,
    #[serde(default)]
    pub critical_success: bool,
    pub attack_mode_before: String,
    pub attack_mode_after: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution_model_assumption_ids: Vec<String>,
}

/// Research-only declaration used when the frozen package lacks an action or packet row.
/// Native rows always take precedence and cannot be silently overridden.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResearchActionMetadata {
    pub hypothesis_id: String,
    pub ability_id: String,
    pub damage_effect_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_gameplay_tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_action_role: Option<crate::simulation::metadata::InputActionRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_type_ex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_source_category: Option<String>,
    /// Explicit interpretation of a missing row's FixedCritRate field. Present
    /// native packet scalars always take precedence and cannot be overridden.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_crit_rate: Option<ResearchFixedCritRate>,
    /// Exact owner-declared UnbalValue for a packet with no native damage row.
    /// Omitted means absent; `Some(0/1)` preserves a declared zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unbal_value: Option<ntesim_core::formula::Ratio>,
    /// Explicit caller-declared native packet GAName for a research-only
    /// action/packet relationship. It must match the frozen row when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_ga_name_binding: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum ResearchFixedCritRate {
    NoOverride,
    Probability { rate: Ratio },
}

impl GameActionCommand {
    pub fn timestamp_us(&self) -> u64 {
        match self {
            Self::TargetGauge { timestamp_us, .. } => *timestamp_us,
            Self::PlayerInput { timestamp_us, .. } => *timestamp_us,
            Self::InvokeAbility { timestamp_us, .. } => *timestamp_us,
            Self::SwitchActor { timestamp_us, .. } => *timestamp_us,
            Self::Wait { timestamp_us, .. } => *timestamp_us,
        }
    }

    pub fn set_timestamp_us(&mut self, time_us: u64) {
        match self {
            Self::TargetGauge { timestamp_us, .. }
            | Self::PlayerInput { timestamp_us, .. }
            | Self::InvokeAbility { timestamp_us, .. }
            | Self::SwitchActor { timestamp_us, .. }
            | Self::Wait { timestamp_us, .. } => *timestamp_us = time_us,
        }
    }
}

/// Explicit versioned research roll policy; no default or game-rule inference.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchCritRollPolicy {
    IndependentPerTargetHitV1,
}

/// Explicit caller hypothesis, never an evidence-bound NTE random rule.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchCritPolicy {
    pub roll_policy: ResearchCritRollPolicy,
    pub hypothesis: String,
    pub eligible_effects: BTreeMap<String, BTreeSet<String>>,
    pub actor_sequences: BTreeMap<String, u64>,
}

pub type StochasticCritPolicy = ResearchCritPolicy;
pub type ActionMetadata = ResearchActionMetadata;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameScenarioRequest {
    pub clock_contract: ntesim_contracts::ClockContract,
    pub timer_policy: ntesim_contracts::TimerPolicy,
    /// Independent absolute execution ceiling; never enlarged for timer pauses.
    pub absolute_limit_us: u64,
    pub schema_version: u32,
    pub content_id: String,
    /// Visible modeling hypotheses; metadata never grants execution support.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub model_assumptions: BTreeMap<String, String>,
    pub relative_duration_us: u64,
    /// Time uses the full horizon; victory uses elapsed time to all-target defeat.
    #[serde(default, skip_serializing_if = "GameStopPolicy::is_time")]
    pub stop: GameStopPolicy,
    /// Explicit party-wide starting Fons balance. Missing remains distinct from zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_fons: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat_stamina: Option<super::super::CombatStaminaProfile>,
    #[serde(default)]
    pub seed: u64,
    /// Live source inputs follow the already-settled work at their timestamp.
    #[serde(
        default,
        skip_serializing_if = "GameActionAdmissionOrder::is_scheduled"
    )]
    pub action_admission_order: GameActionAdmissionOrder,
    /// Executed source reached its end; autonomous effects may outlive a time boundary.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub source_exhausted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_halt: Option<super::GameSourceHalt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_time_contexts: Vec<super::GameTimeContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stochastic_crit: Option<ResearchCritPolicy>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub action_metadata: BTreeMap<usize, ResearchActionMetadata>,
    /// Explicit airborne/height assumptions, keyed by the authored action index.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub plunge_inputs: BTreeMap<usize, GamePlungeInput>,
    /// Explicit movement-approach Normal selections; no distance is inferred.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub approach_inputs: std::collections::BTreeSet<usize>,
    /// Authored hold duration and movement intent; the owner selects the action.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hold_inputs: BTreeMap<usize, GameHoldInput>,
    /// Explicit Branch response, landing and height assumptions, keyed by the
    /// authored action index that starts the held Normal.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub branch_inputs: BTreeMap<usize, GameBranchInput>,
    /// Explicit movement intent for a Dash command; absence preserves front Dodge.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dash_directions: BTreeMap<usize, GameDashDirection>,
    /// Authored aim target per combat command. Absence selects dummy zero;
    /// each owner still controls area recipients, caps and autonomous targeting.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub aim_targets: BTreeMap<usize, usize>,
    pub actors: Vec<GameActorConfig>,
    #[serde(default)]
    pub active_actor_index: usize,
    pub targets: Vec<GameTargetConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incoming_damage_events: Vec<GameActorDamageEvent>,
    /// Explicit anonymous-dummy attack starts and range eligibility changes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dummy_combat_events: Vec<GameDummyCombatEvent>,
    /// Incoming event index -> declared anonymous dummy damage instigator.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub incoming_dummy_sources: BTreeMap<usize, usize>,
    /// Explicit interpretation of the declared amount; omission means externally mitigated.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub incoming_damage_basis: BTreeMap<usize, GameIncomingDamageBasis>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hp_cost_events: Vec<GameActorHpCostEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interrupt_events: Vec<GameActorInterruptEvent>,
    /// Explicit player-tenacity cue applications; no incoming stagger is inferred.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tenacity_events: Vec<GameActorTenacityEvent>,
    /// Explicit successful owner parries, independent from Critical Dodge.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parry_events: Vec<GameActorParryEvent>,
    /// Explicit combat leave/re-entry assumptions driving owner out-of-combat callbacks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub combat_transition_events: Vec<GameActorCombatTransitionEvent>,
    pub actions: Vec<GameActionCommand>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GameStopPolicy {
    #[default]
    Time,
    Victory,
}

impl GameStopPolicy {
    fn is_time(&self) -> bool {
        *self == Self::Time
    }
}

impl GameScenarioRequest {
    pub(crate) fn aim_target(&self, command_index: usize) -> usize {
        self.aim_targets.get(&command_index).copied().unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GameActionAdmissionOrder {
    #[default]
    Scheduled,
    AfterSettledEvents,
}

impl GameActionAdmissionOrder {
    fn is_scheduled(&self) -> bool {
        *self == Self::Scheduled
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct GamePlungeInput {
    /// Positive airborne time to landing; no geometry or inferred jump velocity.
    pub landing_delay_us: u64,
    /// Dimensionless owner height bonus; Baicang accepts the closed range0..1.
    pub height_bonus: Ratio,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GameDashDirection {
    #[default]
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct GameHoldInput {
    /// An explicit owner-supported reservation, never an inferred extra cast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserved_stocks: Option<u8>,
    pub duration_us: u64,
    #[serde(default)]
    pub moving: bool,
}

/// Explicit target response and landing assumptions for a held Normal Branch.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct GameBranchInput {
    /// True keeps Branch/BranchLoop until landing; false takes the AirAttack loop.
    pub can_be_launched: bool,
    /// Authored real time from Branch admission to landing.
    pub landing_delay_us: u64,
    /// Dimensionless fallback height bonus; the AirAttack packet uses1+height.
    #[serde(default = "Ratio::zero")]
    pub height_bonus: Ratio,
}

/// Explicit forced cancellation, or declared armor pressure for a supported owner.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameActorInterruptEvent {
    pub timestamp_us: u64,
    pub actor_index: usize,
    /// Explicit normalized incoming armor pressure for the Zankou owner model.
    /// Omission forces cancellation independently of resistance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_damage: Option<Ratio>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameActorTenacityEvent {
    pub timestamp_us: u64,
    pub actor_index: usize,
}

/// Explicit scenario HP consumption, separate from incoming damage and shields.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameActorHpCostEvent {
    pub timestamp_us: u64,
    pub actor_index: usize,
    pub source: String,
    pub amount: u64,
}

/// Explicit successful parry, distinct from Critical Dodge and incoming-damage authorization.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameActorParryEvent {
    At {
        timestamp_us: u64,
        actor_index: usize,
    },
    /// Assume success immediately after this admitted owner input, before the next input.
    AfterAction { command_index: usize },
}

/// Explicit combat state transition; no enemy state or geometry is inferred from it.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameActorCombatTransitionEvent {
    pub timestamp_us: u64,
    pub actor_index: usize,
    pub leaving: bool,
}

/// A live observation supported by the public NTE compiler.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameLiveBranchCondition {
    TargetHpBelow { target_index: usize, threshold: u64 },
    ActiveActorEnergyBelow { threshold: u64 },
}

/// Narrow live-program instruction emitted by the public NTE compiler. The
/// static request remains the source of action templates; this instruction
/// chooses the delay applied to commands after `after_action_index` for each
/// seeded execution.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GameLiveBranch {
    pub timestamp_us: u64,
    pub after_action_index: usize,
    pub condition: GameLiveBranchCondition,
    pub wait_if_below_us: u64,
    pub wait_otherwise_us: u64,
    pub source_line: u32,
    pub source_column: u32,
}

/// Live program wrapper kept separate from the stable static scenario API.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameScenarioProgramRequest {
    pub request: GameScenarioRequest,
    #[serde(default)]
    pub branches: Vec<GameLiveBranch>,
    #[serde(default)]
    pub energy_waits: Vec<GameLiveEnergyWait>,
}

/// Event-driven wait for a caller-declared whole energy threshold on one actor.
/// The runtime parks actions at and after the boundary until settled energy meets it.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GameLiveEnergyWait {
    pub timestamp_us: u64,
    pub after_action_index: usize,
    pub actor_index: usize,
    pub energy_at_least: u64,
    pub source_line: u32,
    pub source_column: u32,
}

/// Authored combat assumptions, without enemy identities, skills or autonomous AI.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameDummyCombatEvent {
    AttackStart {
        timestamp_us: u64,
        target_index: usize,
        actor_index: usize,
    },
    Proximity {
        timestamp_us: u64,
        target_index: usize,
        in_proximity: bool,
    },
}
impl GameDummyCombatEvent {
    pub fn timestamp_us(&self) -> u64 {
        match self {
            Self::AttackStart { timestamp_us, .. } | Self::Proximity { timestamp_us, .. } => {
                *timestamp_us
            }
        }
    }
    pub(crate) fn target_index(&self) -> usize {
        match self {
            Self::AttackStart { target_index, .. } | Self::Proximity { target_index, .. } => {
                *target_index
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameIncomingDamageBasis {
    ExternallyMitigated,
    BeforeDefense,
}
