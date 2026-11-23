//! Atomic action admission on one explicit local clock. No game legality defaults.
use crate::{scheduler::ClockId, state::Pool};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub recovery_us: u64,
    pub cooldown_us: u64,
    pub costs: BTreeMap<String, u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActionState {
    pub rule: Rule,
    pub ready_at_us: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Gate {
    clock: ClockId,
    last_start_us: u64,
    locked_until_us: u64,
    actions: BTreeMap<String, ActionState>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionError {
    EmptyActions,
    EmptyAction,
    EmptyPool,
    ClockMismatch {
        expected: ClockId,
        provided: ClockId,
    },
    UnknownAction(String),
    TimeReversal {
        last: u64,
        now: u64,
    },
    RecoveryLock {
        until: u64,
    },
    Cooldown {
        action: String,
        until: u64,
    },
    UnknownPool(String),
    InsufficientResource {
        pool: String,
        required: u64,
        available: u64,
    },
    Overflow,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Admission {
    pub action: String,
    pub clock: ClockId,
    pub started_at_us: u64,
    pub locked_until_us: u64,
    pub ready_at_us: u64,
    pub spent: BTreeMap<String, u64>,
}
/// Shared clock admission for adapters whose resource/field ownership is external.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimingState {
    pub last_start_us: u64,
    pub locked_until_us: u64,
    pub ready_at_us: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimingAdmission {
    pub locked_until_us: u64,
    pub ready_at_us: u64,
}
impl TimingState {
    pub fn prepare(
        &self,
        action: &str,
        now_us: u64,
        recovery_us: u64,
        cooldown_us: u64,
    ) -> Result<TimingAdmission, ActionError> {
        if now_us < self.last_start_us {
            return Err(ActionError::TimeReversal {
                last: self.last_start_us,
                now: now_us,
            });
        }
        if now_us < self.locked_until_us {
            return Err(ActionError::RecoveryLock {
                until: self.locked_until_us,
            });
        }
        if now_us < self.ready_at_us {
            return Err(ActionError::Cooldown {
                action: action.into(),
                until: self.ready_at_us,
            });
        }
        Ok(TimingAdmission {
            locked_until_us: now_us
                .checked_add(recovery_us)
                .ok_or(ActionError::Overflow)?,
            ready_at_us: now_us
                .checked_add(cooldown_us)
                .ok_or(ActionError::Overflow)?,
        })
    }
}
impl Gate {
    pub fn new(
        clock: ClockId,
        initial_time_us: u64,
        locked_until_us: u64,
        actions: BTreeMap<String, ActionState>,
    ) -> Result<Self, ActionError> {
        if actions.is_empty() {
            return Err(ActionError::EmptyActions);
        }
        for (id, state) in &actions {
            if id.trim().is_empty() {
                return Err(ActionError::EmptyAction);
            }
            if state.rule.costs.keys().any(|id| id.trim().is_empty()) {
                return Err(ActionError::EmptyPool);
            }
        }
        Ok(Self {
            clock,
            last_start_us: initial_time_us,
            locked_until_us,
            actions,
        })
    }
    pub fn clock(&self) -> ClockId {
        self.clock
    }
    pub fn locked_until_us(&self) -> u64 {
        self.locked_until_us
    }
    pub fn ready_at_us(&self, action: &str) -> Result<u64, ActionError> {
        self.actions
            .get(action)
            .map(|state| state.ready_at_us)
            .ok_or_else(|| ActionError::UnknownAction(action.into()))
    }
    /// Check the entire transaction before charging any pool or changing clocks.
    /// Recovery and cooldown both start at this admitted local timestamp.
    pub fn start(
        &mut self,
        action: &str,
        clock: ClockId,
        now_us: u64,
        resources: &mut BTreeMap<String, Pool>,
    ) -> Result<Admission, ActionError> {
        if clock != self.clock {
            return Err(ActionError::ClockMismatch {
                expected: self.clock,
                provided: clock,
            });
        }
        let state = self
            .actions
            .get(action)
            .ok_or_else(|| ActionError::UnknownAction(action.into()))?;
        let timing = TimingState {
            last_start_us: self.last_start_us,
            locked_until_us: self.locked_until_us,
            ready_at_us: state.ready_at_us,
        }
        .prepare(
            action,
            now_us,
            state.rule.recovery_us,
            state.rule.cooldown_us,
        )?;
        let lock = timing.locked_until_us;
        let ready = timing.ready_at_us;
        for (id, amount) in &state.rule.costs {
            let pool = resources
                .get(id)
                .ok_or_else(|| ActionError::UnknownPool(id.clone()))?;
            if pool.current() < *amount {
                return Err(ActionError::InsufficientResource {
                    pool: id.clone(),
                    required: *amount,
                    available: pool.current(),
                });
            }
        }
        // Borrowed resources cannot change between this preflight and settlement.
        for (id, amount) in &state.rule.costs {
            resources
                .get_mut(id)
                .unwrap()
                .spend(*amount)
                .expect("preflight verified every resource cost");
        }
        let admission = Admission {
            action: action.into(),
            clock,
            started_at_us: now_us,
            locked_until_us: lock,
            ready_at_us: ready,
            spent: state.rule.costs.clone(),
        };
        self.actions.get_mut(action).unwrap().ready_at_us = ready;
        self.last_start_us = now_us;
        self.locked_until_us = lock;
        Ok(admission)
    }
}

#[cfg(test)]
mod tests;
