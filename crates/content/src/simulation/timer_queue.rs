//! Clock-indexed semantic deadlines. Frozen domains do not participate in the
//! absolute scheduler until a real lifecycle transition resumes their clock.
use super::events::{GameEventData, ScheduledGameEvent};
use crate::execution::CausalPriority;
use ntesim_core::kernel::timing::{ClockDomain, ClockProgression, RuntimeModeClock};
use std::cell::Cell;
use std::collections::{BTreeMap, BinaryHeap};

#[derive(Clone, Debug, PartialEq, Eq)]
struct TimedEvent {
    event: ScheduledGameEvent,
    payload_pause_offset: u64,
}
impl Ord for TimedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.event.cmp(&other.event)
    }
}
impl PartialOrd for TimedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Default)]
struct DomainQueue {
    elapsed_us: u64,
    running: bool,
    events: BinaryHeap<TimedEvent>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EventHead {
    pub(crate) time_us: u64,
    priority: CausalPriority,
    sequence: u64,
    pub(crate) kind: HeadKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeadKind {
    Admission,
    EnergyWait { index: usize },
    ResumeEnergyWait,
    Other,
}

impl EventHead {
    fn projected(now_us: u64, elapsed_us: u64, event: &ScheduledGameEvent) -> Self {
        let kind = match event.data {
            GameEventData::ActionAdmission { .. }
            | GameEventData::SwitchActor { .. }
            | GameEventData::LiveBranch { .. } => HeadKind::Admission,
            GameEventData::LiveEnergyWait { wait_index } => {
                HeadKind::EnergyWait { index: wait_index }
            }
            GameEventData::ResumeLiveEnergyWait => HeadKind::ResumeEnergyWait,
            _ => HeadKind::Other,
        };
        Self {
            time_us: projected_time(now_us, elapsed_us, event.time_us),
            priority: event.priority,
            sequence: event.seq,
            kind,
        }
    }
}

fn projected_time(now_us: u64, elapsed_us: u64, deadline: u64) -> u64 {
    now_us.saturating_add(deadline.saturating_sub(elapsed_us))
}

#[derive(Clone, Debug)]
pub(crate) struct EventQueue {
    now_us: u64,
    domains: BTreeMap<ClockDomain, DomainQueue>,
    policy: ClockProgression,
    next_head: Cell<Option<Option<(ClockDomain, EventHead)>>>,
    detached_domains: BTreeMap<u64, ClockDomain>,
    active_stop_actions: std::collections::BTreeSet<ntesim_core::kernel::timing::PauseOwner>,
    mode_contexts: BTreeMap<(usize, u64), ntesim_core::kernel::timing::PauseOwner>,
    bud_stop_exception: bool,
    replay_context: Option<ntesim_core::kernel::timing::PauseOwner>,
    event_replay_context: Option<ntesim_core::kernel::timing::PauseOwner>,
}

impl Default for EventQueue {
    fn default() -> Self {
        Self {
            now_us: 0,
            domains: BTreeMap::new(),
            policy: ClockProgression::default(),
            next_head: Cell::new(None),
            detached_domains: BTreeMap::new(),
            active_stop_actions: Default::default(),
            mode_contexts: Default::default(),
            bud_stop_exception: false,
            replay_context: None,
            event_replay_context: None,
        }
    }
}

impl EventQueue {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn sync(&mut self, clock: &RuntimeModeClock) -> Result<(), String> {
        let delta = clock
            .absolute_us()
            .checked_sub(self.now_us)
            .ok_or("queue clock reversal")?;
        let policy = clock.progression_policy();
        let changed = policy != self.policy;
        if delta != 0 || changed {
            self.next_head.set(None);
            for (domain, queue) in &mut self.domains {
                if queue.running && delta != 0 {
                    queue.elapsed_us = queue
                        .elapsed_us
                        .checked_add(delta)
                        .ok_or("timer coordinate overflow")?;
                }
                if changed {
                    queue.running = policy.progresses(*domain);
                }
            }
            self.now_us = clock.absolute_us();
            self.policy = policy;
        }
        self.domains.retain(|_, queue| !queue.events.is_empty());
        self.detached_domains.clear();
        Ok(())
    }
    pub(crate) fn push(&mut self, event: ScheduledGameEvent) {
        let domain = self.detached_domains.remove(&event.seq).unwrap_or_else(|| {
            let domain = self.domain(&event.data);
            if matches!(domain, ClockDomain::Action { .. })
                && let Some(context) = self.replay_context.or(self.event_replay_context)
            {
                ClockDomain::ReplayAction { context }
            } else if event.time_us == self.now_us
                && super::timer_policy::immediate_callback(&event.data)
            {
                ClockDomain::Absolute
            } else {
                domain
            }
        });
        self.push_on(domain, event);
    }
    pub(crate) fn begin_replay(
        &mut self,
        context: ntesim_core::kernel::timing::PauseOwner,
    ) -> Result<(), String> {
        if self.replay_context.replace(context).is_some() {
            return Err("recursive replay timer context".into());
        }
        Ok(())
    }
    pub(crate) fn end_replay(&mut self) {
        self.replay_context = None;
    }
    pub(crate) fn end_event(&mut self) {
        self.event_replay_context = None;
    }
    pub(crate) fn event_replay(&self) -> Option<ntesim_core::kernel::timing::PauseOwner> {
        self.event_replay_context
    }
    pub(crate) fn has_replay(&self, context: ntesim_core::kernel::timing::PauseOwner) -> bool {
        self.domains
            .get(&ClockDomain::ReplayAction { context })
            .is_some_and(|queue| !queue.events.is_empty())
    }
    pub(crate) fn cancel_action(
        &mut self,
        context: ntesim_core::kernel::timing::PauseOwner,
    ) -> usize {
        self.next_head.set(None);
        let mut cancelled = 0;
        self.domains.retain(|domain, queue| {
            if matches!(domain, ClockDomain::Action { context: owner, .. } | ClockDomain::ReplayAction { context: owner } if *owner == context) {
                cancelled += queue.events.iter().filter(|event| matches!(event.event.data, GameEventData::HitDelivery { .. })).count();
                false
            } else { true }
        });
        cancelled
    }
    fn domain(&self, event: &GameEventData) -> ClockDomain {
        if matches!(event, GameEventData::ReactionPistil(_)) {
            return ClockDomain::AutonomousActivity {
                advance_during_active_stop: self.bud_stop_exception,
            };
        }
        use crate::callbacks::ContentEvent;
        use crate::characters::hotori::{katana, timepiece};
        let mode = match event {
            GameEventData::Content(ContentEvent::HotoriTimepiece(
                timepiece::Event::ModeEntry { owner, generation }
                | timepiece::Event::ModeDeadline { owner, generation }
                | timepiece::Event::Playback { owner, generation },
            )) => Some((*owner, *generation)),
            GameEventData::Content(ContentEvent::HotoriKatana(
                katana::Event::Count {
                    owner,
                    mode_generation,
                }
                | katana::Event::Finisher {
                    owner,
                    mode_generation,
                }
                | katana::Event::Clear {
                    owner,
                    mode_generation,
                },
            )) => Some((*owner, *mode_generation)),
            _ => None,
        };
        if let Some(context) = mode.and_then(|mode| self.mode_contexts.get(&mode)) {
            return ClockDomain::Action {
                context: *context,
                advance_during_active_stop: true,
            };
        }
        super::timer_policy::domain(event)
    }
    pub(crate) fn set_bud_stop_exception(&mut self, enabled: bool) {
        if self.bud_stop_exception == enabled {
            return;
        }
        self.bud_stop_exception = enabled;
        let events = self.drain().collect::<Vec<_>>();
        for event in events {
            if matches!(event.data, GameEventData::ReactionPistil(_)) {
                self.detached_domains.remove(&event.seq);
            }
            self.push(event);
        }
    }
    pub(crate) fn bind_mode(
        &mut self,
        generation: u64,
        context: ntesim_core::kernel::timing::PauseOwner,
    ) {
        self.mode_contexts
            .insert((context.owner, generation), context);
        // Mode setup schedules its finite callbacks before admission commits.
        let events = self.drain().collect::<Vec<_>>();
        for event in events {
            let domain = self.domain(&event.data);
            self.detached_domains.remove(&event.seq);
            self.push_on(domain, event);
        }
    }
    pub(crate) fn push_on(&mut self, mut domain: ClockDomain, mut event: ScheduledGameEvent) {
        self.next_head.set(None);
        if let ClockDomain::Action { context, .. } = domain {
            domain = ClockDomain::Action {
                context,
                advance_during_active_stop: self.active_stop_actions.contains(&context),
            };
        }
        let queue = self.domains.entry(domain).or_insert_with(|| DomainQueue {
            running: self.policy.progresses(domain),
            elapsed_us: self.now_us,
            ..Default::default()
        });
        event.time_us = queue
            .elapsed_us
            .checked_add(event.time_us.saturating_sub(self.now_us))
            .expect("semantic deadline is bounded by its absolute input");
        queue.events.push(TimedEvent {
            event,
            payload_pause_offset: self.now_us - queue.elapsed_us,
        });
    }
    pub(crate) fn allow_active_stop(&mut self, context: ntesim_core::kernel::timing::PauseOwner) {
        if !self.active_stop_actions.insert(context) {
            return;
        }
        let old = ClockDomain::Action {
            context,
            advance_during_active_stop: false,
        };
        if let Some(queue) = self.domains.remove(&old) {
            self.next_head.set(None);
            for event in &queue.events {
                self.push_on(old, self.projected(&queue, event));
            }
        }
    }
    fn projected(&self, queue: &DomainQueue, timed: &TimedEvent) -> ScheduledGameEvent {
        Self::project_owned(self.now_us, queue.elapsed_us, timed.clone())
    }

    fn project_owned(now_us: u64, elapsed_us: u64, mut timed: TimedEvent) -> ScheduledGameEvent {
        let paused = now_us - elapsed_us - timed.payload_pause_offset;
        super::timer_policy::project_payload(&mut timed.event.data, paused);
        timed.event.time_us = projected_time(now_us, elapsed_us, timed.event.time_us);
        timed.event
    }

    fn next_head(&self) -> Option<(ClockDomain, EventHead)> {
        if let Some(cached) = self.next_head.get() {
            return cached;
        }
        let next = self
            .domains
            .iter()
            .filter_map(|(&domain, queue)| {
                let event = &queue.events.peek()?.event;
                (queue.running || event.time_us <= queue.elapsed_us).then(|| {
                    (
                        domain,
                        EventHead::projected(self.now_us, queue.elapsed_us, event),
                    )
                })
            })
            .min_by_key(|(_, head)| (head.time_us, head.priority, head.sequence));
        self.next_head.set(Some(next));
        next
    }
    pub(crate) fn peek(&self) -> Option<EventHead> {
        self.next_head().map(|(_, head)| head)
    }
    pub(crate) fn pop(&mut self) -> Option<ScheduledGameEvent> {
        let (domain, _) = self.next_head()?;
        self.next_head.set(None);
        self.event_replay_context = match domain {
            ClockDomain::ReplayAction { context } => Some(context),
            _ => None,
        };
        let queue = self.domains.get_mut(&domain)?;
        Some(Self::project_owned(
            self.now_us,
            queue.elapsed_us,
            queue.events.pop()?,
        ))
    }
    pub(crate) fn len(&self) -> usize {
        self.domains.values().map(|queue| queue.events.len()).sum()
    }
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = ScheduledGameEvent> + '_ {
        self.domains.values().flat_map(move |queue| {
            queue
                .events
                .iter()
                .map(move |event| self.projected(queue, event))
        })
    }
    pub(crate) fn drain(&mut self) -> std::vec::IntoIter<ScheduledGameEvent> {
        self.next_head.set(None);
        for (&domain, queue) in &self.domains {
            for event in &queue.events {
                self.detached_domains.insert(event.event.seq, domain);
            }
        }
        let events = self.iter().collect::<Vec<_>>();
        self.domains.clear();
        events.into_iter()
    }
    pub(crate) fn replace(&mut self, events: Vec<ScheduledGameEvent>) {
        self.next_head.set(None);
        self.domains.clear();
        self.extend(events);
    }
    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.next_head.set(None);
        self.domains.clear();
        self.detached_domains.clear();
    }
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&ScheduledGameEvent) -> bool) {
        self.next_head.set(None);
        let now = self.now_us;
        for queue in self.domains.values_mut() {
            let elapsed = queue.elapsed_us;
            queue.events.retain(|event| {
                let mut projected = event.event.clone();
                projected.time_us = now.saturating_add(event.event.time_us.saturating_sub(elapsed));
                super::timer_policy::project_payload(
                    &mut projected.data,
                    now - elapsed - event.payload_pause_offset,
                );
                keep(&projected)
            });
        }
    }
    #[cfg(test)]
    pub(crate) fn into_vec(self) -> Vec<ScheduledGameEvent> {
        self.iter().collect()
    }
    #[cfg(test)]
    pub(crate) fn into_sorted_vec(self) -> Vec<ScheduledGameEvent> {
        let mut events = self.into_vec();
        events.sort();
        events
    }
}
impl Extend<ScheduledGameEvent> for EventQueue {
    fn extend<T: IntoIterator<Item = ScheduledGameEvent>>(&mut self, events: T) {
        for event in events {
            self.push(event);
        }
    }
}
impl From<Vec<ScheduledGameEvent>> for EventQueue {
    fn from(events: Vec<ScheduledGameEvent>) -> Self {
        let mut queue = Self::new();
        queue.extend(events);
        queue
    }
}
