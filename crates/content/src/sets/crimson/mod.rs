use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, RefreshPolicy};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;

pub(crate) mod projection;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::SetsCrimsonV1;

pub(crate) const BUFF_ID: &str = "crimson.atk";
pub(crate) const ATK_BONUS_KEY: &str = "atk_percent";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StackGrant {
    pub(crate) actor_index: usize,
    pub(crate) atk_bonus_per_stack: Ratio,
    pub(crate) duration_us: u64,
    pub(crate) max_stacks: u32,
}

pub(crate) fn on_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::CrimsonAtk {
            atk_bonus_per_stack,
            duration_us,
            max_stacks,
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::CrimsonAtkStack(StackGrant {
                    actor_index: ctx.actor_index,
                    atk_bonus_per_stack: *atk_bonus_per_stack,
                    duration_us: *duration_us,
                    max_stacks: *max_stacks,
                })),
            });
        }
    }
    Ok(())
}

pub(crate) fn apply(ctx: &mut CallbackContext<'_>, grant: StackGrant) -> Result<(), String> {
    let StackGrant {
        actor_index,
        atk_bonus_per_stack,
        duration_us,
        max_stacks,
    } = grant;
    let state = &mut *ctx.state;
    let bonus = ExactFraction::from_ratio(atk_bonus_per_stack)?;
    let mut declaration = TimedBuffDeclaration::reset(
        BUFF_ID,
        "set:Suit4",
        BuffRecipient::SelfActor,
        duration_us,
        BuffStackingScope::PerSource,
        std::collections::BTreeMap::from([(ATK_BONUS_KEY.into(), bonus)]),
    );
    declaration.refresh_policy = RefreshPolicy::Independent;
    declaration.max_stacks = max_stacks;
    state.apply_buff(actor_index, declaration)?;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let deadline = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("Crimson expiry overflow")?;
    if deadline <= ctx.request.absolute_limit_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: deadline,
                priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    ContentEvent::BuffLifecycleExpiry,
                ),
            });
    }
    let stacks: u32 = state
        .buffs
        .instances()
        .iter()
        .filter(|buff| buff.id == BUFF_ID && buff.owner_actor == actor_index)
        .map(|buff| buff.stacks)
        .sum();
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "crimson_atk_stack".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            stacks: Some(stacks),
            max_stacks: Some(max_stacks),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn validate_config(
    proc: &GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let GearProcConfig::CrimsonAtk {
        atk_bonus_per_stack,
        duration_us,
        max_stacks,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    if ctx.actor.set_id.as_deref() != Some("set:Suit4") {
        return Err(format!(
            "actor {} Crimson requires equipped set:Suit4",
            ctx.actor_index
        ));
    }
    ExactFraction::from_ratio(*atk_bonus_per_stack)?;
    if *duration_us == 0 || !(1..=64).contains(max_stacks) {
        return Err("Crimson requires positive duration and 1..=64 stacks".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "crimson",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    _ctx: &crate::gear::validation::DeadlineContext<'_>,
    _duration_us: &u64,
) -> Result<(), String> {
    Ok(())
}
