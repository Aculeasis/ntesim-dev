use super::events::Event;
use super::parameters::PARAMETERS;
use super::passive;
use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt};
use ntesim_core::execution::CausalPriority;

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::ShinkuNormalChainV1;
pub(crate) const SKILL_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::ShinkuSkillPacketsV1;
pub(crate) const BURST_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::ShinkuBurstPacketsV1;
pub(crate) const DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ShinkuDirectUltimateFollowupsV1;
pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ShinkuAoeAllInRangeDummyV1;
pub(crate) const PRE_DAWN_REFRESH_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ShinkuPreDawnRefreshV1;
pub(crate) const DEFIANT_SPIRIT_AND_RAGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ShinkuDefiantSpiritAndRageV1;

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        == super::primary_packet("GA_Shinku_Melee")
    {
        ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
        let in_rage = {
            let state = ctx.actor.character.shinku_mut()?;
            state.normal_stage = crate::characters::normal::after_gap(
                state.normal_stage,
                state.normal_started_us,
                ctx.time_us,
            );
            state.surging_crimson_until_us > ctx.time_us
        };
        if in_rage {
            ctx.record_hypothesis(DEFIANT_SPIRIT_AND_RAGE_HYPOTHESIS_ID)?;
        }
        let stage = {
            let state = ctx.actor.character.shinku_mut()?;
            let stage = state.normal_stage;
            state.normal_stage = (stage + 1) % 5;
            state.normal_started_us = Some(ctx.time_us);
            stage
        };
        if in_rage {
            match stage {
                0 => {
                    ctx.replace_native_primary_packet("GE_Player_Shinku_Melee1_Rage_Damage")?;
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_Melee1_1_Rage_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                1 => {
                    ctx.replace_native_primary_packet("GE_Player_Shinku_Melee2_Rage_Damage")?;
                }
                2 => {
                    ctx.replace_native_primary_packet("GE_Player_Shinku_Melee3_Rage_Damage")?;
                    for _ in 0..5 {
                        ctx.schedule_native_child_packet(
                            "GE_Player_Shinku_Melee3_1_Rage_Damage",
                            ctx.target_index,
                            ctx.hit_time_us,
                        )?;
                    }
                }
                3 => {
                    ctx.replace_native_primary_packet("GE_Player_Shinku_Melee4_Rage_Damage")?;
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_Melee4_1_Rage_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                4 => {
                    ctx.replace_native_primary_packet("GE_Player_Shinku_Melee5_Rage_Damage")?;
                }
                _ => unreachable!(),
            }
        } else {
            match stage {
                0 => {}
                1 => {
                    ctx.replace_native_primary_packet("GE_Player_Shinku_Melee1_2_Damage")?;
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_Melee1_2_1_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                2 => {
                    ctx.replace_native_primary_packet("GE_Player_Shinku_Melee1_3_Damage")?;
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_Melee1_3_1_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_Melee1_3_2_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_Melee1_3_3_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                3 => {
                    ctx.replace_native_primary_packet("GE_Player_Shinku_Melee1_4_Damage")?;
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_Melee1_4_1_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                4 => {
                    ctx.replace_native_primary_packet("GE_Player_Shinku_Melee1_5_Damage")?;
                }
                _ => unreachable!(),
            }
        }
    } else {
        let state = ctx.actor.character.shinku_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1076" {
        return Ok(());
    }

    if ability_id == "GA_Shinku_Melee" {
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.shinku_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if matches!(
        ability_id,
        "GA_Shinku_Melee" | "GA_Shinku_Skill" | "GA_Shinku_UltraSkill" | "GA_Shinku_QTE"
    ) {
        passive::on_attack_cast(ctx)?;
    }

    // Ordinary Skill (does not activate Surging Crimson) vs Scarlet Descent (Rage Skill).
    if ability_id == "GA_Shinku_Skill" {
        let packet_id = ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str());
        if packet_id == Some("GE_Player_Shinku_Skill1_Rage_Damage") {
            ctx.record_hypothesis(SKILL_PACKETS_HYPOTHESIS_ID)?;
            ctx.record_hypothesis(DEFIANT_SPIRIT_AND_RAGE_HYPOTHESIS_ID)?;
            if ctx.request.targets.len() > 1 {
                ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Shinku_Skill2_Rage_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            for target_index in 0..ctx.request.targets.len() {
                if target_index == ctx.target_index {
                    continue;
                }
                for packet in [
                    "GE_Player_Shinku_Skill1_Rage_Damage",
                    "GE_Player_Shinku_Skill2_Rage_Damage",
                ] {
                    ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
                }
            }
            let fading_cost = ExactFraction::new(PARAMETERS.fading_reason_max.into(), 1)?;
            let shinku = ctx.actor.character.shinku_mut()?;
            if let Some(pool) = &mut shinku.fading_reason {
                pool.spend_up_to(fading_cost)?;
            }
            shinku.skill_rage_ready_us = ctx
                .time_us
                .checked_add(PARAMETERS.skill_rage_cooldown_us)
                .ok_or("Shinku Scarlet Descent cooldown overflow")?;
            shinku.scarlet_descent_casts += 1;
            shinku.crimson_judgment_stacks = (shinku.crimson_judgment_stacks + 1).min(3);

            if ctx.config.awakening.has_effect(5) {
                ctx.record_hypothesis(passive::MODEL)?;
                crate::runtime_api::increment_sequence(ctx.sequence)?;
                ctx.queue.push(ScheduledGameEvent {
                    time_us: ctx.hit_time_us,
                    priority: CausalPriority::ProcCallback,
                    seq: *ctx.sequence,
                    data: GameEventData::Content(ContentEvent::Shinku(Event::ScarletDescentHeal {
                        owner: ctx.actor_index,
                        command: ctx.command_index,
                    })),
                });
            }
        } else if packet_id == Some("GE_Player_Shinku_Skill1_1_Damage") {
            ctx.record_hypothesis(SKILL_PACKETS_HYPOTHESIS_ID)?;
            if ctx.request.targets.len() > 1 {
                ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            for packet in [
                "GE_Player_Shinku_Skill1_2_Damage",
                "GE_Player_Shinku_Skill1_3_Damage",
            ] {
                ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
            }
            for target_index in 0..ctx.request.targets.len() {
                if target_index == ctx.target_index {
                    continue;
                }
                for packet in [
                    "GE_Player_Shinku_Skill1_1_Damage",
                    "GE_Player_Shinku_Skill1_2_Damage",
                    "GE_Player_Shinku_Skill1_3_Damage",
                ] {
                    ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
                }
            }
        } else {
            return Err("Shinku ordinary Skill train requires the modeled first packet as its admitted primary".into());
        }
    }

    if ability_id == "GA_Shinku_UltraSkill" {
        let packet_id = ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str());
        if packet_id == Some("GE_Player_Shinku_UltraSkillPre_Rage_Damage") {
            ctx.record_hypothesis(DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID)?;

            let is_judgment_dash = ctx
                .actor
                .character
                .shinku()
                .map(|s| {
                    s.surging_crimson_until_us > ctx.time_us
                        && s.surging_crimson_until_us.saturating_sub(ctx.time_us) <= 4_000_000
                        && s.crimson_judgment_stacks > 0
                })
                .unwrap_or(false);
            if is_judgment_dash {
                if ctx.request.targets.len() > 1 {
                    ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                }
                for _ in 0..3 {
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_UltraSkillPre_Rage_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                for target_index in 0..ctx.request.targets.len() {
                    if target_index == ctx.target_index {
                        continue;
                    }
                    for _ in 0..4 {
                        ctx.schedule_native_child_packet(
                            "GE_Player_Shinku_UltraSkillPre_Rage_Damage",
                            target_index,
                            ctx.hit_time_us,
                        )?;
                    }
                }
                let shinku = ctx.actor.character.shinku_mut()?;
                shinku.crimson_judgment_stacks = shinku.crimson_judgment_stacks.saturating_sub(1);
                shinku.crimson_judgment_combo = true;
            }
        } else if packet_id == Some("GE_Player_Shinku_UltraSkill_Rage_Damage") {
            ctx.record_hypothesis(DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID)?;
            let is_verdict = ctx
                .actor
                .character
                .shinku()
                .map(|s| {
                    s.surging_crimson_until_us > ctx.time_us
                        && s.surging_crimson_until_us.saturating_sub(ctx.time_us) <= 4_000_000
                        && s.crimson_judgment_stacks == 0
                })
                .unwrap_or(false);
            if is_verdict {
                ctx.record_hypothesis(DEFIANT_SPIRIT_AND_RAGE_HYPOTHESIS_ID)?;
                if ctx.request.targets.len() > 1 {
                    ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                }
                for _ in 0..9 {
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_UltraSkill_Rage_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill2_Rage_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                for target_index in 0..ctx.request.targets.len() {
                    if target_index == ctx.target_index {
                        continue;
                    }
                    for _ in 0..10 {
                        ctx.schedule_native_child_packet(
                            "GE_Player_Shinku_UltraSkill_Rage_Damage",
                            target_index,
                            ctx.hit_time_us,
                        )?;
                    }
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_UltraSkill2_Rage_Damage",
                        target_index,
                        ctx.hit_time_us,
                    )?;
                }
                // Client GA_Shinku_UltraSkill_Rage.EndAbilityRemoveGEs clears Surging Crimson
                // after Verdict's hits settle, rather than before damage evaluation.
                crate::runtime_api::increment_sequence(ctx.sequence)?;
                ctx.queue.push(ScheduledGameEvent {
                    time_us: ctx.hit_time_us,
                    priority: CausalPriority::ProcCallback,
                    seq: *ctx.sequence,
                    data: GameEventData::Content(ContentEvent::Shinku(Event::EndVerdictRage {
                        owner: ctx.actor_index,
                        command: ctx.command_index,
                    })),
                });
            }
        } else {
            if packet_id != super::primary_packet(ability_id) {
                return Err(
                    "Shinku Burst train requires the modeled first packet as its admitted primary"
                        .into(),
                );
            }
            ctx.record_hypothesis(BURST_PACKETS_HYPOTHESIS_ID)?;
            ctx.record_hypothesis(DEFIANT_SPIRIT_AND_RAGE_HYPOTHESIS_ID)?;
            if ctx.request.targets.len() > 1 {
                ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Shinku_UltraSkill2_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            for _ in 0..5 {
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill3_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Shinku_UltraSkill4_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Shinku_UltraSkill5_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            for target_index in 0..ctx.request.targets.len() {
                if target_index == ctx.target_index {
                    continue;
                }
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill1_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill2_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
                for _ in 0..5 {
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_UltraSkill3_Damage",
                        target_index,
                        ctx.hit_time_us,
                    )?;
                }
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill4_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill5_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
            }
            let fading_max = ExactFraction::new(PARAMETERS.fading_reason_max.into(), 1)?;
            let shinku = ctx.actor.character.shinku_mut()?;
            if let Some(pool) = &mut shinku.fading_reason {
                pool.spend_up_to(fading_max)?;
            }
            shinku.paid_burst_until_us = ctx.time_us + 20_000_000;
            shinku.surging_crimson_until_us = ctx
                .time_us
                .checked_add(PARAMETERS.rage_duration_us)
                .ok_or("Shinku surging_crimson deadline overflow")?;
            shinku.skill_rage_ready_us = 0;
            shinku.scarlet_descent_casts = 0;
            shinku.crimson_judgment_stacks = 0;
            shinku.crimson_judgment_combo = false;
            if ctx
                .actor
                .ability_cooldowns
                .get("GA_Shinku_UltraSkill")
                .copied()
                .unwrap_or(0)
                > ctx.time_us
            {
                ctx.actor
                    .ability_cooldowns
                    .insert("GA_Shinku_UltraSkill".into(), ctx.time_us + 9_000_000);
            }
            ctx.actor.ability_cooldowns.remove("GA_Shinku_Skill");
            if ctx.config.awakening.has_effect(5) {
                ctx.record_hypothesis(passive::MODEL)?;
                passive::apply_a5_def_buff(
                    ctx.buffs,
                    ctx.actor_index,
                    ctx.world_time_us,
                    PARAMETERS.rage_duration_us,
                )?;
            }
        }
    }

    Ok(())
}

pub(crate) fn end_verdict_rage(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    passive::remove_a5_def_buff(&mut ctx.state.buffs, owner);
    let now = ctx.state.current_time_us;
    let paid_burst_until = {
        let shinku = ctx.state.actors[owner].character.shinku_mut()?;
        shinku.surging_crimson_until_us = 0;
        shinku.skill_rage_ready_us = 0;
        shinku.crimson_judgment_stacks = 0;
        shinku.scarlet_descent_casts = 0;
        if let Some(pool) = &mut shinku.fading_reason {
            let max = ExactFraction::new(PARAMETERS.fading_reason_max.into(), 1)?;
            pool.spend_up_to(max)?;
        }
        shinku.paid_burst_until_us
    };
    if paid_burst_until > now {
        ctx.state.actors[owner]
            .ability_cooldowns
            .insert("GA_Shinku_UltraSkill".into(), paid_burst_until);
    }
    Ok(())
}

pub(crate) fn apply_a5_heal(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let base_atk = ctx.request.actors[owner].static_stats.base_atk;
    let base_heal = ExactFraction::new(
        u128::from(base_atk) * u128::from(PARAMETERS.a5_heal_atk_multiplier),
        1,
    )?;
    let settlement = ctx.settle_healing(owner, owner, base_heal)?;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "shinku_a5_scarlet_descent_heal".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            gross: Some(settlement.gross),
            ..Default::default()
        }),
    });
    Ok(())
}

fn hit_resource_gain(
    effect_id: &str,
    in_rage: bool,
    tables: &std::collections::BTreeMap<String, serde_json::Value>,
    content_id: &str,
) -> Result<Option<(bool, ExactFraction)>, String> {
    if effect_id == "GE_Player_Shinku_Watch_Damage" {
        return Ok(Some((in_rage, PARAMETERS.watch_resource_gain)));
    }
    if effect_id == "GE_Player_Shinku_QTE_Damage" {
        if in_rage {
            return Ok(None);
        }
        return Ok(Some((false, ExactFraction::new(53, 10)?)));
    }
    let is_rage_melee = matches!(
        effect_id,
        "GE_Player_Shinku_Melee1_Rage_Damage"
            | "GE_Player_Shinku_Melee1_1_Rage_Damage"
            | "GE_Player_Shinku_Melee2_Rage_Damage"
            | "GE_Player_Shinku_Melee3_Rage_Damage"
            | "GE_Player_Shinku_Melee3_1_Rage_Damage"
            | "GE_Player_Shinku_Melee4_Rage_Damage"
            | "GE_Player_Shinku_Melee4_1_Rage_Damage"
            | "GE_Player_Shinku_Melee5_Rage_Damage"
    );
    let is_ordinary_melee = matches!(
        effect_id,
        "GE_Player_Shinku_Melee1_1_Damage"
            | "GE_Player_Shinku_Melee1_2_Damage"
            | "GE_Player_Shinku_Melee1_2_1_Damage"
            | "GE_Player_Shinku_Melee1_3_Damage"
            | "GE_Player_Shinku_Melee1_3_1_Damage"
            | "GE_Player_Shinku_Melee1_3_2_Damage"
            | "GE_Player_Shinku_Melee1_3_3_Damage"
            | "GE_Player_Shinku_Melee1_4_Damage"
            | "GE_Player_Shinku_Melee1_4_1_Damage"
            | "GE_Player_Shinku_Melee1_5_Damage"
    );
    let is_ordinary_skill = matches!(
        effect_id,
        "GE_Player_Shinku_Skill1_1_Damage"
            | "GE_Player_Shinku_Skill1_2_Damage"
            | "GE_Player_Shinku_Skill1_3_Damage"
    );
    if (!in_rage && !(is_ordinary_melee || is_ordinary_skill)) || (in_rage && !is_rage_melee) {
        return Ok(None);
    }
    let row =
        ntesim_data::native_action_metadata::resolve_damage_effect(content_id, tables, effect_id)?;
    let charge_value = row
        .raw_row
        .as_ref()
        .and_then(|r| r.get("ChargeAdd"))
        .ok_or_else(|| format!("Shinku hit {effect_id} missing ChargeAdd"))?;
    let scalar = ntesim_data::native::parse_native_scalar(charge_value)?;
    let native_charge = ExactFraction::new(scalar.numerator(), scalar.denominator())?;
    let scale = if is_ordinary_melee {
        // 18 / 17.6 = 45 / 44
        ExactFraction::new(45, 44)?
    } else if is_ordinary_skill {
        // 5 / 5.2 = 25 / 26
        ExactFraction::new(25, 26)?
    } else {
        // 13 / 12.998 = 6500 / 6499
        ExactFraction::new(6500, 6499)?
    };
    Ok(Some((in_rage, native_charge.mul(scale)?)))
}

fn grant_gauges_on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let in_rage = {
        let state = ctx.actor.character.shinku_mut()?;
        if let Some(group) = ctx.packet_group_id
            && !state.granted_resource_groups.insert(group.to_owned())
        {
            return Ok(());
        }
        state.surging_crimson_until_us >= ctx.time_us
    };
    let Some((to_fading, amount)) = hit_resource_gain(
        ctx.damage_effect_id,
        in_rage,
        ntesim_data::skill_levels::frozen::tables(),
        &ctx.request.content_id,
    )?
    else {
        return Ok(());
    };
    if amount.is_zero() {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        ctx.execution_assumptions,
        DEFIANT_SPIRIT_AND_RAGE_HYPOTHESIS_ID,
    )?;
    if to_fading {
        let state = ctx.actor.character.shinku_mut()?;
        let pool = state
            .fading_reason
            .as_mut()
            .ok_or("Shinku Fading Reason pool missing")?;
        let (applied, _) = pool.grant(amount)?;
        if !applied.is_zero() {
            ctx.trace.push(GameEventReceipt {
                time_us: ctx.time_us,
                phase: "HitDelivery".into(),
                event_kind: "shinku_fading_reason_grant".into(),
                active_actor: ctx.active_actor_index,
                hit_index: Some(ctx.hit_index),
                from_actor: Some(ctx.actor_index),
                to_actor: Some(ctx.actor_index),
                energy_change: None,
                energy_fanout: None,
                data: Some(GameEventPayload {
                    reason: Some(format!("{}/{}", applied.numerator, applied.denominator)),
                    ..Default::default()
                }),
            });
        }
    } else {
        let snapshot = crate::simulation::energy::EnergySnapshot::capture(ctx.actor);
        ctx.actor.grant_unscaled_fractional_energy(amount)?;
        let receipt = snapshot.receipt(
            ctx.actor,
            crate::simulation::GameEnergySource::ShinkuHit,
            ctx.actor_index,
            ctx.actor_index,
            Some(amount),
        )?;
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.time_us,
            phase: "HitDelivery".into(),
            event_kind: "shinku_ordinary_hit_energy_grant".into(),
            active_actor: ctx.active_actor_index,
            hit_index: Some(ctx.hit_index),
            from_actor: Some(ctx.actor_index),
            to_actor: Some(ctx.actor_index),
            energy_change: Some(receipt),
            energy_fanout: None,
            data: None,
        });
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1076" {
        return Ok(());
    }
    if ctx.config.awakening.has_effect(2) && ctx.ability_id == "GA_Shinku_UltraSkill" {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            ctx.execution_assumptions,
            PRE_DAWN_REFRESH_HYPOTHESIS_ID,
        )?;
        let reductions = std::collections::BTreeMap::from([(
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS".into(),
            PARAMETERS.a2_cosmos_resistance_reduction,
        )]);
        ctx.refresh_status(
            "shinku.pre_dawn",
            PARAMETERS.a2_resistance_duration_us,
            reductions,
        )?;
    }
    grant_gauges_on_hit(ctx)?;
    passive::try_trigger_watch_hits(ctx)?;
    Ok(())
}
