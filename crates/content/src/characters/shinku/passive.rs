use super::events::Event;
use super::parameters::PARAMETERS;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::energy::EnergySnapshot;
use crate::simulation::events::{GameEventData, HitPacket, ScheduledGameEvent};
use crate::simulation::metadata::{
    ActionCategory, ActionMetadata, ActionPacketBindingKind, InputActionRole, MetadataProvenance,
    PrimaryActionSlot,
};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{
    ExactFraction, GameCausalTrigger, GameEnergySource, GameEventPayload, GameEventReceipt,
};
use ntesim_core::buffs::dispatch::{
    BuffManager, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};
use ntesim_core::execution::CausalPriority;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ShinkuPassivesAndAwakeningsV1;
pub(crate) const LONE_STRIDE_ATK_BUFF_ID: &str = "shinku.lone_stride_atk";
pub(crate) const A5_DEF_BUFF_ID: &str = "shinku.a5_def";

pub(crate) fn passive1_unlocked(
    config: &crate::simulation::GameActorConfig,
) -> Result<bool, String> {
    let tier = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?["breakthrough"]
        .as_u64()
        .unwrap_or(0);
    Ok(tier >= u64::from(PARAMETERS.passive1_min_breakthrough))
}

fn watch_stack_cap(config: &crate::simulation::GameActorConfig) -> u32 {
    if config.awakening.has_effect(4) {
        PARAMETERS.watch_a4_max_stacks
    } else {
        PARAMETERS.watch_max_stacks
    }
}

fn schedule_watch_start(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    state: &mut super::awakenings::State,
    owner: usize,
    at_us: u64,
) -> Result<(), String> {
    state.watch_generation = state.watch_generation.wrapping_add(1);
    state.watch_next_tick_us = 0;
    let generation = state.watch_generation;
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: at_us,
        priority: CausalPriority::BuffTickOrExpiry,
        seq: *sequence,
        data: GameEventData::Content(ContentEvent::Shinku(Event::WatchStart {
            owner,
            generation,
        })),
    });
    Ok(())
}

fn schedule_watch_tick(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    state: &mut super::awakenings::State,
    owner: usize,
    generation: u64,
    at_us: u64,
) -> Result<(), String> {
    state.watch_next_tick_us = at_us;
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: at_us,
        priority: CausalPriority::BuffTickOrExpiry,
        seq: *sequence,
        data: GameEventData::Content(ContentEvent::Shinku(Event::WatchTick { owner, generation })),
    });
    Ok(())
}

pub(crate) fn initialize_combat(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let has_a3 = ctx.request.actors[owner].awakening.has_effect(3);
    if ctx.state.active_actor_index == owner || has_a3 {
        let now = ctx.state.current_time_us;
        let state = ctx.state.actors[owner].character.shinku_mut()?;
        schedule_watch_start(
            ctx.queue,
            ctx.sequence,
            state,
            owner,
            now.checked_add(PARAMETERS.watch_unattack_us)
                .ok_or("Shinku Watch start deadline overflow")?,
        )?;
    }
    Ok(())
}

pub(crate) fn on_field_entry(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 || ctx.request.actors[owner].awakening.has_effect(3) {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner].character.shinku_mut()?;
    schedule_watch_start(
        ctx.queue,
        ctx.sequence,
        state,
        owner,
        now.checked_add(PARAMETERS.watch_unattack_us)
            .ok_or("Shinku Watch entry deadline overflow")?,
    )
}

pub(crate) fn on_field_exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let has_a3 = ctx.request.actors[owner].awakening.has_effect(3);
    remove_a5_def_buff(&mut ctx.state.buffs, owner);
    let state = ctx.state.actors[owner].character.shinku_mut()?;
    state.surging_crimson_until_us = 0;
    state.skill_rage_ready_us = 0;
    state.scarlet_descent_casts = 0;
    state.crimson_judgment_stacks = 0;
    if let Some(pool) = &mut state.fading_reason {
        let max = ExactFraction::new(PARAMETERS.fading_reason_max.into(), 1)?;
        pool.spend_up_to(max)?;
    }
    if !has_a3 {
        state.menacing_gaze_active = false;
        state.watch_stacks = 0;
        state.watch_next_tick_us = 0;
        state.watch_generation = state.watch_generation.wrapping_add(1);
    }
    Ok(())
}

pub(crate) fn on_attack_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor = ctx.actor_index;
    let state = ctx.actor.character.shinku_mut()?;
    state.menacing_gaze_active = false;
    schedule_watch_start(
        ctx.queue,
        ctx.sequence,
        state,
        actor,
        ctx.time_us
            .checked_add(PARAMETERS.watch_unattack_us)
            .ok_or("Shinku Watch attack reset overflow")?,
    )
}

pub(crate) fn on_critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1076" {
        return Ok(());
    }
    let cap = watch_stack_cap(ctx.config);
    let state = ctx.actor.character.shinku_mut()?;
    if state.menacing_gaze_active {
        crate::runtime_api::record_hypothesis_id(ctx.request, ctx.execution_assumptions, MODEL)?;
        state.watch_stacks = state.watch_stacks.saturating_add(1).min(cap);
    }
    Ok(())
}

pub(crate) fn on_watch_start(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let has_a3 = ctx.request.actors[owner].awakening.has_effect(3);
    if ctx.state.active_actor_index != owner && !has_a3 {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner].character.shinku_mut()?;
    if state.watch_generation != generation {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    state.menacing_gaze_active = true;
    schedule_watch_tick(
        ctx.queue,
        ctx.sequence,
        state,
        owner,
        generation,
        now.checked_add(PARAMETERS.watch_tick_us)
            .ok_or("Shinku Watch tick deadline overflow")?,
    )
}

pub(crate) fn on_watch_tick(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let has_a3 = ctx.request.actors[owner].awakening.has_effect(3);
    if ctx.state.active_actor_index != owner && !has_a3 {
        return Ok(());
    }
    let cap = watch_stack_cap(&ctx.request.actors[owner]);
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner].character.shinku_mut()?;
    if !state.menacing_gaze_active || state.watch_generation != generation {
        return Ok(());
    }
    state.watch_stacks = state.watch_stacks.saturating_add(1).min(cap);
    state.watch_next_tick_us = 0;
    if state.watch_stacks < cap {
        schedule_watch_tick(
            ctx.queue,
            ctx.sequence,
            state,
            owner,
            generation,
            now.checked_add(PARAMETERS.watch_tick_us)
                .ok_or("Shinku Watch tick deadline overflow")?,
        )?;
    }
    Ok(())
}

pub(crate) fn apply_a5_def_buff(
    buffs: &mut BuffManager,
    actor_index: usize,
    world_now_us: u64,
    duration_us: u64,
) -> Result<(), String> {
    crate::simulation::state::apply_timed_buff(
        buffs,
        TimedBuffDeclaration::reset(
            A5_DEF_BUFF_ID,
            "character:1076",
            BuffRecipient::SelfActor,
            duration_us,
            BuffStackingScope::PerSource,
            BTreeMap::from([("def_percent".into(), PARAMETERS.a5_def_percent)]),
        ),
        actor_index,
        world_now_us,
    )
}

pub(crate) fn remove_a5_def_buff(buffs: &mut BuffManager, actor_index: usize) {
    buffs.remove_owned_buff(A5_DEF_BUFF_ID, "character:1076", actor_index);
}

fn watch_action_metadata(provenance_id: crate::HypothesisId) -> ActionMetadata {
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: provenance_id.into(),
        description: provenance_id.description().into(),
    };
    ActionMetadata {
        categories: BTreeSet::from([ActionCategory::Melee]),
        gameplay_tags: Vec::new(),
        ability_id: "GA_Shinku_Melee".into(),
        native_ability_index: None,
        input_role: Some(InputActionRole::Normal),
        input_role_provenance: Some(provenance.clone()),
        primary_slot: Some(PrimaryActionSlot::Normal),
        source_evidence: None,
        identity_evidence: None,
        provenance,
    }
}

pub(crate) fn try_trigger_watch_hits(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.applied_hp_loss == 0
        || matches!(
            ctx.damage_effect_id,
            "GE_Player_Shinku_Watch_Damage"
                | "GE_Player_Shinku_WatchEx_Damage"
                | "GE_Player_Shinku_ReactionAOE_Damage"
        )
    {
        return Ok(());
    }
    let stacks = {
        let state = ctx.actor.character.shinku_mut()?;
        if state.watch_stacks == 0 {
            return Ok(());
        }
        let stacks = state.watch_stacks;
        state.watch_stacks = 0;
        stacks
    };

    crate::runtime_api::record_hypothesis_id(ctx.request, ctx.execution_assumptions, MODEL)?;
    let coord = crate::simulation::inputs::normal_coordinate(ctx.config, "GA_Shinku_Melee")?;
    let action = watch_action_metadata(MODEL);

    let emit_watch_packet =
        |ctx: &mut ActorHitContext<'_>, packet_id: &str| -> Result<(), String> {
            let (mut rate, scaling_stat) = crate::simulation::inputs::coefficient(
                ctx.request,
                ctx.config,
                "GA_Shinku_Melee",
                packet_id,
                coord,
            )?;
            rate.numerator = rate
                .numerator
                .checked_mul(u64::from(stacks))
                .ok_or("Shinku Watch packet rate overflow")?;
            let mut metadata = crate::simulation::metadata::resolve_packet_for_action(
                &ctx.request.content_id,
                ntesim_data::skill_levels::frozen::tables(),
                &ctx.config.character_id,
                &action,
                packet_id,
                &ctx.request.model_assumptions,
            )?;
            metadata.action = ctx.action_metadata.clone();
            metadata.binding.kind = ActionPacketBindingKind::OwnerChild;
            metadata.binding.initiating_ability_id = ctx.action_metadata.ability_id.clone();
            metadata.binding.provenance = Some(MetadataProvenance::AcceptedModelHypothesis {
                id: MODEL.as_str().into(),
                description: MODEL.description().into(),
            });
            metadata.binding.energy_group_id = None;
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::HitDelivery,
                seq: *ctx.sequence,
                data: GameEventData::FollowupHit {
                    parent_hit_index: ctx.hit_index,
                    packet: Box::new(HitPacket {
                        command_index: ctx.command_index,
                        actor_index: ctx.actor_index,
                        target_index: ctx.target_index,
                        ability_id: ctx.action_metadata.ability_id.clone(),
                        damage_effect_id: packet_id.into(),
                        atk_rate: rate,
                        scaling_stat,
                        metadata: Box::new(metadata),
                    }),
                },
            });
            Ok(())
        };

    emit_watch_packet(ctx, "GE_Player_Shinku_Watch_Damage")?;
    if ctx.config.awakening.has_effect(4) && stacks == PARAMETERS.watch_a4_max_stacks {
        emit_watch_packet(ctx, "GE_Player_Shinku_WatchEx_Damage")?;
    }
    Ok(())
}

pub(crate) fn on_charge_reaction(
    ctx: &mut CallbackContext<'_>,
    recipient: usize,
    snapshot_command: usize,
    raw: u64,
) -> Result<(), String> {
    // Passive 1 BackChargeMul = 1.0: while any BT >= 2 living Shinku is in the party,
    // all living off-field teammates gain equal Charge reaction energy (with their own CE).
    let back_charge_source = (0..ctx.request.actors.len()).find(|&i| {
        ctx.request.actors[i].character_id == "character:1076"
            && ctx.state.actors[i].hp > 0
            && matches!(
                &ctx.state.actors[i].character,
                crate::characters::RuntimeState::Shinku(s)
                    if s.breakthrough >= PARAMETERS.passive1_min_breakthrough
            )
    });
    if let Some(shinku_owner) = back_charge_source {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            MODEL,
        )?;
        let basis = ExactFraction::new(raw.into(), 1)?;
        for teammate in 0..ctx.state.actors.len() {
            if teammate == recipient
                || ctx.state.actors[teammate].hp == 0
                || ctx.request.actors[teammate].character_id == "character:1076"
            {
                continue;
            }
            let actor = &mut ctx.state.actors[teammate];
            let before = EnergySnapshot::capture(actor);
            let gross = actor.grant_energy(raw)?;
            ctx.trace.push(GameEventReceipt {
                time_us: ctx.state.current_time_us,
                phase: "ReactionOrBreak".into(),
                event_kind: "shinku_passive1_back_charge".into(),
                active_actor: recipient,
                hit_index: None,
                from_actor: Some(shinku_owner),
                to_actor: Some(teammate),
                energy_change: Some(before.receipt(
                    actor,
                    GameEnergySource::ReactionCharge,
                    shinku_owner,
                    teammate,
                    Some(basis),
                )?),
                energy_fanout: None,
                data: Some(GameEventPayload {
                    gross: Some(gross),
                    ..Default::default()
                }),
            });
        }
    }

    // Active Shinku (BT >= 2): 1.0s ICD AoE + Lone Stride +5% ATK stack (30s, max 10).
    if ctx.request.actors[recipient].character_id != "character:1076"
        || ctx.state.actors[recipient].hp == 0
    {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let should_trigger = {
        let state = ctx.state.actors[recipient].character.shinku_mut()?;
        if state.breakthrough < PARAMETERS.passive1_min_breakthrough
            || now < state.passive1_aoe_icd_until_us
        {
            false
        } else {
            state.passive1_aoe_icd_until_us = now
                .checked_add(PARAMETERS.passive1_aoe_icd_us)
                .ok_or("Shinku Passive 1 AoE ICD overflow")?;
            true
        }
    };
    if !should_trigger {
        return Ok(());
    }

    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;

    let mut buff = TimedBuffDeclaration::reset(
        LONE_STRIDE_ATK_BUFF_ID,
        "character:1076",
        BuffRecipient::SelfActor,
        PARAMETERS.passive1_atk_duration_us,
        BuffStackingScope::PerSource,
        BTreeMap::from([("atk_percent".into(), PARAMETERS.passive1_atk_per_stack)]),
    );
    buff.refresh_policy = RefreshPolicy::Reset;
    buff.max_stacks = PARAMETERS.passive1_atk_max_stacks;
    buff.decay_policy = DecayPolicy::AllAtOnce;
    ctx.state.apply_buff(recipient, buff)?;

    let living_targets: Vec<usize> = (0..ctx.state.targets.len())
        .filter(|&t| ctx.state.targets[t].hp_current > 0)
        .collect();
    if living_targets.is_empty() {
        return Ok(());
    }

    let config = &ctx.request.actors[recipient];
    let coord = crate::simulation::inputs::normal_coordinate(config, "GA_Shinku_Melee")?;
    let (rate, scaling_stat) = crate::simulation::inputs::coefficient(
        ctx.request,
        config,
        "GA_Shinku_Melee",
        "GE_Player_Shinku_ReactionAOE_Damage",
        coord,
    )?;
    let action = watch_action_metadata(MODEL);
    let mut metadata = crate::simulation::metadata::resolve_packet_for_action(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        &action,
        "GE_Player_Shinku_ReactionAOE_Damage",
        &ctx.request.model_assumptions,
    )?;
    metadata.binding.kind = ActionPacketBindingKind::OwnerChild;
    metadata.binding.initiating_ability_id = "GA_Shinku_Melee".into();
    metadata.binding.provenance = Some(MetadataProvenance::AcceptedModelHypothesis {
        id: MODEL.as_str().into(),
        description: MODEL.description().into(),
    });
    metadata.binding.energy_group_id = Some(crate::runtime_api::next_packet_group(
        recipient,
        &mut ctx.state.actors[recipient],
    )?);

    let trigger = match ctx.state.origins.identify(ctx.request, snapshot_command)? {
        (None, Some(autonomous_event_index)) => GameCausalTrigger::OwnerDeadline {
            autonomous_event_index,
        },
        (Some(cmd), None) => {
            if matches!(
                ctx.request.actions.get(cmd),
                Some(crate::simulation::GameActionCommand::SwitchActor { .. })
            ) {
                GameCausalTrigger::FieldEntry { command_index: cmd }
            } else {
                GameCausalTrigger::Action { command_index: cmd }
            }
        }
        (None, None) => GameCausalTrigger::InitialState,
        _ => return Err("unexpected causal identity for Shinku Reaction AOE".into()),
    };
    let key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: recipient,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: "GA_Shinku_Melee".into(),
            time_us: now,
            trigger,
            recipient_healing: None,
        },
    )?;

    for target in living_targets {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: now,
            priority: CausalPriority::HitDelivery,
            seq: *ctx.sequence,
            data: GameEventData::HitDelivery {
                command_index: key,
                actor_index: recipient,
                target_index: target,
                ability_id: "GA_Shinku_Melee".into(),
                damage_effect_id: "GE_Player_Shinku_ReactionAOE_Damage".into(),
                atk_rate: rate,
                scaling_stat,
                metadata: Box::new(metadata.clone()),
            },
        });
    }
    Ok(())
}
