use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TeamLifecycle {
    pub(in super::super) field: TeamFieldState,
    pub(in super::super) actors: Vec<ActorLifecycle>,
    pub(in super::super) resources: BTreeMap<String, Pool>,
    pub(in super::super) last_time_us: u64,
}

impl TeamLifecycle {
    pub fn field(&self) -> &TeamFieldState {
        &self.field
    }
    pub fn actors(&self) -> &[ActorLifecycle] {
        &self.actors
    }
    pub fn resources(&self) -> &BTreeMap<String, Pool> {
        &self.resources
    }
    pub fn last_time_us(&self) -> u64 {
        self.last_time_us
    }

    pub fn new(
        team_size: usize,
        character_ids: Vec<String>,
        initial_active: usize,
        switch_cooldown_us: u64,
        resources: BTreeMap<String, Pool>,
    ) -> Result<Self, LifecycleError> {
        if character_ids.len() != team_size {
            return Err(LifecycleError::CharacterCountMismatch {
                expected: team_size,
                actual: character_ids.len(),
            });
        }
        let field = TeamFieldState::new(team_size, initial_active, switch_cooldown_us)?;
        let actors = character_ids
            .into_iter()
            .enumerate()
            .map(|(i, id)| ActorLifecycle::new(i, id))
            .collect();
        Ok(Self {
            field,
            actors,
            resources,
            last_time_us: 0,
        })
    }

    fn transition<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, LifecycleError>,
    ) -> Result<T, LifecycleError> {
        let mut next = self.clone();
        let result = operation(&mut next)?;
        *self = next;
        Ok(result)
    }

    pub fn field_status(&self, actor_index: usize) -> Result<FieldStatus, LifecycleError> {
        if actor_index >= self.field.team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: actor_index,
                max: self.field.team_size.saturating_sub(1),
            });
        }
        if self.field.is_on_field(actor_index) {
            Ok(FieldStatus::OnField)
        } else {
            Ok(FieldStatus::OffField)
        }
    }

    pub fn advance_time(&mut self, now_us: u64) -> Result<Vec<PhaseTransition>, LifecycleError> {
        if now_us < self.last_time_us {
            return Err(LifecycleError::TimeReversal {
                last_us: self.last_time_us,
                now_us,
            });
        }
        for actor in &self.actors {
            actor.action_state.validate()?;
            actor.form_state.validate()?;
            actor.recast_state.validate()?;
            if let Some(last_us) = match &actor.action_state.phase {
                super::super::model::ActionPhase::Idle => None,
                super::super::model::ActionPhase::Startup { started_at_us, .. }
                | super::super::model::ActionPhase::Active { started_at_us, .. }
                | super::super::model::ActionPhase::Recovery { started_at_us, .. } => {
                    Some(*started_at_us)
                }
            } && now_us < last_us
            {
                return Err(LifecycleError::TimeReversal { last_us, now_us });
            }
        }
        self.last_time_us = now_us;
        let mut transitions = Vec::new();
        for actor in &mut self.actors {
            transitions.extend(actor.action_state.advance_time(actor.actor_index, now_us)?);
            actor.form_state.advance_time(now_us);
            actor.recast_state.advance_time(now_us);
        }
        Ok(transitions)
    }

    pub fn enter_form(
        &mut self,
        actor_index: usize,
        form_id: String,
        duration_us: Option<u64>,
        now_us: u64,
    ) -> Result<(), LifecycleError> {
        self.transition(|next| {
            next.require_live_actor(actor_index)?;
            next.advance_time(now_us)?;
            next.actors[actor_index]
                .form_state
                .enter_form(form_id, duration_us, now_us)
        })
    }

    pub fn open_recast(
        &mut self,
        actor_index: usize,
        base_action: String,
        recast_action: String,
        duration_us: u64,
        max_uses: u32,
        now_us: u64,
    ) -> Result<(), LifecycleError> {
        self.transition(|next| {
            next.require_live_actor(actor_index)?;
            next.advance_time(now_us)?;
            next.actors[actor_index].recast_state.open_recast(
                base_action,
                recast_action,
                duration_us,
                max_uses,
                now_us,
            )
        })
    }

    pub fn consume_recast(
        &mut self,
        actor_index: usize,
        recast_action: &str,
        now_us: u64,
    ) -> Result<u32, LifecycleError> {
        self.transition(|next| {
            next.require_live_actor(actor_index)?;
            next.advance_time(now_us)?;
            next.actors[actor_index]
                .recast_state
                .consume_recast(recast_action, now_us)
        })
    }

    fn require_live_actor(&self, actor_index: usize) -> Result<(), LifecycleError> {
        if actor_index >= self.field.team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: actor_index,
                max: self.field.team_size.saturating_sub(1),
            });
        }
        if !self.field.is_alive(actor_index) {
            return Err(LifecycleError::ActorDead(actor_index));
        }
        Ok(())
    }

    pub fn switch_to(
        &mut self,
        target_index: usize,
        now_us: u64,
    ) -> Result<SwitchRecord, LifecycleError> {
        self.transition(|next| next.switch_to_inner(target_index, now_us))
    }
    fn switch_to_inner(
        &mut self,
        target_index: usize,
        now_us: u64,
    ) -> Result<SwitchRecord, LifecycleError> {
        if now_us < self.last_time_us {
            return Err(LifecycleError::TimeReversal {
                last_us: self.last_time_us,
                now_us,
            });
        }
        let _ = self.advance_time(now_us)?;

        // Validate destination and cooldown before mutating the outgoing action.
        self.field.can_switch_to(target_index, now_us)?;

        let current_active = self.field.active_actor();
        let active_actor = &mut self.actors[current_active];

        if !active_actor.action_state.is_idle() {
            if let Err(e) = active_actor
                .action_state
                .can_cancel(CancelType::Switch, now_us)
            {
                return match e {
                    LifecycleError::CancelNotAllowed { phase, .. } => {
                        let ends = active_actor.action_state.phase.ends_at_us().unwrap_or(0);
                        Err(LifecycleError::ActiveActorLockedInPhase {
                            phase,
                            until_us: ends,
                        })
                    }
                    LifecycleError::CancelTooEarly { earliest_us, .. } => {
                        Err(LifecycleError::ActiveActorLockedInPhase {
                            phase: active_actor.action_state.phase.kind(),
                            until_us: earliest_us,
                        })
                    }
                    other => Err(other),
                };
            }
            let _ = active_actor
                .action_state
                .cancel(current_active, CancelType::Switch, now_us)?;
        }

        self.field.switch_to(target_index, now_us)
    }

    pub fn start_action(
        &mut self,
        actor_index: usize,
        action_id: String,
        timings: ActionPhaseTimings,
        costs: BTreeMap<String, u64>,
        now_us: u64,
        require_on_field: bool,
    ) -> Result<(), LifecycleError> {
        self.transition(|next| {
            next.start_action_inner(
                actor_index,
                action_id,
                timings,
                costs,
                now_us,
                require_on_field,
            )
        })
    }
    fn start_action_inner(
        &mut self,
        actor_index: usize,
        action_id: String,
        timings: ActionPhaseTimings,
        costs: BTreeMap<String, u64>,
        now_us: u64,
        require_on_field: bool,
    ) -> Result<(), LifecycleError> {
        if now_us < self.last_time_us {
            return Err(LifecycleError::TimeReversal {
                last_us: self.last_time_us,
                now_us,
            });
        }
        if actor_index >= self.field.team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: actor_index,
                max: self.field.team_size.saturating_sub(1),
            });
        }
        if !self.field.is_alive(actor_index) {
            return Err(LifecycleError::ActorDead(actor_index));
        }
        if require_on_field && !self.field.is_on_field(actor_index) {
            return Err(LifecycleError::RequiresOnField {
                actor_index,
                active_actor: self.field.active_actor(),
            });
        }

        let _ = self.advance_time(now_us)?;

        for (pool_name, &cost) in &costs {
            let pool = self
                .resources
                .get(pool_name)
                .ok_or_else(|| LifecycleError::UnknownPool(pool_name.clone()))?;
            if pool.current() < cost {
                return Err(LifecycleError::InsufficientResource {
                    pool: pool_name.clone(),
                    required: cost,
                    available: pool.current(),
                });
            }
        }

        // Pool deductions and action admission commit together through transition.
        for (pool_name, &cost) in &costs {
            self.resources
                .get_mut(pool_name)
                .unwrap()
                .spend(cost)
                .map_err(|_| LifecycleError::InsufficientResource {
                    pool: pool_name.clone(),
                    required: cost,
                    available: self.resources[pool_name].current(),
                })?;
        }
        self.actors[actor_index]
            .action_state
            .start(action_id, timings, costs, now_us)?;

        Ok(())
    }

    pub fn cancel_action(
        &mut self,
        actor_index: usize,
        cancel_type: CancelType,
        now_us: u64,
    ) -> Result<CancelReceipt, LifecycleError> {
        self.transition(|next| next.cancel_action_inner(actor_index, cancel_type, now_us))
    }
    fn cancel_action_inner(
        &mut self,
        actor_index: usize,
        cancel_type: CancelType,
        now_us: u64,
    ) -> Result<CancelReceipt, LifecycleError> {
        if now_us < self.last_time_us {
            return Err(LifecycleError::TimeReversal {
                last_us: self.last_time_us,
                now_us,
            });
        }
        if actor_index >= self.field.team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: actor_index,
                max: self.field.team_size.saturating_sub(1),
            });
        }
        self.require_live_actor(actor_index)?;
        let _ = self.advance_time(now_us)?;
        self.actors[actor_index]
            .action_state
            .cancel(actor_index, cancel_type, now_us)
    }

    pub fn interrupt_action(
        &mut self,
        actor_index: usize,
        cause: InterruptionCause,
        refund_policy: &RefundPolicy,
        now_us: u64,
    ) -> Result<InterruptionReceipt, LifecycleError> {
        self.transition(|next| {
            next.interrupt_action_inner(actor_index, cause, refund_policy, now_us)
        })
    }
    fn interrupt_action_inner(
        &mut self,
        actor_index: usize,
        cause: InterruptionCause,
        refund_policy: &RefundPolicy,
        now_us: u64,
    ) -> Result<InterruptionReceipt, LifecycleError> {
        if now_us < self.last_time_us {
            return Err(LifecycleError::TimeReversal {
                last_us: self.last_time_us,
                now_us,
            });
        }
        if actor_index >= self.field.team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: actor_index,
                max: self.field.team_size.saturating_sub(1),
            });
        }
        self.require_live_actor(actor_index)?;
        let _ = self.advance_time(now_us)?;
        let mut receipt = self.actors[actor_index].action_state.interrupt(
            actor_index,
            cause,
            refund_policy,
            now_us,
        )?;

        for (pool_name, refund) in &mut receipt.refunded_costs {
            let grant = self
                .resources
                .get_mut(pool_name)
                .ok_or_else(|| LifecycleError::UnknownPool(pool_name.clone()))?
                .grant(*refund);
            *refund = grant.applied;
            if grant.excess > 0 {
                receipt
                    .refund_excess
                    .insert(pool_name.clone(), grant.excess);
            }
        }

        Ok(receipt)
    }

    pub fn kill_actor(
        &mut self,
        actor_index: usize,
        now_us: u64,
    ) -> Result<Option<SwitchRecord>, LifecycleError> {
        self.transition(|next| next.kill_actor_inner(actor_index, now_us))
    }
    fn kill_actor_inner(
        &mut self,
        actor_index: usize,
        now_us: u64,
    ) -> Result<Option<SwitchRecord>, LifecycleError> {
        if now_us < self.last_time_us {
            return Err(LifecycleError::TimeReversal {
                last_us: self.last_time_us,
                now_us,
            });
        }
        if actor_index >= self.field.team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: actor_index,
                max: self.field.team_size.saturating_sub(1),
            });
        }
        self.require_live_actor(actor_index)?;
        let _ = self.advance_time(now_us)?;

        if !self.actors[actor_index].action_state.is_idle() {
            let _ = self.actors[actor_index].action_state.interrupt(
                actor_index,
                InterruptionCause::Death,
                &RefundPolicy::NoRefund,
                now_us,
            )?;
        }
        self.actors[actor_index].form_state.exit_form();
        self.actors[actor_index].recast_state.clear();

        self.field.mark_dead(actor_index, now_us)
    }
}
