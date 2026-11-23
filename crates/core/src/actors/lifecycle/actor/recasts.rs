use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(try_from = "RecastStateSnapshot")]
pub struct RecastState {
    pub(crate) active_recast: Option<RecastInstance>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecastStateSnapshot {
    active_recast: Option<RecastInstance>,
}

impl TryFrom<RecastStateSnapshot> for RecastState {
    type Error = String;

    fn try_from(snapshot: RecastStateSnapshot) -> Result<Self, Self::Error> {
        let state = Self {
            active_recast: snapshot.active_recast,
        };
        state
            .validate()
            .map_err(|error| format!("invalid recast state: {error:?}"))?;
        Ok(state)
    }
}

impl RecastState {
    pub fn validate(&self) -> Result<(), LifecycleError> {
        if let Some(recast) = &self.active_recast
            && (recast.base_action.is_empty()
                || recast.recast_action.is_empty()
                || recast.expires_at_us <= recast.opened_at_us
                || recast.uses_remaining == 0)
        {
            return Err(LifecycleError::InvalidRecastState);
        }
        Ok(())
    }

    pub fn active_recast(&self) -> Option<&RecastInstance> {
        self.active_recast.as_ref()
    }

    pub fn open_recast(
        &mut self,
        base_action: String,
        recast_action: String,
        duration_us: u64,
        max_uses: u32,
        now_us: u64,
    ) -> Result<(), LifecycleError> {
        self.validate()?;
        if base_action.is_empty() || recast_action.is_empty() {
            return Err(LifecycleError::InvalidRecastState);
        }
        if duration_us == 0 {
            return Err(LifecycleError::InvalidRecastDuration);
        }
        if max_uses == 0 {
            return Err(LifecycleError::InvalidRecastUses);
        }
        let expires_at_us = now_us
            .checked_add(duration_us)
            .ok_or(LifecycleError::DeadlineOverflow)?;
        self.active_recast = Some(RecastInstance {
            base_action,
            recast_action,
            opened_at_us: now_us,
            expires_at_us,
            uses_remaining: max_uses,
        });
        self.validate()?;
        Ok(())
    }

    pub fn can_recast(&self, recast_action: &str, now_us: u64) -> Result<(), LifecycleError> {
        self.validate()?;
        let recast = self
            .active_recast
            .as_ref()
            .ok_or(LifecycleError::NoActiveRecast)?;
        if now_us < recast.opened_at_us {
            return Err(LifecycleError::TimeReversal {
                last_us: recast.opened_at_us,
                now_us,
            });
        }
        if now_us >= recast.expires_at_us {
            return Err(LifecycleError::RecastExpired {
                expired_at_us: recast.expires_at_us,
                now_us,
            });
        }
        if recast.recast_action != recast_action {
            return Err(LifecycleError::RecastActionMismatch {
                expected: recast.recast_action.clone(),
                actual: recast_action.to_string(),
            });
        }
        if recast.uses_remaining == 0 {
            return Err(LifecycleError::RecastNoUsesRemaining);
        }
        Ok(())
    }

    pub fn consume_recast(
        &mut self,
        recast_action: &str,
        now_us: u64,
    ) -> Result<u32, LifecycleError> {
        self.can_recast(recast_action, now_us)?;
        let recast = self.active_recast.as_mut().unwrap();
        recast.uses_remaining = recast
            .uses_remaining
            .checked_sub(1)
            .ok_or(LifecycleError::RecastNoUsesRemaining)?;
        let rem = recast.uses_remaining;
        if rem == 0 {
            self.active_recast = None;
        }
        Ok(rem)
    }

    pub fn advance_time(&mut self, now_us: u64) {
        if self
            .active_recast
            .as_ref()
            .is_some_and(|recast| now_us >= recast.expires_at_us)
        {
            self.active_recast = None;
        }
    }

    pub fn clear(&mut self) {
        self.active_recast = None;
    }
}
