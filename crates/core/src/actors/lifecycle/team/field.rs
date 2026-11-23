use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TeamFieldSnapshot")]
pub struct TeamFieldState {
    pub(crate) team_size: usize,
    pub(crate) active_actor_index: usize,
    pub(crate) vitality: Vec<ActorVitality>,
    pub(crate) switch_cooldown_us: u64,
    pub(crate) switch_locked_until_us: u64,
    pub(crate) switch_history: Vec<SwitchRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TeamFieldSnapshot {
    team_size: usize,
    active_actor_index: usize,
    vitality: Vec<ActorVitality>,
    switch_cooldown_us: u64,
    switch_locked_until_us: u64,
    switch_history: Vec<SwitchRecord>,
}

impl TryFrom<TeamFieldSnapshot> for TeamFieldState {
    type Error = String;

    fn try_from(snapshot: TeamFieldSnapshot) -> Result<Self, Self::Error> {
        let state = Self {
            team_size: snapshot.team_size,
            active_actor_index: snapshot.active_actor_index,
            vitality: snapshot.vitality,
            switch_cooldown_us: snapshot.switch_cooldown_us,
            switch_locked_until_us: snapshot.switch_locked_until_us,
            switch_history: snapshot.switch_history,
        };
        state
            .validate()
            .map_err(|error| format!("invalid team field state: {error:?}"))?;
        Ok(state)
    }
}

impl TeamFieldState {
    pub fn validate(&self) -> Result<(), LifecycleError> {
        if self.team_size == 0 || self.team_size > 4 {
            return Err(LifecycleError::InvalidTeamSize(self.team_size));
        }
        if self.vitality.len() != self.team_size {
            return Err(LifecycleError::InvalidTeamSize(self.vitality.len()));
        }
        if self.active_actor_index >= self.team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: self.active_actor_index,
                max: self.team_size.saturating_sub(1),
            });
        }
        if self.vitality.contains(&ActorVitality::Alive) && !self.is_alive(self.active_actor_index)
        {
            return Err(LifecycleError::InvalidActionState);
        }
        let mut last_switch_us = 0;
        for record in &self.switch_history {
            if record.from_index >= self.team_size
                || record.to_index >= self.team_size
                || record.from_index == record.to_index
                || record.timestamp_us < last_switch_us
            {
                return Err(LifecycleError::InvalidActionState);
            }
            last_switch_us = record.timestamp_us;
        }
        if self.switch_locked_until_us < last_switch_us {
            return Err(LifecycleError::InvalidActionState);
        }
        Ok(())
    }

    pub fn team_size(&self) -> usize {
        self.team_size
    }

    pub fn vitality(&self) -> &[ActorVitality] {
        &self.vitality
    }

    pub fn new(
        team_size: usize,
        initial_active: usize,
        switch_cooldown_us: u64,
    ) -> Result<Self, LifecycleError> {
        if team_size == 0 || team_size > 4 {
            return Err(LifecycleError::InvalidTeamSize(team_size));
        }
        if initial_active >= team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: initial_active,
                max: team_size.saturating_sub(1),
            });
        }
        Ok(Self {
            team_size,
            active_actor_index: initial_active,
            vitality: vec![ActorVitality::Alive; team_size],
            switch_cooldown_us,
            switch_locked_until_us: 0,
            switch_history: Vec::new(),
        })
    }

    pub fn active_actor(&self) -> usize {
        self.active_actor_index
    }

    pub fn is_on_field(&self, actor_index: usize) -> bool {
        self.active_actor_index == actor_index
    }

    pub fn is_alive(&self, actor_index: usize) -> bool {
        self.vitality.get(actor_index).copied() == Some(ActorVitality::Alive)
    }

    pub fn can_switch_to(&self, target_index: usize, now_us: u64) -> Result<(), LifecycleError> {
        now_us
            .checked_add(self.switch_cooldown_us)
            .ok_or(LifecycleError::DeadlineOverflow)?;
        if target_index >= self.team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: target_index,
                max: self.team_size.saturating_sub(1),
            });
        }
        if target_index == self.active_actor_index {
            return Err(LifecycleError::CannotSwitchToSelf(target_index));
        }
        if !self.is_alive(target_index) {
            return Err(LifecycleError::ActorDead(target_index));
        }
        if now_us < self.switch_locked_until_us {
            return Err(LifecycleError::SwitchOnCooldown {
                until_us: self.switch_locked_until_us,
                now_us,
            });
        }
        Ok(())
    }

    pub fn switch_to(
        &mut self,
        target_index: usize,
        now_us: u64,
    ) -> Result<SwitchRecord, LifecycleError> {
        self.can_switch_to(target_index, now_us)?;
        let deadline = now_us
            .checked_add(self.switch_cooldown_us)
            .ok_or(LifecycleError::DeadlineOverflow)?;
        let from_index = self.active_actor_index;
        self.active_actor_index = target_index;
        self.switch_locked_until_us = deadline;
        let record = SwitchRecord {
            from_index,
            to_index: target_index,
            timestamp_us: now_us,
        };
        self.switch_history.push(record.clone());
        Ok(record)
    }

    pub fn mark_dead(
        &mut self,
        actor_index: usize,
        now_us: u64,
    ) -> Result<Option<SwitchRecord>, LifecycleError> {
        if actor_index >= self.team_size {
            return Err(LifecycleError::ActorIndexOutOfBounds {
                index: actor_index,
                max: self.team_size.saturating_sub(1),
            });
        }
        self.vitality[actor_index] = ActorVitality::Dead;
        if self.active_actor_index == actor_index {
            let next_alive =
                (0..self.team_size).find(|&i| self.vitality[i] == ActorVitality::Alive);
            if let Some(next) = next_alive {
                self.active_actor_index = next;
                let record = SwitchRecord {
                    from_index: actor_index,
                    to_index: next,
                    timestamp_us: now_us,
                };
                self.switch_history.push(record.clone());
                return Ok(Some(record));
            }
            return Ok(None);
        }
        Ok(None)
    }
}
