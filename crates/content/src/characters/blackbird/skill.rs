use crate::HypothesisId;
use crate::runtime_api::CastContext;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) evolve_melee_stage: usize,
    pub(crate) evolve_melee_started_us: Option<u64>,
    pub(crate) skill_permission_until_us: Option<u64>,
    pub(crate) finisher_extra_commands: std::collections::BTreeSet<usize>,
    pub(crate) nova: std::collections::BTreeMap<(usize, u64), super::nova::Accumulator>,
    pub(crate) malice: Option<ntesim_core::kernel::state::ExactPool>,
    pub(crate) malice_packets: std::collections::BTreeSet<String>,
    pub(crate) malice_skill_casts: std::collections::BTreeSet<usize>,
    pub(crate) malice_extras: std::collections::BTreeMap<String, super::malice::PendingExtra>,
    pub(crate) bird: super::bird::Controller,
    pub(crate) clock: super::clock::State,
    pub(crate) riposte_ready: bool,
    pub(crate) qte_switch_until_us: u64,
    pub(crate) qte_malice_commands: std::collections::BTreeSet<usize>,
    pub(crate) form: Option<super::form::Form>,
    pub(crate) form_generation: u64,
    pub(crate) out_of_combat: bool,
    pub(crate) combat_generation: u64,
}

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::BlackbirdNormalChainV1;
pub(crate) const EVOLVE_HYPOTHESIS_ID: HypothesisId = HypothesisId::BlackbirdEvolveActionsV1;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::BlackbirdSkillBurstPacketsV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::BlackbirdAoeAllInRangeDummyV1;
pub(crate) const GROUPS_HYPOTHESIS_ID: HypothesisId = HypothesisId::BlackbirdPrimaryPacketGroupsV1;
pub(crate) const SKILL_CYCLE_HYPOTHESIS_ID: HypothesisId = HypothesisId::BlackbirdSkillCycleFillV1;

pub(crate) fn evolve_accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(EVOLVE_HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(EVOLVE_HYPOTHESIS_ID.description())
}

pub(crate) fn validate_ability(
    request: &crate::simulation::GameScenarioRequest,
    actor: &crate::simulation::state::ActorRuntime,
    ability: &str,
) -> Result<(), String> {
    let in_form = super::form::active(actor);
    let is_evolved = matches!(
        ability,
        "GA_BlackBird_Evolve_Melee"
            | "GA_BlackBird_Evolve_Skill"
            | "GA_BlackBird_Evolve_UltraSkill"
            | "GA_BlackBird_Evolve_QTE"
    );
    if !evolve_accepted(request) {
        if in_form || is_evolved {
            return Err("UNSUPPORTED_BLACKBIRD_EVOLVE_INPUT: active replacement form requires its evolved action binding".into());
        }
        return Ok(());
    }
    if in_form {
        if !is_evolved {
            return Err(
                "ordinary abilities cannot be cast while Blackbird transformation form is active"
                    .into(),
            );
        }
    } else if is_evolved {
        return Err("Blackbird evolved abilities require active transformation form".into());
    }
    Ok(())
}

pub(crate) fn permission_admission_failure(
    actor: &crate::simulation::state::ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    let crate::characters::RuntimeState::Blackbird(state) = &actor.character else {
        return None;
    };
    if ability != "GA_BlackBird_Evolve_Skill"
        || state
            .skill_permission_until_us
            .is_some_and(|until| now <= until)
    {
        return None;
    }
    let message = "Blackbird Evolve Skill requires active skill permission".to_string();
    Some(crate::simulation::rejection::AdmissionFailure {
        reason: crate::simulation::GameAdmissionReason::Other {
            message: message.clone(),
        },
        message,
    })
}

pub(crate) fn validate_player_input(
    request: &crate::simulation::GameScenarioRequest,
    actor: &crate::simulation::state::ActorRuntime,
) -> Result<(), String> {
    if super::form::active(actor) && !evolve_accepted(request) {
        return Err("UNSUPPORTED_BLACKBIRD_EVOLVE_INPUT: active replacement form requires its evolved action binding".into());
    }
    Ok(())
}

fn cast_normal_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err(
            "Blackbird normal chain requires the modeled first packet as its admitted input".into(),
        );
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.blackbird_mut()?;
    state.normal_stage = crate::characters::normal::after_gap(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let stage = state.normal_stage;
    state.normal_stage = (stage + 1) % 5;
    state.normal_started_us = Some(ctx.time_us);
    let packet = [
        "GE_Player_BlackBird_Melee1_Damage",
        "GE_Player_BlackBird_Melee2_Damage",
        "GE_Player_BlackBird_Melee3_Damage",
        "GE_Player_BlackBird_Melee4_Damage",
        "GE_Player_BlackBird_Melee5_Damage",
    ][stage];
    if stage != 0 {
        ctx.replace_native_primary_packet(packet)?;
    }
    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(GROUPS_HYPOTHESIS_ID)?;
    }
    ctx.scope_primary_aoe(packet)?;
    for _ in 1..[1, 2, 4, 5, 5][stage] {
        ctx.schedule_native_aoe_packet(packet, ctx.hit_time_us)?;
    }
    Ok(())
}

fn cast_evolve_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !evolve_accepted(ctx.request) {
        return Err("UNSUPPORTED_BLACKBIRD_EVOLVE_INPUT: active replacement form requires its evolved action binding".into());
    }
    if !super::form::active(ctx.actor) {
        return Err("Blackbird Evolve Melee requires active transformation form".into());
    }
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err(
            "Blackbird evolved normal chain requires the modeled first packet as its admitted input".into(),
        );
    }
    ctx.record_hypothesis(EVOLVE_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.blackbird_mut()?;
    state.evolve_melee_stage = crate::characters::normal::after_gap(
        state.evolve_melee_stage,
        state.evolve_melee_started_us,
        ctx.time_us,
    );
    let stage = state.evolve_melee_stage;
    state.evolve_melee_stage = (stage + 1) % 5;
    state.evolve_melee_started_us = Some(ctx.time_us);
    let packet = [
        "GE_Player_BlackBird_Evolve_Melee1_Damage",
        "GE_Player_BlackBird_Evolve_Melee2_Damage",
        "GE_Player_BlackBird_Evolve_Melee3_Damage",
        "GE_Player_BlackBird_Evolve_Melee4_Damage",
        "GE_Player_BlackBird_Evolve_Melee5_Damage",
    ][stage];
    if stage != 0 {
        ctx.replace_native_primary_packet(packet)?;
    }
    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(GROUPS_HYPOTHESIS_ID)?;
    }
    ctx.scope_primary_aoe(packet)?;
    for _ in 1..[2, 5, 6, 1, 6][stage] {
        ctx.schedule_native_aoe_packet(packet, ctx.hit_time_us)?;
    }
    if stage == 4 {
        let progress_us = 351_288;
        let perm_time_us = ctx.time_us + progress_us;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: perm_time_us,
                priority: ntesim_core::execution::CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::Blackbird(
                        super::events::Event::SkillPermission {
                            actor_index: ctx.actor_index,
                            command_index: ctx.command_index,
                            progress_us,
                        },
                    ),
                ),
            });
    }
    Ok(())
}

fn cast_children(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let children: &[&str] = match ctx.ability_id {
        "GA_BlackBird_Skill" => &[
            "GE_Player_BlackBird_Skill_Damage",
            "GE_Player_BlackBird_Skill_Damage",
            "GE_Player_BlackBird_Skill_Damage",
        ],
        "GA_BlackBird_UltraSkill" => &[
            "GE_Player_BlackBird_UltraSkill_Damage",
            "GE_Player_BlackBird_UltraSkill_Damage",
            "GE_Player_BlackBird_UltraSkill_Damage",
            "GE_Player_BlackBird_UltraSkill_Damage",
            "GE_Player_BlackBird_UltraSkill_Damage",
            "GE_Player_BlackBird_UltraSkill_Damage",
            "GE_Player_BlackBird_UltraSkill_Damage",
            "GE_Player_BlackBird_UltraSkill_Damage",
        ],
        "GA_BlackBird_Evolve_Skill" => &[
            "GE_Player_BlackBird_Evolve_Skill_Damage",
            "GE_Player_BlackBird_Evolve_Skill_Damage",
            "GE_Player_BlackBird_Evolve_Skill_Damage",
        ],
        "GA_BlackBird_Evolve_UltraSkill" => &[
            "GE_Player_BlackBird_Evolve_UltraSkill_Damage",
            "GE_Player_BlackBird_Evolve_UltraSkill_Damage",
            "GE_Player_BlackBird_Evolve_UltraSkill_Damage",
            "GE_Player_BlackBird_Evolve_UltraSkill_Damage",
            "GE_Player_BlackBird_Evolve_UltraSkill_Damage",
            "GE_Player_BlackBird_Evolve_UltraSkill_Damage",
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
                "Blackbird packet train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
        if ctx.request.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            ctx.record_hypothesis(GROUPS_HYPOTHESIS_ID)?;
        }
        ctx.scope_primary_aoe(
            super::primary_packet(ctx.ability_id).ok_or("Blackbird primary missing")?,
        )?;
        for child in children {
            ctx.schedule_native_aoe_packet(child, ctx.hit_time_us)?;
        }
    }
    Ok(())
}

fn cast_evolve_skill(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !evolve_accepted(ctx.request) {
        return Err("UNSUPPORTED_BLACKBIRD_EVOLVE_INPUT: active replacement form requires its evolved action binding".into());
    }
    if !super::form::active(ctx.actor) {
        return Err("Blackbird Evolve Skill requires active transformation form".into());
    }
    let state = ctx.actor.character.blackbird_mut()?;
    if state
        .skill_permission_until_us
        .is_none_or(|until| ctx.time_us > until)
    {
        return Err("Blackbird Evolve Skill requires active skill permission".into());
    }
    state.skill_permission_until_us = None;
    ctx.record_hypothesis(EVOLVE_HYPOTHESIS_ID)?;
    ctx.actor.ability_cooldowns.insert(
        ctx.ability_id.into(),
        ctx.time_us
            .checked_add(1_000_000)
            .ok_or("cooldown overflow")?,
    );
    if ctx.config.awakening.has_effect(2) {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: ntesim_core::execution::CausalPriority::ResourceCost,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::Blackbird(super::events::Event::SkillHeal {
                        actor_index: ctx.actor_index,
                        command_index: ctx.command_index,
                    }),
                ),
            });
    }
    Ok(())
}

fn cast_evolve_ultra(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !evolve_accepted(ctx.request) {
        return Err("UNSUPPORTED_BLACKBIRD_EVOLVE_INPUT: active replacement form requires its evolved action binding".into());
    }
    if !super::form::active(ctx.actor) {
        return Err("Blackbird Evolve UltraSkill requires active transformation form".into());
    }
    ctx.record_hypothesis(EVOLVE_HYPOTHESIS_ID)?;
    super::form::end_cast(ctx)?;
    ctx.actor
        .character
        .blackbird_mut()?
        .skill_permission_until_us = None;
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1042" {
        return Ok(());
    }
    let ability_id = ctx.ability_id;
    super::support::on_cast(ctx)?;
    if super::branches::on_cast(ctx)? {
        return super::clock::on_cast(ctx);
    }
    if super::secondary::on_cast(ctx)? {
        return super::clock::on_cast(ctx);
    }
    ctx.actor.character.blackbird_mut()?.riposte_ready = false;

    if ability_id == "GA_BlackBird_Melee" {
        cast_normal_melee(ctx)?;
    } else if ability_id == "GA_BlackBird_Evolve_Melee" {
        cast_evolve_melee(ctx)?;
    } else {
        let state = ctx.actor.character.blackbird_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
        state.evolve_melee_stage = 0;
        state.evolve_melee_started_us = None;
    }

    cast_children(ctx)?;

    if ability_id == "GA_BlackBird_Skill" {
        super::bird::on_skill(ctx)?;
        super::malice::on_skill(ctx)?;
        ctx.record_hypothesis(SKILL_CYCLE_HYPOTHESIS_ID)?;
        ctx.actor.cycle_ledger.fill(&mut ctx.actor.cycle)?;
    } else if ability_id == "GA_BlackBird_Evolve_Skill" {
        cast_evolve_skill(ctx)?;
    } else if ability_id == "GA_BlackBird_Evolve_UltraSkill" {
        cast_evolve_ultra(ctx)?;
    }

    if ability_id == "GA_BlackBird_UltraSkill" && !super::form::accepted(ctx.request) {
        super::buffs::burst(ctx.buffs, ctx.config, ctx.actor_index, ctx.world_time_us)?;
    }

    super::form::on_cast(ctx)?;
    super::clock::on_cast(ctx)
}

pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    super::clock::on_hit(ctx)?;
    super::bird::on_hit(ctx)?;
    super::malice::on_hit(ctx)?;
    if ctx.config.awakening.has_effect(5) && ctx.applied_hp_loss > 0 {
        // Source: DT_BlackBirdEffectFigure.json Buff_BlackBird042_Level5_CutDef_DamageResistPsycheBase = -0.08.
        let reductions = ["PSYCHE", "LAKSHANA", "CHAOS"]
            .into_iter()
            .map(|element| {
                Ok((
                    format!("ECharacterElementType::CHARACTER_ELEMENT_TYPE_{element}"),
                    super::parameters::PARAMETERS.a5_resistance_reduction,
                ))
            })
            .collect::<Result<_, String>>()?;
        ctx.refresh_status(
            "blackbird.revenant",
            super::parameters::PARAMETERS.a5_resistance_duration_us,
            reductions,
        )?;
    }
    Ok(())
}
