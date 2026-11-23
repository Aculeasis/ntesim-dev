use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::CastContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::{ExactFraction, GameEventReceipt};
use ntesim_core::execution::CausalPriority;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::EdgarSkillBurstPacketsV1;
pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::EdgarNormalChainV1;
pub(crate) const HEALING_HYPOTHESIS_ID: HypothesisId = HypothesisId::EdgarUltimatePeriodicHealingV1;
pub(crate) const A6_DEF_HYPOTHESIS_ID: HypothesisId = HypothesisId::EdgarA6WakeTeamDefV1;

pub(crate) const KEY_HYPOTHESIS_ID: HypothesisId = HypothesisId::EdgarKeyOfTruthV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::EdgarAoeAllInRangeDummyV1;

fn schedule_heal_tick(
    ctx: &mut CastContext<'_>,
    generation: u64,
    end_time_us: u64,
    interval_us: u64,
    time_us: u64,
    flat_heal: ExactFraction,
    max_hp_ratio: ExactFraction,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us,
        priority: CausalPriority::BuffTickOrExpiry,
        seq: *ctx.sequence,
        data: GameEventData::Content(ContentEvent::EdgarHealTick {
            actor_index: ctx.actor_index,
            generation,
            end_time_us,
            interval_us,
            flat_heal,
            max_hp_ratio,
        }),
    });
    Ok(())
}

fn handle_melee(ctx: &mut CastContext<'_>, ability_id: &str) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ability_id)
    {
        return Err(
            "Edgar normal chain requires the modeled first packet as its admitted input".into(),
        );
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.edgar_mut()?;
    state.normal_stage = crate::characters::normal::after_gap(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let stage = state.normal_stage;
    state.normal_stage = (stage + 1) % 5;
    state.normal_started_us = Some(ctx.time_us);
    let packet = [
        "GE_Player_Edgar_Melee1_Damage",
        "GE_Player_Edgar_Melee2_Damage",
        "GE_Player_Edgar_Melee3_Damage",
        "GE_Player_Edgar_Melee4_Damage",
        "GE_Player_Edgar_Melee5_Damage",
    ][stage];
    if stage != 0 {
        ctx.replace_native_primary_packet(packet)?;
    }
    if stage == 2 {
        ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if super::clock::accepted(ctx.request) {
        return super::clock::cast(ctx);
    }
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1021" {
        return Ok(());
    }

    if ability_id == "GA_Edgar_Melee" {
        handle_melee(ctx, ability_id)?;
    } else {
        let state = ctx.actor.character.edgar_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    let children: &[&str] = match ability_id {
        "GA_Edgar_Skill" => &[
            "GE_Player_Edgar_Skill_Damage",
            "GE_Player_Edgar_Skill_Damage",
            "GE_Player_Edgar_Skill_Damage",
            "GE_Player_Edgar_Skill_Damage",
            "GE_Player_Edgar_Skill_Damage",
            "GE_Player_Edgar_Skill_Damage",
        ],
        _ => &[],
    };
    if !children.is_empty() {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != super::primary_packet(ctx.ability_id)
        {
            return Err(
                "Edgar packet train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
        for child in children {
            ctx.schedule_native_child_packet(child, ctx.target_index, ctx.hit_time_us)?;
        }
    }

    if ctx.request.targets.len() > 1
        && (ability_id == "GA_Edgar_Skill" || ability_id == "GA_Edgar_UltraSkill")
    {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        let all_packets: &[&str] = match ability_id {
            "GA_Edgar_Skill" => &[
                "GE_Player_Edgar_Skill_Damage",
                "GE_Player_Edgar_Skill_Damage",
                "GE_Player_Edgar_Skill_Damage",
                "GE_Player_Edgar_Skill_Damage",
                "GE_Player_Edgar_Skill_Damage",
                "GE_Player_Edgar_Skill_Damage",
                "GE_Player_Edgar_Skill_Damage",
            ],
            "GA_Edgar_UltraSkill" => &["GE_Player_Edgar_UltraSkill_Damage"],
            _ => &[],
        };
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            for packet in all_packets {
                ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
            }
        }
    }

    // GA_Edgar_UltraSkill enters Finnegan's Wake state
    // Base duration: 10s (10_000_000 us) per DT_EdgarEffectFigure GA_UltraSkill_Duration
    // Plus 1s per Key of Truth per GA_UltraSkill_PerKeyAddDuration
    if ability_id == "GA_Edgar_UltraSkill" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != super::primary_packet(ability_id)
        {
            return Err("Edgar healing requires the native Ultimate packet".into());
        }
        ctx.record_hypothesis(HEALING_HYPOTHESIS_ID)?;
        let array_index = match ctx.request.actions.get(ctx.command_index) {
            Some(crate::simulation::GameActionCommand::InvokeAbility { array_index, .. }) => {
                *array_index
            }
            _ => {
                return Err("Edgar healing requires an admitted native Ultimate coordinate".into());
            }
        };
        let coordinate: u32 = array_index
            .checked_add(1)
            .ok_or("Edgar healing talent coordinate overflow")?
            .try_into()
            .map_err(|_| "Edgar healing talent coordinate exceeds u32")?;
        let healing = super::parameters::healing(coordinate)?;
        let keys = ctx.actor.character.edgar_mut()?.keys_of_truth;
        let base_duration_us = healing.base_duration_us;
        let per_key_duration_us = healing.per_key_duration_us;
        let bonus_duration_us = u64::from(keys)
            .checked_mul(per_key_duration_us)
            .ok_or("Edgar Finnegan's Wake key duration overflow")?;
        let duration_us = base_duration_us
            .checked_add(bonus_duration_us)
            .ok_or("Edgar Finnegan's Wake duration overflow")?;
        if actor_config.awakening.has_effect(6) {
            ctx.record_hypothesis(A6_DEF_HYPOTHESIS_ID)?;
            let def_percent = healing.a6_def_ratio;
            ctx.refresh_shared_buff_while_alive(
                "edgar.a6_wake_team_def",
                ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
                duration_us,
                std::collections::BTreeMap::from([("def_percent".into(), def_percent)]),
            )?;
        }
        ctx.actor.character.edgar_mut()?.finnegan_wake_until_us = ctx
            .time_us
            .checked_add(duration_us)
            .ok_or("Edgar Finnegan's Wake deadline overflow")?;
        let interval_us = healing.interval_us;
        let flat_heal = healing.flat;
        let max_hp_ratio = healing.max_hp_ratio;
        let (generation, end_time_us) = {
            let state = ctx.actor.character.edgar_mut()?;
            state.healing_generation = state
                .healing_generation
                .checked_add(1)
                .ok_or("Edgar healing generation overflow")?;
            (state.healing_generation, state.finnegan_wake_until_us)
        };
        let first_tick_us = ctx
            .time_us
            .checked_add(interval_us)
            .ok_or("Edgar first healing tick overflow")?;
        if first_tick_us <= end_time_us {
            schedule_heal_tick(
                ctx,
                generation,
                end_time_us,
                interval_us,
                first_tick_us,
                flat_heal,
                max_hp_ratio,
            )?;
        }
    }

    Ok(())
}

pub(crate) fn heal_tick(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    generation: u64,
    _end_time_us: u64,
    interval_us: u64,
    flat_heal: ExactFraction,
    max_hp_ratio: ExactFraction,
) -> Result<(), String> {
    let end_time_us = match &ctx.state.actors[actor_index].character {
        crate::characters::RuntimeState::Edgar(state) => state.finnegan_wake_until_us,
        _ => return Err("periodic producer owner mismatch".into()),
    };

    let is_current = matches!(
        ctx.state.actors[actor_index].character,
        crate::characters::RuntimeState::Edgar(ref state)
            if state.healing_generation == generation
    );
    if !is_current {
        return Ok(());
    }
    if ctx.state.actors[actor_index].hp == 0 {
        if let crate::characters::RuntimeState::Edgar(state) =
            &mut ctx.state.actors[actor_index].character
        {
            state.healing_generation = state
                .healing_generation
                .checked_add(1)
                .ok_or("Edgar healing generation overflow")?;
        }
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "BuffTickOrExpiry".into(),
            event_kind: "edgar_healing_cancelled".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                reason: Some("source_defeated".into()),
                ..Default::default()
            }),
        });
        return Ok(());
    }
    let target_index = ctx.state.active_actor_index;
    let source_hp_max = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        ctx.state.actors[actor_index].hp_max_base,
        &ctx.state.actors[actor_index].hp_max_static_bonus,
        actor_index,
        ctx.state.actors[actor_index].hp_max,
    )?;
    let base_heal =
        flat_heal.add(ExactFraction::new(source_hp_max.into(), 1)?.mul(max_hp_ratio)?)?;
    let settlement = ctx.settle_healing(actor_index, target_index, base_heal)?;
    let gross_heal = settlement.gross;
    let before = settlement.before_hp;
    if before == 0 {
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "BuffTickOrExpiry".into(),
            event_kind: "edgar_healing_tick".into(),
            active_actor: target_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(target_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                target_alive: Some(false),
                gross: Some(gross_heal),
                applied: Some(0),
                overflow: Some(gross_heal),
                ..Default::default()
            }),
        });
    } else {
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "BuffTickOrExpiry".into(),
            event_kind: "edgar_healing_tick".into(),
            active_actor: target_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(target_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                target_alive: Some(true),
                hp_max: Some(source_hp_max),
                gross: Some(gross_heal),
                applied: Some(settlement.applied),
                overflow: Some(settlement.excess),
                hp_before: Some(before),
                hp_after: Some(settlement.after_hp),
                ..Default::default()
            }),
        });
    }
    crate::weapons::wrong_gate::on_healing(ctx, actor_index, target_index, gross_heal)?;
    let next_tick_us = ctx
        .state
        .current_time_us
        .checked_add(interval_us)
        .ok_or("Edgar next healing tick overflow")?;
    if next_tick_us <= end_time_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: next_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::EdgarHealTick {
                actor_index,
                generation,
                end_time_us,
                interval_us,
                flat_heal,
                max_hp_ratio,
            }),
        });
    }
    Ok(())
}
