use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, RefreshPolicy};
use ntesim_core::formula::Ratio;

pub(crate) mod projection;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::SetsQuietManorV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StackGrant {
    pub(crate) actor_index: usize,
    pub(crate) mental_bonus_per_stack: Ratio,
    pub(crate) duration_us: u64,
    pub(crate) max_stacks: u32,
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::QuietManorMental {
            mental_bonus_per_stack,
            duration_us,
            max_stacks,
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.action_metadata.input_role == Some(InputActionRole::Normal) {
                crate::runtime_api::increment_sequence(ctx.sequence)?;
                ctx.queue
                    .push(crate::simulation::events::ScheduledGameEvent {
                        time_us: ctx.time_us,
                        priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                        seq: *ctx.sequence,
                        data: crate::simulation::events::GameEventData::Content(
                            ContentEvent::QuietManorMentalStack(StackGrant {
                                actor_index: ctx.actor_index,
                                mental_bonus_per_stack: *mental_bonus_per_stack,
                                duration_us: *duration_us,
                                max_stacks: *max_stacks,
                            }),
                        ),
                    });
            }
        }
    }
    Ok(())
}

pub(crate) fn apply(ctx: &mut CallbackContext<'_>, grant: StackGrant) -> Result<(), String> {
    let StackGrant {
        actor_index,
        mental_bonus_per_stack,
        duration_us,
        max_stacks,
    } = grant;
    let state = &mut *ctx.state;
    let bonus = ExactFraction::from_ratio(mental_bonus_per_stack)?;
    let mut declaration = TimedBuffDeclaration::reset(
        "quiet_manor.mental_damage",
        "set:Suit12",
        BuffRecipient::SelfActor,
        duration_us,
        BuffStackingScope::PerSource,
        std::collections::BTreeMap::from([("mental_damage_bonus".into(), bonus)]),
    );
    declaration.refresh_policy = RefreshPolicy::Independent;
    declaration.max_stacks = max_stacks;
    state.apply_buff(actor_index, declaration)?;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let deadline = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("Quiet Manor expiry overflow")?;
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
        .filter(|buff| buff.id == "quiet_manor.mental_damage" && buff.owner_actor == actor_index)
        .map(|buff| buff.stacks)
        .sum();
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "quiet_manor_mental_stack".into(),
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
    let GearProcConfig::QuietManorMental {
        mental_bonus_per_stack,
        duration_us,
        max_stacks,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    if ctx.actor.set_id.as_deref() != Some("set:Suit12") {
        return Err(format!(
            "actor {} Quiet Manor requires equipped set:Suit12",
            ctx.actor_index
        ));
    }
    ExactFraction::from_ratio(*mental_bonus_per_stack)?;
    if *duration_us == 0 || !(1..=64).contains(max_stacks) {
        return Err("Quiet Manor requires positive duration and 1..=64 stacks".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "quiet_manor",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    if ctx.action.input_role == Some(InputActionRole::Normal) {
        ctx.time_us
            .checked_add(*duration_us)
            .ok_or_else(|| format!("action {} Quiet Manor deadline overflow", ctx.action_index))?;
    }
    Ok(())
}
