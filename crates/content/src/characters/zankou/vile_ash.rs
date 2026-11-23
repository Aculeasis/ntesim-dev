use super::dot_kind::DotKind;
use crate::HypothesisId;
use crate::formula::Ratio;
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::ActionPacketMetadata;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameActionCommand, GameActorConfig, GameScenarioRequest, NativeScalingStat,
};
use ntesim_core::execution::CausalPriority;

pub(crate) const DOT_ULTRA_DAMAGE_EFFECT_ID: &str = "GE_Player_Zankou_DotUltraDamage";
pub(crate) const ULTRA_ABILITY_ID: &str = "GA_Zankou_UltraSkill";
pub(crate) const VILE_ASH_TICK_PERIOD_US: u64 = 1_000_000;
pub(crate) const VILE_ASH_DURATION_US: u64 = 30_000_000;
pub(crate) const VILE_ASH_STACK_CAP: u32 = 10;
pub(crate) const VILE_ASH_ULTRA_STACKS: u32 = 5;

pub(crate) const VILE_ASH_SCHEDULING_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::CharacterZankouVileAshDotSchedulingV2;

#[derive(Clone, Debug)]
pub(crate) struct VileAshTrain {
    kind: DotKind,
    pub(crate) actor_index: usize,
    pub(crate) command_index: usize,
    pub(crate) pending_tick_seq: Option<u64>,
    pub(crate) stacks: u32,
    pub(crate) next_tick_us: u64,
    pub(crate) deadline_us: u64,
    pub(crate) k_native: Ratio,
    pub(crate) scaling_stat: NativeScalingStat,
    metadata: Box<ActionPacketMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VileAshApplication {
    kind: DotKind,
    pub(crate) actor_index: usize,
    pub(crate) command_index: usize,
    metadata: Box<ActionPacketMetadata>,
    k_native: Ratio,
    scaling_stat: NativeScalingStat,
    stacks: u32,
    target: Option<usize>,
}

pub(crate) fn is_vile_ash_packet(damage_effect_id: &str) -> bool {
    matches!(
        damage_effect_id,
        "GE_Player_Zankou_DotDamage" | "GE_Player_Zankou_DotUltraDamage"
    )
}

pub(crate) fn calc_zankou_dot_stack_coef(
    effect_id: &str,
    expand_multiplier: u32,
    stacks: u32,
) -> u32 {
    if !is_vile_ash_packet(effect_id) || expand_multiplier > 0 {
        return 0;
    }
    stacks.saturating_sub(1).min(VILE_ASH_STACK_CAP)
}

fn native_dot_rate(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    array_index: u32,
    kind: DotKind,
) -> Result<(Ratio, NativeScalingStat), String> {
    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &request.content_id,
            character_id: &config.character_id,
            ability_id: kind.native_ability(),
            damage_effect_id: kind.packet(),
            array_index,
            research_metadata: None,
            model_assumptions: &request.model_assumptions,
        },
    )?;
    let k_native: Ratio = serde_json::from_value(projection["compiled_rate"].clone())
        .map_err(|error| format!("Vile Ash native coefficient invalid: {error}"))?;
    let scaling_stat: NativeScalingStat =
        serde_json::from_value(projection["compiled_scaling_stat"].clone())
            .map_err(|error| format!("Vile Ash native scaling stat invalid: {error}"))?;
    Ok((k_native, scaling_stat))
}

fn schedule_tick(
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    train: &mut VileAshTrain,
    target_index: usize,
) -> Result<(), String> {
    let additional = calc_zankou_dot_stack_coef(train.kind.packet(), 0, train.stacks);
    let factor = u64::from(additional) + 1;
    let atk_rate = Ratio {
        numerator: train
            .k_native
            .numerator
            .checked_mul(factor)
            .ok_or("Vile Ash tick coefficient overflow")?,
        denominator: train.k_native.denominator,
    };
    crate::runtime_api::increment_sequence(sequence)?;
    train.pending_tick_seq = Some(*sequence);
    queue.push(ScheduledGameEvent {
        time_us: train.next_tick_us,
        priority: CausalPriority::HitDelivery,
        seq: *sequence,
        data: GameEventData::HitDelivery {
            command_index: train.command_index,
            actor_index: train.actor_index,
            target_index,
            ability_id: train.metadata.action.ability_id.clone(),
            damage_effect_id: train.kind.packet().to_owned(),
            atk_rate,
            scaling_stat: train.scaling_stat,
            metadata: train.metadata.clone(),
        },
    });
    Ok(())
}

pub(crate) fn apply_vile_ash(ctx: &mut CastContext<'_>, application_us: u64) -> Result<(), String> {
    let array_index: u32 = match ctx.request.actions.get(ctx.command_index) {
        Some(GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            array_index,
            ..
        }) => {
            if *actor_index != ctx.actor_index || ability_id.as_str() != ctx.ability_id {
                return Err("Vile Ash producer disagrees with its admitted owner/action".into());
            }
            (*array_index)
                .try_into()
                .map_err(|_| "Vile Ash array coordinate exceeds u32")?
        }
        _ => return Err("Vile Ash producer requires an admitted ability command".into()),
    };
    prepare_application(ctx, DotKind::VileAsh, array_index, application_us)
}

pub(crate) fn apply_heartwrench(
    ctx: &mut CastContext<'_>,
    application_us: u64,
) -> Result<(), String> {
    let coordinate = crate::simulation::inputs::normal_coordinate(ctx.config, "GA_Zankou_Melee")?;
    prepare_application(
        ctx,
        DotKind::Heartwrench,
        coordinate
            .try_into()
            .map_err(|_| "Heartwrench coordinate exceeds u32")?,
        application_us,
    )
}

fn prepare_application(
    ctx: &mut CastContext<'_>,
    kind: DotKind,
    array_index: u32,
    application_us: u64,
) -> Result<(), String> {
    ctx.record_hypothesis(kind.hypothesis())?;
    application_us
        .checked_add(VILE_ASH_DURATION_US)
        .ok_or("Zankou DoT deadline overflow")?;
    let (k_native, scaling_stat) = native_dot_rate(ctx.request, ctx.config, array_index, kind)?;
    // Resolve reached packet assumptions during admission, before any costs
    // or primary hits commit; subsequent ticks retain these frozen bytes.
    let metadata = Box::new(crate::simulation::metadata::resolve_packet_for_action(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &ctx.config.character_id,
        ctx.action_metadata,
        kind.packet(),
        &ctx.request.model_assumptions,
    )?);
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: application_us,
        priority: CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::ZankouVileAshApply(
            VileAshApplication {
                kind,
                actor_index: ctx.actor_index,
                command_index: ctx.command_index,
                metadata,
                k_native,
                scaling_stat,
                stacks: kind.application_stacks(),
                target: None,
            },
        )),
    });
    Ok(())
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    application: VileAshApplication,
) -> Result<(), String> {
    let VileAshApplication {
        kind,
        actor_index,
        command_index,
        metadata,
        k_native,
        scaling_stat,
        stacks,
        target,
    } = application;
    let apply_us = ctx.state.current_time_us;
    if ctx.state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    let deadline_us = apply_us
        .checked_add(VILE_ASH_DURATION_US)
        .ok_or("Vile Ash deadline overflow")?;
    let first_tick_us = apply_us
        .checked_add(VILE_ASH_TICK_PERIOD_US)
        .ok_or("Vile Ash first tick overflow")?;
    let state = ctx.state.actors[actor_index].character.zankou_mut()?;
    for target_index in 0..ctx.state.targets.len() {
        if target.is_some_and(|target| target != target_index)
            || ctx.state.targets[target_index].hp_current == 0
        {
            continue;
        }
        let train = kind
            .trains(state)
            .entry(target_index)
            .or_insert(VileAshTrain {
                kind,
                actor_index,
                command_index,
                pending_tick_seq: None,
                stacks: 0,
                next_tick_us: first_tick_us,
                deadline_us,
                k_native,
                scaling_stat,
                metadata: metadata.clone(),
            });
        train.actor_index = actor_index;
        train.command_index = command_index;
        // Reapplication replaces the pending delivery as well as its deadline.
        // Otherwise an obsolete snapshot can deal damage and retrigger gear.
        if let Some(stale_seq) = train.pending_tick_seq.take() {
            ctx.queue.retain(|event| event.seq != stale_seq);
        }
        let active = train.stacks > 0 && kind.active_at(apply_us, train.deadline_us);
        let next_tick_us = if active {
            train.next_tick_us
        } else {
            first_tick_us
        };
        if !active {
            train.stacks = 0;
        }
        train.stacks = train.stacks.saturating_add(stacks).min(VILE_ASH_STACK_CAP);
        train.next_tick_us = next_tick_us;
        train.deadline_us = deadline_us;
        train.k_native = k_native;
        train.scaling_stat = scaling_stat;
        train.metadata = metadata.clone();
        schedule_tick(ctx.queue, ctx.sequence, train, target_index)?;
        sync_status(
            &mut ctx.state.buffs,
            train,
            target_index,
            apply_us,
            ctx.state.mode_clock.relative_us(),
        )?;
    }
    Ok(())
}

fn sync_status(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    train: &VileAshTrain,
    target: usize,
    now: u64,
    world_time_us: u64,
) -> Result<(), String> {
    use ntesim_core::buffs::dispatch::*;
    let id = match train.kind {
        DotKind::Heartwrench => "zankou.heartwrench",
        DotKind::VileAsh => "zankou.vile_ash",
    };
    buffs.remove_target_buff(id, "character:1036", &target.to_string());
    let definition = BuffDefinition {
        id: id.into(),
        source: "character:1036".into(),
        recipient: BuffRecipient::SpecificTarget(target.into()),
        source_lifetime: BuffSourceLifetime::Snapshot,
        stacking_scope: BuffStackingScope::PerSource,
        duration_us: Some(train.deadline_us.saturating_sub(now)),
        refresh_policy: RefreshPolicy::Reset,
        max_stacks: VILE_ASH_STACK_CAP,
        decay_policy: DecayPolicy::AllAtOnce,
        semantic_tags: std::collections::BTreeSet::from([
            crate::characters::packet_semantics::DOT_DAMAGE_TAG.into(),
        ]),
        stat_modifiers: Default::default(),
    };
    for _ in 0..train.stacks {
        buffs
            .apply_buff(definition.clone(), train.actor_index, world_time_us)
            .map_err(|e| format!("Zankou DoT status: {e:?}"))?;
    }
    Ok(())
}

pub(crate) fn carriers(
    actor: &crate::simulation::state::ActorRuntime,
    target: usize,
    now: u64,
) -> Vec<VileAshApplication> {
    let crate::characters::RuntimeState::Zankou(state) = &actor.character else {
        return Vec::new();
    };
    if actor.hp == 0 {
        return Vec::new();
    }
    [&state.heartwrench_trains, &state.vile_ash_trains]
        .into_iter()
        .filter_map(|trains| trains.get(&target))
        .filter(|train| train.stacks > 0 && train.kind.active_at(now, train.deadline_us))
        .map(|train| VileAshApplication {
            kind: train.kind,
            actor_index: train.actor_index,
            command_index: train.command_index,
            metadata: train.metadata.clone(),
            k_native: train.k_native,
            scaling_stat: train.scaling_stat,
            stacks: train.stacks,
            target: None,
        })
        .collect()
}

pub(crate) fn target_defeated(ctx: &mut CallbackContext<'_>, target: usize) -> Result<(), String> {
    for actor in &mut ctx.state.actors {
        if let crate::characters::RuntimeState::Zankou(state) = &mut actor.character {
            for trains in [&mut state.heartwrench_trains, &mut state.vile_ash_trains] {
                if let Some(train) = trains.remove(&target)
                    && let Some(seq) = train.pending_tick_seq
                {
                    ctx.queue.retain(|e| e.seq != seq);
                }
            }
        }
    }
    for id in ["zankou.heartwrench", "zankou.vile_ash"] {
        ctx.state
            .buffs
            .remove_target_buff(id, "character:1036", &target.to_string());
    }
    Ok(())
}
pub(crate) fn owner_defeated(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.state.actors[owner].hp > 0 {
        return Ok(());
    }
    let state = ctx.state.actors[owner].character.zankou_mut()?;
    state.heartwrench_trains.clear();
    state.vile_ash_trains.clear();
    // Target-only stat queries have no living-source context. These opaque
    // stack carriers use Snapshot and are removed by this actual defeat hook.
    for id in ["zankou.heartwrench", "zankou.vile_ash"] {
        ctx.state
            .buffs
            .remove_owned_buff(id, "character:1036", owner);
    }
    ctx.queue.retain(|e| match &e.data {
        GameEventData::HitDelivery { actor_index, .. } => *actor_index != owner,
        GameEventData::Content(crate::callbacks::ContentEvent::ZankouVileAshApply(app)) => {
            app.actor_index != owner
        }
        GameEventData::Content(crate::callbacks::ContentEvent::ZankouSpread {
            source_actor,
            ..
        }) => *source_actor != owner,
        _ => true,
    });
    Ok(())
}

pub(crate) fn continue_dot_train(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let Some(kind) = DotKind::from_packet(ctx.damage_effect_id) else {
        return Ok(());
    };
    continue_clock(
        DotClockContext {
            request: ctx.request,
            execution_assumptions: ctx.execution_assumptions,
            actor: ctx.actor,
            time_us: ctx.time_us,
            queue: ctx.queue,
            sequence: ctx.sequence,
        },
        kind,
        ctx.target_index,
    )
}

pub(crate) fn continue_zero_packet(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    effect_id: &str,
) -> Result<(), String> {
    let Some(kind) = DotKind::from_packet(effect_id) else {
        return Ok(());
    };
    continue_clock(
        DotClockContext {
            request: ctx.request,
            execution_assumptions: &mut ctx.state.execution_model_assumptions,
            actor: &mut ctx.state.actors[actor_index],
            time_us: ctx.state.current_time_us,
            queue: ctx.queue,
            sequence: ctx.sequence,
        },
        kind,
        target_index,
    )
}

struct DotClockContext<'a> {
    request: &'a GameScenarioRequest,
    execution_assumptions: &'a mut std::collections::BTreeMap<String, String>,
    actor: &'a mut crate::simulation::state::ActorRuntime,
    time_us: u64,
    queue: &'a mut BinaryHeap,
    sequence: &'a mut u64,
}

fn continue_clock(
    ctx: DotClockContext<'_>,
    kind: DotKind,
    target_index: usize,
) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        ctx.execution_assumptions,
        kind.hypothesis(),
    )?;
    let source_alive = ctx.actor.hp > 0;
    let time_us = ctx.time_us;
    let state = ctx.actor.character.zankou_mut()?;
    let Some(train) = kind.trains(state).get_mut(&target_index) else {
        return Ok(());
    };
    if time_us != train.next_tick_us {
        return Ok(());
    }
    train.pending_tick_seq = None;
    if !source_alive {
        return Ok(());
    }
    let next_tick_us = time_us
        .checked_add(VILE_ASH_TICK_PERIOD_US)
        .ok_or("Vile Ash next tick overflow")?;
    train.next_tick_us = next_tick_us;
    if !kind.active_at(next_tick_us, train.deadline_us) {
        return Ok(());
    }
    schedule_tick(ctx.queue, ctx.sequence, train, target_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_coef_matches_frozen_probes() {
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Zankou_DotDamage", 0, 0),
            0
        );
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Zankou_DotDamage", 0, 1),
            0
        );
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Zankou_DotDamage", 0, 2),
            1
        );
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Zankou_DotDamage", 0, 3),
            2
        );
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Zankou_DotDamage", 0, 10),
            9
        );
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Zankou_DotDamage", 0, 20),
            10
        );
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Zankou_DotUltraDamage", 0, 5),
            4
        );
    }

    #[test]
    fn stack_coef_rejects_expansion_bypass_and_foreign_classes() {
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Zankou_DotDamage", 1, 10),
            0
        );
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Lacrimosa_Blood_Damage", 0, 10),
            0
        );
        assert_eq!(
            calc_zankou_dot_stack_coef("GE_Player_Zankou_UltraSkill1_Damage", 0, 10),
            0
        );
    }
}
