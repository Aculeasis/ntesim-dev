//! Universal modifier and buff infrastructure.
//!
//! Provides data-driven declarations and triggers for stat modifiers and timed buffs,
//! reducing ad-hoc callback boilerplate and unifying the combat event pipeline.

use crate::runtime_api::CallbackContext;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use ntesim_core::formula::Ratio;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

/// Universal modifier representation describing a stat alteration.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ModifierKind {
    Additive,
    Multiplicative,
    Override,
}

/// A structured modifier with target stat, modification kind, and optional tags.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UniversalModifier {
    pub stat: String,
    pub kind: ModifierKind,
    pub value: ExactFraction,
    pub duration_us: Option<u64>,
    pub tags: Vec<String>,
}

/// Standard trigger hooks for combat callbacks and procs.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TriggerHook {
    Cast {
        role: Option<crate::simulation::metadata::InputActionRole>,
    },
    Hit {
        is_crit: bool,
    },
    DamageReceived {
        actual_hp_loss: u64,
    },
    TargetDefeated {
        target_index: usize,
    },
}

/// Unified data-driven timed buff grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TimedBuffGrant {
    pub actor_index: usize,
    pub declarations: Vec<TimedBuffDeclaration>,
    pub event_kind: String,
    pub data: Option<GameEventPayload>,
}

impl TimedBuffGrant {
    /// Applies all declared buffs to the simulation state and records a single trace receipt.
    pub(crate) fn apply(&self, ctx: &mut CallbackContext<'_>) -> Result<(), String> {
        ctx.state.max_proc_depth_reached = ctx.state.max_proc_depth_reached.max(1);
        for decl in &self.declarations {
            ctx.state.apply_buff(self.actor_index, decl.clone())?;
        }
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "BuffTickOrExpiry".into(),
            event_kind: self.event_kind.clone(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(self.actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: self.data.clone(),
        });
        Ok(())
    }
}

/// Fluent builder for constructing structured timed buff grants.
#[derive(Clone, Debug)]
pub(crate) struct BuffGrantBuilder {
    actor_index: usize,
    id: String,
    source: String,
    recipient: BuffRecipient,
    duration_us: u64,
    refresh_policy: RefreshPolicy,
    stacking_scope: BuffStackingScope,
    decay_policy: DecayPolicy,
    max_stacks: u32,
    stat_modifiers: BTreeMap<String, ExactFraction>,
    extra_declarations: Vec<TimedBuffDeclaration>,
    event_kind: String,
    data: Option<GameEventPayload>,
}

impl BuffGrantBuilder {
    pub(crate) fn new(
        actor_index: usize,
        id: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            actor_index,
            id: id.into(),
            source: source.into(),
            recipient: BuffRecipient::SelfActor,
            duration_us: 0,
            refresh_policy: RefreshPolicy::Reset,
            stacking_scope: BuffStackingScope::PerSource,
            decay_policy: DecayPolicy::AllAtOnce,
            max_stacks: 1,
            stat_modifiers: BTreeMap::new(),
            extra_declarations: Vec::new(),
            event_kind: String::new(),
            data: None,
        }
    }

    pub(crate) fn recipient(mut self, recipient: BuffRecipient) -> Self {
        self.recipient = recipient;
        self
    }

    pub(crate) fn duration_us(mut self, duration_us: u64) -> Self {
        self.duration_us = duration_us;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn refresh_policy(mut self, policy: RefreshPolicy) -> Self {
        self.refresh_policy = policy;
        self
    }

    pub(crate) fn stacking_scope(mut self, scope: BuffStackingScope) -> Self {
        self.stacking_scope = scope;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn decay_policy(mut self, policy: DecayPolicy) -> Self {
        self.decay_policy = policy;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn max_stacks(mut self, max_stacks: u32) -> Self {
        self.max_stacks = max_stacks;
        self
    }

    pub(crate) fn stat(mut self, key: impl Into<String>, value: ExactFraction) -> Self {
        self.stat_modifiers.insert(key.into(), value);
        self
    }

    pub(crate) fn stat_ratio(
        mut self,
        key: impl Into<String>,
        ratio: Ratio,
    ) -> Result<Self, String> {
        let frac = ExactFraction::from_ratio(ratio)?;
        self.stat_modifiers.insert(key.into(), frac);
        Ok(self)
    }

    pub(crate) fn add_declaration(mut self, declaration: TimedBuffDeclaration) -> Self {
        self.extra_declarations.push(declaration);
        self
    }

    pub(crate) fn receipt(mut self, event_kind: impl Into<String>, data: GameEventPayload) -> Self {
        self.event_kind = event_kind.into();
        self.data = Some(data);
        self
    }

    pub(crate) fn build(self) -> TimedBuffGrant {
        let mut declarations = Vec::with_capacity(1 + self.extra_declarations.len());
        declarations.push(TimedBuffDeclaration {
            id: self.id,
            source: self.source,
            recipient: self.recipient,
            duration_us: self.duration_us,
            refresh_policy: self.refresh_policy,
            stacking_scope: self.stacking_scope,
            decay_policy: self.decay_policy,
            max_stacks: self.max_stacks,
            stat_modifiers: self.stat_modifiers,
        });
        declarations.extend(self.extra_declarations);

        TimedBuffGrant {
            actor_index: self.actor_index,
            declarations,
            event_kind: self.event_kind,
            data: self.data,
        }
    }

    pub(crate) fn into_event(self) -> crate::callbacks::ContentEvent {
        crate::callbacks::ContentEvent::TimedBuffGrant(Box::new(self.build()))
    }
}
