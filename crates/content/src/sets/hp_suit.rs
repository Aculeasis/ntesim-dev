use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, RefreshPolicy};
use ntesim_core::formula::Ratio;

pub(crate) mod projection;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::SetsHpSuitV1;

pub(crate) const BUFF_ID: &str = "hp_suit.max_hp";
pub(crate) const MAX_HP_BONUS_KEY: &str = "max_hp_bonus";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StackGrant {
    pub(crate) actor_index: usize,
    pub(crate) max_hp_bonus_per_stack: Ratio,
    pub(crate) duration_us: u64,
    pub(crate) max_stacks: u32,
    pub(crate) stack_count: u32,
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::HpSuitMaxHp {
            max_hp_bonus_per_stack,
            duration_us,
            max_stacks,
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.action_metadata.input_role == Some(InputActionRole::Ultimate) {
                crate::gear::schedule_cast_buff(
                    ctx,
                    ContentEvent::HpSuitMaxHpStack(StackGrant {
                        actor_index: ctx.actor_index,
                        max_hp_bonus_per_stack: *max_hp_bonus_per_stack,
                        duration_us: *duration_us,
                        max_stacks: *max_stacks,
                        stack_count: *max_stacks,
                    }),
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn on_damage_received(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    actual_hp_loss: u64,
) -> Result<(), String> {
    if actual_hp_loss == 0 {
        return Ok(());
    }
    let Some((max_hp_bonus_per_stack, duration_us, max_stacks)) = ctx.request.actors[actor_index]
        .gear_procs
        .iter()
        .find_map(|proc| match proc {
            GearProcConfig::HpSuitMaxHp {
                max_hp_bonus_per_stack,
                duration_us,
                max_stacks,
            } => Some((*max_hp_bonus_per_stack, *duration_us, *max_stacks)),
            _ => None,
        })
    else {
        return Ok(());
    };
    apply_damage_stack(
        ctx,
        actor_index,
        max_hp_bonus_per_stack,
        duration_us,
        max_stacks,
    )
}

fn apply_damage_stack(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    max_hp_bonus_per_stack: Ratio,
    duration_us: u64,
    max_stacks: u32,
) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    apply(
        ctx,
        StackGrant {
            actor_index,
            max_hp_bonus_per_stack,
            duration_us,
            max_stacks,
            stack_count: 1,
        },
    )
}

pub(crate) fn apply(ctx: &mut CallbackContext<'_>, grant: StackGrant) -> Result<(), String> {
    let StackGrant {
        actor_index,
        max_hp_bonus_per_stack,
        duration_us,
        max_stacks,
        stack_count,
    } = grant;
    let state = &mut *ctx.state;
    let bonus = ExactFraction::from_ratio(max_hp_bonus_per_stack)?;
    let mut declaration = TimedBuffDeclaration::reset(
        BUFF_ID,
        "set:Suit10",
        BuffRecipient::SelfActor,
        duration_us,
        BuffStackingScope::PerSource,
        std::collections::BTreeMap::from([(MAX_HP_BONUS_KEY.into(), bonus)]),
    );
    declaration.refresh_policy = RefreshPolicy::Independent;
    declaration.max_stacks = max_stacks;
    for _ in 0..stack_count {
        state.apply_buff(actor_index, declaration.clone())?;
    }
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let deadline = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("HP suit expiry overflow")?;
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
    let effective = crate::simulation::state::effective_hp_max(
        &state.buffs,
        state.actors[actor_index].hp_max_base,
        &state.actors[actor_index].hp_max_static_bonus,
        actor_index,
        state.actors[actor_index].hp_max,
    )?;
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "hp_suit_max_hp_stack".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            stacks: Some(stacks),
            max_stacks: Some(max_stacks),
            hp_max: Some(effective),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn validate_config(
    proc: &GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let GearProcConfig::HpSuitMaxHp {
        max_hp_bonus_per_stack,
        duration_us,
        max_stacks,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    if ctx.actor.set_id.as_deref() != Some("set:Suit10") {
        return Err(format!(
            "actor {} HP suit requires equipped set:Suit10",
            ctx.actor_index
        ));
    }
    ExactFraction::from_ratio(*max_hp_bonus_per_stack)?;
    if *duration_us == 0 || !(1..=64).contains(max_stacks) {
        return Err("HP suit requires positive duration and 1..=64 stacks".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "hp_suit",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    if ctx.action.input_role == Some(InputActionRole::Ultimate) {
        ctx.time_us
            .checked_add(*duration_us)
            .ok_or_else(|| format!("action {} HP suit deadline overflow", ctx.action_index))?;
    }
    Ok(())
}
