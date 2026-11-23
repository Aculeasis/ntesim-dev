//! Branch and Lucid Dream held coordinates, shot schedules and charge stages.
use super::*;

pub(super) const LUCID_SHOTS: [u64; 35] = [
    669_243, 834_435, 1_004_162, 1_168_673, 1_334_449, 1_498_395, 1_670_721, 1_834_352, 2_001_511,
    2_165_533, 2_332_957, 2_499_105, 2_666_801, 2_833_325, 3_001_303, 3_168_681, 3_333_705,
    3_501_114, 3_665_626, 3_832_992, 4_001_725, 4_167_952, 4_333_087, 4_501_369, 4_669_203,
    4_835_725, 5_002_131, 5_167_420, 5_331_114, 5_500_718, 5_666_907, 5_834_234, 5_998_520,
    6_169_262, 6_333_174,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BranchStage {
    Stage1,
    Stage2,
    Stage3Early,
    Stage3Full,
}

impl BranchStage {
    pub(crate) fn from_hold_us(hold_us: u64) -> Self {
        if hold_us < 600_000 {
            Self::Stage1
        } else if hold_us < 1_266_667 {
            Self::Stage2
        } else if hold_us < 2_066_667 {
            Self::Stage3Early
        } else {
            Self::Stage3Full
        }
    }

    pub(crate) fn tag_progress(self, hold_us: u64) -> u64 {
        match self {
            Self::Stage1 | Self::Stage2 | Self::Stage3Early => hold_us.saturating_add(316_437),
            Self::Stage3Full => 2_376_751,
        }
    }

    pub(crate) fn any_move(self, hold_us: u64) -> u64 {
        match self {
            Self::Stage1 | Self::Stage2 | Self::Stage3Early => hold_us.saturating_add(727_402),
            Self::Stage3Full => 2_799_293,
        }
    }

    pub(crate) fn jump(self, hold_us: u64) -> u64 {
        match self {
            Self::Stage1 | Self::Stage2 | Self::Stage3Early => hold_us.saturating_add(333_931),
            Self::Stage3Full => 0,
        }
    }
}

pub(super) fn cast_transform(ctx: &mut CastContext<'_>, command: usize) -> Result<bool, String> {
    ctx.record_hypothesis(MODEL)?;
    ctx.cancel_primary_delivery()?;
    let mut clock = ctx
        .actor
        .character
        .iroi_mut()?
        .clock
        .take()
        .unwrap_or_default();
    clock.lag = HitlagTracker::default();
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command,
        start: ctx.time_us,
        kind: Kind::Transform,
        source: OwnedAction::capture(ctx)?,
        groups: Default::default(),
        cooldown_started: true,
    });
    ctx.actor.ability_cooldowns.insert(
        skill::ABILITY.into(),
        ctx.time_us
            .checked_add(12_000_000)
            .ok_or("Iroi Skill CD overflow")?,
    );
    ctx.actor.switch_action_locked_until_us = ctx.actor.switch_action_locked_until_us.max(
        ctx.time_us
            .checked_add(600_000)
            .ok_or("Iroi switch lock overflow")?,
    );
    super::super::regression::voluntary_regress_all_allies(ctx)?;
    let state = ctx.actor.character.iroi_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.riposte_ready = false;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.iroi_mut()?.clock = Some(clock);
    Ok(true)
}

pub(super) fn fallback_to_normal1(
    ctx: &mut CastContext<'_>,
    command: usize,
) -> Result<bool, String> {
    ctx.record_hypothesis(MODEL)?;
    ctx.record_hypothesis(super::super::branch::HYPOTHESIS_ID)?;
    ctx.cancel_primary_delivery()?;
    let mut clock = ctx
        .actor
        .character
        .iroi_mut()?
        .clock
        .take()
        .unwrap_or_default();
    clock.lag = HitlagTracker::default();
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command,
        start: ctx.time_us,
        kind: Kind::Normal(0),
        source: OwnedAction::capture(ctx)?,
        groups: Default::default(),
        cooldown_started: false,
    });
    for &progress in profile::HITS[0] {
        push(
            ctx.queue,
            ctx.sequence,
            ctx.time_us,
            Event::Montage {
                owner: ctx.actor_index,
                command,
                progress,
                notify: Notify::NormalHit,
            },
        )?;
    }
    let state = ctx.actor.character.iroi_mut()?;
    state.normal_stage = 1;
    state.normal_started_us = Some(ctx.time_us);
    state.riposte_ready = false;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.iroi_mut()?.clock = Some(clock);
    Ok(true)
}

pub(super) fn cast_lucid_dream(
    ctx: &mut CastContext<'_>,
    command: usize,
    hold_duration: u64,
) -> Result<bool, String> {
    let in_dream = ctx.actor.character.iroi()?.dream_active;
    let cost = crate::simulation::ExactFraction::new(10, 1)?;
    let has_moon = in_dream
        || ctx
            .actor
            .character
            .iroi()?
            .moon
            .try_cmp(cost)
            .map_err(|e| e.to_string())?
            .is_ge();

    if !has_moon {
        return fallback_to_normal1(ctx, command);
    }

    ctx.record_hypothesis(MODEL)?;
    ctx.record_hypothesis(super::super::branch::HYPOTHESIS_ID)?;
    ctx.cancel_primary_delivery()?;
    if !in_dream {
        let state = ctx.actor.character.iroi_mut()?;
        state.moon = state.moon.sub(cost)?;
    }
    let mut clock = ctx
        .actor
        .character
        .iroi_mut()?
        .clock
        .take()
        .unwrap_or_default();
    clock.lag = HitlagTracker::default();
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command,
        start: ctx.time_us,
        kind: Kind::LucidDream {
            hold_us: hold_duration,
        },
        source: OwnedAction::capture(ctx)?,
        groups: Default::default(),
        cooldown_started: false,
    });
    for (idx, &offset) in LUCID_SHOTS.iter().enumerate() {
        if hold_duration > 0 && offset > hold_duration {
            break;
        }
        push(
            ctx.queue,
            ctx.sequence,
            ctx.time_us,
            Event::Montage {
                owner: ctx.actor_index,
                command,
                progress: offset,
                notify: Notify::LucidShot(idx),
            },
        )?;
    }
    let state = ctx.actor.character.iroi_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.riposte_ready = false;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.iroi_mut()?.clock = Some(clock);
    Ok(true)
}

pub(super) fn cast_daydream(
    ctx: &mut CastContext<'_>,
    command: usize,
    hold_duration: u64,
    packet_id: Option<&str>,
) -> Result<bool, String> {
    let in_dream = ctx.actor.character.iroi()?.dream_active;
    let cost = crate::simulation::ExactFraction::new(20, 1)?;
    let has_moon = in_dream
        || ctx
            .actor
            .character
            .iroi()?
            .moon
            .try_cmp(cost)
            .map_err(|e| e.to_string())?
            .is_ge();

    if !has_moon {
        return fallback_to_normal1(ctx, command);
    }

    ctx.record_hypothesis(MODEL)?;
    ctx.record_hypothesis(super::super::branch::HYPOTHESIS_ID)?;
    ctx.cancel_primary_delivery()?;
    if !in_dream {
        let state = ctx.actor.character.iroi_mut()?;
        state.moon = state.moon.sub(cost)?;
    }
    let stage = match packet_id {
        Some(super::super::branch::BRANCH1) => BranchStage::Stage1,
        Some(super::super::branch::BRANCH2) => BranchStage::Stage2,
        Some(super::super::branch::BRANCH3) => BranchStage::Stage3Full,
        _ => BranchStage::from_hold_us(hold_duration),
    };
    let mut clock = ctx
        .actor
        .character
        .iroi_mut()?
        .clock
        .take()
        .unwrap_or_default();
    clock.lag = HitlagTracker::default();
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command,
        start: ctx.time_us,
        kind: Kind::Branch {
            stage,
            hold_us: hold_duration,
        },
        source: OwnedAction::capture(ctx)?,
        groups: Default::default(),
        cooldown_started: false,
    });
    push(
        ctx.queue,
        ctx.sequence,
        ctx.time_us,
        Event::Montage {
            owner: ctx.actor_index,
            command,
            progress: stage.tag_progress(hold_duration),
            notify: Notify::BranchTag { stage },
        },
    )?;
    let state = ctx.actor.character.iroi_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.riposte_ready = false;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.iroi_mut()?.clock = Some(clock);
    Ok(true)
}

pub(super) fn cast_normal(
    ctx: &mut CastContext<'_>,
    command: usize,
    stage: usize,
) -> Result<bool, String> {
    ctx.record_hypothesis(MODEL)?;
    ctx.cancel_primary_delivery()?;
    let mut clock = ctx
        .actor
        .character
        .iroi_mut()?
        .clock
        .take()
        .unwrap_or_default();
    let mut live = HitlagTracker::default();
    for window in clock
        .lag
        .windows()
        .iter()
        .filter(|w| w.start_real_us.saturating_add(w.duration_real_us) > ctx.time_us)
    {
        live.add_window(window.clone())
            .map_err(|e| format!("Iroi inherited pause: {e:?}"))?;
    }
    clock.lag = live;
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command,
        start: ctx.time_us,
        kind: Kind::Normal(stage),
        source: OwnedAction::capture(ctx)?,
        groups: Default::default(),
        cooldown_started: false,
    });
    for &progress in profile::HITS[stage] {
        push(
            ctx.queue,
            ctx.sequence,
            ctx.time_us,
            Event::Montage {
                owner: ctx.actor_index,
                command,
                progress,
                notify: Notify::NormalHit,
            },
        )?;
    }
    if stage == 2 {
        push(
            ctx.queue,
            ctx.sequence,
            ctx.time_us,
            Event::Montage {
                owner: ctx.actor_index,
                command,
                progress: 268_779,
                notify: Notify::NormalActor,
            },
        )?;
    }
    ctx.actor.character.iroi_mut()?.normal_stage = (stage + 1) % 5;
    ctx.actor.character.iroi_mut()?.riposte_ready = false;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.iroi_mut()?.clock = Some(clock);
    Ok(true)
}

pub(super) fn cast_skill(ctx: &mut CastContext<'_>, command: usize) -> Result<bool, String> {
    ctx.record_hypothesis(MODEL)?;
    ctx.cancel_primary_delivery()?;
    let mut clock = ctx
        .actor
        .character
        .iroi_mut()?
        .clock
        .take()
        .unwrap_or_default();
    let mut live = HitlagTracker::default();
    for window in clock
        .lag
        .windows()
        .iter()
        .filter(|w| w.start_real_us.saturating_add(w.duration_real_us) > ctx.time_us)
    {
        live.add_window(window.clone())
            .map_err(|e| format!("Iroi inherited pause: {e:?}"))?;
    }
    clock.lag = live;
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command,
        start: ctx.time_us,
        kind: Kind::Skill,
        source: OwnedAction::capture(ctx)?,
        groups: Default::default(),
        cooldown_started: false,
    });
    let state = ctx.actor.character.iroi_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    ctx.actor.ability_cooldowns.remove(skill::ABILITY);
    skill::schedule(ctx)?;
    ctx.actor.character.iroi_mut()?.riposte_ready = false;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.iroi_mut()?.clock = Some(clock);
    Ok(true)
}

pub(super) fn cast_burst(ctx: &mut CastContext<'_>, command: usize) -> Result<bool, String> {
    ctx.record_hypothesis(MODEL)?;
    ctx.cancel_primary_delivery()?;
    let mut clock = ctx
        .actor
        .character
        .iroi_mut()?
        .clock
        .take()
        .unwrap_or_default();
    let mut live = HitlagTracker::default();
    for window in clock
        .lag
        .windows()
        .iter()
        .filter(|w| w.start_real_us.saturating_add(w.duration_real_us) > ctx.time_us)
    {
        live.add_window(window.clone())
            .map_err(|e| format!("Iroi inherited pause: {e:?}"))?;
    }
    clock.lag = live;
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command,
        start: ctx.time_us,
        kind: Kind::Burst,
        source: OwnedAction::capture(ctx)?,
        groups: Default::default(),
        cooldown_started: false,
    });
    let state = ctx.actor.character.iroi_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    burst::begin(ctx)?;
    ctx.actor.character.iroi_mut()?.riposte_ready = false;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.iroi_mut()?.clock = Some(clock);
    Ok(true)
}
