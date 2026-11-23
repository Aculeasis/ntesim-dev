use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::CastContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use ntesim_core::execution::CausalPriority;

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::BaicangNormalChainV1;
pub(crate) const DOMAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::BaicangJudgmentOfAutumnDomainV1;
pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::BaicangAoeAllInRangeDummyV1;
pub(crate) const EXECUTE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::BaicangExecuteAndFatalHitDebtV1;

pub(crate) fn validate_hit_target(
    request: &crate::simulation::GameScenarioRequest,
    ability_id: &str,
    packet: &crate::simulation::metadata::PacketMetadata,
    target: &crate::simulation::state::TargetRuntime,
) -> Result<(), String> {
    if request
        .model_assumptions
        .contains_key(EXECUTE_HYPOTHESIS_ID.as_str())
    {
        return Ok(());
    }
    let domain_packet = ability_id == "GA_Cang_UltraSkill"
        && matches!(
            packet.damage_effect_id.as_str(),
            "GE_Player_Cang_UltraSkill2_Damage" | "GE_Player_Cang_UltraSkill_Damage"
        );
    if domain_packet
        && target.hp_current > 0
        && u128::from(target.hp_current) * 10 < u128::from(target.hp_max)
    {
        return Err("UNSUPPORTED_BAICANG_EXECUTE: Judgment of Autumn reached its unsupported below-10%-HP execute branch".into());
    }
    Ok(())
}

fn handle_domain_cast(
    ctx: &mut CastContext<'_>,
    ability_id: &str,
    awakenings: &ntesim_data::AwakeningSelection,
) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ability_id)
    {
        return Err("Baicang domain requires the native Ultimate primary packet".into());
    }
    ctx.record_hypothesis(DOMAIN_HYPOTHESIS_ID)?;
    let duration_us =
        super::buff_duration(awakenings, ability_id).ok_or("Baicang domain duration missing")?;
    let field_start = if super::words::accepted(ctx.request) {
        ctx.time_us
            .checked_add(2_664_744)
            .ok_or("Baicang field start overflow")?
    } else {
        ctx.time_us
    };
    if super::words::accepted(ctx.request) {
        super::timing::primary(ctx, 2_665_367, 2_871_112)?;
    }
    let baicang = ctx.actor.character.baicang_mut()?;
    baicang.domain_until_us = ctx
        .time_us
        .checked_add(field_start - ctx.time_us)
        .ok_or("Baicang field start overflow")?
        .checked_add(duration_us)
        .ok_or("Baicang domain deadline overflow")?;
    baicang.domain_generation = baicang
        .domain_generation
        .checked_add(1)
        .ok_or("Baicang domain generation overflow")?;
    let generation = baicang.domain_generation;
    let end_time_us = baicang.domain_until_us;
    let interval_us = super::DOMAIN_TICK_INTERVAL_US;

    baicang.fatal_hit_armed = true;
    baicang.domain_start_us = field_start;

    let coordinate: u32 = match ctx.request.actions.get(ctx.command_index) {
        Some(crate::simulation::GameActionCommand::InvokeAbility { array_index, .. }) => {
            (*array_index)
                .try_into()
                .map_err(|_| "Baicang domain coordinate exceeds u32")?
        }
        _ => return Err("Baicang domain requires an admitted native Burst coordinate".into()),
    };
    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &ctx.request.content_id,
            character_id: &ctx.config.character_id,
            ability_id,
            damage_effect_id: "GE_Player_Cang_UltraSkill_Damage",
            array_index: coordinate,
            research_metadata: None,
            model_assumptions: &ctx.request.model_assumptions,
        },
    )?;
    let atk_rate = serde_json::from_value(projection["compiled_rate"].clone())
        .map_err(|error| format!("Baicang domain coefficient invalid: {error}"))?;
    let scaling_stat = serde_json::from_value(projection["compiled_scaling_stat"].clone())
        .map_err(|error| format!("Baicang domain scaling stat invalid: {error}"))?;
    let metadata = ctx.child_packet_metadata("GE_Player_Cang_UltraSkill_Damage")?;
    let first_tick_us = field_start
        .checked_add(interval_us)
        .ok_or("Baicang first domain tick overflow")?;
    if first_tick_us <= end_time_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: first_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::BaicangDomainTick {
                actor_index: ctx.actor_index,
                target_index: ctx.target_index,
                command_index: ctx.command_index,
                generation,
                end_time_us,
                interval_us,
                ability_id: ability_id.to_owned(),
                damage_effect_id: "GE_Player_Cang_UltraSkill_Damage".into(),
                atk_rate,
                scaling_stat,
                metadata: Box::new(metadata),
            }),
        });
    }

    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: end_time_us,
        priority: CausalPriority::SettlementAndDeath,
        seq: *ctx.sequence,
        data: GameEventData::Content(ContentEvent::BaicangDomainEnd {
            actor_index: ctx.actor_index,
            generation,
        }),
    });

    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Cang_UltraSkill2_Damage",
                target_index,
                ctx.hit_time_us,
            )?;
        }
    }

    let power_words_accepted = ctx
        .request
        .model_assumptions
        .contains_key("baicang.power_words_v1");

    {
        let state = ctx.actor.character.baicang_mut()?;
        state.power_words_mode = power_words_accepted;
    }
    if power_words_accepted {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: field_start,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::BaicangFieldWords {
                actor_index: ctx.actor_index,
                command_index: ctx.command_index,
                generation,
                array_index: coordinate,
            }),
        });
    }

    Ok(())
}

fn handle_melee_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if super::etd::on_cast(ctx)? {
        return Ok(());
    }
    if super::branches::on_cast(ctx)? {
        return Ok(());
    }
    if super::plunge::on_cast(ctx)? {
        return Ok(());
    }
    if super::riposte::on_cast(ctx)? {
        return Ok(());
    }
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet("GA_Cang_Melee")
    {
        return Err(
            "Baicang normal chain requires the modeled first packet as its admitted input".into(),
        );
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.baicang_mut()?;
    state.normal_stage = crate::characters::normal::after_gap(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let stage = state.normal_stage;
    state.normal_stage = (stage + 1) % 5;
    state.normal_started_us = Some(if super::words::accepted(ctx.request) {
        ctx.time_us
            .saturating_add(super::timing::NORMAL_RECOVERY[stage])
    } else {
        ctx.time_us
    });
    let packet = [
        "GE_Player_Cang_Melee1_Damage",
        "GE_Player_Cang_Melee2_Damage",
        "GE_Player_Cang_Melee3_Damage",
        "GE_Player_Cang_Melee4_Damage",
        "GE_Player_Cang_Melee5_Damage",
    ][stage];
    if stage != 0 {
        ctx.replace_native_primary_packet(packet)?;
    }
    if super::words::accepted(ctx.request) {
        super::timing::normal(ctx, stage, packet)?;
        if super::clock::accepted(ctx.request) {
            ctx.actor.character.baicang_mut()?.normal_started_us = Some(ctx.actor.locked_until_us);
        }
    } else {
        for _ in 1..[1, 1, 1, 7, 1][stage] {
            ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
        }
    }

    let power_words_accepted = ctx
        .request
        .model_assumptions
        .contains_key("baicang.power_words_v1");

    {
        let state = ctx.actor.character.baicang_mut()?;
        state.power_words_mode = power_words_accepted;
    }

    Ok(())
}

fn handle_skill_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let power_words_accepted = ctx
        .request
        .model_assumptions
        .contains_key("baicang.power_words_v1");

    {
        let state = ctx.actor.character.baicang_mut()?;
        state.power_words_mode = power_words_accepted;
    }

    let (derived_stage, is_derived_active) = {
        let state = ctx.actor.character.baicang_mut()?;
        (
            state.derived_skill_stage,
            state.derived_skill_stage_expires_at_us > ctx.time_us,
        )
    };

    if power_words_accepted && is_derived_active && derived_stage == 3 {
        ctx.replace_native_primary_packet("GE_Player_Cang_SkillA_Damage")?;
        super::timing::primary(ctx, 216_575, 974_591)?;
        ctx.schedule_native_child_packet(
            "GE_Player_Cang_SkillA_Damage",
            ctx.target_index,
            ctx.hit_time_us,
        )?;
        ctx.schedule_native_child_packet(
            "GE_Player_Cang_SkillA_Damage",
            ctx.target_index,
            ctx.hit_time_us,
        )?;
        let t2 = ctx.hit_time_us.saturating_add(6);
        ctx.schedule_native_child_packet("GE_Player_Cang_SkillA_Damage2", ctx.target_index, t2)?;
        ctx.schedule_native_child_packet("GE_Player_Cang_SkillA_Damage2", ctx.target_index, t2)?;
        let t3 = ctx.hit_time_us.saturating_add(108);
        ctx.schedule_native_child_packet("GE_Player_Cang_SkillA_Damage2", ctx.target_index, t3)?;
        ctx.schedule_native_child_packet("GE_Player_Cang_SkillA_Damage2", ctx.target_index, t3)?;
        let exp_t = ctx.hit_time_us.saturating_add(300_000);
        ctx.schedule_native_child_packet("GE_Player_Cang_SkillA_Damage1", ctx.target_index, exp_t)?;
    } else if power_words_accepted && is_derived_active && derived_stage == 2 {
        ctx.replace_native_primary_packet("GE_Player_Cang_SkillB_Damage")?;
        super::timing::primary(ctx, 666_652, 971_565)?;
        ctx.scope_primary_aoe("GE_Player_Cang_SkillB_Damage")?;
    } else if power_words_accepted {
        super::timing::primary(ctx, 668_070, 961_682)?;
        ctx.scope_primary_aoe("GE_Player_Cang_Skill_Damage")?;
    }

    if power_words_accepted {
        ctx.actor.ability_cooldowns.insert(
            "GA_Cang_Skill".into(),
            ctx.time_us.saturating_add(14_000_000),
        );
        let state = ctx.actor.character.baicang_mut()?;
        state.derived_skill_stage = 0;
        if ctx.config.awakening.has_effect(1) {
            state
                .skill_stacks_by_command
                .insert(ctx.command_index, state.a1_silence_stacks);
            state.a1_silence_stacks = 0;
        }

        let credits = state.a1_objurgate_credits;
        state.a1_objurgate_credits = 0;
        if credits > 0 {
            let reduction_us = (credits as u64) * 4_000_000;
            if let Some(cd) = ctx.actor.ability_cooldowns.get("GA_Cang_Skill").copied() {
                ctx.actor.ability_cooldowns.insert(
                    "GA_Cang_Skill".into(),
                    cd.saturating_sub(reduction_us).max(ctx.time_us),
                );
            }
        }
    }

    if !power_words_accepted && ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        let packet = ctx
            .packet_metadata
            .map(|p| p.damage_effect_id.as_str())
            .unwrap_or("GE_Player_Cang_Skill_Damage");
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
        }
    }

    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1023" {
        return Ok(());
    }

    super::entry_jump::on_cast(ctx)?;
    super::support::on_cast(ctx)?;

    let awakenings = &actor_config.awakening;

    if ability_id == "GA_Cang_Melee" {
        handle_melee_cast(ctx)?;
    } else {
        let state = ctx.actor.character.baicang_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
        state.riposte_ready = false;
    }
    ctx.actor.character.baicang_mut()?.heavy_dodge_until_us = 0;

    if ability_id == "GA_Cang_Skill" {
        handle_skill_cast(ctx)?;
    }

    if ability_id == "GA_Cang_UltraSkill" {
        handle_domain_cast(ctx, ability_id, awakenings)?;
    }
    if ability_id == "GA_Cang_QTE" && super::words::accepted(ctx.request) {
        let recovery = if super::clock::accepted(ctx.request) {
            1_356_945
        } else {
            1_566_945
        };
        super::timing::primary(ctx, 1_000_358, recovery)?;
        ctx.scope_primary_aoe("GE_Player_Cang_QTE_Damage")?;
    }

    let power_words_accepted = ctx
        .request
        .model_assumptions
        .contains_key("baicang.power_words_v1");

    {
        let state = ctx.actor.character.baicang_mut()?;
        state.power_words_mode = power_words_accepted;
    }

    if !power_words_accepted {
        let generates_word = ability_id == "GA_Cang_Skill"
            || ability_id == "GA_Cang_QTE"
            || ability_id == "GA_Cang_Melee";

        if generates_word && awakenings.has_effect(2) {
            let baicang = ctx.actor.character.baicang_mut()?;
            if baicang.power_word_until_us <= ctx.time_us {
                baicang.power_word_stacks = 1;
            } else {
                baicang.power_word_stacks = (baicang.power_word_stacks + 1).min(4);
            }
            baicang.power_word_until_us = ctx
                .time_us
                .checked_add(10_000_000)
                .ok_or("Baicang power word deadline overflow")?;
        }
    }

    super::cancel::capture(ctx)?;
    super::clock::capture(ctx)
}

/// The field's own timers are born at its actual montage marker. Time before
/// deployment cannot consume or extend its lifetime/first periodic phase.
pub(crate) fn domain_started(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let state = actor.character.baicang_mut()?;
    if actor.hp == 0 || state.domain_generation != generation {
        return Ok(());
    }
    let duration = super::buff_duration(&ctx.request.actors[owner].awakening, "GA_Cang_UltraSkill")
        .ok_or("Baicang field lifetime missing")?;
    state.domain_start_us = now;
    state.domain_until_us = now
        .checked_add(duration)
        .ok_or("Baicang field lifetime overflow")?;
    let end = state.domain_until_us;
    let events = ctx.queue.drain().collect::<Vec<_>>();
    for mut event in events {
        match &event.data {
            GameEventData::Content(ContentEvent::BaicangDomainTick {
                actor_index,
                generation: live,
                interval_us,
                ..
            }) if *actor_index == owner && *live == generation => {
                event.time_us = now
                    .checked_add(*interval_us)
                    .ok_or("Baicang first tick overflow")?;
                ctx.queue
                    .push_on(ntesim_core::kernel::timing::ClockDomain::World, event);
            }
            GameEventData::Content(ContentEvent::BaicangDomainEnd {
                actor_index,
                generation: live,
            }) if *actor_index == owner && *live == generation => {
                event.time_us = end;
                ctx.queue.push_on(
                    ntesim_core::kernel::timing::ClockDomain::ObjectLifetime,
                    event,
                );
            }
            _ => ctx.queue.push(event),
        }
    }
    Ok(())
}

pub(crate) fn on_domain_end(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    generation: u64,
) -> Result<(), String> {
    let field = ctx.state.actors[actor_index].character.baicang_mut()?;
    if field.domain_generation == generation && ctx.state.current_time_us >= field.domain_until_us {
        field.fatal_hit_armed = false;
        ctx.queue.retain(|event| !matches!(&event.data,
            GameEventData::Content(crate::callbacks::ContentEvent::BaicangDomainTick { actor_index: source, generation: live, .. }
                | crate::callbacks::ContentEvent::BaicangFieldWords { actor_index: source, generation: live, .. })
                if *source == actor_index && *live == generation));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn domain_tick(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    command_index: usize,
    generation: u64,
    _end_time_us: u64,
    interval_us: u64,
    ability_id: String,
    damage_effect_id: String,
    atk_rate: ntesim_core::formula::Ratio,
    scaling_stat: crate::simulation::NativeScalingStat,
    metadata: crate::simulation::metadata::ActionPacketMetadata,
) -> Result<(), String> {
    let end_time_us = match &ctx.state.actors[actor_index].character {
        crate::characters::RuntimeState::Baicang(state) => state.domain_until_us,
        _ => return Err("periodic producer owner mismatch".into()),
    };

    let current = matches!(
        ctx.state.actors[actor_index].character,
        crate::characters::RuntimeState::Baicang(ref state)
            if state.domain_generation == generation && state.domain_until_us >= ctx.state.current_time_us
    );
    if !current {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        DOMAIN_HYPOTHESIS_ID,
    )?;
    if ctx.state.actors[actor_index].hp == 0 {
        if let crate::characters::RuntimeState::Baicang(state) =
            &mut ctx.state.actors[actor_index].character
        {
            state.domain_generation = state
                .domain_generation
                .checked_add(1)
                .ok_or("Baicang domain generation overflow")?;
        }
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "BuffTickOrExpiry".into(),
            event_kind: "baicang_domain_cancelled".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                reason: Some("owner_defeated".into()),
                ..Default::default()
            }),
        });
        return Ok(());
    }
    if ctx.state.targets[target_index].hp_current == 0 && !super::words::accepted(ctx.request) {
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "BuffTickOrExpiry".into(),
            event_kind: "baicang_domain_target_defeated".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                target_index: Some(target_index),
                ..Default::default()
            }),
        });
        return Ok(());
    }
    let recipients: Vec<_> = if super::words::accepted(ctx.request) {
        (0..ctx.state.targets.len()).collect()
    } else {
        vec![target_index]
    };
    for recipient in recipients {
        if ctx.state.targets[recipient].hp_current == 0 {
            continue;
        }
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.state.current_time_us,
            priority: CausalPriority::HitDelivery,
            seq: *ctx.sequence,
            data: GameEventData::HitDelivery {
                command_index,
                actor_index,
                target_index: recipient,
                ability_id: ability_id.clone(),
                damage_effect_id: damage_effect_id.clone(),
                atk_rate,
                scaling_stat,
                metadata: Box::new(metadata.clone()),
            },
        });
    }
    let next_tick_us = ctx
        .state
        .current_time_us
        .checked_add(interval_us)
        .ok_or("Baicang next domain tick overflow")?;
    if next_tick_us <= end_time_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: next_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::BaicangDomainTick {
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
                metadata: Box::new(metadata),
            }),
        });
    }
    Ok(())
}
