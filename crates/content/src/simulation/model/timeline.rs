//! Event-boundary observations, never scheduler checkpoints or inferred inputs.
use super::{
    ExactFraction, GameActorShieldSummary, GameActorStateSummary, GameCycleLedgerReport,
    GameFirefliesStack, GameTargetStateSummary,
};
use ntesim_core::buffs::dispatch::{
    ActiveBuffInstance, BuffDuration, BuffRecipient, BuffSourceLifetime, BuffStackingScope,
    DecayPolicy, RefreshPolicy,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod delta_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(Some)
    }

    pub fn serialize<T, S>(value: &Option<Option<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        match value {
            Some(Some(inner)) => inner.serialize(serializer),
            Some(None) | None => serializer.serialize_none(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "kind")]
pub enum GameStateBoundary {
    Initial,
    Event { ordinal: usize },
    Final,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameStaminaState {
    pub current: ExactFraction,
    pub spent: ExactFraction,
    pub recovered: ExactFraction,
    pub recovery_ready_at_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTimerDefinition {
    pub owner: Option<usize>,
    pub key: String,
    pub clock_basis: ntesim_core::kernel::timing::ClockDomain,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTimerRef {
    pub timer_def_id: usize,
    /// Deadline in the cataloged clock's coordinate, never an absolute projection.
    pub expires_at_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameBuffCatalogEntry {
    pub clock_basis: ntesim_core::kernel::timing::ClockDomain,
    pub id: String,
    pub source: String,
    pub owner_actor: usize,
    pub recipient: BuffRecipient,
    #[serde(default)]
    pub stacking_scope: BuffStackingScope,
    #[serde(default)]
    pub source_lifetime: BuffSourceLifetime,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub semantic_tags: BTreeSet<String>,
    pub refresh_policy: RefreshPolicy,
    pub max_stacks: u32,
    pub decay_policy: DecayPolicy,
    pub stat_modifiers: BTreeMap<String, crate::formula::Ratio>,
}

impl GameBuffCatalogEntry {
    pub(crate) fn from_instance(buff: &ActiveBuffInstance) -> Self {
        Self {
            clock_basis: ntesim_core::kernel::timing::ClockDomain::World,
            id: buff.id.clone(),
            source: buff.source.clone(),
            owner_actor: buff.owner_actor,
            recipient: buff.recipient.clone(),
            stacking_scope: buff.stacking_scope,
            source_lifetime: buff.source_lifetime,
            semantic_tags: buff.semantic_tags.clone(),
            refresh_policy: buff.refresh_policy.clone(),
            max_stacks: buff.max_stacks,
            decay_policy: buff.decay_policy,
            stat_modifiers: buff.stat_modifiers.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveBuffRef {
    pub instance_id: u64,
    pub buff_def_id: usize,
    pub stacks: u32,
    pub duration: BuffDuration,
}

impl ActiveBuffRef {
    pub fn into_instance(&self, def: &GameBuffCatalogEntry) -> ActiveBuffInstance {
        ActiveBuffInstance {
            instance_id: self.instance_id,
            stacking_scope: def.stacking_scope,
            semantic_tags: def.semantic_tags.clone(),
            source_lifetime: def.source_lifetime,
            id: def.id.clone(),
            source: def.source.clone(),
            owner_actor: def.owner_actor,
            recipient: def.recipient.clone(),
            duration: self.duration.clone(),
            refresh_policy: def.refresh_policy.clone(),
            stacks: self.stacks,
            max_stacks: def.max_stacks,
            decay_policy: def.decay_policy,
            stat_modifiers: def.stat_modifiers.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuffTimelineDiff {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upsert: Vec<ActiveBuffRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remove: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameActorStateDelta {
    pub slot: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_max: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_capacity: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_shields: Option<Vec<GameActorShieldSummary>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fireflies_stacks: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_fireflies_stacks: Option<Vec<GameFirefliesStack>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_conserved: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_energy: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub granted_energy: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spent_energy: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow_energy: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "delta_option"
    )]
    pub charge_remainder: Option<Option<ExactFraction>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "delta_option"
    )]
    pub cycle: Option<Option<GameCycleLedgerReport>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<BTreeMap<String, ntesim_core::kernel::state::Pool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fractional_resources:
        Option<BTreeMap<String, ntesim_core::kernel::state::ResourcePoolSummary>>,
}

impl GameActorStateDelta {
    pub fn from_summary(summary: &GameActorStateSummary) -> Self {
        Self {
            slot: summary.slot,
            character_id: Some(summary.character_id.clone()),
            attack_mode: Some(summary.attack_mode.clone()),
            hp: Some(summary.hp),
            hp_max: Some(summary.hp_max),
            energy: Some(summary.energy),
            energy_capacity: Some(summary.energy_capacity),
            active_shields: (!summary.active_shields.is_empty())
                .then(|| summary.active_shields.clone()),
            fireflies_stacks: Some(summary.fireflies_stacks),
            active_fireflies_stacks: Some(summary.active_fireflies_stacks.clone()),
            energy_conserved: Some(summary.energy_conserved),
            initial_energy: Some(summary.initial_energy),
            granted_energy: Some(summary.granted_energy),
            spent_energy: Some(summary.spent_energy),
            overflow_energy: Some(summary.overflow_energy),
            charge_remainder: summary.charge_remainder.map(Some),
            cycle: summary.cycle.clone().map(Some),
            resources: (!summary.resources.is_empty()).then(|| summary.resources.clone()),
            fractional_resources: (!summary.fractional_resources.is_empty())
                .then(|| summary.fractional_resources.clone()),
        }
    }

    pub fn into_summary(&self) -> Result<GameActorStateSummary, String> {
        Ok(GameActorStateSummary {
            slot: self.slot,
            character_id: self
                .character_id
                .clone()
                .ok_or("missing character_id in boundary actor state")?,
            attack_mode: self
                .attack_mode
                .clone()
                .ok_or("missing attack_mode in boundary actor state")?,
            hp: self.hp.ok_or("missing hp in boundary actor state")?,
            hp_max: self
                .hp_max
                .ok_or("missing hp_max in boundary actor state")?,
            energy: self
                .energy
                .ok_or("missing energy in boundary actor state")?,
            energy_capacity: self
                .energy_capacity
                .ok_or("missing energy_capacity in boundary actor state")?,
            active_shields: self.active_shields.clone().unwrap_or_default(),
            fireflies_stacks: self
                .fireflies_stacks
                .ok_or("missing fireflies_stacks in boundary actor state")?,
            active_fireflies_stacks: self
                .active_fireflies_stacks
                .clone()
                .ok_or("missing active_fireflies_stacks in boundary actor state")?,
            energy_conserved: self
                .energy_conserved
                .ok_or("missing energy_conserved in boundary actor state")?,
            initial_energy: self
                .initial_energy
                .ok_or("missing initial_energy in boundary actor state")?,
            granted_energy: self
                .granted_energy
                .ok_or("missing granted_energy in boundary actor state")?,
            spent_energy: self
                .spent_energy
                .ok_or("missing spent_energy in boundary actor state")?,
            overflow_energy: self
                .overflow_energy
                .ok_or("missing overflow_energy in boundary actor state")?,
            charge_remainder: self.charge_remainder.flatten(),
            cycle: self.cycle.clone().flatten(),
            resources: self.resources.clone().unwrap_or_default(),
            fractional_resources: self.fractional_resources.clone().unwrap_or_default(),
        })
    }

    pub fn diff(
        previous: &GameActorStateSummary,
        current: &GameActorStateSummary,
    ) -> Result<Option<Self>, String> {
        if previous.slot != current.slot {
            return Err("actor delta slot mismatch".into());
        }
        let delta = Self {
            slot: current.slot,
            character_id: (current.character_id != previous.character_id)
                .then(|| current.character_id.clone()),
            attack_mode: (current.attack_mode != previous.attack_mode)
                .then(|| current.attack_mode.clone()),
            hp: (current.hp != previous.hp).then_some(current.hp),
            hp_max: (current.hp_max != previous.hp_max).then_some(current.hp_max),
            energy: (current.energy != previous.energy).then_some(current.energy),
            energy_capacity: (current.energy_capacity != previous.energy_capacity)
                .then_some(current.energy_capacity),
            active_shields: (current.active_shields != previous.active_shields)
                .then(|| current.active_shields.clone()),
            fireflies_stacks: (current.fireflies_stacks != previous.fireflies_stacks)
                .then_some(current.fireflies_stacks),
            active_fireflies_stacks: (current.active_fireflies_stacks
                != previous.active_fireflies_stacks)
                .then(|| current.active_fireflies_stacks.clone()),
            energy_conserved: (current.energy_conserved != previous.energy_conserved)
                .then_some(current.energy_conserved),
            initial_energy: (current.initial_energy != previous.initial_energy)
                .then_some(current.initial_energy),
            granted_energy: (current.granted_energy != previous.granted_energy)
                .then_some(current.granted_energy),
            spent_energy: (current.spent_energy != previous.spent_energy)
                .then_some(current.spent_energy),
            overflow_energy: (current.overflow_energy != previous.overflow_energy)
                .then_some(current.overflow_energy),
            charge_remainder: (current.charge_remainder != previous.charge_remainder)
                .then_some(current.charge_remainder),
            cycle: (current.cycle != previous.cycle).then(|| current.cycle.clone()),
            resources: (current.resources != previous.resources).then(|| current.resources.clone()),
            fractional_resources: (current.fractional_resources != previous.fractional_resources)
                .then(|| current.fractional_resources.clone()),
        };
        Ok(delta.has_changes().then_some(delta))
    }

    pub fn has_changes(&self) -> bool {
        self.character_id.is_some()
            || self.attack_mode.is_some()
            || self.hp.is_some()
            || self.hp_max.is_some()
            || self.energy.is_some()
            || self.energy_capacity.is_some()
            || self.active_shields.is_some()
            || self.fireflies_stacks.is_some()
            || self.active_fireflies_stacks.is_some()
            || self.energy_conserved.is_some()
            || self.initial_energy.is_some()
            || self.granted_energy.is_some()
            || self.spent_energy.is_some()
            || self.overflow_energy.is_some()
            || self.charge_remainder.is_some()
            || self.cycle.is_some()
            || self.resources.is_some()
            || self.fractional_resources.is_some()
    }

    pub fn apply_to(&self, actor: &mut GameActorStateSummary) -> Result<(), String> {
        if actor.slot != self.slot {
            return Err("actor delta slot does not match folded actor".into());
        }
        if let Some(character_id) = &self.character_id {
            actor.character_id = character_id.clone();
        }
        if let Some(attack_mode) = &self.attack_mode {
            actor.attack_mode = attack_mode.clone();
        }
        if let Some(hp) = self.hp {
            actor.hp = hp;
        }
        if let Some(hp_max) = self.hp_max {
            actor.hp_max = hp_max;
        }
        if let Some(energy) = self.energy {
            actor.energy = energy;
        }
        if let Some(energy_capacity) = self.energy_capacity {
            actor.energy_capacity = energy_capacity;
        }
        if let Some(active_shields) = &self.active_shields {
            actor.active_shields = active_shields.clone();
        }
        if let Some(fireflies_stacks) = self.fireflies_stacks {
            actor.fireflies_stacks = fireflies_stacks;
        }
        if let Some(active_fireflies_stacks) = &self.active_fireflies_stacks {
            actor.active_fireflies_stacks = active_fireflies_stacks.clone();
        }
        if let Some(energy_conserved) = self.energy_conserved {
            actor.energy_conserved = energy_conserved;
        }
        if let Some(initial_energy) = self.initial_energy {
            actor.initial_energy = initial_energy;
        }
        if let Some(granted_energy) = self.granted_energy {
            actor.granted_energy = granted_energy;
        }
        if let Some(spent_energy) = self.spent_energy {
            actor.spent_energy = spent_energy;
        }
        if let Some(overflow_energy) = self.overflow_energy {
            actor.overflow_energy = overflow_energy;
        }
        if let Some(charge_remainder) = self.charge_remainder {
            actor.charge_remainder = charge_remainder;
        }
        if let Some(cycle) = &self.cycle {
            actor.cycle = cycle.clone();
        }
        if let Some(resources) = &self.resources {
            actor.resources = resources.clone();
        }
        if let Some(fractional_resources) = &self.fractional_resources {
            actor.fractional_resources = fractional_resources.clone();
        }
        Ok(())
    }
}

/// Replaces only the listed actor/target slots and changed buff diffs.
/// `Initial` and `Final` frames contain complete actor/target/buff snapshots;
/// intermediate `Event` frames emit `buff_diff` and sparse actor deltas.
/// Equal-time frames retain settlement order; expiry fields retain actual
/// deadlines even when an idle gap is observed at a later event boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameStateFrame {
    pub time_us: u64,
    pub boundary: GameStateBoundary,
    pub active_actor_index: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actors: Vec<GameActorStateDelta>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<GameTargetStateSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buffs: Option<Vec<ActiveBuffRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buff_diff: Option<BuffTimelineDiff>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stamina: Option<GameStaminaState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timers: Option<Vec<GameTimerRef>>,
    /// Exclusive receipt-prefix ends: every newly emitted receipt belongs to
    /// this boundary, including multiple callbacks inside a single event.
    pub hits_end: usize,
    pub healing_end: usize,
    pub trace_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameHealingReceipt {
    pub time_us: u64,
    pub source_actor_index: usize,
    pub recipient_actor_index: usize,
    pub base: ExactFraction,
    pub outgoing_bonus: ExactFraction,
    pub incoming_bonus: ExactFraction,
    pub gross: u64,
    pub applied: u64,
    pub excess: u64,
    pub hp_before: u64,
    pub hp_after: u64,
}
