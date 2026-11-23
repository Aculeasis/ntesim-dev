use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::ZankouNormalChainV1;
pub(crate) const HEARTWRENCH_HYPOTHESIS_ID: HypothesisId = super::forms::MODEL;

pub(crate) fn public_skill_timing() -> (u64, u64, u64) {
    (
        PARAMETERS.skill_recovery_us,
        PARAMETERS.skill_cooldown_us,
        PARAMETERS.skill_primary_delay_us,
    )
}

pub(crate) fn direct_packet_energy(
    ability: &str,
    packet: &str,
) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
    ((ability == "GA_Zankou_QTE" && packet == "GE_Player_Zankou_QTE_Damage")
        || (ability == "GA_Zankou_Skill"
            && matches!(
                packet,
                "GE_Player_Zankou_Skill1_Damage"
                    | "GE_Player_Zankou_Skill1_1_Damage"
                    | "GE_Player_Zankou_Skill2_Damage"
                    | "GE_Player_Zankou_Skill2_1_Damage"
                    | "GE_Player_Zankou_Skill3_Damage"
                    | "GE_Player_Zankou_Skill3_1_Damage"
                    | "GE_Player_Zankou_Skill3_2_Damage"
                    | "GE_Player_Zankou_Skill4_Damage"
                    | "GE_Player_Zankou_Skill4_1_Damage"
                    | "GE_Player_Zankou_Skill4_2_Damage"
            )))
    .then_some(crate::runtime_api::DirectPacketEnergyPolicy {
        trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
        hypothesis_id: HEARTWRENCH_HYPOTHESIS_ID,
        event_kind: "zankou_skill_direct_packet_energy_grant",
    })
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1036" {
        return Ok(());
    }
    if ability_id == "GA_Zankou_QTE" {
        let kind =
            super::secondary::kind(ctx)?.ok_or("Zankou Support requires its native packet")?;
        return super::secondary::cast(ctx, kind);
    }

    let state = ctx.actor.character.zankou_mut()?;
    if state.form == super::state::Form::Illusion && ctx.time_us >= state.form_until_us {
        state.form = super::state::Form::Reality;
        state.form_until_us = 0;
        state.form_generation = state
            .form_generation
            .checked_add(1)
            .ok_or("Zankou form generation overflow")?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if ability_id == "GA_Zankou_Melee" {
        return super::normal::cast(ctx);
    }
    if ability_id != "GA_Zankou_Skill" {
        super::forms::stop(ctx.actor, ctx.queue, ctx.buffs, ctx.time_us)?;
    }
    {
        let state = ctx.actor.character.zankou_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    let is_skill = ability_id == "GA_Zankou_Skill";
    let is_ultra = ability_id == "GA_Zankou_UltraSkill";
    if is_skill {
        return super::forms::cast(ctx);
    }
    if is_ultra {
        return super::burst::handle_burst(ctx);
    }

    Ok(())
}

pub(crate) fn initialize_team_effect(
    config: &crate::simulation::GameActorConfig,
    actor_index: usize,
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
) -> Result<(), String> {
    super::modifiers::initialize(config, actor_index, buffs)
}

pub(crate) fn validate_packet_delivery(
    _config: &crate::simulation::GameActorConfig,
    actor: &crate::simulation::state::ActorRuntime,
    packet: &crate::simulation::metadata::PacketMetadata,
    target_has_scorch: bool,
    scorch_replacement_modeled: bool,
    _target_would_be_defeated: bool,
) -> Result<(), String> {
    if actor.hp == 0 || !target_has_scorch {
        return Ok(());
    }
    if !scorch_replacement_modeled
        && packet
            .semantic_tags
            .contains(crate::characters::packet_semantics::DOT_DAMAGE_TAG)
    {
        return Err(
            "unsupported reached Zankou Passive1 dependency: DoT delivery on a Scorch target requires source-bound Scorch refresh and damage payload".into(),
        );
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    super::clock::contact(ctx)?;
    if super::vile_ash::is_vile_ash_packet(ctx.damage_effect_id) {
        super::vile_ash::continue_dot_train(ctx)?;
    }
    if ctx.config.awakening.count() >= 6 {
        crate::simulation::state::apply_timed_buff(
            ctx.buffs,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "zankou_resonance_6_atk",
                "character:1036",
                ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
                PARAMETERS.resonance_duration_us,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([(
                    "atk_percent".into(),
                    PARAMETERS.resonance_atk_percent,
                )]),
            ),
            ctx.actor_index,
            ctx.world_time_us,
        )?;
    }
    Ok(())
}

pub(crate) fn on_scorch_target_defeated(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target: usize,
) -> Result<(), String> {
    super::vile_ash::target_defeated(ctx, target)?;
    let now_us = ctx.state.current_time_us;
    let had_scorch = ctx
        .state
        .reactions
        .scorch
        .get(&target)
        .is_some_and(|s| s.expires_at_us > now_us)
        || ctx
            .state
            .buffs
            .active_tagged_stacks_for_target(
                &target.to_string(),
                crate::characters::packet_semantics::SCORCH_STATUS_TAG,
                now_us,
            )
            .unwrap_or(0)
            > 0;
    if !had_scorch {
        return Ok(());
    }
    for (actor_idx, actor_config) in ctx.request.actors.iter().enumerate() {
        if actor_config.character_id != "character:1036" {
            continue;
        }
        if ctx.state.actors[actor_idx].hp == 0 {
            continue;
        }
        if actor_config.awakening.count() >= 6 {
            let kindling_stacks = {
                let zankou = ctx.state.actors[actor_idx].character.zankou_mut()?;
                zankou.r6_enhancement_until_us = now_us
                    .checked_add(60_000_000)
                    .ok_or("Zankou R6 enhancement overflow")?;
                zankou.kindling_stacks
            };

            ctx.trace.push(crate::simulation::GameEventReceipt {
                time_us: now_us,
                phase: "TargetDefeated".into(),
                event_kind: "zankou_r6_scorch_kill_enhancement".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: Some(actor_idx),
                to_actor: Some(actor_idx),
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    duration_us: Some(60_000_000),
                    stacks: Some(kindling_stacks),
                    ..Default::default()
                }),
            });
        }
    }
    Ok(())
}

pub(crate) fn on_reaction_participation(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    participants: [usize; 2],
    reaction: &ntesim_data::reaction_values::ReactionResult,
) -> Result<(), String> {
    if !matches!(
        reaction,
        ntesim_data::reaction_values::ReactionResult::Scorch
            | ntesim_data::reaction_values::ReactionResult::Hexed
    ) {
        return Ok(());
    }
    for &actor_idx in &participants {
        if actor_idx >= ctx.request.actors.len() {
            continue;
        }
        let config = &ctx.request.actors[actor_idx];
        if config.character_id != "character:1036" || !config.awakening.has_effect(2) {
            continue;
        }
        if ctx.state.actors[actor_idx].hp == 0 {
            continue;
        }
        let zankou = ctx.state.actors[actor_idx].character.zankou_mut()?;
        zankou.kindling_stacks = zankou
            .kindling_stacks
            .saturating_add(1)
            .min(super::passives::KINDLING_CAP);
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ReactionParticipation".into(),
            event_kind: "zankou_a2_kindling_generated".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_idx),
            to_actor: Some(actor_idx),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                reaction: Some(
                    match reaction {
                        ntesim_data::reaction_values::ReactionResult::Scorch => "scorch",
                        _ => "hexed",
                    }
                    .into(),
                ),
                stacks: Some(zankou.kindling_stacks),
                ..Default::default()
            }),
        });
    }
    Ok(())
}
