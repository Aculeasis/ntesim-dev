//! Deterministic local-clock tasks. Rates are explicit technical parameters,
//! never inferred asset FPS, animation playback speed or game hitstop rules.
use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
};

pub const RATE_SCALE: u32 = 1_000_000;
pub type ClockId = u32;
pub type EventId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClockReading {
    pub local_time_us: u64,
    /// Fraction of a microsecond with denominator RATE_SCALE.
    pub phase_millionths: u32,
    pub rate_ppm: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    UnknownClock(ClockId),
    DuplicateClock(ClockId),
    UnknownCause(EventId),
    UnknownEvent(EventId),
    TimeReversal,
    Overflow,
    InternalInvariant(&'static str),
}

struct Task<T> {
    deadline: u128,
    id: EventId,
    cause: Option<EventId>,
    payload: T,
}
impl<T> PartialEq for Task<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.deadline, self.id) == (other.deadline, other.id)
    }
}
impl<T> Eq for Task<T> {}
impl<T> PartialOrd for Task<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Task<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.deadline, other.id).cmp(&(self.deadline, self.id))
    }
}

struct Clock<T> {
    phase: u128,
    rate: u32,
    tasks: BinaryHeap<Task<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Delivery<T> {
    pub event_id: EventId,
    pub cause: Option<EventId>,
    pub real_time_us: u64,
    pub clock: ClockId,
    pub local_time_us: u64,
    pub payload: T,
}

pub struct Scheduler<T> {
    real_time: u64,
    next_id: EventId,
    clocks: BTreeMap<ClockId, Clock<T>>,
    pending: BTreeSet<EventId>,
    delivered: BTreeSet<EventId>,
    canceled: BTreeSet<EventId>,
}

impl<T> Default for Scheduler<T> {
    fn default() -> Self {
        Self {
            real_time: 0,
            next_id: 0,
            clocks: BTreeMap::new(),
            pending: BTreeSet::new(),
            delivered: BTreeSet::new(),
            canceled: BTreeSet::new(),
        }
    }
}

impl<T> Scheduler<T> {
    pub fn real_time_us(&self) -> u64 {
        self.real_time
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    pub fn clock_readings(&self) -> Result<BTreeMap<ClockId, ClockReading>, ScheduleError> {
        self.clocks
            .iter()
            .map(|(id, clock)| {
                let local_time_us = u64::try_from(clock.phase / u128::from(RATE_SCALE))
                    .map_err(|_| ScheduleError::Overflow)?;
                Ok((
                    *id,
                    ClockReading {
                        local_time_us,
                        phase_millionths: (clock.phase % u128::from(RATE_SCALE)) as u32,
                        rate_ppm: clock.rate,
                    },
                ))
            })
            .collect()
    }
    pub fn add_clock(&mut self, id: ClockId, rate: u32) -> Result<(), ScheduleError> {
        if self.clocks.contains_key(&id) {
            return Err(ScheduleError::DuplicateClock(id));
        }
        self.clocks.insert(
            id,
            Clock {
                phase: 0,
                rate,
                tasks: BinaryHeap::new(),
            },
        );
        Ok(())
    }
    pub fn local_time_us(&self, id: ClockId) -> Result<u64, ScheduleError> {
        u64::try_from(
            self.clocks
                .get(&id)
                .ok_or(ScheduleError::UnknownClock(id))?
                .phase
                / u128::from(RATE_SCALE),
        )
        .map_err(|_| ScheduleError::Overflow)
    }
    pub fn set_rate(&mut self, id: ClockId, rate: u32) -> Result<(), ScheduleError> {
        self.clocks
            .get_mut(&id)
            .ok_or(ScheduleError::UnknownClock(id))?
            .rate = rate;
        Ok(())
    }
    pub fn schedule_after(
        &mut self,
        clock: ClockId,
        delay_us: u64,
        cause: Option<EventId>,
        payload: T,
    ) -> Result<EventId, ScheduleError> {
        if let Some(cause) = cause
            && !self.delivered.contains(&cause)
        {
            return Err(ScheduleError::UnknownCause(cause));
        }
        let id = self.next_id;
        let next = id.checked_add(1).ok_or(ScheduleError::Overflow)?;
        let clock = self
            .clocks
            .get_mut(&clock)
            .ok_or(ScheduleError::UnknownClock(clock))?;
        let deadline = clock
            .phase
            .checked_add(u128::from(delay_us) * u128::from(RATE_SCALE))
            .ok_or(ScheduleError::Overflow)?;
        clock.tasks.push(Task {
            deadline,
            id,
            cause,
            payload,
        });
        self.pending.insert(id);
        self.next_id = next;
        Ok(id)
    }
    pub fn schedule_batch_after(
        &mut self,
        clock: ClockId,
        delay_us: u64,
        cause: Option<EventId>,
        payloads: Vec<T>,
    ) -> Result<Vec<EventId>, ScheduleError> {
        if let Some(cause) = cause
            && !self.delivered.contains(&cause)
        {
            return Err(ScheduleError::UnknownCause(cause));
        }
        let count = u64::try_from(payloads.len()).map_err(|_| ScheduleError::Overflow)?;
        let next = self
            .next_id
            .checked_add(count)
            .ok_or(ScheduleError::Overflow)?;
        let clock_state = self
            .clocks
            .get_mut(&clock)
            .ok_or(ScheduleError::UnknownClock(clock))?;
        let deadline = clock_state
            .phase
            .checked_add(u128::from(delay_us) * u128::from(RATE_SCALE))
            .ok_or(ScheduleError::Overflow)?;
        let ids: Vec<_> = (self.next_id..next).collect();
        for (id, payload) in ids.iter().copied().zip(payloads) {
            clock_state.tasks.push(Task {
                deadline,
                id,
                cause,
                payload,
            });
            self.pending.insert(id);
        }
        self.next_id = next;
        Ok(ids)
    }
    pub fn cancel(&mut self, id: EventId) -> Result<(), ScheduleError> {
        if !self.pending.remove(&id) {
            return Err(ScheduleError::UnknownEvent(id));
        }
        self.canceled.insert(id);
        Ok(())
    }
    fn advance(&mut self, time: u64) -> Result<(), ScheduleError> {
        let delta = time
            .checked_sub(self.real_time)
            .ok_or(ScheduleError::TimeReversal)?;
        let phases = self
            .clocks
            .values()
            .map(|c| {
                c.phase
                    .checked_add(u128::from(delta) * u128::from(c.rate))
                    .filter(|phase| *phase / u128::from(RATE_SCALE) <= u128::from(u64::MAX))
                    .ok_or(ScheduleError::Overflow)
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (clock, phase) in self.clocks.values_mut().zip(phases) {
            clock.phase = phase;
        }
        self.real_time = time;
        Ok(())
    }
    /// Deliver the earliest eligible task, or advance to the caller's horizon.
    /// A zero-rate clock can deliver an already-due immediate task; future
    /// local deadlines stay paused. All equal real timestamps use global IDs.
    pub fn pop_before(&mut self, horizon_us: u64) -> Result<Option<Delivery<T>>, ScheduleError> {
        if horizon_us < self.real_time {
            return Err(ScheduleError::TimeReversal);
        }
        let mut next: Option<(u128, EventId, ClockId)> = None;
        for (id, clock) in &mut self.clocks {
            while let Some(task) = clock.tasks.peek_mut() {
                if !self.canceled.contains(&task.id) {
                    break;
                }
                let task = std::collections::binary_heap::PeekMut::pop(task);
                self.canceled.remove(&task.id);
            }
            let Some(task) = clock.tasks.peek() else {
                continue;
            };
            let delay = if task.deadline <= clock.phase {
                0
            } else if clock.rate == 0 {
                continue;
            } else {
                (task.deadline - clock.phase).div_ceil(u128::from(clock.rate))
            };
            // Several local deadlines can round to the same real tick when
            // the clock runs faster than real time. Order that entire bucket
            // by global IDs, rather than whichever local deadline is first.
            let eligible_phase = clock.phase + delay * u128::from(clock.rate);
            let event_id = clock
                .tasks
                .iter()
                .filter(|task| task.deadline <= eligible_phase && !self.canceled.contains(&task.id))
                .map(|task| task.id)
                .min()
                .ok_or(ScheduleError::InternalInvariant("eligible task missing"))?;
            let candidate = (u128::from(self.real_time) + delay, event_id, *id);
            if next.is_none_or(|n| candidate < n) {
                next = Some(candidate);
            }
        }
        let Some((time, event_id, clock)) =
            next.filter(|(time, _, _)| *time <= u128::from(horizon_us))
        else {
            self.advance(horizon_us)?;
            return Ok(None);
        };
        let time = u64::try_from(time).map_err(|_| ScheduleError::Overflow)?;
        let selected_clock = self
            .clocks
            .get(&clock)
            .ok_or(ScheduleError::InternalInvariant("selected clock missing"))?;
        let local = selected_clock.phase
            + u128::from(time - self.real_time) * u128::from(selected_clock.rate);
        let local_time_us =
            u64::try_from(local / u128::from(RATE_SCALE)).map_err(|_| ScheduleError::Overflow)?;
        self.advance(time)?;
        let clock_state = self
            .clocks
            .get_mut(&clock)
            .ok_or(ScheduleError::InternalInvariant(
                "selected clock missing after advance",
            ))?;
        let mut postponed = Vec::new();
        let mut selected = None;
        while let Some(task) = clock_state.tasks.peek_mut() {
            if task.deadline > clock_state.phase {
                break;
            }
            let task = std::collections::binary_heap::PeekMut::pop(task);
            if self.canceled.remove(&task.id) {
                continue;
            }
            if task.id == event_id {
                selected = Some(task);
            } else {
                postponed.push(task);
            }
        }
        clock_state.tasks.extend(postponed);
        let task = selected.ok_or(ScheduleError::InternalInvariant("selected task missing"))?;
        self.pending.remove(&event_id);
        self.delivered.insert(event_id);
        Ok(Some(Delivery {
            event_id,
            cause: task.cause,
            real_time_us: time,
            clock,
            local_time_us,
            payload: task.payload,
        }))
    }
}

#[cfg(test)]
mod tests;
