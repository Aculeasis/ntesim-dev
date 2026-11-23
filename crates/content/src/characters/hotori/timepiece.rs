//! World's Tide Timepiece account, rewind lease and Time Stop clock.
use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::ExactFraction as F;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::state::ActorRuntime;
use ntesim_core::execution::CausalPriority;
use ntesim_core::kernel::state::{ExactPool, ExactPoolSummary};

pub(crate) const MODEL: HypothesisId = HypothesisId::HotoriTimepieceV1;

pub(crate) const CLOCK_MODEL: HypothesisId = HypothesisId::HotoriTimeStopClockV1;

pub(crate) const PASSIVE_PER_SEC: u128 = 6;

pub(crate) const REWIND_US: u64 = 5_167_475;

pub(crate) const ENTRY_US: u64 = 2_183_333;

pub(crate) const DEADLINE_US: u64 = 14_183_333;

pub(crate) const BURST_SWITCH_LOCK_US: u64 = 4_783_333;

pub(crate) const PLAYBACK_US: u64 = 4_216_431;

pub(crate) const A3_DURATION_US: u64 = 10_000_000;

pub(crate) const A3_ICD_US: u64 = 60_000_000;

const SKILL_COST: F = F {
    numerator: 60,
    denominator: 1,
};

const BURST_COST: F = F {
    numerator: 120,
    denominator: 1,
};

pub(crate) fn initial_pool(initial: Option<crate::formula::Ratio>) -> Result<ExactPool, String> {
    let value = initial
        .map(F::from_ratio)
        .transpose()?
        .unwrap_or(F::new(0, 1)?);
    if value.try_cmp(BURST_COST)?.is_gt() {
        return Err("start_timepiece exceeds Hotori's 120 Timepiece capacity.".into());
    }
    ExactPool::new(BURST_COST, value)
}

const A4_GAIN: F = F {
    numerator: 10,
    denominator: 1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Mode {
    pub(crate) entry_us: u64,
    pub(crate) end_us: u64,
    pub(crate) records_n: u32,
    /// Root Burst command whose weapon family bonuses cover this mode.
    pub(crate) root_command_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Record {
    pub(crate) owner: usize,
    pub(crate) command_index: usize,
    pub(crate) time_us: u64,
    pub(crate) ability_id: String,
    pub(crate) array_index: usize,
    pub(crate) damage_effect_id: String,
    pub(crate) atk_rate: ntesim_core::formula::Ratio,
    pub(crate) scaling_stat: crate::simulation::NativeScalingStat,
    pub(crate) hit_delay_us: u64,
    pub(crate) metadata: crate::simulation::metadata::ActionPacketMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct Timepiece {
    pub(crate) pool: ExactPool,
    pub(crate) settled_until_us: u64,
    pub(crate) passive: bool,
    pub(crate) rewind_until_us: Option<u64>,
    pub(crate) rewind_generation: u64,
    pub(crate) a3_until_us: Option<u64>,
    pub(crate) a3_icd_until_us: u64,
    pub(crate) a3_generation: u64,
    pub(crate) mode: Option<Mode>,
    pub(crate) mode_generation: u64,
    pub(crate) katana_count: u32,
    pub(crate) katana_next_input_us: Option<u64>,
    pub(crate) kills: u32,
    pub(crate) switch_locked_until_us: u64,
    pub(crate) breakthrough: u8,
    pub(crate) records: Vec<Record>,
}

impl Default for Timepiece {
    fn default() -> Self {
        Self {
            pool: initial_pool(None).expect("default Timepiece balance is valid"),
            settled_until_us: 0,
            passive: true,
            rewind_until_us: None,
            rewind_generation: 0,
            a3_until_us: None,
            a3_icd_until_us: 0,
            a3_generation: 0,
            mode: None,
            mode_generation: 0,
            katana_count: 0,
            katana_next_input_us: None,
            kills: 0,
            switch_locked_until_us: 0,
            breakthrough: 0,
            records: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    RewindSettled { owner: usize, generation: u64 },
    A3Complete { owner: usize, generation: u64 },
    ModeEntry { owner: usize, generation: u64 },
    ModeDeadline { owner: usize, generation: u64 },
    Playback { owner: usize, generation: u64 },
}

impl Timepiece {
    pub(crate) fn rewinding(&self, now: u64) -> bool {
        self.rewind_until_us.is_some_and(|end| now < end)
    }

    /// A committed World's Tide occupies setup and the active interval; the
    /// mode is cleared at the finisher, the resource deadline or death.
    pub(crate) fn mode_busy(&self, now: u64) -> bool {
        self.mode.as_ref().is_some_and(|mode| now < mode.end_us)
    }

    pub(crate) fn mode_active(&self, now: u64) -> bool {
        self.mode
            .as_ref()
            .is_some_and(|mode| now >= mode.entry_us && now < mode.end_us)
    }

    fn grant_recovery(
        &mut self,
        from_us: u64,
        to_us: u64,
        rate_per_sec: u128,
    ) -> Result<(), String> {
        let elapsed = to_us.saturating_sub(from_us);
        if elapsed == 0 {
            return Ok(());
        }
        self.pool
            .grant(F::new(u128::from(elapsed) * rate_per_sec, 1_000_000)?)?;
        Ok(())
    }

    pub(crate) fn settle_to(&mut self, now_us: u64) -> Result<(), String> {
        if now_us <= self.settled_until_us {
            return Ok(());
        }
        let mut cursor = self.settled_until_us;
        if let Some(a3_end) = self.a3_until_us {
            let segment_end = a3_end.min(now_us);
            if segment_end > cursor {
                if self.passive {
                    self.grant_recovery(cursor, segment_end, PASSIVE_PER_SEC)?;
                }
                self.grant_recovery(cursor, segment_end, PASSIVE_PER_SEC)?;
                cursor = segment_end;
            }
            if segment_end >= a3_end {
                self.a3_until_us = None;
            }
        }
        if cursor < now_us {
            if self.passive {
                self.grant_recovery(cursor, now_us, PASSIVE_PER_SEC)?;
            }
            cursor = now_us;
        }
        self.settled_until_us = cursor;
        Ok(())
    }

    pub(crate) fn projected(&self, now_us: u64) -> Result<Self, String> {
        let mut projected = self.clone();
        projected.settle_to(now_us)?;
        Ok(projected)
    }

    pub(crate) fn summary(&self, now_us: u64) -> Result<ExactPoolSummary, String> {
        self.projected(now_us)?.pool.summary()
    }
}

fn schedule_content(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    time_us: u64,
    priority: CausalPriority,
    event: Event,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us,
        priority,
        seq: *sequence,
        data: GameEventData::Content(ContentEvent::HotoriTimepiece(event)),
    });
    Ok(())
}

fn rejection(message: String) -> crate::simulation::rejection::AdmissionFailure {
    crate::simulation::rejection::AdmissionFailure {
        reason: crate::simulation::GameAdmissionReason::Other {
            message: message.clone(),
        },
        message,
    }
}

pub(crate) fn resource_admission_failure(
    actor: &ActorRuntime,
    ability: &str,
    now_us: u64,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if ability != "GA_Jin_Skill" && ability != "GA_Jin_UltraSkill" {
        return None;
    }
    let state = match &actor.character {
        crate::characters::RuntimeState::Hotori(state) => state,
        _ => return None,
    };
    let timepiece = match state.timepiece.projected(now_us) {
        Ok(timepiece) => timepiece,
        Err(message) => return Some(rejection(message)),
    };
    if ability == "GA_Jin_Skill" {
        if timepiece.mode_busy(now_us) {
            return Some(rejection(
                "Present Replay cannot be admitted during World's Tide".into(),
            ));
        }
        if timepiece.rewinding(now_us) {
            return Some(rejection(
                "Present Replay cannot be admitted while its rewind is active".into(),
            ));
        }
        match timepiece.pool.current().try_cmp(SKILL_COST) {
            Ok(ordering) if ordering.is_gt() => None,
            Ok(_) => Some(rejection(format!(
                "Present Replay requires more than 60 Timepiece; available {}",
                timepiece.pool.current().to_f64()
            ))),
            Err(message) => Some(rejection(String::from(message))),
        }
    } else {
        if timepiece.mode_busy(now_us) {
            return Some(rejection(
                "World's Tide cannot be admitted during an active Time Stop".into(),
            ));
        }
        if timepiece.rewinding(now_us) {
            return Some(rejection(
                "World's Tide cannot be admitted while Present Replay rewinds".into(),
            ));
        }
        match timepiece.pool.current().try_cmp(BURST_COST) {
            Ok(ordering) if ordering.is_eq() => None,
            Ok(_) => Some(rejection(format!(
                "World's Tide requires exactly 120 Timepiece; available {}",
                timepiece.pool.current().to_f64()
            ))),
            Err(message) => Some(rejection(String::from(message))),
        }
    }
}

/// World's Tide forbids leaving the field until the active mode ends.
pub(crate) fn switch_lock(
    actor: &ActorRuntime,
    ability_id: &str,
    now_us: u64,
) -> Option<(u64, HypothesisId)> {
    if ability_id != "switch_actor" {
        return None;
    }
    let state = match &actor.character {
        crate::characters::RuntimeState::Hotori(state) => state,
        _ => return None,
    };
    let timepiece = &state.timepiece;
    if let Some(mode) = &timepiece.mode
        && now_us < mode.end_us
    {
        return Some((mode.end_us, CLOCK_MODEL));
    }
    (now_us < timepiece.switch_locked_until_us)
        .then_some((timepiece.switch_locked_until_us, CLOCK_MODEL))
}

pub(crate) fn commit_skill(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let now_us = ctx.time_us;
    let owner = ctx.actor_index;
    let deadline_us = now_us
        .checked_add(REWIND_US)
        .ok_or("Present Replay rewind deadline overflow")?;
    let generation = {
        let state = ctx.actor.character.hotori_mut()?;
        let timepiece = &mut state.timepiece;
        timepiece.settle_to(now_us)?;
        if timepiece.mode_busy(now_us) {
            return Err("Present Replay cannot be admitted during World's Tide".into());
        }
        if timepiece.rewinding(now_us) {
            return Err("Present Replay cannot be admitted while its rewind is active".into());
        }
        match timepiece.pool.current().try_cmp(SKILL_COST)? {
            ordering if ordering.is_gt() => {}
            _ => return Err("Present Replay requires more than 60 Timepiece".into()),
        }
        timepiece.a3_until_us = None;
        timepiece.a3_generation = timepiece.a3_generation.saturating_add(1);
        timepiece.pool.spend_up_to(SKILL_COST)?;
        timepiece.passive = false;
        timepiece.rewind_until_us = Some(deadline_us);
        timepiece.rewind_generation = timepiece.rewind_generation.saturating_add(1);
        timepiece.records.clear();
        timepiece.rewind_generation
    };
    schedule_content(
        ctx.queue,
        ctx.sequence,
        deadline_us,
        CausalPriority::ModePause,
        Event::RewindSettled { owner, generation },
    )?;
    crate::runtime_api::record_hypothesis_id(ctx.request, ctx.execution_assumptions, MODEL)?;
    Ok(())
}

pub(crate) fn commit_burst(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let now_us = ctx.time_us;
    let owner = ctx.actor_index;
    let entry_us = now_us
        .checked_add(ENTRY_US)
        .ok_or("World's Tide entry deadline overflow")?;
    let deadline_us = now_us
        .checked_add(DEADLINE_US)
        .ok_or("World's Tide resource deadline overflow")?;
    let switch_lock_us = now_us
        .checked_add(BURST_SWITCH_LOCK_US)
        .ok_or("World's Tide switch lock overflow")?;
    let playback_us = now_us
        .checked_add(PLAYBACK_US)
        .ok_or("World's Tide playback deadline overflow")?;
    let generation = {
        let state = ctx.actor.character.hotori_mut()?;
        let records_n =
            u32::try_from(state.timepiece.records.len()).map_err(|_| "Too many records")?;
        let timepiece = &mut state.timepiece;
        timepiece.settle_to(now_us)?;
        if timepiece.mode_busy(now_us) {
            return Err("World's Tide cannot be admitted during an active Time Stop".into());
        }
        if timepiece.rewinding(now_us) {
            return Err("World's Tide cannot be admitted while Present Replay rewinds".into());
        }
        match timepiece.pool.current().try_cmp(BURST_COST)? {
            ordering if ordering.is_eq() => {}
            _ => return Err("World's Tide requires exactly 120 Timepiece".into()),
        }
        timepiece.a3_until_us = None;
        timepiece.a3_generation = timepiece.a3_generation.saturating_add(1);
        timepiece.pool.spend_up_to(BURST_COST)?;
        timepiece.passive = false;
        timepiece.kills = 0;
        timepiece.katana_count = 0;
        timepiece.katana_next_input_us = None;
        timepiece.mode_generation = timepiece.mode_generation.saturating_add(1);
        timepiece.mode = Some(Mode {
            entry_us,
            end_us: deadline_us,
            records_n,
            root_command_index: ctx.command_index,
        });
        timepiece.switch_locked_until_us = switch_lock_us;
        timepiece.mode_generation
    };
    schedule_content(
        ctx.queue,
        ctx.sequence,
        entry_us,
        CausalPriority::ModePause,
        Event::ModeEntry { owner, generation },
    )?;
    schedule_content(
        ctx.queue,
        ctx.sequence,
        deadline_us,
        CausalPriority::ModePause,
        Event::ModeDeadline { owner, generation },
    )?;
    schedule_content(
        ctx.queue,
        ctx.sequence,
        playback_us,
        CausalPriority::ModePause,
        Event::Playback { owner, generation },
    )?;
    crate::runtime_api::record_hypothesis_id(ctx.request, ctx.execution_assumptions, MODEL)?;
    Ok(())
}

pub(crate) fn record_trace(
    ctx: &mut CallbackContext<'_>,
    phase: &str,
    event_kind: &str,
    from_actor: Option<usize>,
    to_actor: Option<usize>,
    data: Option<crate::simulation::GameEventPayload>,
) -> Result<(), String> {
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: phase.into(),
        event_kind: event_kind.into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor,
        to_actor,
        energy_change: None,
        energy_fanout: None,
        data,
    });
    Ok(())
}

fn rewind_settled(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let now_us = ctx.state.current_time_us;
    if ctx.request.actors[owner].character_id != "character:1052" || ctx.state.actors[owner].hp == 0
    {
        return Ok(());
    }
    let a3_selected = ctx.request.actors[owner].awakening.has_effect(3);
    let (started_a3, current) = {
        let state = ctx.state.actors[owner].character.hotori_mut()?;
        let timepiece = &mut state.timepiece;
        if timepiece.rewind_generation != generation || timepiece.rewind_until_us != Some(now_us) {
            return Ok(());
        }
        timepiece.settle_to(now_us)?;
        timepiece.rewind_until_us = None;
        timepiece.rewind_generation = timepiece.rewind_generation.saturating_add(1);
        timepiece.passive = true;
        let mut started = None;
        if a3_selected && timepiece.a3_icd_until_us <= now_us {
            let a3_end = now_us
                .checked_add(A3_DURATION_US)
                .ok_or("Momentary Gaze deadline overflow")?;
            timepiece.a3_until_us = Some(a3_end);
            timepiece.a3_icd_until_us = now_us
                .checked_add(A3_ICD_US)
                .ok_or("Momentary Gaze cooldown overflow")?;
            timepiece.a3_generation = timepiece.a3_generation.saturating_add(1);
            started = Some((a3_end, timepiece.a3_generation));
        }
        (started, timepiece.pool.current())
    };
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    record_trace(
        ctx,
        "ModePause",
        "hotori_rewind_settled",
        Some(owner),
        Some(owner),
        Some(crate::simulation::GameEventPayload {
            exact_amount: Some(current),
            exact_capacity: Some(BURST_COST),
            ..Default::default()
        }),
    )?;
    if let Some((a3_end, a3_generation)) = started_a3 {
        schedule_content(
            ctx.queue,
            ctx.sequence,
            a3_end,
            CausalPriority::ModePause,
            Event::A3Complete {
                owner,
                generation: a3_generation,
            },
        )?;
        record_trace(
            ctx,
            "ModePause",
            "hotori_a3_started",
            Some(owner),
            Some(owner),
            Some(crate::simulation::GameEventPayload {
                expires_at_us: Some(a3_end),
                ..Default::default()
            }),
        )?;
    }
    Ok(())
}

fn a3_complete(ctx: &mut CallbackContext<'_>, owner: usize, generation: u64) -> Result<(), String> {
    let now_us = ctx.state.current_time_us;
    if ctx.request.actors[owner].character_id != "character:1052" {
        return Ok(());
    }
    let current = {
        let state = ctx.state.actors[owner].character.hotori_mut()?;
        let timepiece = &mut state.timepiece;
        if timepiece.a3_generation != generation || timepiece.a3_until_us.is_none() {
            return Ok(());
        }
        timepiece.settle_to(now_us)?;
        timepiece.a3_until_us = None;
        timepiece.pool.current()
    };
    record_trace(
        ctx,
        "ModePause",
        "hotori_a3_complete",
        Some(owner),
        Some(owner),
        Some(crate::simulation::GameEventPayload {
            exact_amount: Some(current),
            exact_capacity: Some(BURST_COST),
            ..Default::default()
        }),
    )
}

fn mode_entry(ctx: &mut CallbackContext<'_>, owner: usize, generation: u64) -> Result<(), String> {
    let now_us = ctx.state.current_time_us;
    if ctx.request.actors[owner].character_id != "character:1052" || ctx.state.actors[owner].hp == 0
    {
        return Ok(());
    }
    let (deadline_us, records_n, root_command_index) = {
        let state = &ctx.state.actors[owner].character;
        let crate::characters::RuntimeState::Hotori(state) = state else {
            return Ok(());
        };
        let timepiece = &state.timepiece;
        if timepiece.mode_generation != generation {
            return Ok(());
        }
        let Some(mode) = &timepiece.mode else {
            return Ok(());
        };
        if now_us < mode.entry_us {
            return Ok(());
        }
        (mode.end_us, mode.records_n, mode.root_command_index)
    };
    ctx.state
        .mode_clock
        .transition_pause(
            ntesim_core::kernel::timing::PauseOwner {
                owner,
                generation: root_command_index as u64,
            },
            ntesim_core::kernel::timing::ModePauseKind::ActiveStop,
            now_us,
        )
        .map_err(|e| e.to_string())?;
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        CLOCK_MODEL,
    )?;
    record_trace(
        ctx,
        "ModePause",
        "hotori_time_stop_enter",
        Some(owner),
        Some(owner),
        Some(crate::simulation::GameEventPayload {
            expires_at_us: Some(deadline_us),
            count: Some(usize::try_from(records_n).map_err(|_| "Hotori records count overflow")?),
            ..Default::default()
        }),
    )
}

fn end_mode(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
    event_kind: &str,
    reason: &str,
) -> Result<(), String> {
    let now_us = ctx.state.current_time_us;
    if ctx.request.actors[owner].character_id != "character:1052" {
        return Ok(());
    }
    let alive = ctx.state.actors[owner].hp > 0;
    let (current, _entry_us, root_command_index) = {
        let state = ctx.state.actors[owner].character.hotori_mut()?;
        let timepiece = &mut state.timepiece;
        if timepiece.mode_generation != generation || timepiece.mode.is_none() {
            return Ok(());
        }
        timepiece.settle_to(now_us)?;
        let entry_us = timepiece.mode.as_ref().map(|mode| mode.entry_us);
        let root_command_index = timepiece.mode.as_ref().map(|mode| mode.root_command_index);
        timepiece.mode = None;
        timepiece.mode_generation = timepiece.mode_generation.saturating_add(1);
        timepiece.passive = alive;
        timepiece.katana_count = 0;
        timepiece.katana_next_input_us = None;
        (timepiece.pool.current(), entry_us, root_command_index)
    };
    if let Some(command) = root_command_index {
        ctx.state
            .mode_clock
            .end_pause(
                ntesim_core::kernel::timing::PauseOwner {
                    owner,
                    generation: command as u64,
                },
                now_us,
            )
            .map_err(|e| e.to_string())?;
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        CLOCK_MODEL,
    )?;
    record_trace(
        ctx,
        "ModePause",
        event_kind,
        Some(owner),
        Some(owner),
        Some(crate::simulation::GameEventPayload {
            reason: Some(reason.into()),
            exact_amount: Some(current),
            exact_capacity: Some(BURST_COST),
            active: Some(alive),
            ..Default::default()
        }),
    )
}

/// Explicit scenario cancellation discards the committed reservation.
pub(crate) fn interrupt(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    let now = ctx.state.current_time_us;
    let before = ctx.state.actors[owner].locked_until_us;
    let state = ctx.state.actors[owner].character.hotori_mut()?;
    // This is the explicit forced-cancellation input, independent of ordinary
    // time-control resistance. The reservation is never returned to the pool.
    let setup = state
        .timepiece
        .mode
        .as_ref()
        .map(|mode| (state.timepiece.mode_generation, mode.root_command_index));
    let mut cancelled = 0;
    if let Some((generation, command)) = setup {
        end_mode(
            ctx,
            owner,
            generation,
            "hotori_time_stop_cancelled",
            "forced_cancellation",
        )?;
        cancelled = ctx
            .queue
            .cancel_action(ntesim_core::kernel::timing::PauseOwner {
                owner,
                generation: command as u64,
            });
        ctx.state.actors[owner].locked_until_us = now;
        ctx.state.actors[owner]
            .character
            .hotori_mut()?
            .timepiece
            .switch_locked_until_us = now;
    }
    Ok(crate::simulation::GameActorInterruptReceipt {
        time_us: now,
        actor_index: owner,
        event_index,
        interrupted_command_index: setup.map(|(_, command)| command),
        cancelled_hit_deliveries: cancelled,
        field_cancelled: false,
        recovery_before_us: before,
        recovery_after_us: ctx.state.actors[owner].locked_until_us,
        skill_cooldown_before_us: None,
        skill_cooldown_after_us: None,
    })
}

/// The Katana finisher ends the mode at its own damage delivery.
pub(crate) fn finisher_delivered(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    end_mode(
        ctx,
        owner,
        generation,
        "hotori_time_stop_finisher",
        "finisher",
    )
}

pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::RewindSettled { owner, generation } => rewind_settled(ctx, owner, generation),
        Event::A3Complete { owner, generation } => a3_complete(ctx, owner, generation),
        Event::ModeEntry { owner, generation } => mode_entry(ctx, owner, generation),
        Event::ModeDeadline { owner, generation } => end_mode(
            ctx,
            owner,
            generation,
            "hotori_time_stop_expired",
            "deadline",
        ),
        Event::Playback { owner, generation } => super::replay::playback(ctx, owner, generation),
    }
}

pub(crate) fn defeat(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.request.actors[owner].character_id != "character:1052" {
        return Ok(());
    }
    let now_us = ctx.state.current_time_us;
    let (current, _entry_us, root_command_index) = {
        let state = ctx.state.actors[owner].character.hotori_mut()?;
        let timepiece = &mut state.timepiece;
        timepiece.settle_to(now_us)?;
        timepiece.passive = false;
        timepiece.a3_until_us = None;
        timepiece.a3_generation = timepiece.a3_generation.saturating_add(1);
        timepiece.rewind_until_us = None;
        timepiece.rewind_generation = timepiece.rewind_generation.saturating_add(1);
        let entry_us = timepiece.mode.as_ref().map(|mode| mode.entry_us);
        let root_command_index = timepiece.mode.as_ref().map(|mode| mode.root_command_index);
        timepiece.mode = None;
        timepiece.mode_generation = timepiece.mode_generation.saturating_add(1);
        timepiece.katana_count = 0;
        timepiece.katana_next_input_us = None;
        timepiece.records.clear();
        (timepiece.pool.current(), entry_us, root_command_index)
    };
    if let Some(command) = root_command_index {
        ctx.state
            .mode_clock
            .end_pause(
                ntesim_core::kernel::timing::PauseOwner {
                    owner,
                    generation: command as u64,
                },
                now_us,
            )
            .map_err(|e| e.to_string())?;
    }
    record_trace(
        ctx,
        "SettlementAndDeath",
        "hotori_timepiece_death",
        Some(owner),
        Some(owner),
        Some(crate::simulation::GameEventPayload {
            exact_amount: Some(current),
            exact_capacity: Some(BURST_COST),
            ..Default::default()
        }),
    )
}

pub(crate) fn on_vitals_changed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    changes: &[crate::simulation::vitals::HpChange],
) -> Result<(), String> {
    if ctx.request.actors[owner].character_id != "character:1052" {
        return Ok(());
    }
    let Some(change) = changes.iter().find(|change| change.actor_index == owner) else {
        return Ok(());
    };
    if change.before == 0 && change.after > 0 {
        let state = ctx.state.actors[owner].character.hotori_mut()?;
        state.timepiece.settle_to(ctx.state.current_time_us)?;
        state.timepiece.passive = true;
    }
    Ok(())
}

pub(crate) fn on_team_target_defeated(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    target: usize,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
) -> Result<(), String> {
    let now_us = state.current_time_us;
    for (owner_idx, (config, actor)) in request
        .actors
        .iter()
        .zip(state.actors.iter_mut())
        .enumerate()
    {
        if config.character_id != "character:1052"
            || actor.hp == 0
            || !config.awakening.has_effect(4)
        {
            continue;
        }
        let crate::characters::RuntimeState::Hotori(hotori) = &mut actor.character else {
            continue;
        };
        let timepiece = &mut hotori.timepiece;
        if timepiece.kills >= 3 || timepiece.rewinding(now_us) || timepiece.mode_busy(now_us) {
            continue;
        }
        timepiece.settle_to(now_us)?;
        timepiece.kills = timepiece.kills.saturating_add(1);
        let (applied, excess) = timepiece.pool.grant(A4_GAIN)?;
        crate::runtime_api::record_hypothesis_id(
            request,
            &mut state.execution_model_assumptions,
            MODEL,
        )?;
        trace.push(crate::simulation::GameEventReceipt {
            time_us: now_us,
            phase: "ProcCallback".into(),
            event_kind: "hotori_a4_gain".into(),
            active_actor: state.active_actor_index,
            hit_index: None,
            from_actor: Some(owner_idx),
            to_actor: Some(owner_idx),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                target_index: Some(target),
                stacks: Some(timepiece.kills),
                max_stacks: Some(3),
                exact_amount: Some(applied),
                exact_overflow: Some(excess),
                ..Default::default()
            }),
        });
    }
    Ok(())
}
