use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsSpiderV1;

pub(crate) const BUFF_KNOWLEDGE_ID: &str = "spider.knowledge";
pub(crate) const BUFF_TEAM_ATK_ID: &str = "buff_fork_spider_team_atk";
pub(crate) const SOURCE: &str = "weapon:fork_spider";
pub(crate) const ICD_US: u64 = 500_000;
pub(crate) const KNOWLEDGE_DURATION_US: u64 = 3_600_000_000;
const CATEGORY_A: &str = "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A";

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) ready_at_us: u64,
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::SpiderTeamAtk {
            atk_bonus_per_stack,
            atk_bonus_max_stack,
            duration_us,
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.action_metadata.input_role == Some(InputActionRole::Ultimate) {
                let stacks = ctx
                    .buffs
                    .instances()
                    .iter()
                    .filter(|buff| {
                        buff.id == BUFF_KNOWLEDGE_ID
                            && buff.source == SOURCE
                            && buff.owner_actor == ctx.actor_index
                    })
                    .map(|buff| buff.stacks)
                    .sum::<u32>();
                ctx.buffs
                    .remove_owned_buff(BUFF_KNOWLEDGE_ID, SOURCE, ctx.actor_index);

                if stacks > 0 {
                    let per_stack_frac = ExactFraction::from_ratio(*atk_bonus_per_stack)?;
                    let mut team_bonus_frac =
                        per_stack_frac.mul(ExactFraction::new(u128::from(stacks), 1)?)?;
                    if stacks == 8 {
                        let max_bonus_frac = ExactFraction::from_ratio(*atk_bonus_max_stack)?;
                        team_bonus_frac = team_bonus_frac.add(max_bonus_frac)?;
                    }
                    let team_atk_bonus = crate::build::ratio(team_bonus_frac)?;

                    crate::runtime_api::increment_sequence(ctx.sequence)?;
                    ctx.queue.push(ScheduledGameEvent {
                        time_us: ctx.time_us,
                        priority: CausalPriority::BuffTickOrExpiry,
                        seq: *ctx.sequence,
                        data: GameEventData::Content(ContentEvent::SpiderTeamAtkGrant {
                            actor_index: ctx.actor_index,
                            team_atk_bonus,
                            duration_us: *duration_us,
                        }),
                    });
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let is_normal = ctx.action_metadata.input_role == Some(InputActionRole::Normal)
        || ctx
            .packet_metadata
            .as_ref()
            .and_then(|p| p.native_damage_source_category.as_deref())
            == Some(CATEGORY_A);
    if !is_normal {
        return Ok(());
    }

    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::SpiderTeamAtk { .. } = proc {
            if ctx.time_us < ctx.actor.equipment.spider.ready_at_us {
                continue;
            }
            ctx.actor.equipment.spider.ready_at_us = ctx
                .time_us
                .checked_add(ICD_US)
                .ok_or("Spider Knowledge ICD deadline overflow")?;

            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::SpiderKnowledgeStack {
                    actor_index: ctx.actor_index,
                }),
            });
        }
    }
    Ok(())
}

pub(crate) fn apply_knowledge_stack(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    state.apply_buff(
        actor_index,
        TimedBuffDeclaration {
            id: BUFF_KNOWLEDGE_ID.into(),
            source: SOURCE.into(),
            recipient: BuffRecipient::SpecificActor(actor_index),
            duration_us: KNOWLEDGE_DURATION_US,
            stacking_scope: BuffStackingScope::PerSource,
            refresh_policy: RefreshPolicy::Reset,
            max_stacks: 8,
            decay_policy: DecayPolicy::AllAtOnce,
            stat_modifiers: std::collections::BTreeMap::new(),
        },
    )?;

    let stacks = state
        .buffs
        .instances()
        .iter()
        .filter(|buff| {
            buff.id == BUFF_KNOWLEDGE_ID && buff.source == SOURCE && buff.owner_actor == actor_index
        })
        .map(|buff| buff.stacks)
        .sum::<u32>();

    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "spider_knowledge_stack".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            stacks: Some(stacks),
            max_stacks: Some(8),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn apply_team_atk(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    team_atk_bonus: Ratio,
    duration_us: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let atk_percent = ExactFraction::from_ratio(team_atk_bonus)?;

    state.apply_buff(
        actor_index,
        TimedBuffDeclaration::reset(
            BUFF_TEAM_ATK_ID,
            SOURCE,
            BuffRecipient::TeamAll,
            duration_us,
            BuffStackingScope::Shared,
            std::collections::BTreeMap::from([("atk_percent".into(), atk_percent)]),
        ),
    )?;

    let deadline = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("Spider team ATK deadline overflow")?;
    if deadline <= ctx.request.absolute_limit_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: deadline,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::BuffLifecycleExpiry),
        });
    }

    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "spider_team_atk".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            atk_percent: Some(atk_percent),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    });
    Ok(())
}
