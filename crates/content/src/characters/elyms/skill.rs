//! Elyms Connection Strength, Skill Redirect (`Skill1`/`Skill2`/`Skill3`), QTE, and `Do You Get Deja Vu`.
use super::parameters::PARAMETERS;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::rejection::AdmissionFailure;
use crate::simulation::state::ActorRuntime;
use crate::simulation::{GameActionCommand, GameAdmissionReason, GameScenarioRequest};
use std::collections::BTreeMap;

pub(crate) const CHAIN_HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::ElymsNormalChainV1;
pub(crate) const SKILL_REDIRECT_HYPOTHESIS_ID: crate::HypothesisId =
    crate::HypothesisId::ElymsSkillRedirectV1;
pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: crate::HypothesisId =
    crate::HypothesisId::ElymsAoeAllInRangeDummyV1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Connection {
    #[default]
    Weak = 0,
    Medium = 1,
    Strong = 2,
}

impl Connection {
    pub(crate) fn as_u64(self) -> u64 {
        match self {
            Self::Weak => 0,
            Self::Medium => 1,
            Self::Strong => 2,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) connection: Connection,
    pub(crate) power_e1_active: bool,
    pub(crate) power_e1_generation: u64,
    pub(crate) power_e2_active: bool,
    pub(crate) power_e2_generation: u64,
    pub(crate) optical_illusion: u8,
    pub(crate) optical_illusion_generation: u64,
    pub(crate) total_consumed_stacks: u8,
    pub(crate) skill3_consumed_by_cmd: BTreeMap<usize, u8>,
    pub(crate) burst_snap_connection: BTreeMap<usize, Connection>,
    pub(crate) q_atk_buff_active: bool,
    pub(crate) q_atk_buff_generation: u64,
    pub(crate) a6_stacks: u8,
    pub(crate) a6_until_rel_us: u64,
    pub(crate) passive2_unlocked: bool,
    pub(crate) passive1_remora_icd_until_rel_us: u64,
    pub(crate) passive1_remora_generation: u64,
    pub(crate) entry_normal_ready: bool,
    pub(crate) riposte_ready: bool,
    pub(crate) on_the_line: Option<super::on_the_line::OnTheLineState>,
    pub(crate) clock: Option<super::clock::Clock>,
}

impl State {
    pub(crate) fn can_skill2(&self) -> bool {
        self.connection == Connection::Weak && self.power_e1_active
    }

    pub(crate) fn can_skill3(&self) -> bool {
        self.connection >= Connection::Medium && self.power_e2_active
    }

    pub(crate) fn reset_normal(&mut self) {
        self.normal_stage = 0;
        self.normal_started_us = None;
    }
}

pub(crate) fn initialize_actor(
    request: &GameScenarioRequest,
    config: &crate::simulation::GameActorConfig,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    let breakthrough = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?["breakthrough"]
        .as_u64()
        .unwrap_or(0);
    if config.awakening.has_effect(5) {
        actor.cycle_ledger.fill(&mut actor.cycle)?;
    }
    let state = actor.character.elyms_mut()?;
    state.passive2_unlocked = breakthrough >= 4;
    state.connection = Connection::Weak;
    super::clock::initialize(request, actor)?;
    Ok(())
}

pub(crate) fn initialize_combat(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    grant_power_e1(
        ctx.state,
        ctx.queue,
        ctx.sequence,
        owner,
        ctx.state.current_time_us,
    )?;
    ctx.queue.sync(&ctx.state.mode_clock)?;
    Ok(())
}

pub(crate) fn grant_power_e1(
    state: &mut crate::simulation::state::GameSimulatorState,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    owner: usize,
    abs_now: u64,
) -> Result<(), String> {
    let elyms = state.actors[owner].character.elyms_mut()?;
    if elyms.connection != Connection::Weak {
        return Ok(());
    }
    elyms.power_e1_active = true;
    elyms.power_e1_generation = elyms.power_e1_generation.saturating_add(1);
    let generation = elyms.power_e1_generation;
    queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::ExpirePowerE1 { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
    let deadline = abs_now
        .checked_add(PARAMETERS.power_e1_duration_us)
        .ok_or("Elyms PowerE1 expiry overflow")?;
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: deadline,
        priority: crate::execution::CausalPriority::BuffTickOrExpiry,
        seq: *sequence,
        data: GameEventData::Content(ContentEvent::Elyms(super::events::Event::ExpirePowerE1 {
            owner,
            generation,
        })),
    });
    Ok(())
}

fn clear_power_e1(
    elyms: &mut State,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    owner: usize,
) {
    elyms.power_e1_active = false;
    elyms.power_e1_generation = elyms.power_e1_generation.saturating_add(1);
    queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::ExpirePowerE1 { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
}

pub(crate) fn grant_power_e2(
    state: &mut crate::simulation::state::GameSimulatorState,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    owner: usize,
    abs_now: u64,
) -> Result<(), String> {
    let elyms = state.actors[owner].character.elyms_mut()?;
    grant_power_e2_on(elyms, queue, sequence, owner, abs_now)
}

fn grant_power_e2_on(
    elyms: &mut State,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    owner: usize,
    abs_now: u64,
) -> Result<(), String> {
    elyms.power_e2_active = true;
    elyms.power_e2_generation = elyms.power_e2_generation.saturating_add(1);
    let generation = elyms.power_e2_generation;
    queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::ExpirePowerE2 { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
    let deadline = abs_now
        .checked_add(PARAMETERS.power_e2_duration_us)
        .ok_or("Elyms PowerE2 expiry overflow")?;
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: deadline,
        priority: crate::execution::CausalPriority::BuffTickOrExpiry,
        seq: *sequence,
        data: GameEventData::Content(ContentEvent::Elyms(super::events::Event::ExpirePowerE2 {
            owner,
            generation,
        })),
    });
    Ok(())
}

fn clear_power_e2(
    elyms: &mut State,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    owner: usize,
) {
    elyms.power_e2_active = false;
    elyms.power_e2_generation = elyms.power_e2_generation.saturating_add(1);
    queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::ExpirePowerE2 { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
}

fn grant_optical_illusion(
    config: &crate::simulation::GameActorConfig,
    elyms: &mut State,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    owner: usize,
    amount: u8,
    abs_now: u64,
) -> Result<(), String> {
    let cap = if config.awakening.has_effect(3) {
        PARAMETERS.optical_illusion_a3_cap
    } else {
        PARAMETERS.optical_illusion_base_cap
    };
    elyms.optical_illusion = elyms.optical_illusion.saturating_add(amount).min(cap);
    elyms.optical_illusion_generation = elyms.optical_illusion_generation.saturating_add(1);
    let generation = elyms.optical_illusion_generation;
    queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::ExpireOpticalIllusion { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
    let deadline = abs_now
        .checked_add(PARAMETERS.optical_illusion_duration_us)
        .ok_or("Elyms Optical Illusion expiry overflow")?;
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: deadline,
        priority: crate::execution::CausalPriority::BuffTickOrExpiry,
        seq: *sequence,
        data: GameEventData::Content(ContentEvent::Elyms(
            super::events::Event::ExpireOpticalIllusion { owner, generation },
        )),
    });
    Ok(())
}

fn clear_optical_illusion(
    elyms: &mut State,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    owner: usize,
) {
    elyms.optical_illusion = 0;
    elyms.optical_illusion_generation = elyms.optical_illusion_generation.saturating_add(1);
    queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::ExpireOpticalIllusion { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
}

pub(crate) fn expire_power_e1(
    state: &mut crate::simulation::state::GameSimulatorState,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let elyms = state.actors[owner].character.elyms_mut()?;
    if elyms.power_e1_generation == generation {
        elyms.power_e1_active = false;
    }
    Ok(())
}

pub(crate) fn expire_power_e2(
    state: &mut crate::simulation::state::GameSimulatorState,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let elyms = state.actors[owner].character.elyms_mut()?;
    if elyms.power_e2_generation == generation {
        elyms.power_e2_active = false;
    }
    Ok(())
}

pub(crate) fn expire_optical_illusion(
    state: &mut crate::simulation::state::GameSimulatorState,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let elyms = state.actors[owner].character.elyms_mut()?;
    if elyms.optical_illusion_generation == generation {
        elyms.optical_illusion = 0;
    }
    Ok(())
}

pub(crate) fn expire_q_atk_buff(
    state: &mut crate::simulation::state::GameSimulatorState,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let elyms = state.actors[owner].character.elyms_mut()?;
    if elyms.q_atk_buff_generation == generation {
        elyms.q_atk_buff_active = false;
    }
    Ok(())
}

pub(crate) fn on_defeat(actor: &mut ActorRuntime) {
    if let Ok(state) = actor.character.elyms_mut() {
        state.power_e1_active = false;
        state.power_e2_active = false;
        state.optical_illusion = 0;
        state.q_atk_buff_active = false;
        state.a6_stacks = 0;
        state.on_the_line = None;
        state.riposte_ready = false;
        state.entry_normal_ready = false;
        state.reset_normal();
    }
}

fn promote_medium_to_strong(
    config: &crate::simulation::GameActorConfig,
    elyms: &mut State,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    owner: usize,
    abs_now: u64,
) -> Result<(), String> {
    elyms.connection = Connection::Strong;
    if config.awakening.has_effect(3) {
        grant_optical_illusion(
            config,
            elyms,
            queue,
            sequence,
            owner,
            PARAMETERS.a3_med_to_strong_stacks,
            abs_now,
        )?;
    }
    grant_power_e2_on(elyms, queue, sequence, owner, abs_now)
}

fn advance_connection(
    config: &crate::simulation::GameActorConfig,
    elyms: &mut State,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    owner: usize,
    abs_now: u64,
) -> Result<(), String> {
    let r3 = config.awakening.has_resonance_3();
    match elyms.connection {
        Connection::Weak => {
            clear_power_e1(elyms, queue, owner);
            elyms.connection = Connection::Medium;
            grant_optical_illusion(config, elyms, queue, sequence, owner, 1, abs_now)?;
            if r3 {
                promote_medium_to_strong(config, elyms, queue, sequence, owner, abs_now)?;
            }
        }
        Connection::Medium if r3 => {
            promote_medium_to_strong(config, elyms, queue, sequence, owner, abs_now)?;
        }
        Connection::Medium | Connection::Strong => {}
    }
    Ok(())
}

pub(crate) fn on_skill2_manage_notify(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let elyms = ctx.state.actors[owner].character.elyms_mut()?;
    advance_connection(
        &ctx.request.actors[owner],
        elyms,
        ctx.queue,
        ctx.sequence,
        owner,
        ctx.state.current_time_us,
    )
}

pub(crate) fn on_add_e_notify(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    grant_e1: bool,
    grant_e2: bool,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let abs_now = ctx.state.current_time_us;
    if grant_e1 {
        grant_power_e1(ctx.state, ctx.queue, ctx.sequence, owner, abs_now)?;
    }
    if grant_e2 && ctx.state.actors[owner].character.elyms()?.connection >= Connection::Medium {
        grant_power_e2(ctx.state, ctx.queue, ctx.sequence, owner, abs_now)?;
    }
    Ok(())
}

pub(crate) fn select_source_command(
    actor: &ActorRuntime,
    command: &mut GameActionCommand,
) -> Result<(), String> {
    let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        cooldown_us,
        recovery_us,
        hit_delay_us,
        ..
    } = command
    else {
        return Ok(());
    };
    let Ok(state) = actor.character.elyms() else {
        return Ok(());
    };
    if ability_id == "GA_Elyms_Skill"
        && !matches!(
            damage_effect_id.as_str(),
            "GE_Player_Elyms_Skill2_Damage" | "GE_Player_Elyms_Skill3_Damage"
        )
    {
        if state.can_skill2() {
            *damage_effect_id = "GE_Player_Elyms_Skill2_Damage".into();
            *cooldown_us = 0;
            *recovery_us = super::clock::profile::SKILL2_RECOVERY;
            *hit_delay_us = super::clock::profile::SKILL2_HITS[0].1;
        } else if state.can_skill3() {
            *damage_effect_id = "GE_Player_Elyms_Skill3_Damage".into();
            *cooldown_us = 0;
            *recovery_us = super::clock::profile::SKILL3_RECOVERY;
            *hit_delay_us = super::clock::profile::SKILL3_HITS[0].1;
        }
    }
    Ok(())
}

pub(crate) fn cooldown_override(
    actor: &ActorRuntime,
    ability_id: &str,
    damage_effect_id: &str,
) -> Option<u64> {
    let state = actor.character.elyms().ok()?;
    if ability_id == "GA_Elyms_Skill"
        && ((damage_effect_id == "GE_Player_Elyms_Skill2_Damage" && state.can_skill2())
            || (damage_effect_id == "GE_Player_Elyms_Skill3_Damage" && state.can_skill3()))
    {
        Some(0)
    } else {
        None
    }
}

pub(crate) fn preserves_cooldown(ability_id: &str, damage_effect_id: &str) -> bool {
    ability_id == "GA_Elyms_Skill"
        && matches!(
            damage_effect_id,
            "GE_Player_Elyms_Skill2_Damage" | "GE_Player_Elyms_Skill3_Damage"
        )
}

pub(crate) fn resource_admission_failure(
    _request: &GameScenarioRequest,
    _command: usize,
    actor: &ActorRuntime,
    ability: &str,
    _now: u64,
) -> Option<AdmissionFailure> {
    let state = actor.character.elyms().ok()?;
    let cmd_packet = match _request.actions.get(_command) {
        Some(GameActionCommand::InvokeAbility {
            damage_effect_id, ..
        }) => Some(damage_effect_id.as_str()),
        _ => None,
    };
    if ability == "GA_Elyms_Skill"
        && cmd_packet == Some("GE_Player_Elyms_Skill2_Damage")
        && !state.can_skill2()
    {
        return Some(AdmissionFailure {
            reason: GameAdmissionReason::Other {
                message: "Elyms Skill2 requires Weak Connection and active PowerE1Dur".into(),
            },
            message: "Elyms Skill2 requires Weak Connection and active PowerE1Dur".into(),
        });
    }
    if ability == "GA_Elyms_Skill"
        && cmd_packet == Some("GE_Player_Elyms_Skill3_Damage")
        && !state.can_skill3()
    {
        return Some(AdmissionFailure {
            reason: GameAdmissionReason::Other {
                message: "Elyms Skill3 requires >= Medium Connection and active PowerE2Dur".into(),
            },
            message: "Elyms Skill3 requires >= Medium Connection and active PowerE2Dur".into(),
        });
    }
    None
}

fn push_event(
    ctx: &mut CastContext<'_>,
    progress: u64,
    event: super::events::Event,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    let time_us = ctx
        .time_us
        .checked_add(progress)
        .ok_or("Elyms event deadline overflow")?;
    ctx.queue.push(ScheduledGameEvent {
        time_us,
        priority: crate::execution::CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(ContentEvent::Elyms(event)),
    });
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let ability_id = ctx.ability_id;
    let packet = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();

    if ability_id == "GA_Elyms_QTE" {
        if ctx.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        }
        let is_switch_entry = matches!(
            ctx.request.actions.get(ctx.command_index),
            Some(GameActionCommand::SwitchActor { .. })
        );
        if !is_switch_entry {
            ctx.scope_primary_aoe_group("GE_Player_Elyms_QTE_Damage")?;
        }
        {
            let state = ctx.actor.character.elyms_mut()?;
            state.riposte_ready = false;
            state.entry_normal_ready = true;
            state.reset_normal();
        }
        // Apply Weak -> Medium (and -> Strong with R3), or Medium -> Strong with R3 for QTE.
        let elyms = ctx.actor.character.elyms_mut()?;
        advance_connection(
            ctx.config,
            elyms,
            ctx.queue,
            ctx.sequence,
            ctx.actor_index,
            ctx.time_us,
        )?;
        return super::clock::on_cast(ctx, super::clock::Kind::Qte);
    }

    if let Some(secondary_kind) = super::secondary::on_cast(ctx)? {
        return super::clock::on_cast(ctx, secondary_kind);
    }

    if ability_id == "GA_Elyms_Melee" {
        if packet != "GE_Player_Elyms_Melee1_Damage" {
            return Err("Elyms Melee requires GE_Player_Elyms_Melee1_Damage primary packet".into());
        }
        ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
        let (is_crosstalk, stage) = {
            let state = ctx.actor.character.elyms_mut()?;
            let appear = state.entry_normal_ready;
            state.entry_normal_ready = false;
            state.riposte_ready = false;
            let crosstalk = state.connection >= Connection::Medium;
            let stages = if crosstalk { 3 } else { 5 };
            let raw_stage = if appear {
                0
            } else {
                crate::characters::normal::after_gap(
                    state.normal_stage,
                    state.normal_started_us,
                    ctx.time_us,
                ) % stages
            };
            state.normal_stage = (raw_stage + 1) % stages;
            state.normal_started_us = Some(ctx.time_us);
            (crosstalk, raw_stage)
        };
        if ctx.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        }
        let kind = if is_crosstalk {
            match stage {
                0 => {
                    ctx.replace_native_primary_packet("GE_Player_Elyms_MeleeB1_Damage")?;
                    ctx.scope_primary_aoe_group("GE_Player_Elyms_MeleeB1_Damage")?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_MeleeB1_1_Damage",
                        ctx.hit_time_us,
                    )?;
                }
                1 => {
                    ctx.replace_native_primary_packet("GE_Player_Elyms_MeleeB2_Damage")?;
                    ctx.scope_primary_aoe_group("GE_Player_Elyms_MeleeB2_Damage")?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_MeleeB2_1_Damage",
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_MeleeB2_Damage",
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_MeleeB2_1_Damage",
                        ctx.hit_time_us,
                    )?;
                }
                2 => {
                    ctx.replace_native_primary_packet("GE_Player_Elyms_MeleeB3_Damage")?;
                    ctx.scope_primary_aoe_group("GE_Player_Elyms_MeleeB3_Damage")?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_MeleeB3_Damage",
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_MeleeB3_Damage",
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_MeleeB3_1_Damage",
                        ctx.hit_time_us,
                    )?;
                    let progress = if super::clock::accepted(ctx.request) {
                        super::clock::profile::MELEEB3_ADDE_PROGRESS
                    } else {
                        0
                    };
                    push_event(
                        ctx,
                        progress,
                        super::events::Event::ActionAddE {
                            owner: ctx.actor_index,
                            command: ctx.command_index,
                            progress,
                            grant_e1: false,
                            grant_e2: true,
                        },
                    )?;
                }
                _ => unreachable!(),
            }
            super::clock::Kind::Crosstalk(stage)
        } else {
            match stage {
                0 => {
                    ctx.scope_primary_aoe_group("GE_Player_Elyms_Melee1_Damage")?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_Melee1_Damage",
                        ctx.hit_time_us,
                    )?;
                }
                1 => {
                    ctx.replace_native_primary_packet("GE_Player_Elyms_Melee2_Damage")?;
                    ctx.scope_primary_aoe_group("GE_Player_Elyms_Melee2_Damage")?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_Melee2_Damage",
                        ctx.hit_time_us,
                    )?;
                }
                2 => {
                    ctx.replace_native_primary_packet("GE_Player_Elyms_Melee3_Damage")?;
                    ctx.scope_primary_aoe_group("GE_Player_Elyms_Melee3_Damage")?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_Melee3_Damage",
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_Melee3_1_Damage",
                        ctx.hit_time_us,
                    )?;
                }
                3 => {
                    ctx.replace_native_primary_packet("GE_Player_Elyms_Melee4_Damage")?;
                    ctx.scope_primary_aoe_group("GE_Player_Elyms_Melee4_Damage")?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_Melee4_1_Damage",
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_Melee4_Damage",
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_Melee4_1_Damage",
                        ctx.hit_time_us,
                    )?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Elyms_Melee4_Damage",
                        ctx.hit_time_us,
                    )?;
                }
                4 => {
                    ctx.replace_native_primary_packet("GE_Player_Elyms_Melee5_Damage")?;
                    ctx.scope_primary_aoe_group("GE_Player_Elyms_Melee5_Damage")?;
                    let progress = if super::clock::accepted(ctx.request) {
                        super::clock::profile::MELEE5_ADDE_PROGRESS
                    } else {
                        0
                    };
                    push_event(
                        ctx,
                        progress,
                        super::events::Event::ActionAddE {
                            owner: ctx.actor_index,
                            command: ctx.command_index,
                            progress,
                            grant_e1: true,
                            grant_e2: false,
                        },
                    )?;
                }
                _ => unreachable!(),
            }
            super::clock::Kind::Normal(stage)
        };
        return super::clock::on_cast(ctx, kind);
    }

    if ability_id == "GA_Elyms_Skill" {
        ctx.record_hypothesis(SKILL_REDIRECT_HYPOTHESIS_ID)?;
        if ctx.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        }
        {
            let state = ctx.actor.character.elyms_mut()?;
            state.riposte_ready = false;
            state.entry_normal_ready = false;
            state.reset_normal();
        }
        let kind = if packet == "GE_Player_Elyms_Skill2_Damage" {
            let owner = ctx.actor_index;
            let queue = &mut *ctx.queue;
            let state = ctx.actor.character.elyms_mut()?;
            if !state.can_skill2() {
                return Err("Elyms Skill2 requires Weak Connection and active PowerE1Dur".into());
            }
            clear_power_e1(state, queue, owner);
            ctx.scope_primary_aoe_group("GE_Player_Elyms_Skill2_Damage")?;
            for _ in 0..4 {
                ctx.schedule_native_aoe_group("GE_Player_Elyms_Skill2_Damage", ctx.hit_time_us)?;
            }
            ctx.schedule_native_aoe_group("GE_Player_Elyms_Skill2_1_Damage", ctx.hit_time_us)?;
            let progress = if super::clock::accepted(ctx.request) {
                super::clock::profile::SKILL2_MANAGE_PROGRESS
            } else {
                0
            };
            push_event(
                ctx,
                progress,
                super::events::Event::ActionManage {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    progress,
                },
            )?;
            super::clock::Kind::Skill2
        } else if packet == "GE_Player_Elyms_Skill3_Damage" {
            let owner = ctx.actor_index;
            let queue = &mut *ctx.queue;
            let r6 = ctx.config.awakening.has_resonance_6();
            let consumed = {
                let state = ctx.actor.character.elyms_mut()?;
                if !state.can_skill3() {
                    return Err(
                        "Elyms Skill3 requires >= Medium Connection and active PowerE2Dur".into(),
                    );
                }
                clear_power_e2(state, queue, owner);
                let consumed = state.optical_illusion;
                clear_optical_illusion(state, queue, owner);
                if consumed > 0 {
                    state.total_consumed_stacks = state
                        .total_consumed_stacks
                        .saturating_add(consumed)
                        .min(PARAMETERS.total_consumed_cap);
                    state
                        .skill3_consumed_by_cmd
                        .insert(ctx.command_index, consumed);
                }
                state.connection = Connection::Strong;
                consumed
            };
            ctx.scope_primary_aoe_group("GE_Player_Elyms_Skill3_Damage")?;
            for _ in 0..4 {
                ctx.schedule_native_aoe_group("GE_Player_Elyms_Skill3_Damage", ctx.hit_time_us)?;
            }
            ctx.schedule_native_aoe_group("GE_Player_Elyms_Skill3_1_Damage", ctx.hit_time_us)?;
            let extra = r6 && consumed > 0;
            if extra {
                ctx.record_hypothesis(super::awakenings::MODEL)?;
                ctx.schedule_native_aoe_group(
                    "GE_Player_Elyms_Skill3Extra_Damage",
                    ctx.hit_time_us,
                )?;
            }
            super::clock::Kind::Skill3 { extra }
        } else {
            if packet != "GE_Player_Elyms_Skill1_Damage" {
                return Err(
                    "Elyms Skill1 requires GE_Player_Elyms_Skill1_Damage canonical primary".into(),
                );
            }
            ctx.actor
                .cycle_ledger
                .grant_direct(&mut ctx.actor.cycle, PARAMETERS.skill1_cycle_refill)?;
            ctx.scope_primary_aoe_group("GE_Player_Elyms_Skill1_Damage")?;
            for _ in 0..4 {
                ctx.schedule_native_aoe_group("GE_Player_Elyms_Skill1_1_Damage", ctx.hit_time_us)?;
            }
            ctx.schedule_native_aoe_group("GE_Player_Elyms_Skill1_2_Damage", ctx.hit_time_us)?;
            let progress = if super::clock::accepted(ctx.request) {
                super::clock::profile::SKILL1_ADDE_PROGRESS
            } else {
                0
            };
            push_event(
                ctx,
                progress,
                super::events::Event::ActionAddE {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    progress,
                    grant_e1: true,
                    grant_e2: false,
                },
            )?;
            super::clock::Kind::Skill1
        };
        return super::clock::on_cast(ctx, kind);
    }

    if ability_id == "GA_Elyms_UltraSkill" {
        ctx.record_hypothesis(super::on_the_line::MODEL)?;
        let living_targets = ctx.targets.iter().filter(|t| t.hp_current > 0).count();
        let (snap_conn, consumed_before_q) = {
            let owner = ctx.actor_index;
            let queue = &mut *ctx.queue;
            let state = ctx.actor.character.elyms_mut()?;
            let snap = state.connection;
            let consumed = state.total_consumed_stacks;
            state.riposte_ready = false;
            state.entry_normal_ready = false;
            state.reset_normal();
            clear_power_e1(state, queue, owner);
            clear_power_e2(state, queue, owner);
            state.connection = Connection::Weak;
            (snap, consumed)
        };
        let is_st = living_targets == 1 && snap_conn >= Connection::Medium;
        let kind = if is_st {
            if packet != "GE_Player_Elyms_UltraSkillB1_Damage" {
                ctx.replace_native_primary_packet("GE_Player_Elyms_UltraSkillB1_Damage")?;
            }
            ctx.scope_primary_target_group("GE_Player_Elyms_UltraSkillB1_Damage")?;
            for _ in 0..2 {
                ctx.schedule_native_target_group(
                    "GE_Player_Elyms_UltraSkillB1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            for _ in 0..4 {
                ctx.schedule_native_target_group(
                    "GE_Player_Elyms_UltraSkillB2_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            ctx.schedule_native_target_group(
                "GE_Player_Elyms_UltraSkillB3_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            super::clock::Kind::BurstB
        } else {
            if ctx.targets.len() > 1 {
                ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            ctx.scope_primary_aoe_group("GE_Player_Elyms_UltraSkillA1_Damage")?;
            for _ in 0..2 {
                ctx.schedule_native_aoe_group(
                    "GE_Player_Elyms_UltraSkillA1_Damage",
                    ctx.hit_time_us,
                )?;
            }
            ctx.schedule_native_aoe_group("GE_Player_Elyms_UltraSkillA2_Damage", ctx.hit_time_us)?;
            super::clock::Kind::BurstA
        };
        super::awakenings::apply_q_buffs(ctx, consumed_before_q)?;
        super::on_the_line::arm(ctx, is_st, snap_conn)?;
        return super::clock::on_cast(ctx, kind);
    }

    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    super::clock::on_hit(ctx)
}

pub(crate) fn on_field_exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    super::clock::exit(ctx, owner)?;
    super::on_the_line::on_field_exit(ctx, owner)
}

pub(crate) fn on_field_entry(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    super::on_the_line::on_field_entry(ctx, owner)
}
