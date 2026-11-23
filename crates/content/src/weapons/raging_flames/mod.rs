use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext, DamageContext, HitModifiers};
use crate::simulation::metadata::{DamageElement, InputActionRole, PacketMetadata};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use ntesim_core::formula::Ratio;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsRagingFlamesV1;

const WINDOW_ID: &str = "raging_flames.redirect_ultimate_window";
const STACK_ID: &str = "raging_flames.redirect_skill_stacks";
const SOURCE: &str = "weapon:fork_wushoutieyu";

fn active_window_expiry(ctx: &CastContext<'_>) -> Option<u64> {
    ctx.buffs.instances().iter().find_map(|buff| {
        (buff.id == WINDOW_ID
            && buff.source == SOURCE
            && buff.owner_actor == ctx.actor_index
            && buff.recipient == BuffRecipient::SelfActor)
            .then_some(&buff.duration)
            .and_then(|duration| match duration {
                ntesim_core::buffs::dispatch::BuffDuration::Timed { expires_at_us, .. }
                    if *expires_at_us > ctx.world_time_us =>
                {
                    Some(*expires_at_us)
                }
                _ => None,
            })
    })
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::RagingFlames {
            redirect_ultimate_bonus,
            redirect_skill_stack_bonus,
            duration_us,
            ..
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            match ctx.action_metadata.input_role {
                Some(InputActionRole::Ultimate) => crate::gear::schedule_cast_buff(
                    ctx,
                    ContentEvent::RagingFlamesUltimateGrant {
                        actor_index: ctx.actor_index,
                        redirect_ultimate_bonus: *redirect_ultimate_bonus,
                        duration_us: *duration_us,
                    },
                )?,
                Some(InputActionRole::RedirectSkill) if active_window_expiry(ctx).is_some() => {
                    ctx.record_hypothesis(crate::gear::SKILL_CAST_BUFF_HYPOTHESIS_ID)?;
                    crate::runtime_api::increment_sequence(ctx.sequence)?;
                    ctx.queue
                        .push(crate::simulation::events::ScheduledGameEvent {
                            time_us: ctx.time_us,
                            priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                            seq: *ctx.sequence,
                            data: crate::simulation::events::GameEventData::Content(
                                ContentEvent::RagingFlamesSkillStack {
                                    actor_index: ctx.actor_index,
                                    redirect_skill_stack_bonus: *redirect_skill_stack_bonus,
                                },
                            ),
                        });
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::RagingFlames { lakshana_bonus, .. } = proc else {
            continue;
        };
        if packet.element == Some(DamageElement::Lakshana) {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*lakshana_bonus)?)?;
        }
        if !matches!(
            packet.native_damage_source_category.as_deref(),
            Some(
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E"
                    | "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            )
        ) {
            continue;
        }
        let active = ctx.buff_stats;
        let base = active
            .get("raging_flames_redirect_ultimate_bonus")
            .copied()
            .map(ExactFraction::from_ratio)
            .transpose()?
            .unwrap_or(ExactFraction::new(0, 1)?);
        modifiers = modifiers.add_damage(base)?;
        if packet.native_damage_source_category.as_deref()
            == Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E")
        {
            let stacks = active
                .get("raging_flames_redirect_skill_bonus")
                .copied()
                .map(ExactFraction::from_ratio)
                .transpose()?
                .unwrap_or(ExactFraction::new(0, 1)?);
            modifiers = modifiers.add_damage(stacks)?;
        }
    }
    Ok(modifiers)
}

pub(crate) fn apply_ultimate(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    redirect_ultimate_bonus: Ratio,
    duration_us: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    state.buffs.remove_owned_buff(STACK_ID, SOURCE, actor_index);
    state.apply_buff(
        actor_index,
        TimedBuffDeclaration::reset(
            WINDOW_ID,
            SOURCE,
            BuffRecipient::SelfActor,
            duration_us,
            BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([(
                "raging_flames_redirect_ultimate_bonus".into(),
                ExactFraction::from_ratio(redirect_ultimate_bonus)?,
            )]),
        ),
    )?;
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "raging_flames_window".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn apply_skill_stack(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    redirect_skill_stack_bonus: Ratio,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let expiry = state.buffs.instances().iter().find_map(|buff| {
        (buff.id == WINDOW_ID && buff.source == SOURCE && buff.owner_actor == actor_index)
            .then_some(&buff.duration)
            .and_then(|duration| match duration {
                ntesim_core::buffs::dispatch::BuffDuration::Timed { expires_at_us, .. }
                    if *expires_at_us > state.mode_clock.relative_us() =>
                {
                    Some(*expires_at_us)
                }
                _ => None,
            })
    });
    let Some(expiry) = expiry else {
        return Ok(());
    };
    let remaining_us = expiry - state.mode_clock.relative_us();
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let mut declaration = TimedBuffDeclaration::reset(
        STACK_ID,
        SOURCE,
        BuffRecipient::SelfActor,
        remaining_us,
        BuffStackingScope::PerSource,
        std::collections::BTreeMap::from([(
            "raging_flames_redirect_skill_bonus".into(),
            ExactFraction::from_ratio(redirect_skill_stack_bonus)?,
        )]),
    );
    declaration.max_stacks = 2;
    declaration.refresh_policy = RefreshPolicy::Reset;
    declaration.decay_policy = DecayPolicy::AllAtOnce;
    state.apply_buff(actor_index, declaration)?;
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "raging_flames_skill_stack".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            expires_at_us: Some(expiry),
            ..Default::default()
        }),
    });
    Ok(())
}
