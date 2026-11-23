use super::events::{GameEventData, ScheduledGameEvent};
use super::state::GameSimulatorState;
use super::{
    ExactFraction as F, GameBreakContribution, GameEventPayload, GameEventReceipt, GameHitReceipt,
    GameScenarioRequest,
};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::{HypothesisId, execution::CausalPriority, formula::Ratio};
use std::collections::BTreeMap;

mod metadata;
mod values;
pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::CombatLiveBreakDamageV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BreakPacket {
    pub command_index: usize,
    pub actor_index: usize,
    pub target_index: usize,
    pub damage: F,
    pub contribution: GameBreakContribution,
    pub metadata: super::metadata::ActionPacketMetadata,
}

pub(crate) fn schedule(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    command_index: usize,
    target_index: usize,
) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(
        request,
        &mut state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    let bar = state.targets[target_index]
        .break_state
        .as_ref()
        .ok_or("Break contribution requires a bar")?;
    let capacity = bar.original_capacity;
    let generation = bar.generation;
    let living_sources = state
        .actors
        .iter()
        .enumerate()
        .filter_map(|(i, a)| (a.hp > 0).then_some(i))
        .collect();
    let mut assumptions = BTreeMap::new();
    let mut packets = Vec::new();
    let mut repeats = Vec::new();
    for (actor_index, config) in request.actors.iter().enumerate() {
        if state.actors[actor_index].hp == 0 {
            continue;
        }
        let buffs = state
            .buffs
            .get_stat_modifiers_for_actor_with_living_sources(
                actor_index,
                state.active_actor_index,
                state.mode_clock.relative_us(),
                &living_sources,
            )
            .map_err(|e| format!("Break source modifiers failed: {e:?}"))?;
        let buff = |key: &str| F::from_ratio(buffs.get(key).copied().unwrap_or(Ratio::zero()));
        let intensity = F::from_ratio(config.static_stats.break_intensity)?
            .add(buff("break_intensity")?)?
            .add(crate::gear::break_intensity::bonus(
                request,
                state,
                actor_index,
                &mut assumptions,
            )?)?;
        let kit = crate::registry::break_modifiers(&crate::runtime_api::BreakContext {
            world_time_us: state.mode_clock.relative_us(),
            config,
            actor_index,
            target_index,
            buffs: &state.buffs,
        })?;
        let bonus = buff("break_damage_bonus")?.add(kit.damage_bonus)?;
        let ignore =
            crate::runtime_api::aggregate_defense_ignore(kit.defense_ignore, buff("def_ignore")?)?;
        let base_break = F::new(values::base_at_level(config.character_level)?.into(), 1)?;
        let element =
            crate::registry::get_character_descriptor_by_id(&config.character_id)?.element;
        let name = element.as_str();
        let resistance_key = format!("ECharacterElementType::CHARACTER_ELEMENT_TYPE_{name}");
        let target = &state.targets[target_index];
        let resistance = request.targets[target_index]
            .elemental_resistances
            .get(name)
            .copied()
            .unwrap_or(target.resistance);
        let damage = ntesim_core::combat::exact_break_damage(
            &ntesim_core::combat::ExactBreakDamageRequest {
                base_break,
                intensity,
                original_cap: capacity,
                attacker_level: config.character_level,
                enemy_defense: crate::runtime_api::target_defense(
                    &state.buffs,
                    target,
                    target_index,
                    state.mode_clock.relative_us(),
                )?,
                resistance: F::from_ratio(resistance)?,
                bonus,
                shred: F::new(0, 1)?,
                ignore,
                resistance_reduction: super::state::target_resistance_reduction(
                    &state.buffs,
                    target_index,
                    &resistance_key,
                )?,
                special: F::new(1, 1)?,
            },
        )?;
        let packet = BreakPacket {
            command_index,
            actor_index,
            target_index,
            damage,
            contribution: GameBreakContribution {
                generation,
                character_level: config.character_level,
                base_break,
                original_capacity: capacity,
                intensity,
                bonus,
                defense_ignore: ignore,
            },
            metadata: metadata::packet(name)?,
        };
        for _ in 0..kit.extra_contributions {
            repeats.push(packet.clone());
        }
        packets.push(packet);
    }
    packets.extend(repeats);
    state.execution_model_assumptions.extend(assumptions);
    let count = u64::try_from(packets.len()).map_err(|_| "Break packet count overflow")?;
    sequence
        .checked_add(count)
        .ok_or("Break event sequence overflow")?;
    for packet in packets {
        *sequence += 1;
        queue.push(ScheduledGameEvent {
            time_us: state.current_time_us,
            priority: CausalPriority::ReactionOrBreak,
            seq: *sequence,
            data: GameEventData::TargetBreakDamage(Box::new(packet)),
        });
    }
    Ok(())
}

pub(super) fn settle(
    ctx: &mut super::context::EventContext<'_>,
    packet: BreakPacket,
) -> Result<(), String> {
    let target = &ctx.state.targets[packet.target_index];
    if target.hp_current == 0 {
        return Ok(());
    }
    let mut vitals = ntesim_core::state::VitalState::new(
        target.hp_current,
        target.hp_max,
        ntesim_core::state::ShieldOrder::OldestFirst,
    )
    .map_err(|e| format!("Break target vitals: {e:?}"))?;
    let rounded = packet.damage.round_u64()?;
    let outcome = vitals
        .damage(rounded)
        .map_err(|e| format!("Break HP settlement: {e:?}"))?;
    ctx.state.targets[packet.target_index].hp_current = vitals.hp();
    crate::registry::observe_applied_hp(
        ctx.request,
        ctx.state,
        packet.target_index,
        packet.command_index,
        &packet.metadata,
        outcome.hp_lost,
    )?;
    ctx.state.hits_settled = ctx
        .state
        .hits_settled
        .checked_add(1)
        .ok_or("Break hit count overflow")?;
    ctx.state.add_rounded_damage(rounded)?;
    let (report_command_index, autonomous_event_index) = ctx
        .state
        .origins
        .identify(ctx.request, packet.command_index)?;
    let execution_model_assumption_ids = ctx.state.record_receipt_assumptions([(
        HYPOTHESIS_ID.as_str().into(),
        HYPOTHESIS_ID.description().into(),
    )])?;
    let (metadata_id, energy_group_id) =
        ctx.state.intern_action_packet_metadata(&packet.metadata)?;
    let hit_index = ctx.hits.len();
    ctx.hits.push(GameHitReceipt {
        active_actor_index: ctx.state.active_actor_index,
        parent_hit_index: None,
        damage_base_override: None,
        autonomous_event_index,
        control_applications: Vec::new(),
        switch_origin: super::GameSwitchOrigin::support(ctx.request, packet.command_index),
        break_contribution: Some(packet.contribution),
        execution_model_assumption_ids,
        command_index: report_command_index,
        time_us: ctx.state.current_time_us,
        actor_index: packet.actor_index,
        target_index: packet.target_index,
        ability_id: "ntesim.break".into(),
        damage_effect_id: "ntesim.break.contribution".into(),
        scaling_stat: super::NativeScalingStat::Atk,
        scaling_rate: Ratio::zero(),
        metadata_id,
        energy_group_id,
        static_general_damage_bonus: F::new(0, 1)?,
        static_element_damage_bonus: F::new(0, 1)?,
        static_mental_damage_bonus: F::new(0, 1)?,
        non_crit_damage: packet.damage,
        crit_damage: packet.damage,
        crit_rate: Ratio::zero(),
        critical: false,
        damage: packet.damage,
        rounded_damage: rounded,
        applied_hp_loss: outcome.hp_lost,
        crit_sample: None,
    });
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "break_damage".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: Some(hit_index),
        from_actor: Some(packet.actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(packet.target_index),
            ..Default::default()
        }),
    });
    crate::gear::on_target_break_damage(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        packet.target_index,
    )?;
    if outcome.died {
        observe_defeat(ctx, packet.actor_index, packet.target_index)?;
    }
    Ok(())
}

fn observe_defeat(
    ctx: &mut super::context::EventContext<'_>,
    owner: usize,
    target: usize,
) -> Result<(), String> {
    crate::registry::on_party_target_defeated(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        owner,
        target,
    )?;
    crate::characters::team::on_target_defeated(ctx.request, ctx.state, target, ctx.trace)?;
    for (config, actor) in ctx.request.actors.iter().zip(&mut ctx.state.actors) {
        crate::registry::on_target_defeated(config, actor, ctx.state.current_time_us)?;
    }
    crate::gear::on_target_defeated(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        owner,
        target,
    )?;

    Ok(())
}
