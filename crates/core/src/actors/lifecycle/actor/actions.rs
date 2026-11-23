use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ActionStateMachineSnapshot")]
pub struct ActionStateMachine {
    pub(crate) phase: ActionPhase,
    last_time_us: u64,
    pub(crate) timings: Option<ActionPhaseTimings>,
    pub(crate) pending_costs: BTreeMap<String, u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionStateMachineSnapshot {
    last_time_us: u64,
    phase: ActionPhase,
    timings: Option<ActionPhaseTimings>,
    pending_costs: BTreeMap<String, u64>,
}

impl TryFrom<ActionStateMachineSnapshot> for ActionStateMachine {
    type Error = String;

    fn try_from(snapshot: ActionStateMachineSnapshot) -> Result<Self, Self::Error> {
        let state = Self {
            phase: snapshot.phase,
            last_time_us: snapshot.last_time_us,
            timings: snapshot.timings,
            pending_costs: snapshot.pending_costs,
        };
        state
            .validate()
            .map_err(|error| format!("invalid action state: {error:?}"))?;
        Ok(state)
    }
}

impl Default for ActionStateMachine {
    fn default() -> Self {
        Self {
            phase: ActionPhase::Idle,
            last_time_us: 0,
            timings: None,
            pending_costs: BTreeMap::new(),
        }
    }
}

impl ActionStateMachine {
    pub fn phase(&self) -> &ActionPhase {
        &self.phase
    }
    pub fn last_time_us(&self) -> u64 {
        self.last_time_us
    }

    fn check_time(&self, now_us: u64) -> Result<(), LifecycleError> {
        if now_us < self.last_time_us {
            Err(LifecycleError::TimeReversal {
                last_us: self.last_time_us,
                now_us,
            })
        } else {
            Ok(())
        }
    }

    fn validate_timings(timings: &ActionPhaseTimings) -> Result<(), LifecycleError> {
        let validate_windows = |windows: &[CancelWindow], phase_duration: u64| {
            let mut seen = Vec::new();
            windows.iter().all(|window| {
                if window.earliest_offset_us >= phase_duration || seen.contains(&window.cancel_type)
                {
                    return false;
                }
                seen.push(window.cancel_type);
                true
            })
        };
        if !validate_windows(&timings.startup_cancels, timings.startup_us)
            || !validate_windows(&timings.recovery_cancels, timings.recovery_us)
        {
            return Err(LifecycleError::InvalidActionTimings);
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), LifecycleError> {
        if let ActionPhase::Startup {
            started_at_us,
            ends_at_us,
            ..
        }
        | ActionPhase::Active {
            started_at_us,
            ends_at_us,
            ..
        }
        | ActionPhase::Recovery {
            started_at_us,
            ends_at_us,
            ..
        } = &self.phase
            && (*started_at_us > self.last_time_us || self.last_time_us >= *ends_at_us)
        {
            return Err(LifecycleError::InvalidActionState);
        }
        match (&self.phase, &self.timings) {
            (ActionPhase::Idle, None) if self.pending_costs.is_empty() => Ok(()),
            (
                ActionPhase::Startup {
                    action_id,
                    started_at_us,
                    ends_at_us,
                },
                Some(timings),
            ) if !action_id.is_empty() && timings.startup_us > 0 => {
                Self::validate_timings(timings)?;
                if started_at_us.checked_add(timings.startup_us) == Some(*ends_at_us)
                    && ends_at_us
                        .checked_add(timings.active_us)
                        .and_then(|end| end.checked_add(timings.recovery_us))
                        .is_some()
                {
                    Ok(())
                } else {
                    Err(LifecycleError::InvalidActionState)
                }
            }
            (
                ActionPhase::Active {
                    action_id,
                    started_at_us,
                    ends_at_us,
                },
                Some(timings),
            ) if !action_id.is_empty() && timings.active_us > 0 => {
                Self::validate_timings(timings)?;
                if started_at_us.checked_add(timings.active_us) == Some(*ends_at_us)
                    && ends_at_us.checked_add(timings.recovery_us).is_some()
                {
                    Ok(())
                } else {
                    Err(LifecycleError::InvalidActionState)
                }
            }
            (
                ActionPhase::Recovery {
                    action_id,
                    started_at_us,
                    ends_at_us,
                },
                Some(timings),
            ) if !action_id.is_empty() && timings.recovery_us > 0 => {
                Self::validate_timings(timings)?;
                if started_at_us.checked_add(timings.recovery_us) == Some(*ends_at_us) {
                    Ok(())
                } else {
                    Err(LifecycleError::InvalidActionState)
                }
            }
            _ => Err(LifecycleError::InvalidActionState),
        }
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.phase, ActionPhase::Idle)
    }

    pub fn start(
        &mut self,
        action_id: String,
        timings: ActionPhaseTimings,
        costs: BTreeMap<String, u64>,
        now_us: u64,
    ) -> Result<(), LifecycleError> {
        self.validate()?;
        self.check_time(now_us)?;
        if action_id.is_empty() {
            return Err(LifecycleError::InvalidActionTimings);
        }
        if !self.is_idle() {
            return Err(LifecycleError::ActionBusy {
                current_phase: self.phase.kind(),
            });
        }
        Self::validate_timings(&timings)?;
        now_us
            .checked_add(timings.startup_us)
            .and_then(|time| time.checked_add(timings.active_us))
            .and_then(|time| time.checked_add(timings.recovery_us))
            .ok_or(LifecycleError::DeadlineOverflow)?;
        if timings.startup_us == 0 && timings.active_us == 0 && timings.recovery_us == 0 {
            self.phase = ActionPhase::Idle;
            self.timings = None;
            self.pending_costs.clear();
            self.last_time_us = now_us;
            return Ok(());
        }
        self.pending_costs = costs;

        if timings.startup_us > 0 {
            let ends_at_us = now_us
                .checked_add(timings.startup_us)
                .ok_or(LifecycleError::DeadlineOverflow)?;
            self.phase = ActionPhase::Startup {
                action_id,
                started_at_us: now_us,
                ends_at_us,
            };
        } else if timings.active_us > 0 {
            let ends_at_us = now_us
                .checked_add(timings.active_us)
                .ok_or(LifecycleError::DeadlineOverflow)?;
            self.phase = ActionPhase::Active {
                action_id,
                started_at_us: now_us,
                ends_at_us,
            };
        } else if timings.recovery_us > 0 {
            let ends_at_us = now_us
                .checked_add(timings.recovery_us)
                .ok_or(LifecycleError::DeadlineOverflow)?;
            self.phase = ActionPhase::Recovery {
                action_id,
                started_at_us: now_us,
                ends_at_us,
            };
        }
        self.timings = Some(timings);
        self.last_time_us = now_us;
        self.validate()?;
        Ok(())
    }

    pub fn advance_time(
        &mut self,
        actor_index: usize,
        now_us: u64,
    ) -> Result<Vec<PhaseTransition>, LifecycleError> {
        self.validate()?;
        self.check_time(now_us)?;
        let mut transitions = Vec::new();
        loop {
            let next_step = match &self.phase {
                ActionPhase::Idle => None,
                ActionPhase::Startup {
                    action_id,
                    ends_at_us,
                    ..
                } => {
                    if now_us >= *ends_at_us {
                        let timings = self
                            .timings
                            .as_ref()
                            .ok_or(LifecycleError::InvalidActionState)?;
                        let next = if timings.active_us > 0 {
                            ActionPhaseKind::Active
                        } else if timings.recovery_us > 0 {
                            ActionPhaseKind::Recovery
                        } else {
                            ActionPhaseKind::Idle
                        };
                        Some((
                            action_id.clone(),
                            ActionPhaseKind::Startup,
                            *ends_at_us,
                            next,
                        ))
                    } else {
                        None
                    }
                }
                ActionPhase::Active {
                    action_id,
                    ends_at_us,
                    ..
                } => {
                    if now_us >= *ends_at_us {
                        let next = if self.timings.as_ref().is_some_and(|t| t.recovery_us > 0) {
                            ActionPhaseKind::Recovery
                        } else {
                            ActionPhaseKind::Idle
                        };
                        Some((
                            action_id.clone(),
                            ActionPhaseKind::Active,
                            *ends_at_us,
                            next,
                        ))
                    } else {
                        None
                    }
                }
                ActionPhase::Recovery {
                    action_id,
                    ends_at_us,
                    ..
                } => {
                    if now_us >= *ends_at_us {
                        Some((
                            action_id.clone(),
                            ActionPhaseKind::Recovery,
                            *ends_at_us,
                            ActionPhaseKind::Idle,
                        ))
                    } else {
                        None
                    }
                }
            };

            if let Some((action_id, from_kind, end_t, to_kind)) = next_step {
                match to_kind {
                    ActionPhaseKind::Active | ActionPhaseKind::Recovery => {
                        let timings = self
                            .timings
                            .as_ref()
                            .ok_or(LifecycleError::InvalidActionState)?;
                        let duration = if to_kind == ActionPhaseKind::Active {
                            timings.active_us
                        } else {
                            timings.recovery_us
                        };
                        let ends_at_us = end_t
                            .checked_add(duration)
                            .ok_or(LifecycleError::DeadlineOverflow)?;
                        self.phase = if to_kind == ActionPhaseKind::Active {
                            ActionPhase::Active {
                                action_id: action_id.clone(),
                                started_at_us: end_t,
                                ends_at_us,
                            }
                        } else {
                            ActionPhase::Recovery {
                                action_id: action_id.clone(),
                                started_at_us: end_t,
                                ends_at_us,
                            }
                        };
                    }
                    ActionPhaseKind::Idle => {
                        self.phase = ActionPhase::Idle;
                        self.timings = None;
                        self.pending_costs.clear();
                    }
                    _ => return Err(LifecycleError::InvalidActionState),
                }

                transitions.push(PhaseTransition {
                    actor_index,
                    action_id,
                    from_kind,
                    to_kind,
                    timestamp_us: end_t,
                });
            } else {
                break;
            }
        }
        self.last_time_us = now_us;
        self.validate()?;
        Ok(transitions)
    }

    pub fn can_cancel(&self, cancel_type: CancelType, now_us: u64) -> Result<(), LifecycleError> {
        self.validate()?;
        self.check_time(now_us)?;
        if self
            .phase
            .ends_at_us()
            .is_some_and(|ends_at_us| now_us >= ends_at_us)
        {
            return Err(LifecycleError::CancelNotAllowed {
                phase: self.phase.kind(),
                cancel_type,
            });
        }
        if let ActionPhase::Startup { started_at_us, .. }
        | ActionPhase::Recovery { started_at_us, .. } = self.phase
            && now_us < started_at_us
        {
            return Err(LifecycleError::TimeReversal {
                last_us: started_at_us,
                now_us,
            });
        }
        let timings = self
            .timings
            .as_ref()
            .ok_or(LifecycleError::CancelNotAllowed {
                phase: self.phase.kind(),
                cancel_type,
            })?;

        let (allowed_cancels, started_at_us, current_kind) = match &self.phase {
            ActionPhase::Idle => {
                return Err(LifecycleError::CancelNotAllowed {
                    phase: ActionPhaseKind::Idle,
                    cancel_type,
                });
            }
            ActionPhase::Startup { started_at_us, .. } => (
                &timings.startup_cancels,
                *started_at_us,
                ActionPhaseKind::Startup,
            ),
            ActionPhase::Active { .. } => {
                return Err(LifecycleError::CancelNotAllowed {
                    phase: ActionPhaseKind::Active,
                    cancel_type,
                });
            }
            ActionPhase::Recovery { started_at_us, .. } => (
                &timings.recovery_cancels,
                *started_at_us,
                ActionPhaseKind::Recovery,
            ),
        };

        let window = allowed_cancels
            .iter()
            .find(|w| w.cancel_type == cancel_type)
            .ok_or(LifecycleError::CancelNotAllowed {
                phase: current_kind,
                cancel_type,
            })?;

        let earliest = started_at_us
            .checked_add(window.earliest_offset_us)
            .ok_or(LifecycleError::DeadlineOverflow)?;
        if now_us < earliest {
            return Err(LifecycleError::CancelTooEarly {
                earliest_us: earliest,
                now_us,
            });
        }

        Ok(())
    }

    pub fn cancel(
        &mut self,
        actor_index: usize,
        cancel_type: CancelType,
        now_us: u64,
    ) -> Result<CancelReceipt, LifecycleError> {
        self.can_cancel(cancel_type, now_us)?;
        let action_id = self.phase.action_id().unwrap_or("unknown").to_string();
        let phase = self.phase.kind();
        self.phase = ActionPhase::Idle;
        self.timings = None;
        self.pending_costs.clear();
        self.last_time_us = now_us;
        Ok(CancelReceipt {
            actor_index,
            action_id,
            phase,
            cancel_type,
            timestamp_us: now_us,
        })
    }

    pub fn interrupt(
        &mut self,
        actor_index: usize,
        cause: InterruptionCause,
        refund_policy: &RefundPolicy,
        now_us: u64,
    ) -> Result<InterruptionReceipt, LifecycleError> {
        self.validate()?;
        self.check_time(now_us)?;
        if self.is_idle() || self.phase.ends_at_us().is_some_and(|end| now_us >= end) {
            return Err(LifecycleError::NoActionToInterrupt);
        }
        if let RefundPolicy::PartialRefund { ratio } = refund_policy
            && (ratio.denominator == 0 || ratio.numerator > ratio.denominator)
        {
            return Err(LifecycleError::InvalidRefundRatio);
        }
        let action_id = self.phase.action_id().unwrap_or("unknown").to_string();
        let from_phase = self.phase.kind();

        let mut refunded_costs = BTreeMap::new();
        if from_phase == ActionPhaseKind::Startup {
            match refund_policy {
                RefundPolicy::FullRefund => {
                    refunded_costs = self.pending_costs.clone();
                }
                RefundPolicy::PartialRefund { ratio } => {
                    for (pool, &cost) in &self.pending_costs {
                        let refunded = (cost as u128 * ratio.numerator as u128
                            / ratio.denominator as u128)
                            as u64;
                        if refunded > 0 {
                            refunded_costs.insert(pool.clone(), refunded);
                        }
                    }
                }
                RefundPolicy::NoRefund => {}
            }
        }

        self.phase = ActionPhase::Idle;
        self.timings = None;
        self.pending_costs.clear();

        self.last_time_us = now_us;
        Ok(InterruptionReceipt {
            actor_index,
            action_id,
            from_phase,
            cause,
            timestamp_us: now_us,
            refunded_costs,
            refund_excess: BTreeMap::new(),
        })
    }
}
