//! Ordinary Malice, separately conserved from integer Charge and Cycle ledgers.
use crate::runtime_api::{ActorHitContext, CastContext};
use crate::simulation::{ExactFraction, GameActorConfig, GameScenarioRequest, state::ActorRuntime};
use ntesim_core::kernel::state::{ExactPool, ExactPoolSummary};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::BlackbirdOrdinaryMaliceV1;

#[derive(Clone, Debug)]
pub(crate) struct PendingExtra {
    command_index: usize,
    delivery_us: u64,
    coefficient: crate::formula::Ratio,
    metadata: crate::simulation::metadata::ActionPacketMetadata,
    recipients: std::collections::BTreeSet<usize>,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(crate) fn initialize(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    if accepted(request) {
        let mut pool = initial_pool(config.initial_resources.malice)?;
        if actor.hp > 0 {
            pool.grant(ExactFraction::new(20, 1)?)?;
        }
        actor.character.blackbird_mut()?.malice = Some(pool);
    }
    Ok(())
}

pub(crate) fn initial_pool(initial: Option<crate::formula::Ratio>) -> Result<ExactPool, String> {
    ExactPool::new(
        ExactFraction::new(60, 1)?,
        initial
            .map(ExactFraction::from_ratio)
            .transpose()?
            .unwrap_or(ExactFraction::new(0, 1)?),
    )
}

pub(crate) fn record_initial(
    config: &GameActorConfig,
    request: &GameScenarioRequest,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    if config.character_id == "character:1042" && accepted(request) {
        crate::runtime_api::record_hypothesis(request, assumptions, MODEL, MODEL.description())?;
    }
    Ok(())
}

pub(crate) fn summary(
    state: &super::skill::State,
) -> Result<BTreeMap<String, ExactPoolSummary>, String> {
    state
        .malice
        .as_ref()
        .map(|pool| {
            Ok(BTreeMap::from([(
                "blackbird.malice".into(),
                pool.summary()?,
            )]))
        })
        .unwrap_or_else(|| Ok(BTreeMap::new()))
}

pub(crate) fn on_skill(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let state = ctx.actor.character.blackbird_mut()?;
    if let Some(pool) = &mut state.malice
        && !state.malice_skill_casts.contains(&ctx.command_index)
    {
        pool.grant(ExactFraction::new(40, 1)?)?;
        state.malice_skill_casts.insert(ctx.command_index);
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.applied_hp_loss == 0 || super::form::active(ctx.actor) {
        return Ok(());
    }
    // These are exact native amounts, per causal packet, never per recipient.
    let millionths = match ctx.damage_effect_id {
        "GE_Player_BlackBird_Melee1_Damage" => 857_919,
        "GE_Player_BlackBird_Melee2_Damage" => 1_762_896,
        "GE_Player_BlackBird_Melee3_Damage" => 1_089_593,
        "GE_Player_BlackBird_Melee4_Damage" => 933_937,
        "GE_Player_BlackBird_Melee5_Damage" => 1_317_647,
        super::secondary::PLUNGE => 1_723_077,
        super::secondary::RIPOSTE => 4_615_385,
        super::branches::ROOSTCALL_PACKET => 4_123_077,
        super::bird::PACKET if ctx.config.awakening.has_effect(6) => 4_000_000,
        _ => return Ok(()),
    };
    let state = ctx.actor.character.blackbird_mut()?;
    let Some(pool) = &mut state.malice else {
        return Ok(());
    };
    let group = ctx
        .packet_group_id
        .ok_or("Blackbird Malice gain requires a causal packet group")?;
    if state.malice_packets.contains(group) {
        return Ok(());
    }
    let to_actor = ctx.actor_index;
    let group_id = group.to_owned();
    let (gain, overflow) = pool.grant(ExactFraction::new(millionths, 1_000_000)?)?;
    state.malice_packets.insert(group.into());
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.time_us,
        phase: "HitDelivery".into(),
        event_kind: "blackbird_malice_gain".into(),
        active_actor: ctx.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: Some(to_actor),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            packet_group_id: Some(group_id),
            exact_amount: Some(gain),
            exact_overflow: Some(overflow),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn connected(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    key: usize,
    group: Option<&str>,
    target_index: usize,
    effect: &str,
) -> Result<(), String> {
    if !super::skill::evolve_accepted(ctx.request) {
        return Ok(());
    }
    if let Some(group) = group {
        let pending = ctx.state.actors[owner]
            .character
            .blackbird_mut()?
            .malice_extras
            .get_mut(group);
        if let Some(pending) = pending {
            if pending.command_index != key {
                return Err("Blackbird extra group changed causal command".into());
            }
            if !pending.recipients.insert(target_index) {
                return Ok(());
            }
            let pending = pending.clone();
            return enqueue_extra(ctx, owner, target_index, &pending);
        }
    }
    let threshold = if effect == "GE_Player_BlackBird_Evolve_UltraSkill_Damage" {
        if ctx.request.actors[owner].awakening.has_effect(1) {
            let state = ctx.state.actors[owner].character.blackbird_mut()?;
            if state.finisher_extra_commands.insert(key) {
                Some(ExactFraction::new(60, 1)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        super::parameters::malice_threshold(effect)
    };
    let Some(threshold) = threshold else {
        return Ok(());
    };
    // One spend per causal packet even when its full AoE scope has several
    // recipients; the group is cloned across those recipient deliveries.
    let group = group.ok_or("Blackbird evolved Malice spend requires a causal packet group")?;
    let state = ctx.state.actors[owner].character.blackbird_mut()?;
    let Some(pool) = &mut state.malice else {
        return Ok(());
    };
    if !state.malice_packets.insert(group.into()) {
        return Ok(());
    }
    let spent = pool.spend_up_to(threshold)?;
    if spent.is_zero() {
        return Ok(());
    }
    let remaining = pool.current();
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "HitDelivery".into(),
        event_kind: "blackbird_malice_spend".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            damage_effect_id: Some(effect.into()),
            exact_amount: Some(spent),
            exact_after: Some(remaining),
            ..Default::default()
        }),
    });
    let config = &ctx.request.actors[owner];
    let k_idx = crate::simulation::inputs::normal_coordinate(config, "GA_BlackBird_Melee")?;
    let k = ExactFraction::new(
        *super::parameters::K_MALICE_MILLIONTHS
            .get(k_idx)
            .ok_or("Blackbird Malice requires effective Normal talent in 1..=13")?,
        1_000_000,
    )?;
    let total_k = if config.awakening.has_effect(3) {
        let k3 = ExactFraction::new(
            *super::parameters::K3_MALICE_MILLIONTHS
                .get(k_idx)
                .ok_or("Blackbird A3 Malice requires effective Normal talent in 1..=13")?,
            1_000_000,
        )?;
        k.add(k3)?
    } else {
        k
    };
    let extra_coef = total_k.mul(spent)?;
    let atk_rate = ntesim_core::formula::Ratio {
        numerator: extra_coef.numerator as u64,
        denominator: extra_coef.denominator as u64,
    };
    let extra_effect = super::parameters::extra_packet(effect).ok_or("extra packet missing")?;
    let extra_ability = super::parameters::extra_ability(effect).ok_or("extra ability missing")?;
    let delay_us = super::parameters::extra_delay_us(effect);
    let deliver_time_us = ctx
        .state
        .current_time_us
        .checked_add(delay_us)
        .ok_or("extra delivery overflow")?;
    let mut metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        extra_ability,
        extra_effect,
        None,
        &ctx.request.model_assumptions,
    )?;
    metadata.binding.energy_group_id = Some(crate::runtime_api::next_packet_group(
        owner,
        &mut ctx.state.actors[owner],
    )?);
    let pending = PendingExtra {
        command_index: key,
        delivery_us: deliver_time_us,
        coefficient: atk_rate,
        metadata,
        recipients: std::collections::BTreeSet::from([target_index]),
    };
    ctx.state.actors[owner]
        .character
        .blackbird_mut()?
        .malice_extras
        .insert(group.into(), pending.clone());
    // All simultaneous recipients settle before this callback. Queued extras
    // retain their own captured metadata; the temporary shared plan can go.
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx.state.current_time_us,
            priority: ntesim_core::execution::CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::Blackbird(super::events::Event::ExtraCaptureEnd {
                    actor_index: owner,
                    group: group.into(),
                }),
            ),
        });
    enqueue_extra(ctx, owner, target_index, &pending)
}

fn enqueue_extra(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    target_index: usize,
    pending: &PendingExtra,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: pending.delivery_us,
            priority: ntesim_core::execution::CausalPriority::HitDelivery,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::HitDelivery {
                command_index: pending.command_index,
                actor_index: owner,
                target_index,
                ability_id: pending.metadata.action.ability_id.clone(),
                damage_effect_id: pending.metadata.packet.damage_effect_id.clone(),
                atk_rate: pending.coefficient,
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                metadata: Box::new(pending.metadata.clone()),
            },
        });
    Ok(())
}
