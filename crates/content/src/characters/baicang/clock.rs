//! Causal caster progress; persistent effects follow their declared shared clock domains.
use crate::HypothesisId;
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::{GameScenarioRequest, events::GameEventData};
use ntesim_core::kernel::timing::{HitlagTracker, HitlagWindow};

mod projection;
use projection::{Action, project};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::BaicangActorHitstopV1;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    aim_target: usize,
    tracker: HitlagTracker,
    current: Option<Action>,
    contact_time: Option<u64>,
    contacts: std::collections::BTreeSet<String>,
}

impl State {
    pub(crate) fn end_action(&mut self) {
        self.current = None;
    }

    pub(super) fn emitted_at(&self, seq: u64) -> Option<u64> {
        self.current.as_ref()?.emitted_at(seq)
    }

    pub(super) fn first_skill_emission(&self) -> Option<u64> {
        self.current.as_ref()?.first_skill_emission()
    }
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if accepted(request) && !super::words::accepted(request) {
        return Err("baicang.actor_hitstop_v1 requires baicang.power_words_v1".into());
    }
    Ok(())
}

pub(crate) fn capture(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let action = Action::capture(ctx)?;
    let mut clock = std::mem::take(&mut ctx.actor.character.baicang_mut()?.clock);
    clock.aim_target = ctx.target_index;
    // Past windows are only needed until the next action starts. Carry the
    // unexpired tail, including a projectile pause from an earlier command.
    let mut carried = HitlagTracker::default();
    for window in clock.tracker.windows() {
        if window.start_real_us.saturating_add(window.duration_real_us) > ctx.time_us {
            carried
                .add_window(window.clone())
                .map_err(|e| format!("Baicang hitstop: {e:?}"))?;
        }
    }
    clock.tracker = carried;
    clock.current = Some(action);
    project(
        ctx.actor,
        ctx.queue,
        &mut clock,
        ctx.actor_index,
        ctx.time_us,
    )?;
    ctx.actor.character.baicang_mut()?.clock = clock;
    Ok(())
}

fn pause(effect: &str) -> Option<(u64, u64, u32)> {
    Some(match effect {
        "GE_Player_Cang_Melee1_Damage"
        | "GE_Player_Cang_Melee2_Damage"
        | "GE_Player_Cang_Melee5_Damage" => (0, 33_000, 100_000),
        "GE_Player_Cang_Melee3_Damage" => (40_000, 67_000, 100_000),
        "GE_Player_Cang_Melee4_Damage" => (33_000, 33_000, 100_000),
        super::branches::BRANCH1_PACKET | super::branches::BRANCH1_1_PACKET => {
            (0, 100_000, 300_000)
        }
        super::riposte::PACKET_ID => (0, 300_000, 10_000),
        "GE_Player_Cang_QTE_Damage" => (0, 300_000, 300_000),
        _ => return None,
    })
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let Some((delay, duration, rate)) = pause(ctx.damage_effect_id) else {
        return Ok(());
    };
    let clock = &mut ctx.actor.character.baicang_mut()?.clock;
    if clock.contact_time != Some(ctx.time_us) {
        clock.contact_time = Some(ctx.time_us);
        clock.contacts.clear();
    }
    // One source contact time, irrespective of AoE recipients or coincident actors.
    if !clock.contacts.insert(ctx.damage_effect_id.into()) {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        ctx.execution_assumptions,
        HYPOTHESIS_ID,
    )?;
    if delay != 0 {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: ctx
                    .time_us
                    .checked_add(delay)
                    .ok_or("Baicang hitstop start overflow")?,
                priority: ntesim_core::execution::CausalPriority::ModePause,
                seq: *ctx.sequence,
                data: GameEventData::Content(crate::callbacks::ContentEvent::BaicangHitstop {
                    command_index: ctx.command_index,
                    actor_index: ctx.actor_index,
                    duration_us: duration,
                    rate_ppm: rate,
                }),
            });
        return Ok(());
    }
    apply(
        ctx.actor,
        ctx.queue,
        ctx.trace,
        Pause {
            index: ctx.actor_index,
            now: ctx.time_us,
            duration,
            rate,
            active_actor: ctx.active_actor_index,
        },
    )
}

pub(crate) fn event(
    ctx: &mut CallbackContext<'_>,
    actor: usize,
    duration: u64,
    rate: u32,
) -> Result<(), String> {
    if ctx.state.actors[actor].hp == 0 {
        return Ok(());
    }
    apply(
        &mut ctx.state.actors[actor],
        ctx.queue,
        ctx.trace,
        Pause {
            index: actor,
            now: ctx.state.current_time_us,
            duration,
            rate,
            active_actor: ctx.state.active_actor_index,
        },
    )
}

pub(crate) fn execute(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
) -> Result<(), String> {
    // Execute pauses compare the defeated dummy with the owner's locked aim.
    if accepted(ctx.request)
        && target
            == ctx.state.actors[owner]
                .character
                .baicang_mut()?
                .clock
                .aim_target
    {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            HYPOTHESIS_ID,
        )?;
        event(ctx, owner, 300_000, 300_000)?;
    }
    Ok(())
}

struct Pause {
    index: usize,
    now: u64,
    duration: u64,
    rate: u32,
    active_actor: usize,
}

fn apply(
    actor: &mut crate::simulation::state::ActorRuntime,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
    pause: Pause,
) -> Result<(), String> {
    let Pause {
        index,
        now,
        duration,
        rate,
        active_actor,
    } = pause;
    let mut clock = std::mem::take(&mut actor.character.baicang_mut()?.clock);
    let before = actor.locked_until_us;
    clock
        .tracker
        .add_window(HitlagWindow {
            actor_index: index,
            start_real_us: now,
            duration_real_us: duration,
            dilated_rate_ppm: rate,
        })
        .map_err(|e| format!("Baicang hitstop: {e:?}"))?;
    project(actor, queue, &mut clock, index, now)?;
    actor.character.baicang_mut()?.clock = clock;
    trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: "baicang_hitstop".into(),
        active_actor,
        hit_index: None,
        from_actor: None,
        to_actor: Some(index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            duration_us: Some(duration),
            rate_ppm: Some(rate),
            recovery_before_us: Some(before),
            recovery_after_us: Some(actor.locked_until_us),
            ..Default::default()
        }),
    });
    Ok(())
}

impl State {
    pub(crate) fn global_freeze(
        &mut self,
        owner: usize,
        start: u64,
        duration: u64,
    ) -> Result<(), String> {
        self.tracker
            .add_freeze(owner, start, duration)
            .map_err(|error| error.to_string())
    }
}
