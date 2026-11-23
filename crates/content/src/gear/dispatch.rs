use super::config::GearProcConfig;
use super::source_stats;
use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) fn needs_successful_skill_finish(config: &crate::simulation::GameActorConfig) -> bool {
    config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::GoldWool { .. }))
}

pub(crate) fn on_successful_skill_finish(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    crate::weapons::gold_wool::on_skill_finish(ctx, owner)
}

pub(crate) fn on_switch(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    previous_actor: usize,
) -> Result<(), String> {
    crate::weapons::motor_candy::on_switch(ctx, previous_actor)?;
    crate::weapons::bit_game::on_switch(ctx, previous_actor)
}

pub(crate) fn on_reaction_participation(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    participants: [usize; 2],
    reaction: &ntesim_data::reaction_values::ReactionResult,
) -> Result<(), String> {
    if participants[0] == participants[1]
        || participants
            .iter()
            .any(|index| ctx.state.actors[*index].hp == 0)
    {
        return Ok(());
    }
    // Prepare participant and team-listener effects before publishing any state.
    let mut buffs = ctx.state.buffs.clone();
    let mut assumptions = ctx.state.execution_model_assumptions.clone();
    let mut receipts = Vec::new();
    for actor_index in 0..ctx.state.actors.len() {
        if ctx.state.actors[actor_index].hp == 0 {
            continue;
        }
        let participant_triggered = participants.contains(&actor_index)
            && crate::sets::diabolos::on_reaction_participation(
                ctx.request,
                actor_index,
                ctx.state.mode_clock.relative_us(),
                reaction,
                &mut buffs,
                &mut assumptions,
            )?;
        let team_triggered = crate::sets::street_boxer::on_team_reaction(
            ctx.request,
            actor_index,
            ctx.state.mode_clock.relative_us(),
            reaction,
            &mut buffs,
            &mut assumptions,
        )?;
        if participant_triggered || team_triggered {
            receipts.push(crate::simulation::GameEventReceipt {
                time_us: ctx.state.current_time_us,
                phase: "ReactionOrBreak".into(),
                event_kind: "reaction_equipment_window".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: None,
                to_actor: Some(actor_index),
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    reaction: Some(format!("{reaction:?}")),
                    ..Default::default()
                }),
            });
        }
    }
    ctx.state.buffs = buffs;
    ctx.state.execution_model_assumptions = assumptions;
    ctx.trace.extend(receipts);
    Ok(())
}

pub(crate) const CAST_BUFF_HYPOTHESIS_ID: HypothesisId = HypothesisId::GearCastBuffBeforeHitsV1;
pub(crate) const SKILL_CAST_BUFF_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::GearSkillCastBuffBeforeHitsV1;
pub(crate) const QTE_CAST_BUFF_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::GearQteCastBuffBeforeHitsV1;

pub(crate) fn schedule_cast_buff(
    ctx: &mut CastContext<'_>,
    event: crate::callbacks::ContentEvent,
) -> Result<(), String> {
    match ctx.action_metadata.input_role {
        Some(crate::simulation::metadata::InputActionRole::RedirectSkill) => {
            ctx.record_hypothesis(SKILL_CAST_BUFF_HYPOTHESIS_ID)?;
        }
        Some(crate::simulation::metadata::InputActionRole::Qte) => {
            ctx.record_hypothesis(QTE_CAST_BUFF_HYPOTHESIS_ID)?;
        }
        _ => {
            ctx.record_hypothesis(CAST_BUFF_HYPOTHESIS_ID)?;
        }
    }
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(event),
        });
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    crate::sets::quiet_manor::on_cast(ctx)?;
    crate::sets::hp_suit::on_cast(ctx)?;
    crate::weapons::be_happy::on_cast(ctx)?;
    crate::weapons::nakupeda::on_cast(ctx)?;
    crate::weapons::noros::on_cast(ctx)?;
    crate::weapons::bopu::on_cast(ctx)?;
    crate::weapons::gold_wool::on_cast(ctx)?;
    crate::weapons::motor_candy::on_cast(ctx)?;
    crate::sets::shadow_creed::on_cast(ctx)?;
    crate::sets::lost_radiance::on_cast(ctx)?;
    crate::weapons::knight_candy::on_cast(ctx)?;
    crate::weapons::oraora::on_cast(ctx)?;
    crate::weapons::boxing_candy::on_cast(ctx)?;
    crate::weapons::paper_plane::on_cast(ctx)?;
    crate::weapons::worldrain::on_cast(ctx)?;
    crate::weapons::prokaryon::on_cast(ctx)?;
    crate::weapons::appliance::on_cast(ctx)?;
    crate::weapons::blast_candy::on_cast(ctx)?;
    crate::weapons::twinbirds::on_cast(ctx)?;
    crate::weapons::eternal_waltz::on_cast(ctx)?;
    crate::weapons::castle::on_cast(ctx)?;
    crate::weapons::raging_flames::on_cast(ctx)?;
    crate::weapons::far_side_moon::on_cast(ctx)?;
    crate::weapons::lunarphase::on_cast(ctx)?;
    crate::weapons::forgotten::on_cast(ctx)?;
    crate::weapons::koinobori::on_cast(ctx)?;
    crate::weapons::wrong_gate::on_cast(ctx)?;
    crate::weapons::gold_record::on_cast(ctx)?;
    crate::weapons::marching_beyond_time::on_cast(ctx)?;
    crate::weapons::spider::on_cast(ctx)?;
    crate::weapons::bit_game::on_cast(ctx)?;
    crate::weapons::demon_blade::on_cast(ctx)?;
    crate::weapons::moon::on_cast(ctx)?;
    crate::weapons::mind_royale::cast::on_cast(ctx)?;
    crate::weapons::good_boy::cast::on_cast(ctx)?;
    crate::weapons::butterfly::on_cast(ctx)?;
    crate::weapons::kite::on_cast(ctx)?;
    crate::weapons::crowbar::on_cast(ctx)?;
    crate::weapons::thief_candy::on_cast(ctx)?;
    crate::weapons::rose::on_cast(ctx)?;
    crate::weapons::tiger_tally::on_cast(ctx)?;
    crate::sets::speedy_hedgehog::on_cast(ctx)?;
    crate::registry::weapon_passive_on_cast("", ctx)?;
    crate::registry::set_effect_on_cast("", ctx)?;
    let _ = crate::registry::weapon_passive_id("");
    let _ = crate::registry::set_effect_id("");
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &crate::runtime_api::DamageContext<'_>,
    packet: &crate::simulation::metadata::PacketMetadata,
    modifiers: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let modifiers = modifiers.add_atk_percent(source_stats::atk_percent(
        ctx.config,
        ctx.actor,
        ctx.time_us,
        ctx.team_has_three_elements,
    )?)?;
    let modifiers = crate::weapons::paper_plane::damage_bonus(ctx.config, packet, modifiers)?;
    let modifiers = crate::weapons::worldrain::damage_bonus(ctx.config, packet, modifiers)?;
    let modifiers = crate::weapons::gold_wool::damage_bonus(ctx.config, packet, modifiers)?;
    let modifiers = crate::weapons::prokaryon::damage_bonus(ctx.config, packet, modifiers)?;
    let modifiers = crate::weapons::appliance::damage_bonus(ctx.config, packet, modifiers)?;
    let modifiers = crate::weapons::twinbirds::damage_bonus(ctx.config, packet, modifiers)?;
    let modifiers = crate::weapons::raging_flames::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::far_side_moon::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::oraora::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::moon::damage_bonus(ctx.config, packet, modifiers)?;
    let modifiers = crate::weapons::boxing_candy::damage_bonus(ctx, modifiers)?;
    let modifiers = crate::weapons::forgotten::hit_modifiers(ctx, modifiers)?;
    let modifiers = crate::weapons::koinobori::hit_modifiers(ctx, modifiers)?;
    let modifiers = crate::weapons::gold_record::hit_modifiers(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::marching_beyond_time::damage_bonus(ctx, modifiers)?;
    let modifiers = crate::weapons::butterfly::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::mamen::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::kite::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::whale::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::jiaojuan::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::yaodao::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::police_rat::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::rose::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::tiger_tally::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::weapons::black_book::damage_bonus(ctx, packet, modifiers)?;
    let modifiers = crate::registry::weapon_passive_damage_bonus("", ctx, modifiers)?;
    let modifiers = crate::registry::set_effect_damage_bonus("", ctx, modifiers)?;
    let modifiers = crate::sets::devils_blood::damage_bonus(ctx, modifiers)?;
    let modifiers = crate::sets::diabolos::damage_bonus(ctx, packet, modifiers)?;
    crate::weapons::bit_game::damage_bonus(ctx, packet, modifiers)
}

pub(crate) fn observe_team_admission(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    actor_index: usize,
    command_index: usize,
    action: &crate::simulation::metadata::ActionMetadata,
) -> Result<(), String> {
    crate::weapons::marching_beyond_time::observe_team_admission(
        request,
        state,
        actor_index,
        command_index,
        action.input_role,
    )?;
    crate::weapons::black_book::observe_team_admission(
        request,
        state,
        actor_index,
        command_index,
        action,
    )
}

pub(crate) fn on_target_break_damage(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target_index: usize,
) -> Result<(), String> {
    crate::weapons::black_book::on_target_break_damage(ctx, target_index)
}

pub(crate) fn crit_rate_bonus(
    ctx: &crate::runtime_api::DamageContext<'_>,
) -> Result<crate::simulation::ExactFraction, String> {
    crate::weapons::demon_blade::crit_rate_bonus(ctx.config)?
        .add(crate::weapons::far_side_moon::crit_rate_bonus(ctx.config)?)?
        .add(crate::sets::street_boxer::crit_rate_bonus(ctx)?)
}

pub(crate) fn on_hit(
    ctx: &mut CastContext<'_>,
    critical: bool,
    target_index: usize,
) -> Result<(), String> {
    crate::weapons::rose::on_hit(ctx)?;
    crate::weapons::knight_candy::on_hit(ctx, critical)?;
    crate::weapons::oraora::on_hit(ctx)?;
    crate::weapons::demon_blade::on_hit(ctx)?;
    crate::weapons::moon::on_hit(ctx)?;
    crate::weapons::spider::on_hit(ctx)?;
    crate::weapons::bit_game::on_hit(ctx)?;
    crate::weapons::nest_bird::on_hit(ctx, target_index)?;
    crate::weapons::yaodao::on_hit(ctx)?;
    crate::registry::weapon_passive_on_hit("", ctx, critical)?;
    crate::registry::set_effect_on_hit("", ctx, critical)?;
    crate::sets::crimson::on_hit(ctx)
}

pub(crate) fn on_hp_loss_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    crate::weapons::gold_record::on_hit(ctx)
}

pub(crate) fn on_control(
    ctx: &mut CastContext<'_>,
    control: &crate::simulation::GameControlApplication,
) -> Result<(), String> {
    crate::weapons::good_boy::on_control(ctx, control)
}

pub(crate) fn on_broken_target_hit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    execution_assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    crate::weapons::rose::on_broken_target_hit(
        ctx,
        actor_index,
        target_index,
        execution_assumptions,
    )
}

pub(crate) fn validate_hit_delivery(
    request: &crate::simulation::GameScenarioRequest,
    config: &crate::simulation::GameActorConfig,
    metadata: &crate::simulation::metadata::ActionPacketMetadata,
) -> Result<(), String> {
    crate::weapons::yaodao::validate_trigger_assumption(request, config, metadata)
}

pub(crate) fn on_research_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    crate::sets::fireflies::hit::on_research_hit(ctx)
}

pub(crate) fn on_team_hit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    packet: &crate::simulation::metadata::PacketMetadata,
    callback_time_us: u64,
    execution_assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    crate::sets::fireflies::hit::on_team_hit(
        ctx,
        actor_index,
        packet,
        callback_time_us,
        execution_assumptions,
    )
}

pub(crate) fn on_damage_received(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    actual_hp_loss: u64,
) -> Result<(), String> {
    crate::weapons::bittercake::on_damage_received(ctx, actor_index, actual_hp_loss)?;
    crate::sets::hp_suit::on_damage_received(ctx, actor_index, actual_hp_loss)
}

pub(crate) fn on_target_defeated(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
) -> Result<(), String> {
    crate::weapons::be_happy::on_target_defeated(ctx, actor_index, target_index)?;
    crate::weapons::whale::on_target_defeated(ctx, actor_index, target_index)?;
    crate::weapons::rishi::on_target_defeated(ctx, actor_index, target_index)
}
