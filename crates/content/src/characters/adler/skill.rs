use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::ExactFraction;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use ntesim_core::execution::CausalPriority;

pub(crate) const BLESSING_HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerBlessingShieldV1;

pub(crate) const A4_OVERFLOW_HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerBlessingA4OverflowV1;

pub(crate) const R6_HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerBlessingR6V1;

pub(crate) const SKILL_TRAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerSkillTrainV1;
pub(crate) const SKILL_ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::AdlerSkillAllInRangeDummyV1;
pub(crate) const KARMA_HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerKarmaV1;

pub(crate) fn blessing_shield_id(granter_index: usize) -> String {
    format!("adler.blessing:{granter_index}")
}

fn on_blessing_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1033" || ctx.ability_id != "GA_Adler_Skill" {
        return Ok(());
    }
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != Some("GE_Player_Adler_Skill2_Damage")
    {
        return Err(
            "Adler Blessing shield requires the modeled native Skill initial packet".into(),
        );
    }
    ctx.record_hypothesis(BLESSING_HYPOTHESIS_ID)?;
    let array_index = match ctx.request.actions.get(ctx.command_index) {
        Some(crate::simulation::GameActionCommand::InvokeAbility { array_index, .. }) => {
            *array_index
        }
        _ => return Err("Adler Skill shield requires an admitted ability coordinate".into()),
    };
    let level = u32::try_from(array_index)
        .ok()
        .and_then(|index| index.checked_add(1))
        .ok_or("Adler Skill shield level exceeds native curve coordinates")?;
    let scaling = super::values::blessing(level)?;
    let flat = ExactFraction::new(scaling.flat.into(), 1)?;
    let coefficient = ExactFraction::from_ratio(scaling.defense_coefficient)?;
    let living_sources = ctx
        .team_hp_snapshot
        .iter()
        .enumerate()
        .filter_map(|(index, (hp, _))| (*hp > 0).then_some(index))
        .collect::<std::collections::BTreeSet<_>>();
    let live_stats = ctx
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(
            ctx.actor_index,
            ctx.active_actor_index,
            ctx.world_time_us,
            &living_sources,
        )
        .map_err(|error| format!("Adler Skill live DEF query failed: {error:?}"))?;
    let def = crate::runtime_api::source_def(ctx.request, ctx.actor_index, &live_stats)?;
    let amount = flat.add(coefficient.mul(def)?)?;
    let duration_us = if ctx.config.awakening.has_effect(6) {
        12_000_000
    } else {
        8_000_000
    };
    let recipient_index = ctx.active_actor_index;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.time_us,
        priority: CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::AdlerBlessingShield {
            granter_index: ctx.actor_index,
            recipient_index,
            amount,
            duration_us,
        }),
    });
    Ok(())
}

fn on_skill_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != Some("GE_Player_Adler_Skill2_Damage")
    {
        return Err("Adler Skill requires its modeled Skill2 initial packet".into());
    }
    ctx.record_hypothesis(SKILL_TRAIN_HYPOTHESIS_ID)?;
    ctx.record_hypothesis(SKILL_ALL_IN_RANGE_HYPOTHESIS_ID)?;
    ctx.record_hypothesis(KARMA_HYPOTHESIS_ID)?;
    ctx.record_hypothesis(
        crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID,
    )?;
    let state = ctx.actor.character.adler_mut()?;
    let consumed = state.karma_stacks;
    state.karma_stacks = 0;
    state
        .skill_bonus_by_command
        .insert(ctx.command_index, consumed);
    state.skill_generation = state
        .skill_generation
        .checked_add(1)
        .ok_or("Adler Skill generation overflow")?;
    let generation = state.skill_generation;
    state.skill_deadlines.clear();
    let targets = ctx
        .targets
        .iter()
        .enumerate()
        .filter_map(|(index, target)| (target.hp_current > 0).then_some(index))
        .collect::<Vec<_>>();
    for &target_index in &targets {
        if target_index != ctx.target_index {
            ctx.schedule_native_child_packet(
                "GE_Player_Adler_Skill2_Damage",
                target_index,
                ctx.hit_time_us,
            )?;
        }
        ctx.schedule_native_child_packet(
            "GE_Player_Adler_Skill3_Damage",
            target_index,
            ctx.hit_time_us,
        )?;
        ctx.schedule_native_child_packet(
            "GE_Player_Adler_Skill3_Damage",
            target_index,
            ctx.hit_time_us,
        )?;
    }
    let interval_us = super::values::SKILL_TICK_INTERVAL_US;
    let duration_us = super::values::SKILL_TRAIN_DURATION_US;
    let first_tick_us = ctx
        .hit_time_us
        .checked_add(interval_us)
        .ok_or("Adler first Skill tick overflow")?;
    let end_time_us = ctx
        .hit_time_us
        .checked_add(duration_us)
        .ok_or("Adler Skill train end overflow")?;
    let coordinate = match ctx.request.actions.get(ctx.command_index) {
        Some(crate::simulation::GameActionCommand::InvokeAbility { array_index, .. }) => {
            *array_index
        }
        _ => return Err("Adler Skill requires an admitted native coordinate".into()),
    };
    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &ctx.request.content_id,
            character_id: &ctx.config.character_id,
            ability_id: ctx.ability_id,
            damage_effect_id: "GE_Player_Adler_Skill_Damage",
            array_index: coordinate
                .try_into()
                .map_err(|_| "Adler Skill coordinate exceeds u32")?,
            research_metadata: None,
            model_assumptions: &ctx.request.model_assumptions,
        },
    )?;
    let atk_rate = serde_json::from_value(projection["compiled_rate"].clone())
        .map_err(|error| format!("Adler Skill tick coefficient invalid: {error}"))?;
    let scaling_stat = serde_json::from_value(projection["compiled_scaling_stat"].clone())
        .map_err(|error| format!("Adler Skill tick scaling stat invalid: {error}"))?;
    let metadata = ctx.child_packet_metadata("GE_Player_Adler_Skill_Damage")?;
    for target_index in targets {
        ctx.actor
            .character
            .adler_mut()?
            .skill_deadlines
            .insert(target_index, end_time_us);
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: first_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::AdlerSkillTick(
                SkillTick {
                    actor_index: ctx.actor_index,
                    target_index,
                    command_index: ctx.command_index,
                    generation,
                    end_time_us,
                    interval_us,
                    ability_id: ctx.ability_id.to_owned(),
                    damage_effect_id: "GE_Player_Adler_Skill_Damage".into(),
                    atk_rate,
                    scaling_stat,
                    metadata: Box::new(metadata.clone()),
                },
            )),
        });
    }
    Ok(())
}

pub(crate) fn apply_blessing_shield(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    granter_index: usize,
    recipient_index: usize,
    raw_amount: ExactFraction,
    duration_us: u64,
) -> Result<(), String> {
    if ctx.state.actors[granter_index].hp == 0 || ctx.state.actors[recipient_index].hp == 0 {
        return Ok(());
    }
    let outgoing_shield_bonus = ctx.outgoing_shield_bonus(granter_index, recipient_index)?;
    let awakening_bonus = if ctx.request.actors[granter_index].awakening.has_effect(3) {
        ExactFraction::new(1, 5)?
    } else {
        ExactFraction::new(0, 1)?
    };
    let amount = raw_amount
        .mul(
            ExactFraction::new(1, 1)?
                .add(outgoing_shield_bonus)?
                .add(awakening_bonus)?,
        )?
        .round_u64()?;
    let max_hp = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        ctx.state.actors[recipient_index].hp_max_base,
        &ctx.state.actors[recipient_index].hp_max_static_bonus,
        recipient_index,
        ctx.state.actors[recipient_index].hp_max,
    )?;
    let id = blessing_shield_id(granter_index);
    let now_us = ctx.state.current_time_us;
    let expires_at_us = crate::simulation::state::replace_actor_shield(
        &mut ctx.state.actors[recipient_index],
        max_hp,
        now_us,
        &id,
        &id,
        amount,
        duration_us,
    )?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now_us,
        phase: "ProcCallback".into(),
        event_kind: "adler_blessing_shield".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(granter_index),
        to_actor: Some(recipient_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            amount: Some(amount),
            expires_at_us: Some(expires_at_us),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn shield_owner(shield_id: &str) -> Option<usize> {
    shield_id
        .strip_prefix("adler.blessing:")
        .and_then(|index| index.parse().ok())
}

pub(crate) fn is_a4_shield(
    request: &crate::simulation::GameScenarioRequest,
    shield_id: &str,
) -> bool {
    shield_owner(shield_id).is_some_and(|owner| {
        request.actors.get(owner).is_some_and(|actor| {
            actor.character_id == "character:1033" && actor.awakening.has_effect(4)
        })
    })
}

pub(crate) fn recipient_modifiers(
    ctx: &crate::runtime_api::RecipientContext<'_>,
    mut value: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    if ctx.source_config.awakening.count() >= 6
        && ctx.source_actor.hp > 0
        && ctx.packet.is_some_and(|p| {
            p.element == Some(crate::simulation::metadata::DamageElement::Incantation)
        })
    {
        let shield_id = blessing_shield_id(ctx.source_index);
        let shield_active = ctx.team_actors.iter().any(|recipient| {
            recipient.hp > 0
                && recipient.shields.iter().any(|shield| {
                    shield.id == shield_id
                        && shield.remaining_hp > 0
                        && shield.expires_at_us > ctx.time_us
                })
        });
        if shield_active {
            value = value.add_damage(crate::simulation::ExactFraction::new(1, 10)?)?;
            value.record_hypothesis(ctx.request, R6_HYPOTHESIS_ID)?;
        }
    }
    Ok(value)
}

#[cfg(test)]
pub(crate) fn r6_incantation_bonus_active(
    request: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    now_us: u64,
    packet: &crate::simulation::metadata::PacketMetadata,
) -> bool {
    if packet.element != Some(crate::simulation::metadata::DamageElement::Incantation) {
        return false;
    }
    for (granter_index, granter) in request.actors.iter().enumerate() {
        if granter.character_id != "character:1033"
            || granter.awakening.count() < 6
            || state
                .actors
                .get(granter_index)
                .is_none_or(|actor| actor.hp == 0)
        {
            continue;
        }
        let shield_id = blessing_shield_id(granter_index);
        if state.actors.iter().any(|recipient| {
            recipient.hp > 0
                && recipient.shields.iter().any(|shield| {
                    shield.id == shield_id
                        && shield.remaining_hp > 0
                        && shield.expires_at_us > now_us
                })
        }) {
            return true;
        }
    }
    false
}

pub(crate) fn a4_overflow_cap(
    request: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    source_index: usize,
) -> Result<Option<u64>, String> {
    if !request.actors.get(source_index).is_some_and(|actor| {
        actor.character_id == "character:1033" && actor.awakening.has_effect(4)
    }) || state
        .actors
        .get(source_index)
        .is_none_or(|actor| actor.hp == 0)
    {
        return Ok(None);
    }
    let living_sources = state
        .actors
        .iter()
        .enumerate()
        .filter_map(|(index, actor)| (actor.hp > 0).then_some(index))
        .collect::<std::collections::BTreeSet<_>>();
    let live_stats = state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(
            source_index,
            state.active_actor_index,
            state.mode_clock.relative_us(),
            &living_sources,
        )
        .map_err(|error| format!("Adler A4 live DEF query failed: {error:?}"))?;
    let def = crate::runtime_api::source_def(request, source_index, &live_stats)?;
    Ok(Some(def.mul(ExactFraction::new(1, 2)?)?.round_u64()?))
}

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerNormalChainV1;

pub(crate) const BURST_HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerBurstSunyasV1;

pub(crate) const A5_ENERGY_HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerA5BurstEnergyRefundV1;

fn on_burst_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.ability_id != "GA_Adler_UltraSkill" {
        return Ok(());
    }
    let packet = "GE_Player_Adler_UltraSkill_Damage";
    if ctx
        .packet_metadata
        .map(|metadata| metadata.damage_effect_id.as_str())
        != Some(packet)
    {
        return Err("Adler Burst requires its modeled native primary packet".into());
    }
    if ctx.targets.len() != ctx.request.targets.len() {
        return Err("Adler Burst requires live target population at admission".into());
    }
    let use_intervals = ctx
        .request
        .model_assumptions
        .contains_key(HypothesisId::AdlerBurstIntervalsV1.as_str());
    if use_intervals {
        ctx.record_hypothesis(HypothesisId::AdlerBurstIntervalsV1)?;
        let targets: Vec<_> = ctx
            .targets
            .iter()
            .enumerate()
            .filter_map(|(index, target)| (target.hp_current > 0).then_some(index))
            .collect();
        let is_sole = targets.len() == 1;
        ctx.cancel_primary_delivery()?;
        let volley1_time_us = ctx.time_us + 2_185_103;
        for _ in 0..5 {
            for &target in &targets {
                ctx.schedule_native_child_packet(packet, target, volley1_time_us)?;
            }
        }
        if is_sole {
            let volley2_time_us = ctx.time_us + 2_343_074;
            for _ in 0..5 {
                ctx.schedule_native_child_packet(packet, targets[0], volley2_time_us)?;
            }
        }
        let callback_time_us = ctx.time_us + 2_343_074;
        if ctx.config.awakening.has_effect(5) {
            ctx.record_hypothesis(A5_ENERGY_HYPOTHESIS_ID)?;
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push_on(
                crate::simulation::timer_policy::action(ctx.actor_index, ctx.command_index, false),
                ScheduledGameEvent {
                    time_us: callback_time_us,
                    priority: CausalPriority::ProcCallback,
                    seq: *ctx.sequence,
                    data: GameEventData::Content(
                        crate::callbacks::ContentEvent::AdlerA5EnergyGrant {
                            actor_index: ctx.actor_index,
                        },
                    ),
                },
            );
        }
        if ctx.config.awakening.has_effect(1) {
            ctx.record_hypothesis(KARMA_HYPOTHESIS_ID)?;
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push_on(
                crate::simulation::timer_policy::action(ctx.actor_index, ctx.command_index, false),
                ScheduledGameEvent {
                    time_us: callback_time_us,
                    priority: CausalPriority::ProcCallback,
                    seq: *ctx.sequence,
                    data: GameEventData::Content(
                        crate::callbacks::ContentEvent::AdlerA1KarmaGrant {
                            actor_index: ctx.actor_index,
                        },
                    ),
                },
            );
        }
        return Ok(());
    }

    ctx.record_hypothesis(BURST_HYPOTHESIS_ID)?;
    if ctx.config.awakening.has_effect(5) {
        ctx.record_hypothesis(A5_ENERGY_HYPOTHESIS_ID)?;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::AdlerA5EnergyGrant {
                actor_index: ctx.actor_index,
            }),
        });
    }
    if ctx.config.awakening.has_effect(1) {
        ctx.record_hypothesis(KARMA_HYPOTHESIS_ID)?;
        let state = ctx.actor.character.adler_mut()?;
        state.karma_stacks = state.karma_stacks.saturating_add(20).min(20);
    }
    let targets: Vec<_> = ctx
        .targets
        .iter()
        .enumerate()
        .filter_map(|(index, target)| (target.hp_current > 0).then_some(index))
        .collect();
    let strikes = if targets.len() == 1 { 10 } else { 5 };
    for strike in 0..strikes {
        for &target in &targets {
            if strike != 0 || target != ctx.target_index {
                ctx.schedule_native_child_packet(packet, target, ctx.hit_time_us)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    on_blessing_cast(ctx)?;
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1033" {
        return Ok(());
    }

    if super::riposte::on_cast(ctx)? {
        return Ok(());
    }

    if ctx
        .packet_metadata
        .is_some_and(|packet| super::plunge::is_plunge_packet(&packet.damage_effect_id))
    {
        super::plunge::on_cast(ctx)?;
        return Ok(());
    }

    if ability_id == "GA_Adler_Skill" {
        on_skill_cast(ctx)?;
    }

    on_burst_cast(ctx)?;

    if ability_id == "GA_Adler_Melee" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            == super::primary_packet(ability_id)
        {
            ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
            let state = ctx.actor.character.adler_mut()?;
            state.normal_stage = crate::characters::normal::after_gap(
                state.normal_stage,
                state.normal_started_us,
                ctx.time_us,
            );
            let stage = state.normal_stage;
            state.normal_stage = (stage + 1) % 5;
            state.normal_started_us = Some(ctx.time_us);
            match stage {
                0 => {}
                1 => {
                    ctx.replace_native_primary_packet("GE_Player_Adler_Melee2_Damage")?;
                }
                2 => {
                    ctx.replace_native_primary_packet("GE_Player_Adler_Melee3_Damage")?;
                    for _ in 0..6 {
                        ctx.schedule_native_child_packet(
                            "GE_Player_Adler_Melee3_Damage",
                            ctx.target_index,
                            ctx.hit_time_us,
                        )?;
                    }
                }
                3 => {
                    ctx.replace_native_primary_packet("GE_Player_Adler_Melee4_1_Damage")?;
                    ctx.schedule_native_child_packet(
                        "GE_Player_Adler_Melee4_2_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                    if ctx
                        .request
                        .model_assumptions
                        .contains_key(HypothesisId::AdlerNormalChain13V1.as_str())
                    {
                        ctx.record_hypothesis(HypothesisId::AdlerNormalChain13V1)?;
                        ctx.schedule_native_child_packet(
                            "GE_Player_Adler_Melee4_3_Damage",
                            ctx.target_index,
                            ctx.hit_time_us,
                        )?;
                    }
                }
                4 => {
                    ctx.replace_native_primary_packet("GE_Player_Adler_Melee5_Damage")?;
                }
                _ => unreachable!(),
            }
        }
    } else {
        let state = ctx.actor.character.adler_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &crate::runtime_api::DamageContext<'_>,
    mut bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let crate::characters::RuntimeState::Adler(state) = &ctx.actor.character else {
        return Ok(bonus);
    };
    let skill_bonus_stacks = state
        .skill_bonus_by_command
        .get(&ctx.command_index)
        .copied()
        .unwrap_or_default();
    if ctx.ability_id == "GA_Adler_Skill" && skill_bonus_stacks > 0 {
        let per_stack = if ctx.config.awakening.has_effect(2) {
            3
        } else {
            2
        };
        bonus = bonus.add_final(ExactFraction::new(
            u128::from(skill_bonus_stacks) * per_stack,
            100,
        )?)?;
    }
    Ok(bonus)
}

pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    if ctx.target_index != ctx.request.aim_target(ctx.command_index) {
        return Ok(());
    }
    let gain = match ctx.damage_effect_id {
        "GE_Player_Adler_Aim_Damage" => 2,
        "GE_Player_Adler_Melee1_Damage"
        | "GE_Player_Adler_Melee2_Damage"
        | "GE_Player_Adler_Melee3_Damage"
        | "GE_Player_Adler_Melee4_1_Damage"
        | "GE_Player_Adler_Melee4_2_Damage"
        | "GE_Player_Adler_Melee4_3_Damage"
        | "GE_Player_Adler_Melee5_Damage"
        | "GE_Player_Adler_AirAttack_Damage" => 1,
        _ => return Ok(()),
    };
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        ctx.execution_assumptions,
        KARMA_HYPOTHESIS_ID,
    )?;
    let state = ctx.actor.character.adler_mut()?;
    state.karma_stacks = state.karma_stacks.saturating_add(gain).min(20);
    Ok(())
}

pub(crate) fn transfer_party_shield(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    new_active_actor: usize,
) -> Result<(), String> {
    if ctx.state.actors[new_active_actor].hp == 0 {
        return Ok(());
    }
    let interception = ctx
        .request
        .model_assumptions
        .contains_key(HypothesisId::AdlerBlessingPartyInterceptionV1.as_str());
    if !interception {
        return Ok(());
    }
    let now_us = ctx.state.current_time_us;
    for (granter_index, granter) in ctx.request.actors.iter().enumerate() {
        if granter.character_id != "character:1033" || ctx.state.actors[granter_index].hp == 0 {
            continue;
        }
        let shield_id = blessing_shield_id(granter_index);
        let actor_count = ctx.state.actors.len();
        for other in 0..actor_count {
            if other == new_active_actor {
                continue;
            }
            if let Some(pos) = ctx.state.actors[other]
                .shields
                .iter()
                .position(|s| s.id == shield_id && s.remaining_hp > 0 && s.expires_at_us > now_us)
            {
                let shield = ctx.state.actors[other].shields.remove(pos);
                crate::runtime_api::record_hypothesis_id(
                    ctx.request,
                    &mut ctx.state.execution_model_assumptions,
                    HypothesisId::AdlerBlessingPartyInterceptionV1,
                )?;
                let max_hp = crate::simulation::state::effective_hp_max(
                    &ctx.state.buffs,
                    ctx.state.actors[new_active_actor].hp_max_base,
                    &ctx.state.actors[new_active_actor].hp_max_static_bonus,
                    new_active_actor,
                    ctx.state.actors[new_active_actor].hp_max,
                )?;
                let duration_remaining = shield.expires_at_us.saturating_sub(now_us);
                crate::simulation::state::replace_actor_shield(
                    &mut ctx.state.actors[new_active_actor],
                    max_hp,
                    now_us,
                    &shield.id,
                    &shield.source,
                    shield.remaining_hp,
                    duration_remaining,
                )?;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SkillTick {
    actor_index: usize,
    target_index: usize,
    command_index: usize,
    generation: u64,
    end_time_us: u64,
    interval_us: u64,
    ability_id: String,
    damage_effect_id: String,
    atk_rate: ntesim_core::formula::Ratio,
    scaling_stat: crate::simulation::NativeScalingStat,
    metadata: Box<crate::simulation::metadata::ActionPacketMetadata>,
}

pub(crate) fn skill_tick(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    tick: SkillTick,
) -> Result<(), String> {
    let SkillTick {
        actor_index,
        target_index,
        command_index,
        generation,
        mut end_time_us,
        interval_us,
        ability_id,
        damage_effect_id,
        atk_rate,
        scaling_stat,
        metadata,
    } = tick;
    let current = matches!(
        ctx.state.actors[actor_index].character,
        crate::characters::RuntimeState::Adler(ref state)
            if state.skill_generation == generation
    );
    if !current {
        return Ok(());
    }
    if let crate::characters::RuntimeState::Adler(state) = &ctx.state.actors[actor_index].character
    {
        end_time_us = state
            .skill_deadlines
            .get(&target_index)
            .copied()
            .unwrap_or(end_time_us);
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        SKILL_TRAIN_HYPOTHESIS_ID,
    )?;
    if ctx.state.actors[actor_index].hp == 0 || ctx.state.targets[target_index].hp_current == 0 {
        return Ok(());
    }
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.state.current_time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index,
            actor_index,
            target_index,
            ability_id: ability_id.clone(),
            damage_effect_id: damage_effect_id.clone(),
            atk_rate,
            scaling_stat,
            metadata: metadata.clone(),
        },
    });
    let next_tick_us = ctx
        .state
        .current_time_us
        .checked_add(interval_us)
        .ok_or("Adler next Skill tick overflow")?;
    if next_tick_us <= end_time_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: next_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::AdlerSkillTick(
                SkillTick {
                    actor_index,
                    target_index,
                    command_index,
                    generation,
                    end_time_us,
                    interval_us,
                    ability_id,
                    damage_effect_id,
                    atk_rate,
                    scaling_stat,
                    metadata,
                },
            )),
        });
    }
    Ok(())
}

pub(crate) fn spread_source(
    actor: &crate::simulation::state::ActorRuntime,
    target: usize,
    now: u64,
) -> bool {
    actor.hp > 0
        && matches!(&actor.character,crate::characters::RuntimeState::Adler(state) if state.skill_deadlines.get(&target).is_some_and(|end|now<*end))
}
pub(crate) fn spread_train(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    let end = ctx
        .state
        .current_time_us
        .checked_add(super::values::SKILL_TRAIN_DURATION_US)
        .ok_or("Adler spread expiry overflow")?;
    let state = ctx.state.actors[owner].character.adler_mut()?;
    for (target, dummy) in ctx.state.targets.iter().enumerate() {
        if dummy.hp_current > 0 {
            // Adler's producer covers all living stationary dummies. Live
            // copies refresh those same carriers and retain their queued phase.
            let deadline = state
                .skill_deadlines
                .get_mut(&target)
                .ok_or("Adler spread recipient has no attached carrier")?;
            *deadline = end;
        }
    }
    Ok(())
}

pub(crate) fn attached_dot_stacks(
    actor: &crate::simulation::state::ActorRuntime,
    target: usize,
    now: u64,
) -> u32 {
    match &actor.character {
        crate::characters::RuntimeState::Adler(state) => u32::from(
            state
                .skill_deadlines
                .get(&target)
                .is_some_and(|end| now < *end),
        ),
        _ => 0,
    }
}
