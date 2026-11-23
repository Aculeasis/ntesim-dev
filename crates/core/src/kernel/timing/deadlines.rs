use super::*;

/// Semantic deadlines on a binary progressing/paused clock, with cached absolute
/// projections for admission APIs. Advancing an unpaused clock is O(1); a paused
/// segment updates only the live timer map, never the scheduler's unrelated work.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectedDeadlines<K> {
    entries: std::collections::BTreeMap<K, (u64, u64)>,
    absolute_us: u64,
    domain_us: u64,
}

impl<K: Ord> ProjectedDeadlines<K> {
    pub fn insert(&mut self, key: K, absolute_deadline_us: u64) -> Option<u64> {
        let semantic = if absolute_deadline_us >= self.absolute_us {
            // domain_us <= absolute_us, so the sum cannot exceed the supplied
            // u64 absolute deadline, even at the representable boundary.
            self.domain_us + (absolute_deadline_us - self.absolute_us)
        } else {
            self.domain_us
                .saturating_sub(self.absolute_us - absolute_deadline_us)
        };
        self.entries
            .insert(key, (semantic, absolute_deadline_us))
            .map(|(_, absolute)| absolute)
    }

    pub fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&u64>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.entries.get(key).map(|(_, absolute)| absolute)
    }

    pub fn remove<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<u64>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.entries.remove(key).map(|(_, absolute)| absolute)
    }

    pub fn contains_key<Q: Ord + ?Sized>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
    {
        self.entries.contains_key(key)
    }

    pub fn remaining_us<Q: Ord + ?Sized>(&self, key: &Q) -> Option<u64>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.entries
            .get(key)
            .map(|(semantic, _)| semantic.saturating_sub(self.domain_us))
    }

    pub fn semantic_deadlines(&self) -> impl Iterator<Item = (&K, u64)> {
        self.entries
            .iter()
            .map(|(key, (semantic, _))| (key, *semantic))
    }

    pub fn advance(&mut self, absolute_us: u64, domain_us: u64) -> Result<(), TimingError> {
        let delta = absolute_us
            .checked_sub(self.absolute_us)
            .ok_or(TimingError::TimeReversal)?;
        let domain_delta = domain_us
            .checked_sub(self.domain_us)
            .ok_or(TimingError::TimeReversal)?;
        if domain_us > absolute_us || (domain_delta != 0 && domain_delta != delta) {
            return Err(TimingError::InvalidCurve(
                "deadline projection must advance at actual pause boundaries",
            ));
        }
        if domain_delta == 0 && delta > 0 {
            // Validate the complete update before mutating any timer.
            for (semantic, projected) in self.entries.values() {
                if *semantic > self.domain_us {
                    projected
                        .checked_add(delta)
                        .ok_or(TimingError::DeadlineOverflow)?;
                }
            }
            for (semantic, projected) in self.entries.values_mut() {
                if *semantic > self.domain_us {
                    *projected += delta;
                }
            }
        }
        self.absolute_us = absolute_us;
        self.domain_us = domain_us;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ActionTimingSpec {
    pub action_id: String,
    pub startup_us: u64,
    pub hit_offsets_us: Vec<u64>,
    pub recovery_us: u64,
    pub earliest_switch_offset_us: u64,
}

impl ActionTimingSpec {
    pub fn total_duration_us(&self) -> Result<u64, TimingError> {
        self.startup_us
            .checked_add(self.recovery_us)
            .ok_or(TimingError::DeadlineOverflow)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ResolvedActionDeadlines {
    pub action_id: String,
    pub start_real_us: u64,
    pub start_mode_us: u64,
    pub hit_deadlines_real_us: Vec<u64>,
    pub hit_deadlines_mode_us: Vec<u64>,
    pub earliest_switch_real_us: u64,
    pub earliest_switch_mode_us: u64,
    pub recovery_end_real_us: u64,
    pub recovery_end_mode_us: u64,
}

pub fn resolve_action_deadlines(
    actor_index: usize,
    start_real_us: u64,
    spec: &ActionTimingSpec,
    hitlag: &HitlagTracker,
    mode_clock: &ModeClockManager,
) -> Result<ResolvedActionDeadlines, TimingError> {
    let mut hit_real = Vec::with_capacity(spec.hit_offsets_us.len());
    let mut hit_mode = Vec::with_capacity(spec.hit_offsets_us.len());

    for &hit_offset in &spec.hit_offsets_us {
        let r_hit = hitlag.resolve_real_deadline(actor_index, start_real_us, hit_offset)?;
        let m_hit = mode_clock.real_to_mode_time(r_hit);
        hit_real.push(r_hit);
        hit_mode.push(m_hit);
    }

    let r_switch =
        hitlag.resolve_real_deadline(actor_index, start_real_us, spec.earliest_switch_offset_us)?;
    let m_switch = mode_clock.real_to_mode_time(r_switch);

    let r_end =
        hitlag.resolve_real_deadline(actor_index, start_real_us, spec.total_duration_us()?)?;
    let m_end = mode_clock.real_to_mode_time(r_end);

    Ok(ResolvedActionDeadlines {
        action_id: spec.action_id.clone(),
        start_real_us,
        start_mode_us: mode_clock.real_to_mode_time(start_real_us),
        hit_deadlines_real_us: hit_real,
        hit_deadlines_mode_us: hit_mode,
        earliest_switch_real_us: r_switch,
        earliest_switch_mode_us: m_switch,
        recovery_end_real_us: r_end,
        recovery_end_mode_us: m_end,
    })
}
