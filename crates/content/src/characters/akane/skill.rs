use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::rejection::AdmissionFailure;
use crate::simulation::state::ActorRuntime;
use crate::simulation::{GameActionCommand, GameAdmissionReason, GameScenarioRequest};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Mode {
    #[default]
    None,
    Overtone {
        target: usize,
        until_us: u64,
        command: usize,
    },
    ReverbField {
        until_us: u64,
        pull_until_us: u64,
        command: usize,
    },
}

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) string_energy: u8,
    pub(crate) string_awarded_commands: BTreeSet<usize>,
    pub(crate) synced_settled_commands: BTreeSet<usize>,
    pub(crate) mode: Mode,
    pub(crate) mode_generation: u64,
    pub(crate) rhythm_wave_ready_at_us: u64,
    pub(crate) cadenza_started_us: u64,
    pub(crate) cadenza_unlock_us: u64,
    pub(crate) cadenza_until_us: u64,
    pub(crate) stage2_ready_at_us: u64,
    pub(crate) anthem_until_us: u64,
    pub(crate) a1_until_us: u64,
    pub(crate) a4_until_us: u64,
    pub(crate) r6_until_us: u64,
    pub(crate) perfect_pitch_unlocked: bool,
    pub(crate) perfect_pitch_next_tick_us: Option<u64>,
    pub(crate) perfect_pitch_icd_until_us: BTreeMap<usize, u64>,
    pub(crate) perfect_pitch_final_bonuses: BTreeMap<usize, crate::simulation::ExactFraction>,
    pub(crate) entry_normal_ready: bool,
    pub(crate) melee_pre_ready: bool,
    pub(crate) riposte_ready: bool,
    pub(crate) clock: Option<super::clock::Clock>,
}

impl State {
    pub(crate) fn overtone_active(&self, now_us: u64) -> bool {
        matches!(self.mode, Mode::Overtone { until_us, .. } if until_us > now_us)
    }

    pub(crate) fn overtone_target(&self, now_us: u64) -> Option<usize> {
        match self.mode {
            Mode::Overtone {
                target, until_us, ..
            } if until_us > now_us => Some(target),
            _ => None,
        }
    }

    pub(crate) fn reverb_active(&self, now_us: u64) -> bool {
        matches!(self.mode, Mode::ReverbField { until_us, .. } if until_us > now_us)
    }

    pub(crate) fn mode_active(&self, now_us: u64) -> bool {
        self.overtone_active(now_us) || self.reverb_active(now_us)
    }

    pub(crate) fn mode_until_us(&self, now_us: u64) -> u64 {
        match self.mode {
            Mode::Overtone { until_us, .. } | Mode::ReverbField { until_us, .. }
                if until_us > now_us =>
            {
                until_us
            }
            _ => 0,
        }
    }

    pub(crate) fn mode_command_index(&self) -> Option<usize> {
        match self.mode {
            Mode::Overtone { command, .. } | Mode::ReverbField { command, .. } => Some(command),
            Mode::None => None,
        }
    }

    pub(crate) fn cadenza_active(&self, now_us: u64) -> bool {
        self.cadenza_until_us > now_us
    }

    pub(crate) fn can_redirect_skill(&self, now_us: u64) -> bool {
        self.string_energy >= PARAMETERS.max_string_energy && self.mode_active(now_us)
    }
}

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::AkaneNormalChainV1;
pub(crate) const STRING_ENERGY_HYPOTHESIS_ID: HypothesisId = HypothesisId::AkaneStringEnergyV1;
pub(crate) const RHYTHM_WAVE_HYPOTHESIS_ID: HypothesisId = HypothesisId::AkaneRhythmWaveV1;
pub(crate) const SONIC_BOOM_HYPOTHESIS_ID: HypothesisId = HypothesisId::AkaneSonicBoomV1;
pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::AkaneAoeAllInRangeDummyV1;

pub(crate) fn parse_initial_string_energy(value: crate::formula::Ratio) -> Result<u8, String> {
    if value.denominator == 0 || !value.numerator.is_multiple_of(value.denominator) {
        return Err("start_string_energy must be an integer in 0..=30.".into());
    }
    let integer = value.numerator / value.denominator;
    if integer > u64::from(PARAMETERS.max_string_energy) {
        return Err(format!(
            "start_string_energy ({integer}) cannot exceed {}.",
            PARAMETERS.max_string_energy
        ));
    }
    Ok(integer as u8)
}

pub(crate) fn initialize(
    request: &GameScenarioRequest,
    config: &crate::simulation::GameActorConfig,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    let start_configured = config
        .initial_resources
        .string_energy
        .map(parse_initial_string_energy)
        .transpose()?
        .unwrap_or(0);
    let initial = if config.awakening.count() >= 3 {
        start_configured.max(PARAMETERS.max_string_energy)
    } else {
        start_configured
    };
    let state = actor.character.akane_mut()?;
    state.string_energy = initial;
    super::clock::initialize(request, actor)?;
    Ok(())
}

pub(crate) fn select_source_command(
    actor: &ActorRuntime,
    command: &mut GameActionCommand,
) -> Result<(), String> {
    let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        energy_cost,
        cooldown_us,
        recovery_us,
        hit_delay_us,
        timestamp_us,
        ..
    } = command
    else {
        return Ok(());
    };
    let Ok(state) = actor.character.akane() else {
        return Ok(());
    };
    if ability_id == "GA_Akane_Skill"
        && state.can_redirect_skill(*timestamp_us)
        && damage_effect_id != "GE_Player_Akane_UpgradeSkill_Damage"
    {
        let overtone = state.overtone_active(*timestamp_us);
        *damage_effect_id = "GE_Player_Akane_UpgradeSkill_Damage".into();
        *energy_cost = 0;
        *cooldown_us = 0;
        *recovery_us = if overtone {
            super::clock::profile::PROSKILL_1_RECOVERY
        } else {
            super::clock::profile::PROSKILL_2_RECOVERY
        };
        *hit_delay_us = if overtone {
            super::clock::profile::PROSKILL_1_HIT
        } else {
            super::clock::profile::PROSKILL_2_HIT
        };
    } else if ability_id == "GA_Akane_UltraSkill"
        && state.cadenza_active(*timestamp_us)
        && damage_effect_id != "GE_Player_Akane_UltraSkill2_Damage"
    {
        *damage_effect_id = "GE_Player_Akane_UltraSkill2_Damage".into();
        *energy_cost = 0;
        *cooldown_us = PARAMETERS.burst_stage2_cooldown_us;
        *recovery_us = super::clock::profile::BURST2_RECOVERY;
        *hit_delay_us = super::clock::profile::BURST2_HIT;
    }
    Ok(())
}

pub(crate) fn cooldown_override(
    actor: &ActorRuntime,
    ability_id: &str,
    damage_effect_id: &str,
) -> Option<u64> {
    let state = actor.character.akane().ok()?;
    if ability_id == "GA_Akane_Skill" && damage_effect_id == "GE_Player_Akane_UpgradeSkill_Damage" {
        Some(0)
    } else if ability_id == "GA_Akane_UltraSkill"
        && damage_effect_id == "GE_Player_Akane_UltraSkill2_Damage"
    {
        Some(state.cadenza_unlock_us.max(state.stage2_ready_at_us))
    } else {
        None
    }
}

pub(crate) fn preserves_cooldown(ability_id: &str, damage_effect_id: &str) -> bool {
    (ability_id == "GA_Akane_Skill" && damage_effect_id == "GE_Player_Akane_UpgradeSkill_Damage")
        || (ability_id == "GA_Akane_UltraSkill"
            && damage_effect_id == "GE_Player_Akane_UltraSkill2_Damage")
}

pub(crate) fn resource_admission_failure(
    request: &GameScenarioRequest,
    command: usize,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<AdmissionFailure> {
    let state = actor.character.akane().ok()?;
    let cmd_packet = match request.actions.get(command) {
        Some(GameActionCommand::InvokeAbility {
            damage_effect_id, ..
        }) => Some(damage_effect_id.as_str()),
        _ => None,
    };
    let held_us = request
        .hold_inputs
        .get(&command)
        .map_or(0, |input| input.duration_us);
    if ability == "GA_Akane_Skill" && cmd_packet == Some("GE_Player_Akane_UpgradeSkill_Damage") {
        if !state.can_redirect_skill(now) {
            return Some(AdmissionFailure {
                reason: GameAdmissionReason::Other {
                    message: "Synced Resonance requires 30 String Energy and active Overtone or Reverb Field".into(),
                },
                message: "Synced Resonance requires 30 String Energy and active Overtone or Reverb Field".into(),
            });
        }
        if held_us >= PARAMETERS.hold_threshold_us {
            return Some(AdmissionFailure {
                reason: GameAdmissionReason::Other {
                    message: "Synced Resonance is Press-only and rejects hold >= 0.2s".into(),
                },
                message: "Synced Resonance is Press-only and rejects hold >= 0.2s".into(),
            });
        }
    }
    if ability == "GA_Akane_UltraSkill" && cmd_packet == Some("GE_Player_Akane_UltraSkill2_Damage")
    {
        if !state.cadenza_active(now) {
            return Some(AdmissionFailure {
                reason: GameAdmissionReason::Other {
                    message: "Sonic Boom Stage 2 requires active Solo Cadenza".into(),
                },
                message: "Sonic Boom Stage 2 requires active Solo Cadenza".into(),
            });
        }
        let unlock_at = state.cadenza_unlock_us.max(state.stage2_ready_at_us);
        if now < unlock_at {
            return Some(AdmissionFailure {
                reason: GameAdmissionReason::Cooldown {
                    until_us: unlock_at,
                },
                message: format!("Sonic Boom Stage 2 locked until {unlock_at}us"),
            });
        }
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
        .ok_or("Akane event deadline overflow")?;
    ctx.queue.push(ScheduledGameEvent {
        time_us,
        priority: crate::execution::CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(ContentEvent::Akane(event)),
    });
    Ok(())
}

fn redirect_primary_target(ctx: &mut CastContext<'_>, target: usize) {
    let mut events = ctx.queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if let GameEventData::HitDelivery {
            command_index,
            actor_index,
            target_index,
            ..
        } = &mut event.data
            && *command_index == ctx.command_index
            && *actor_index == ctx.actor_index
        {
            *target_index = target;
        }
    }
    ctx.queue.replace(events);
}

pub(crate) fn activate_overtone(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    target: usize,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let until_us = now
        .checked_add(PARAMETERS.mode_duration_us)
        .ok_or("Akane Overtone expiry overflow")?;
    let (mode_gen, next_tick) = {
        let state = ctx.state.actors[owner].character.akane_mut()?;
        state.mode_generation = state.mode_generation.saturating_add(1);
        state.mode = Mode::Overtone {
            target,
            until_us,
            command,
        };
        let tick = if state.perfect_pitch_unlocked {
            let next = now
                .checked_add(PARAMETERS.perfect_pitch_tick_us)
                .ok_or("Akane Perfect Pitch tick overflow")?;
            state.perfect_pitch_next_tick_us = Some(next);
            Some(next)
        } else {
            state.perfect_pitch_next_tick_us = None;
            None
        };
        (state.mode_generation, tick)
    };
    if let Some(next_tick) = next_tick {
        super::passive::schedule_tick(ctx.queue, ctx.sequence, owner, mode_gen, next_tick)?;
    }
    super::awakenings::apply_mode_status_debuff(ctx, owner, Some(target))
}

pub(crate) fn activate_reverb(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let until_us = now
        .checked_add(PARAMETERS.mode_duration_us)
        .ok_or("Akane Reverb Field expiry overflow")?;
    let pull_until_us = if ctx.request.actors[owner].awakening.has_effect(3) {
        now.checked_add(PARAMETERS.a3_pull_duration_us)
            .ok_or("Akane A3 pull expiry overflow")?
    } else {
        0
    };
    let (mode_gen, next_tick) = {
        let state = ctx.state.actors[owner].character.akane_mut()?;
        state.mode_generation = state.mode_generation.saturating_add(1);
        state.mode = Mode::ReverbField {
            until_us,
            pull_until_us,
            command,
        };
        let tick = if state.perfect_pitch_unlocked {
            let next = now
                .checked_add(PARAMETERS.perfect_pitch_tick_us)
                .ok_or("Akane Perfect Pitch tick overflow")?;
            state.perfect_pitch_next_tick_us = Some(next);
            Some(next)
        } else {
            state.perfect_pitch_next_tick_us = None;
            None
        };
        (state.mode_generation, tick)
    };
    if let Some(next_tick) = next_tick {
        super::passive::schedule_tick(ctx.queue, ctx.sequence, owner, mode_gen, next_tick)?;
    }
    super::awakenings::apply_mode_status_debuff(ctx, owner, None)
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let ability_id = ctx.ability_id;
    let packet = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();

    if ability_id == "GA_Akane_QTE" {
        ctx.record_hypothesis(super::secondary::MODEL)?;
        ctx.record_hypothesis(STRING_ENERGY_HYPOTHESIS_ID)?;
        if ctx.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        }
        let is_switch_entry = matches!(
            ctx.request.actions.get(ctx.command_index),
            Some(GameActionCommand::SwitchActor { .. })
        );
        if !is_switch_entry {
            ctx.scope_primary_aoe_group("GE_Player_Akane_QTE_Damage")?;
        }
        let state = ctx.actor.character.akane_mut()?;
        state.riposte_ready = false;
        state.melee_pre_ready = false;
        state.entry_normal_ready = true;
        return super::clock::on_cast(ctx, super::clock::Kind::Qte);
    }

    if let Some(secondary_kind) = super::secondary::on_cast(ctx)? {
        return super::clock::on_cast(ctx, secondary_kind);
    }

    if ability_id == "GA_Akane_Melee" {
        if packet != "GE_Player_Akane_Melee1_Damage" {
            return Err("Akane Melee requires GE_Player_Akane_Melee1_Damage primary packet".into());
        }
        ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
        ctx.record_hypothesis(STRING_ENERGY_HYPOTHESIS_ID)?;

        let (stage, appear, pre, overtone_target, reverb_active) = {
            let state = ctx.actor.character.akane_mut()?;
            let appear = state.entry_normal_ready;
            let pre = state.melee_pre_ready;
            state.entry_normal_ready = false;
            state.melee_pre_ready = false;
            let stage = if appear || pre {
                0
            } else {
                crate::characters::normal::after_gap(
                    state.normal_stage,
                    state.normal_started_us,
                    ctx.time_us,
                )
            };
            state.normal_stage = (stage + 1) % 4;
            state.normal_started_us = Some(ctx.time_us);
            let marked = state
                .overtone_target(ctx.time_us)
                .filter(|&t| ctx.targets.get(t).is_some_and(|tgt| tgt.hp_current > 0));
            let reverb_active = state.reverb_active(ctx.time_us);
            (stage, appear, pre, marked, reverb_active)
        };

        let kind = if appear {
            super::clock::Kind::AppearNormal
        } else if pre {
            super::clock::Kind::MeleePre
        } else {
            super::clock::Kind::Normal(stage)
        };

        match stage {
            0 => {
                if let Some(target) = overtone_target {
                    ctx.scope_primary_target_group("GE_Player_Akane_Melee1_Damage")?;
                    if target != ctx.target_index {
                        redirect_primary_target(ctx, target);
                    }
                    ctx.schedule_native_target_group(
                        "GE_Player_Akane_Melee1_Damage",
                        target,
                        ctx.hit_time_us,
                    )?;
                } else if reverb_active {
                    if ctx.targets.len() > 1 {
                        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                    }
                    ctx.scope_primary_aoe_group("GE_Player_Akane_Melee1_Damage")?;
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Akane_Melee1_Damage",
                        ctx.hit_time_us,
                    )?;
                } else {
                    ctx.scope_primary_target_group("GE_Player_Akane_Melee1_Damage")?;
                    ctx.schedule_native_target_group(
                        "GE_Player_Akane_Melee1_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
            }
            1 => {
                if ctx.targets.len() > 1 {
                    ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                }
                ctx.replace_native_primary_packet("GE_Player_Akane_Melee2_Damage")?;
                ctx.scope_primary_aoe_group("GE_Player_Akane_Melee2_Damage")?;
                ctx.schedule_native_aoe_group("GE_Player_Akane_Melee2_Damage", ctx.hit_time_us)?;
            }
            2 => {
                if ctx.targets.len() > 1 {
                    ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                }
                ctx.replace_native_primary_packet("GE_Player_Akane_Melee3_Damage")?;
                ctx.scope_primary_aoe_group("GE_Player_Akane_Melee3_Damage")?;
                for _ in 0..2 {
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Akane_Melee3_Damage",
                        ctx.hit_time_us,
                    )?;
                }
                for _ in 0..4 {
                    ctx.schedule_native_aoe_group(
                        "GE_Player_Akane_Melee3_1_Damage",
                        ctx.hit_time_us,
                    )?;
                }
            }
            3 => {
                if ctx.targets.len() > 1 {
                    ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                }
                ctx.replace_native_primary_packet("GE_Player_Akane_Melee4_Damage")?;
                ctx.scope_primary_aoe_group("GE_Player_Akane_Melee4_Damage")?;
            }
            _ => unreachable!(),
        }
        return super::clock::on_cast(ctx, kind);
    }

    if ability_id == "GA_Akane_Skill" {
        ctx.record_hypothesis(RHYTHM_WAVE_HYPOTHESIS_ID)?;
        ctx.record_hypothesis(STRING_ENERGY_HYPOTHESIS_ID)?;
        let kind = if packet == "GE_Player_Akane_UpgradeSkill_Damage" {
            let overtone_target = {
                let state = ctx.actor.character.akane()?;
                if !state.can_redirect_skill(ctx.time_us) {
                    return Err("Synced Resonance requires 30 String Energy and active mode".into());
                }
                state.overtone_target(ctx.time_us)
            };
            if let Some(marked) = overtone_target {
                let target = if ctx.targets.get(marked).is_some_and(|t| t.hp_current > 0) {
                    marked
                } else {
                    ctx.target_index
                };
                ctx.scope_primary_target_group("GE_Player_Akane_UpgradeSkill_Damage")?;
                if target != ctx.target_index {
                    redirect_primary_target(ctx, target);
                }
                let progress = if super::clock::accepted(ctx.request) {
                    super::clock::profile::PROSKILL_1_BUFF_PROGRESS
                } else {
                    0
                };
                push_event(
                    ctx,
                    progress,
                    super::events::Event::SyncedBuffs {
                        owner: ctx.actor_index,
                        command: ctx.command_index,
                        progress,
                    },
                )?;
                super::clock::Kind::SyncedOvertone
            } else {
                if ctx.targets.len() > 1 {
                    ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                }
                ctx.scope_primary_aoe_group("GE_Player_Akane_UpgradeSkill_Damage")?;
                let progress = if super::clock::accepted(ctx.request) {
                    super::clock::profile::PROSKILL_2_BUFF_PROGRESS
                } else {
                    0
                };
                push_event(
                    ctx,
                    progress,
                    super::events::Event::SyncedBuffs {
                        owner: ctx.actor_index,
                        command: ctx.command_index,
                        progress,
                    },
                )?;
                super::clock::Kind::SyncedReverb
            }
        } else {
            if packet != "GE_Player_Akane_Skill_Damage" {
                return Err(
                    "Akane Rhythm Wave requires GE_Player_Akane_Skill_Damage canonical primary"
                        .into(),
                );
            }
            let held_us = ctx
                .request
                .hold_inputs
                .get(&ctx.command_index)
                .map_or(0, |input| input.duration_us);
            let ready_at = ctx
                .time_us
                .checked_add(PARAMETERS.skill_cooldown_us)
                .ok_or("Akane Rhythm Wave cooldown overflow")?;
            ctx.actor.character.akane_mut()?.rhythm_wave_ready_at_us = ready_at;
            if held_us >= PARAMETERS.hold_threshold_us {
                if ctx.targets.len() > 1 {
                    ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                }
                ctx.replace_native_primary_packet("GE_Player_Akane_Skill2_Damage")?;
                ctx.scope_primary_aoe_group("GE_Player_Akane_Skill2_Damage")?;
                ctx.schedule_native_aoe_group("GE_Player_Akane_Skill2_Damage", ctx.hit_time_us)?;
                let progress = if super::clock::accepted(ctx.request) {
                    super::clock::profile::SKILL_HOLD_MODE_PROGRESS
                } else {
                    0
                };
                push_event(
                    ctx,
                    progress,
                    super::events::Event::ModeReverb {
                        owner: ctx.actor_index,
                        command: ctx.command_index,
                        progress,
                    },
                )?;
                super::clock::Kind::SkillHold
            } else {
                ctx.scope_primary_target_group("GE_Player_Akane_Skill_Damage")?;
                let progress = if super::clock::accepted(ctx.request) {
                    super::clock::profile::SKILL_PRESS_MODE_PROGRESS
                } else {
                    0
                };
                push_event(
                    ctx,
                    progress,
                    super::events::Event::ModeOvertone {
                        owner: ctx.actor_index,
                        command: ctx.command_index,
                        target: ctx.target_index,
                        progress,
                    },
                )?;
                super::clock::Kind::SkillPress
            }
        };
        let state = ctx.actor.character.akane_mut()?;
        state.riposte_ready = false;
        state.melee_pre_ready = false;
        state.entry_normal_ready = false;
        state.normal_stage = 0;
        state.normal_started_us = None;
        return super::clock::on_cast(ctx, kind);
    }

    if ability_id == "GA_Akane_UltraSkill" {
        ctx.record_hypothesis(SONIC_BOOM_HYPOTHESIS_ID)?;
        if ctx.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        }
        let kind = if packet == "GE_Player_Akane_UltraSkill2_Damage" {
            ctx.scope_primary_aoe_group("GE_Player_Akane_UltraSkill2_Damage")?;
            let ready_at = ctx
                .time_us
                .checked_add(PARAMETERS.burst_stage2_cooldown_us)
                .ok_or("Akane Stage 2 cooldown overflow")?;
            let state = ctx.actor.character.akane_mut()?;
            state.cadenza_started_us = 0;
            state.cadenza_unlock_us = 0;
            state.cadenza_until_us = 0;
            state.stage2_ready_at_us = ready_at;
            super::awakenings::apply_starlit_anthem(ctx)?;
            super::clock::Kind::BurstStage2
        } else {
            if packet != "GE_Player_Akane_UltraSkill1_Damage" {
                return Err(
                    "Akane Sonic Boom Stage 1 requires GE_Player_Akane_UltraSkill1_Damage".into(),
                );
            }
            ctx.scope_primary_aoe_group("GE_Player_Akane_UltraSkill1_Damage")?;
            for _ in 0..9 {
                ctx.schedule_native_aoe_group(
                    "GE_Player_Akane_UltraSkill1_Damage",
                    ctx.hit_time_us,
                )?;
            }
            let state = ctx.actor.character.akane_mut()?;
            state.cadenza_started_us = ctx.time_us;
            state.cadenza_unlock_us = ctx
                .time_us
                .checked_add(PARAMETERS.cadenza_unlock_us)
                .ok_or("Akane Solo Cadenza unlock overflow")?;
            state.cadenza_until_us = ctx
                .time_us
                .checked_add(PARAMETERS.cadenza_duration_us)
                .ok_or("Akane Solo Cadenza expiry overflow")?;
            state.stage2_ready_at_us = 0;
            super::awakenings::apply_starlit_anthem(ctx)?;
            super::clock::Kind::BurstStage1
        };
        let state = ctx.actor.character.akane_mut()?;
        state.riposte_ready = false;
        state.melee_pre_ready = false;
        state.entry_normal_ready = false;
        state.normal_stage = 0;
        state.normal_started_us = None;
        return super::clock::on_cast(ctx, kind);
    }

    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    super::clock::on_hit(ctx)?;
    let gain = match ctx.damage_effect_id {
        "GE_Player_Akane_Melee1_Damage" => 1,
        "GE_Player_Akane_Melee2_Damage" => 1,
        "GE_Player_Akane_Melee3_Damage" => 2,
        "GE_Player_Akane_Melee3_1_Damage" => 0,
        "GE_Player_Akane_Melee4_Damage" => 6,
        "GE_Player_Akane_Airattack_Damage" => 1,
        "GE_Player_Akane_PerfectEvadeAttack_Damage" => 2,
        "GE_Player_Akane_QTE_Damage" => 4,
        "GE_Player_Akane_Skill_Damage" => 6,
        "GE_Player_Akane_Skill2_Damage" => 6,
        "GE_Player_Akane_UpgradeSkill_Damage" => {
            let config = ctx.config;
            let req = ctx.request;
            let cmd = ctx.command_index;
            let r6 = config.awakening.count() >= 6;
            let state = ctx.actor.character.akane_mut()?;
            if state.synced_settled_commands.insert(cmd) {
                state.string_energy = state
                    .string_energy
                    .saturating_sub(PARAMETERS.max_string_energy);
                if r6 {
                    crate::runtime_api::record_hypothesis_id(
                        req,
                        ctx.execution_assumptions,
                        super::awakenings::MODEL,
                    )?;
                    state.string_energy = state
                        .string_energy
                        .saturating_add(PARAMETERS.r6_string_refund)
                        .min(PARAMETERS.max_string_energy);
                }
            }
            return Ok(());
        }
        _ => return Ok(()),
    };
    if gain > 0 {
        let state = ctx.actor.character.akane_mut()?;
        if state.string_awarded_commands.insert(ctx.command_index) {
            state.string_energy = state
                .string_energy
                .saturating_add(gain)
                .min(PARAMETERS.max_string_energy);
        }
    }
    Ok(())
}
