use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext, DamageContext, HitModifiers};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{DamageElement, InputActionRole, PacketMetadata};
use crate::simulation::{ExactFraction, GameActorConfig, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffRecipient, BuffSourceLifetime, BuffStackingScope, DecayPolicy,
    RefreshPolicy,
};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsBitGameV1;

pub(crate) const BUFF_OFF_FIELD_BASE_ID: &str = "buff_fork_BitGame_off_field_base";
pub(crate) const BUFF_OFF_FIELD_STACK_ID: &str = "buff_fork_BitGame_off_field_stack";
pub(crate) const BUFF_ON_FIELD_STACK_ID: &str = "buff_fork_BitGame_on_field_psyche_stack";
pub(crate) const SOURCE: &str = "weapon:fork_BitGame";
pub(crate) const ON_FIELD_ICD_US: u64 = 300_000;
const CATEGORY_A: &str = "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A";

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) off_field_ready_at_us: u64,
    pub(crate) on_field_ready_at_us: u64,
}

fn parameters(config: &GameActorConfig) -> Option<(Ratio, Ratio, u64, Ratio, Ratio)> {
    config.gear_procs.iter().find_map(|proc| match proc {
        GearProcConfig::BitGame {
            off_field_atk_base,
            off_field_atk_per_stack,
            off_field_icd_us,
            on_field_psyche_base,
            on_field_psyche_per_stack,
        } => Some((
            *off_field_atk_base,
            *off_field_atk_per_stack,
            *off_field_icd_us,
            *on_field_psyche_base,
            *on_field_psyche_per_stack,
        )),
        _ => None,
    })
}

pub(crate) fn initialize(
    ctx: &mut CallbackContext<'_>,
) -> Result<BTreeMap<String, String>, String> {
    let mut assumptions = BTreeMap::new();
    let has_bit_game = ctx
        .request
        .actors
        .iter()
        .any(|config| parameters(config).is_some());
    if !has_bit_game {
        return Ok(assumptions);
    }
    crate::runtime_api::record_hypothesis_id(ctx.request, &mut assumptions, HYPOTHESIS_ID)?;

    let active_actor = ctx.state.active_actor_index;
    for (i, actor_config) in ctx.request.actors.iter().enumerate() {
        if let Some((off_field_atk_base, ..)) = parameters(actor_config)
            && i != active_actor
        {
            apply_off_field_base(ctx, i, off_field_atk_base)?;
        }
    }
    Ok(assumptions)
}

pub(crate) fn on_switch(
    ctx: &mut CallbackContext<'_>,
    previous_actor: usize,
) -> Result<(), String> {
    let current_active = ctx.state.active_actor_index;
    for (i, actor_config) in ctx.request.actors.iter().enumerate() {
        if let Some((off_field_atk_base, ..)) = parameters(actor_config) {
            if previous_actor == i {
                ctx.state
                    .buffs
                    .remove_owned_buff(BUFF_ON_FIELD_STACK_ID, SOURCE, i);
                apply_off_field_base(ctx, i, off_field_atk_base)?;
            } else if current_active == i {
                ctx.state
                    .buffs
                    .remove_owned_buff(BUFF_OFF_FIELD_BASE_ID, SOURCE, i);
                ctx.state
                    .buffs
                    .remove_owned_buff(BUFF_OFF_FIELD_STACK_ID, SOURCE, i);
            }
        }
    }
    Ok(())
}

fn apply_off_field_base(
    ctx: &mut CallbackContext<'_>,
    wearer_index: usize,
    off_field_atk_base: Ratio,
) -> Result<(), String> {
    let mut stat_modifiers = BTreeMap::new();
    stat_modifiers.insert("atk_percent".into(), off_field_atk_base);
    ctx.state
        .buffs
        .apply_buff(
            BuffDefinition {
                stacking_scope: BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                source_lifetime: BuffSourceLifetime::WhileAlive,
                id: BUFF_OFF_FIELD_BASE_ID.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::ActiveActor,
                duration_us: None,
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers,
            },
            wearer_index,
            ctx.state.mode_clock.relative_us(),
        )
        .map_err(|error| format!("buff application failed: {error:?}"))?;
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if parameters(ctx.config).is_some() {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let Some((_, off_field_atk_per_stack, off_field_icd_us, _, on_field_psyche_per_stack)) =
        parameters(ctx.config)
    else {
        return Ok(());
    };

    let is_on_field = ctx.actor_index == ctx.active_actor_index;
    if is_on_field {
        let is_basic = ctx.action_metadata.input_role == Some(InputActionRole::Normal)
            || ctx
                .packet_metadata
                .as_ref()
                .and_then(|p| p.native_damage_source_category.as_deref())
                == Some(CATEGORY_A);
        let is_psyche = ctx.packet_metadata.as_ref().is_some_and(|p| {
            p.element == Some(DamageElement::Psyche)
                || p.native_damage_type_ex == "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE"
        });
        if is_basic && is_psyche && ctx.time_us >= ctx.actor.equipment.bit_game.on_field_ready_at_us
        {
            ctx.actor.equipment.bit_game.on_field_ready_at_us = ctx
                .time_us
                .checked_add(ON_FIELD_ICD_US)
                .ok_or("BitGame on-field ICD deadline overflow")?;
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::BitGameOnFieldStack {
                    actor_index: ctx.actor_index,
                    on_field_psyche_per_stack,
                }),
            });
        }
    } else if ctx.time_us >= ctx.actor.equipment.bit_game.off_field_ready_at_us {
        ctx.actor.equipment.bit_game.off_field_ready_at_us = ctx
            .time_us
            .checked_add(off_field_icd_us)
            .ok_or("BitGame off-field ICD deadline overflow")?;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::BitGameOffFieldStack {
                actor_index: ctx.actor_index,
                off_field_atk_per_stack,
            }),
        });
    }
    Ok(())
}

pub(crate) fn apply_off_field_stack(
    ctx: &mut CallbackContext<'_>,
    wearer_index: usize,
    off_field_atk_per_stack: Ratio,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let mut stat_modifiers = BTreeMap::new();
    stat_modifiers.insert("atk_percent".into(), off_field_atk_per_stack);
    ctx.state
        .buffs
        .apply_buff(
            BuffDefinition {
                stacking_scope: BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                source_lifetime: BuffSourceLifetime::WhileAlive,
                id: BUFF_OFF_FIELD_STACK_ID.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::ActiveActor,
                duration_us: None,
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: 4,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers,
            },
            wearer_index,
            ctx.state.mode_clock.relative_us(),
        )
        .map_err(|error| format!("buff application failed: {error:?}"))?;

    let stacks = ctx
        .state
        .buffs
        .instances()
        .iter()
        .filter(|buff| {
            buff.id == BUFF_OFF_FIELD_STACK_ID
                && buff.source == SOURCE
                && buff.owner_actor == wearer_index
        })
        .map(|buff| buff.stacks)
        .sum::<u32>();

    ctx.trace.push(GameEventReceipt {
        time_us: now,
        phase: "ProcCallback".into(),
        event_kind: "bit_game_off_field_stack".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(wearer_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            stacks: Some(stacks),
            max_stacks: Some(4),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn apply_on_field_stack(
    ctx: &mut CallbackContext<'_>,
    wearer_index: usize,
    _on_field_psyche_per_stack: Ratio,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let stat_modifiers = BTreeMap::new();
    ctx.state
        .buffs
        .apply_buff(
            BuffDefinition {
                stacking_scope: BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                source_lifetime: BuffSourceLifetime::WhileAlive,
                id: BUFF_ON_FIELD_STACK_ID.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::SelfActor,
                duration_us: None,
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: 10,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers,
            },
            wearer_index,
            ctx.state.mode_clock.relative_us(),
        )
        .map_err(|error| format!("buff application failed: {error:?}"))?;

    let stacks = ctx
        .state
        .buffs
        .instances()
        .iter()
        .filter(|buff| {
            buff.id == BUFF_ON_FIELD_STACK_ID
                && buff.source == SOURCE
                && buff.owner_actor == wearer_index
        })
        .map(|buff| buff.stacks)
        .sum::<u32>();

    ctx.trace.push(GameEventReceipt {
        time_us: now,
        phase: "ProcCallback".into(),
        event_kind: "bit_game_on_field_stack".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(wearer_index),
        to_actor: Some(wearer_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            stacks: Some(stacks),
            max_stacks: Some(10),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    let is_psyche = packet.element == Some(DamageElement::Psyche)
        || packet.native_damage_type_ex == "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE";
    if !is_psyche {
        return Ok(modifiers);
    }
    let Some((_, _, _, on_field_psyche_base, on_field_psyche_per_stack)) = parameters(ctx.config)
    else {
        return Ok(modifiers);
    };

    let is_on_field = ctx.actor_index == ctx.active_actor_index;
    if is_on_field {
        modifiers = modifiers.add_damage(ExactFraction::from_ratio(on_field_psyche_base)?)?;
        let stacks = ctx
            .buffs
            .instances()
            .iter()
            .filter(|buff| {
                buff.id == BUFF_ON_FIELD_STACK_ID
                    && buff.source == SOURCE
                    && buff.owner_actor == ctx.actor_index
            })
            .map(|buff| buff.stacks)
            .sum::<u32>();
        if stacks > 0 {
            let per_stack = ExactFraction::from_ratio(on_field_psyche_per_stack)?;
            modifiers =
                modifiers.add_damage(per_stack.mul(ExactFraction::new(u128::from(stacks), 1)?)?)?;
        }
    }
    Ok(modifiers)
}
