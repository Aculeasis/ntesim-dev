use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Content supplies a finite cast/mode generation, never just an actor identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct PauseOwner {
    pub owner: usize,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModePauseKind {
    Cinematic,
    ActiveStop,
}

/// Global progression policy. Local montage dilation composes separately with
/// the elapsed action clock; it must not pause world or object clocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClockDomain {
    Absolute,
    World,
    ObjectLifetime,
    Action {
        context: PauseOwner,
        advance_during_active_stop: bool,
    },
    ReplayAction {
        context: PauseOwner,
    },
    AutonomousActivity {
        advance_during_active_stop: bool,
    },
}

impl ClockDomain {
    fn progresses(self, owner: PauseOwner, kind: ModePauseKind) -> bool {
        match kind {
            ModePauseKind::Cinematic => self.progresses_during_cinematic(owner),
            ModePauseKind::ActiveStop => self.progresses_during_active_stop(),
        }
    }

    fn progresses_during_cinematic(self, owner: PauseOwner) -> bool {
        self == Self::Absolute || matches!(self, Self::Action { context, .. } if context == owner)
    }

    fn progresses_during_active_stop(self) -> bool {
        match self {
            Self::Absolute | Self::ObjectLifetime | Self::ReplayAction { .. } => true,
            Self::World => false,
            Self::Action {
                advance_during_active_stop,
                ..
            }
            | Self::AutonomousActivity {
                advance_during_active_stop,
            } => advance_during_active_stop,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Cinematics {
    #[default]
    None,
    One(PauseOwner),
    Overlapping,
}

/// The active restrictions that affect scheduler deadlines. Multiple active
/// stops impose the same restriction. Two cinematic owners share no running
/// domain except Absolute.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClockProgression {
    cinematics: Cinematics,
    active_stop: bool,
}

impl ClockProgression {
    pub fn progresses(self, domain: ClockDomain) -> bool {
        let cinematic_allows = match self.cinematics {
            Cinematics::None => true,
            Cinematics::One(owner) => domain.progresses_during_cinematic(owner),
            Cinematics::Overlapping => domain == ClockDomain::Absolute,
        };
        cinematic_allows && (!self.active_stop || domain.progresses_during_active_stop())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedModePause {
    pub owner: PauseOwner,
    pub kind: ModePauseKind,
    pub start_absolute_us: u64,
    pub end_absolute_us: u64,
}

/// Incremental live counterpart of the interval-based timing research service.
/// Work per advance depends on registered live domains and active pauses, not
/// the length of combat or its historical pause list. No future release is guessed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeModeClock {
    absolute_us: u64,
    clocks: BTreeMap<ClockDomain, u64>,
    active: BTreeMap<PauseOwner, (ModePauseKind, u64)>,
    retired: BTreeSet<PauseOwner>,
    completed: Vec<OwnedModePause>,
    record_pauses: bool,
}

impl Default for RuntimeModeClock {
    fn default() -> Self {
        Self {
            absolute_us: 0,
            clocks: [
                (ClockDomain::Absolute, 0),
                (ClockDomain::World, 0),
                (ClockDomain::ObjectLifetime, 0),
                (
                    ClockDomain::AutonomousActivity {
                        advance_during_active_stop: false,
                    },
                    0,
                ),
                (
                    ClockDomain::AutonomousActivity {
                        advance_during_active_stop: true,
                    },
                    0,
                ),
            ]
            .into(),
            active: BTreeMap::new(),
            retired: BTreeSet::new(),
            completed: Vec::new(),
            record_pauses: true,
        }
    }
}

impl RuntimeModeClock {
    pub fn new(record_pauses: bool) -> Self {
        Self {
            record_pauses,
            ..Self::default()
        }
    }
    pub fn absolute_us(&self) -> u64 {
        self.absolute_us
    }

    pub fn relative_us(&self) -> u64 {
        self.clocks[&ClockDomain::World]
    }

    pub fn is_paused(&self) -> bool {
        !self.active.is_empty()
    }

    pub fn active_pauses(&self) -> impl Iterator<Item = (PauseOwner, ModePauseKind)> + '_ {
        self.active.iter().map(|(&owner, &(kind, _))| (owner, kind))
    }

    pub fn progresses(&self, domain: ClockDomain) -> bool {
        self.active_pauses()
            .all(|(owner, kind)| domain.progresses(owner, kind))
    }

    /// A small current policy snapshot without historical observations.
    pub fn progression_policy(&self) -> ClockProgression {
        let mut policy = ClockProgression::default();
        for (owner, kind) in self.active_pauses() {
            match kind {
                ModePauseKind::Cinematic => {
                    policy.cinematics = match policy.cinematics {
                        Cinematics::None => Cinematics::One(owner),
                        _ => Cinematics::Overlapping,
                    };
                }
                ModePauseKind::ActiveStop => policy.active_stop = true,
            }
        }
        policy
    }

    /// Exact reconstructed state for replay identity, independent of whether
    /// detailed historical pause observations were retained for a sample.
    pub fn replay_state(
        &self,
    ) -> (
        u64,
        Vec<(ClockDomain, u64)>,
        Vec<(PauseOwner, ModePauseKind, u64)>,
        Vec<PauseOwner>,
    ) {
        (
            self.absolute_us,
            self.clocks
                .iter()
                .map(|(&domain, &time)| (domain, time))
                .collect(),
            self.active
                .iter()
                .map(|(&owner, &(kind, start))| (owner, kind, start))
                .collect(),
            self.retired.iter().copied().collect(),
        )
    }

    /// Register a finite action context at its creation boundary. Its coordinate
    /// starts at zero; registering it again never resets its elapsed progress.
    pub fn register_action(&mut self, context: PauseOwner, advance_during_active_stop: bool) {
        self.clocks
            .entry(ClockDomain::Action {
                context,
                advance_during_active_stop,
            })
            .or_insert(0);
    }

    pub fn remove_action(&mut self, context: PauseOwner) {
        self.clocks.retain(|domain, _| {
            !matches!(domain,
            ClockDomain::Action { context: registered, .. } if *registered == context)
        });
    }

    pub fn elapsed_us(&self, domain: ClockDomain) -> Option<u64> {
        self.clocks.get(&domain).copied()
    }

    pub fn advance_to(&mut self, absolute_us: u64) -> Result<(), TimingError> {
        let delta = absolute_us
            .checked_sub(self.absolute_us)
            .ok_or(TimingError::TimeReversal)?;
        if delta == 0 {
            return Ok(());
        }
        // Every elapsed coordinate is bounded by absolute_us, so these additions
        // cannot overflow once the checked absolute difference succeeds.
        for (domain, elapsed) in &mut self.clocks {
            if self
                .active
                .iter()
                .all(|(&owner, &(kind, _))| domain.progresses(owner, kind))
            {
                *elapsed += delta;
            }
        }
        self.absolute_us = absolute_us;
        Ok(())
    }

    /// Duplicate delivery cannot restart a pause or change its classification.
    pub fn start_pause(
        &mut self,
        owner: PauseOwner,
        kind: ModePauseKind,
        at_us: u64,
    ) -> Result<bool, TimingError> {
        self.advance_to(at_us)?;
        if self.active.contains_key(&owner) || self.retired.contains(&owner) {
            return Ok(false);
        }
        self.active.insert(owner, (kind, at_us));
        Ok(true)
    }

    /// Unknown/stale releases cannot close a different generation's pause.
    pub fn end_pause(&mut self, owner: PauseOwner, at_us: u64) -> Result<bool, TimingError> {
        self.advance_to(at_us)?;
        let Some((kind, start_absolute_us)) = self.active.remove(&owner) else {
            return Ok(false);
        };
        if self.record_pauses {
            self.completed.push(OwnedModePause {
                owner,
                kind,
                start_absolute_us,
                end_absolute_us: at_us,
            });
        }
        self.retired.insert(owner);
        Ok(true)
    }

    /// A cinematic-to-active handoff keeps ownership and the union continuous,
    /// while retaining the two distinct timer policies in observations.
    pub fn transition_pause(
        &mut self,
        owner: PauseOwner,
        kind: ModePauseKind,
        at_us: u64,
    ) -> Result<bool, TimingError> {
        self.advance_to(at_us)?;
        let Some((previous, start_absolute_us)) = self.active.get_mut(&owner) else {
            return Ok(false);
        };
        if *previous == kind {
            return Ok(false);
        }
        if self.record_pauses {
            self.completed.push(OwnedModePause {
                owner,
                kind: *previous,
                start_absolute_us: *start_absolute_us,
                end_absolute_us: at_us,
            });
        }
        *previous = kind;
        *start_absolute_us = at_us;
        Ok(true)
    }

    /// Observe only executed time, including clipped open pauses. This does not
    /// release a live owner or imply that its cinematic naturally completed.
    pub fn observed_pauses(&self, through_us: u64) -> Result<Vec<OwnedModePause>, TimingError> {
        if !self.record_pauses || through_us > self.absolute_us {
            return Err(TimingError::InvalidCurve(
                "pause observation exceeds executed time",
            ));
        }
        let mut pauses: Vec<_> = self
            .completed
            .iter()
            .copied()
            .chain(
                self.active
                    .iter()
                    .map(|(&owner, &(kind, start_absolute_us))| OwnedModePause {
                        owner,
                        kind,
                        start_absolute_us,
                        end_absolute_us: self.absolute_us,
                    }),
            )
            .filter_map(|mut pause| {
                pause.end_absolute_us = pause.end_absolute_us.min(through_us);
                (pause.start_absolute_us < pause.end_absolute_us).then_some(pause)
            })
            .collect();
        pauses.sort_unstable_by_key(|pause| (pause.start_absolute_us, pause.owner));
        Ok(pauses)
    }

    /// Project the relative endpoint only while the world is running. A pause
    /// requires the caller to advance to actual release work or its absolute cap.
    pub fn relative_deadline(&self, target_us: u64) -> Result<Option<u64>, TimingError> {
        if target_us <= self.relative_us() {
            return Ok(Some(self.absolute_us));
        }
        if self.is_paused() {
            return Ok(None);
        }
        self.absolute_us
            .checked_add(target_us - self.relative_us())
            .map(Some)
            .ok_or(TimingError::DeadlineOverflow)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ModePauseInterval {
    pub start_real_us: u64,
    pub duration_us: u64,
    pub reason: String,
}

impl ModePauseInterval {
    pub fn end_real_us(&self) -> Result<u64, TimingError> {
        self.start_real_us
            .checked_add(self.duration_us)
            .ok_or(TimingError::DeadlineOverflow)
    }
}

pub fn merge_intervals(mut intervals: Vec<[u64; 2]>) -> Vec<[u64; 2]> {
    if intervals.is_empty() {
        return Vec::new();
    }
    intervals.retain(|&[s, e]| e > s);
    if intervals.is_empty() {
        return Vec::new();
    }
    intervals.sort_unstable_by_key(|&[s, _]| s);

    let mut merged = Vec::with_capacity(intervals.len());
    let [mut cur_start, mut cur_end] = intervals[0];

    for &[s, e] in &intervals[1..] {
        if s <= cur_end {
            cur_end = cur_end.max(e);
        } else {
            merged.push([cur_start, cur_end]);
            cur_start = s;
            cur_end = e;
        }
    }
    merged.push([cur_start, cur_end]);
    merged
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ModeClockManager {
    pauses: Vec<ModePauseInterval>,
    merged_intervals: Vec<[u64; 2]>,
}

impl ModeClockManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pauses(&self) -> &[ModePauseInterval] {
        &self.pauses
    }
    pub fn merged_intervals(&self) -> &[[u64; 2]] {
        &self.merged_intervals
    }

    pub fn add_pause(&mut self, pause: ModePauseInterval) -> Result<(), TimingError> {
        pause.end_real_us()?;
        self.pauses.push(pause);
        self.recompute_merged();
        Ok(())
    }

    pub(crate) fn recompute_merged(&mut self) {
        let raw: Vec<[u64; 2]> = self
            .pauses
            .iter()
            .map(|p| [p.start_real_us, p.end_real_us().expect("validated pause")])
            .collect();
        self.merged_intervals = merge_intervals(raw);
    }

    /// Total paused real microseconds in the range [0, real_time_us].
    pub fn paused_time_up_to(&self, real_time_us: u64) -> u64 {
        let mut total = 0u64;
        for &[s, e] in &self.merged_intervals {
            if s >= real_time_us {
                break;
            }
            let overlap_end = e.min(real_time_us);
            if overlap_end > s {
                total += overlap_end - s;
            }
        }
        total
    }

    pub fn real_to_mode_time(&self, real_time_us: u64) -> u64 {
        let paused = self.paused_time_up_to(real_time_us);
        real_time_us - paused
    }

    pub fn mode_to_real_time(&self, mode_time_us: u64) -> Result<u64, TimingError> {
        if self.merged_intervals.is_empty() {
            return Ok(mode_time_us);
        }
        let mut current_real = 0u64;
        let mut accumulated_mode = 0u64;

        for &[s, e] in &self.merged_intervals {
            let active_before = s - current_real;
            if accumulated_mode + active_before >= mode_time_us {
                let needed = mode_time_us - accumulated_mode;
                return current_real
                    .checked_add(needed)
                    .ok_or(TimingError::DeadlineOverflow);
            }
            accumulated_mode += active_before;
            current_real = e;
        }

        let remaining = mode_time_us - accumulated_mode;
        current_real
            .checked_add(remaining)
            .ok_or(TimingError::DeadlineOverflow)
    }
}
