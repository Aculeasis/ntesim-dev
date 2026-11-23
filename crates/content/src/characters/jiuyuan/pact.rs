//! Natural Rose Pact attachment train; active settlement remains a separate owner branch.
use crate::HypothesisId;
use crate::formula::Ratio;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::ExactFraction;
use crate::simulation::NativeScalingStat;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::ActionPacketMetadata;
use ntesim_core::execution::CausalPriority;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::JiuyuanNaturalPactV1;
pub(crate) const NATURAL_DURATION_US: u64 = 30_000_000;
const PERIOD_US: u64 = 1_000_000;
const SEED: &str = "GE_Player_Kuhara_Seed_Damage";
const END: &str = "GE_Player_Kuhara_BudEnd_Damage";
const BOOM: &str = "GE_Player_Kuhara_BudBoom_Damage";

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Packet {
    pub(crate) ratio: Ratio,
    pub(crate) scaling: NativeScalingStat,
    pub(crate) metadata: Box<ActionPacketMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Application {
    pub(crate) actor_index: usize,
    pub(crate) command_index: usize,
    pub(crate) target_index: usize,
    pub(crate) seed: Packet,
    pub(crate) end: Packet,
    pub(crate) bud_boom: Packet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PactPhase {
    Seed,
    Bud,
    Closing { closes_at_us: u64 },
}

#[derive(Clone, Debug)]
pub(crate) struct Pact {
    pub(crate) generation: u64,
    pub(crate) planted_us: u64,
    pub(crate) matures_us: u64,
    pub(crate) deadline_us: u64,
    pub(crate) phase: PactPhase,
    pub(crate) accumulated_hp_loss: u64,
    pub(crate) application: Application,
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .request
        .model_assumptions
        .contains_key(HYPOTHESIS_ID.as_str())
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    if !accepted(ctx.request) {
        return Ok(());
    }
    let packet = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str());
    let planting = ctx.ability_id == "GA_Kuhara_Skill"
        || (ctx.ability_id == "GA_Kuhara_Melee"
            && (packet == Some("GE_Player_Kuhara_Melee5_Damage")
                || (packet == Some("GE_Player_Kuhara_Melee1_Damage")
                    && ctx.actor.character.jiuyuan_mut()?.normal_stage == 0)));
    if !planting {
        return Ok(());
    }
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let config = ctx.config;
    let progression = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?;
    let level = crate::build::preview_projection::effective_skill_level(
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        "GA_Kuhara_Melee",
        config
            .talent_levels
            .map_or(config.purchased_skill_level, |levels| levels[0]),
        progression["breakthrough"]
            .as_u64()
            .ok_or("Pact breakthrough missing")?
            .try_into()
            .map_err(|_| "Pact breakthrough exceeds u32")?,
        &config.awakening,
    )?;
    let coordinate = level["array_index"]
        .as_u64()
        .ok_or("Pact Normal coordinate missing")?
        .try_into()
        .map_err(|_| "Pact coordinate exceeds u32")?;
    let seed = project(ctx, SEED, coordinate)?;
    let end = project(ctx, END, coordinate)?;
    let bud_boom = project(ctx, BOOM, coordinate)?;
    // The existing Normal5 public model delivers to target zero; Skill fans out.
    let count = if ctx.ability_id == "GA_Kuhara_Skill" {
        ctx.targets.len()
    } else {
        1
    };
    for target_index in 0..count {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.hit_time_us,
            priority: CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::JiuyuanPactApply(
                Application {
                    actor_index: ctx.actor_index,
                    command_index: ctx.command_index,
                    target_index,
                    seed: seed.clone(),
                    end: end.clone(),
                    bud_boom: bud_boom.clone(),
                },
            )),
        });
    }
    Ok(())
}

fn project(ctx: &CastContext<'_>, effect: &str, coordinate: u32) -> Result<Packet, String> {
    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &ctx.request.content_id,
            character_id: &ctx.config.character_id,
            ability_id: "GA_Kuhara_Melee",
            damage_effect_id: effect,
            array_index: coordinate,
            research_metadata: None,
            model_assumptions: &ctx.request.model_assumptions,
        },
    )?;
    let mut ratio: Ratio = serde_json::from_value(projection["compiled_rate"].clone())
        .map_err(|e| format!("Pact coefficient invalid: {e}"))?;
    if effect == SEED && ctx.config.awakening.has_effect(2) {
        ratio.numerator = ratio
            .numerator
            .checked_mul(2)
            .ok_or("Pact A2 coefficient overflow")?;
    }
    let scaling = serde_json::from_value(projection["compiled_scaling_stat"].clone())
        .map_err(|e| format!("Pact scaling invalid: {e}"))?;
    let mut metadata = Box::new(crate::simulation::metadata::resolve_packet_for_action(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &ctx.config.character_id,
        ctx.action_metadata,
        effect,
        &ctx.request.model_assumptions,
    )?);
    for tag in [
        "State.Damage",
        "State.Damage.Attachment",
        "ntesim.damage.attachment",
    ] {
        metadata.packet.semantic_tags.insert(tag.into());
    }
    Ok(Packet {
        ratio,
        scaling,
        metadata,
    })
}

pub(crate) fn apply(ctx: &mut CallbackContext<'_>, application: Application) -> Result<(), String> {
    let Application {
        actor_index,
        target_index,
        ..
    } = application;
    if ctx.state.actors[actor_index].hp == 0 || ctx.state.targets[target_index].hp_current == 0 {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[actor_index].character.jiuyuan_mut()?;
    let pacts = &mut state.pacts;
    if let Some(existing) = pacts.get(&target_index)
        && !matches!(existing.phase, PactPhase::Closing { .. })
    {
        return Ok(()); // Seed and Bud both NeverRefresh/NeverReset.
    }
    state.pact_generation = state
        .pact_generation
        .checked_add(1)
        .ok_or("Pact generation overflow")?;
    let generation = state.pact_generation;
    let reductions = if ctx.request.actors[actor_index].awakening.has_resonance_6() {
        std::collections::BTreeMap::from([(
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE".into(),
            super::parameters::PARAMETERS.resonance_resistance_reduction,
        )])
    } else {
        Default::default()
    };
    crate::simulation::state::apply_target_status(
        &mut ctx.state.buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: actor_index,
            source: "character:1055".into(),
            target_index,
            id: "jiuyuan.rose_pact".into(),
            now_us: ctx.state.mode_clock.relative_us(),
            duration_us: NATURAL_DURATION_US + 200_000,
            reductions,
        },
    )?;
    pacts.insert(
        target_index,
        Pact {
            generation,
            planted_us: now,
            matures_us: now.checked_add(5_000_000).ok_or("Pact matures overflow")?,
            deadline_us: now
                .checked_add(NATURAL_DURATION_US)
                .ok_or("Pact deadline overflow")?,
            phase: PactPhase::Seed,
            accumulated_hp_loss: 0,
            application,
        },
    );
    schedule_tick(
        ctx,
        actor_index,
        target_index,
        now,
        now.checked_add(PERIOD_US).ok_or("Pact tick overflow")?,
    )
}

fn schedule_tick(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    planted_us: u64,
    time_us: u64,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us,
        priority: CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::JiuyuanPactTick {
            actor_index,
            target_index,
            planted_us,
        }),
    });
    Ok(())
}

pub(crate) fn tick(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    planted_us: u64,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[actor_index];
    let pacts = &mut actor.character.jiuyuan_mut()?.pacts;
    if actor.hp == 0 || ctx.state.targets[target_index].hp_current == 0 {
        pacts.remove(&target_index);
        return Ok(());
    }
    let Some(pact) = pacts
        .get_mut(&target_index)
        .filter(|pact| pact.planted_us == planted_us)
    else {
        return Ok(());
    };
    if matches!(pact.phase, PactPhase::Closing { .. }) {
        return Ok(());
    }
    let seed_packet = pact.application.seed.clone();
    let end_packet = pact.application.end.clone();
    let app = pact.application.clone();
    let deadline = pact.deadline_us;
    let matures = pact.matures_us;

    match pact.phase {
        PactPhase::Seed => {
            emit(ctx, &app, &seed_packet)?;
            let actor = &mut ctx.state.actors[actor_index];
            let pact = actor
                .character
                .jiuyuan_mut()?
                .pacts
                .get_mut(&target_index)
                .unwrap();
            if now.saturating_add(PERIOD_US) >= matures {
                pact.phase = PactPhase::Bud;
            }
            schedule_tick(
                ctx,
                actor_index,
                target_index,
                planted_us,
                now.checked_add(PERIOD_US)
                    .ok_or("Pact next tick overflow")?,
            )?;
        }
        PactPhase::Bud => {
            emit(ctx, &app, &seed_packet)?;
            if now >= deadline {
                emit(ctx, &app, &end_packet)?;
                ctx.state.actors[actor_index]
                    .character
                    .jiuyuan_mut()?
                    .pacts
                    .remove(&target_index);
            } else {
                schedule_tick(
                    ctx,
                    actor_index,
                    target_index,
                    planted_us,
                    now.checked_add(PERIOD_US)
                        .ok_or("Pact next tick overflow")?,
                )?;
            }
        }
        PactPhase::Closing { .. } => {}
    }
    Ok(())
}

pub(crate) fn consume_bud(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    is_burst: bool,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[actor_index];
    let pacts = &mut actor.character.jiuyuan_mut()?.pacts;
    let Some(pact) = pacts.get_mut(&target_index) else {
        return Ok(());
    };
    if pact.phase != PactPhase::Bud {
        return Ok(());
    }
    let closes_at_us = now.checked_add(200_000).ok_or("closing overflow")?;
    let generation = pact.generation;
    pact.phase = PactPhase::Closing { closes_at_us };

    let mut multiplier_num: u32 = if is_burst { 4 } else { 1 };
    let mut multiplier_den: u32 = 1;
    if ctx.request.actors[actor_index].awakening.has_effect(4) {
        multiplier_num = multiplier_num.checked_mul(3).ok_or("A4 mult overflow")?;
        multiplier_den = multiplier_den.checked_mul(2).ok_or("A4 mult overflow")?;
    }

    let mut bud_boom = pact.application.bud_boom.clone();
    bud_boom.ratio.numerator = bud_boom
        .ratio
        .numerator
        .checked_mul(multiplier_num as u64)
        .ok_or("BudBoom ratio numerator overflow")?;
    bud_boom.ratio.denominator = bud_boom
        .ratio
        .denominator
        .checked_mul(multiplier_den as u64)
        .ok_or("BudBoom ratio denominator overflow")?;

    let app = pact.application.clone();
    let accumulated_hp = pact.accumulated_hp_loss;

    if is_burst {
        let state = &mut *ctx.state;
        let trace = &mut *ctx.trace;
        state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
        let actor = &mut state.actors[actor_index];
        let before = crate::simulation::energy::EnergySnapshot::capture(actor);
        let charge_basis = ExactFraction::new(40, 1)?;
        let _ = actor.grant_energy(40)?;
        trace.push(crate::simulation::GameEventReceipt {
            time_us: state.current_time_us,
            phase: "ProcCallback".into(),
            event_kind: "jiuyuan_bud_boom_energy".into(),
            active_actor: state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(actor_index),
            energy_change: Some(before.receipt(
                actor,
                crate::simulation::GameEnergySource::JiuyuanBudBoom,
                actor_index,
                actor_index,
                Some(charge_basis),
            )?),
            energy_fanout: None,
            data: None,
        });

        if ctx.request.actors[actor_index].awakening.has_effect(2) && accumulated_hp > 0 {
            let base_heal =
                ExactFraction::new(accumulated_hp.into(), 1)?.mul(ExactFraction::new(5, 100)?)?;
            for recipient in 0..ctx.state.actors.len() {
                if ctx.state.actors[recipient].hp > 0 {
                    ctx.settle_healing(actor_index, recipient, base_heal)?;
                }
            }
        }
    }

    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: now.checked_add(100_000).ok_or("BudBoom delay overflow")?,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: app.command_index,
            actor_index: app.actor_index,
            target_index: app.target_index,
            ability_id: bud_boom.metadata.action.ability_id.clone(),
            damage_effect_id: bud_boom.metadata.packet.damage_effect_id.clone(),
            atk_rate: bud_boom.ratio,
            scaling_stat: bud_boom.scaling,
            metadata: bud_boom.metadata.clone(),
        },
    });

    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: closes_at_us,
        priority: CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::JiuyuanPactClose {
            actor_index,
            target_index,
            generation,
        }),
    });

    Ok(())
}

pub(crate) fn close(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    generation: u64,
) -> Result<(), String> {
    let pacts = &mut ctx.state.actors[actor_index].character.jiuyuan_mut()?.pacts;
    if let Some(pact) = pacts.get(&target_index)
        && pact.generation == generation
        && matches!(pact.phase, PactPhase::Closing { .. })
    {
        pacts.remove(&target_index);
        ctx.state
            .buffs
            .remove_owned_buff("jiuyuan.rose_pact", "character:1055", actor_index);
    }
    Ok(())
}

pub(crate) fn force_grow(ctx: &mut CallbackContext<'_>, actor_index: usize) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let pacts = &mut ctx.state.actors[actor_index].character.jiuyuan_mut()?.pacts;
    for pact in pacts.values_mut() {
        if pact.phase == PactPhase::Seed {
            pact.phase = PactPhase::Bud;
            pact.deadline_us = now.checked_add(25_000_000).ok_or("force grow overflow")?;
        }
    }
    Ok(())
}

pub(crate) fn on_ultra4_hit(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
) -> Result<(), String> {
    consume_bud(ctx, actor_index, target_index, true)
}

pub(crate) fn on_branch_power_hit(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[actor_index];
    let pacts = &mut actor.character.jiuyuan_mut()?.pacts;
    if let Some(pact) = pacts.get_mut(&target_index) {
        match pact.phase {
            PactPhase::Bud => {
                consume_bud(ctx, actor_index, target_index, false)?;
            }
            PactPhase::Seed => {
                pact.matures_us = pact.matures_us.saturating_sub(2_000_000);
                if pact.matures_us <= now {
                    pact.phase = PactPhase::Bud;
                    pact.deadline_us = now.checked_add(25_000_000).ok_or("deadline overflow")?;
                }
            }
            PactPhase::Closing { .. } => {}
        }
    }
    Ok(())
}

pub(crate) fn observe_damage(
    state: &mut crate::simulation::state::GameSimulatorState,
    target: usize,
    effect: &str,
    applied_hp: u64,
) {
    if matches!(effect, SEED | "GE_Player_Kuhara_SeedReaction_Damage") {
        for actor in &mut state.actors {
            if let Ok(jiuyuan) = actor.character.jiuyuan_mut()
                && let Some(pact) = jiuyuan.pacts.get_mut(&target)
            {
                pact.accumulated_hp_loss = pact.accumulated_hp_loss.saturating_add(applied_hp);
            }
        }
    }
}

fn emit(
    ctx: &mut CallbackContext<'_>,
    application: &Application,
    packet: &Packet,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.state.current_time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: application.command_index,
            actor_index: application.actor_index,
            target_index: application.target_index,
            ability_id: packet.metadata.action.ability_id.clone(),
            damage_effect_id: packet.metadata.packet.damage_effect_id.clone(),
            atk_rate: packet.ratio,
            scaling_stat: packet.scaling,
            metadata: packet.metadata.clone(),
        },
    });
    Ok(())
}

pub(crate) fn is_packet(effect: &str) -> bool {
    matches!(
        effect,
        SEED | END | BOOM | "GE_Player_Kuhara_SeedReaction_Damage"
    )
}

pub(crate) fn on_defeat(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.request.actors[owner].character_id == "character:1055" && accepted(ctx.request) {
        ctx.state.actors[owner]
            .character
            .jiuyuan_mut()?
            .pacts
            .clear();
        ctx.state
            .buffs
            .remove_owned_buff("jiuyuan.rose_pact", "character:1055", owner);
    }
    Ok(())
}
