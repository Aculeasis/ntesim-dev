use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::{ActorHitContext, CastContext};
use crate::simulation::ExactFraction;
use crate::simulation::GameAdmissionReason;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::rejection::AdmissionFailure;
use crate::simulation::state::{TargetStatusDeclaration, apply_target_status};
use ntesim_core::buffs::dispatch::BuffRecipient;
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::LinkoNormalChainV1;
pub(crate) const SKILL_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::LinkoSkillFourHitsV1;
pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::LinkoAoeAllInRangeDummyV1;
pub(crate) const PET_HYPOTHESIS_ID: HypothesisId = HypothesisId::LinkoXiaozhenPetV1;
pub(crate) const HEXED_STATUS_TAG: &str = "ntesim.reaction.hexed";

pub(crate) fn oversync_remaining_us(state: &super::awakenings::State, now: u64) -> u64 {
    state.oversync_until_us.unwrap_or(0).saturating_sub(now)
}

pub(crate) fn on_defeat(actor: &mut crate::simulation::state::ActorRuntime) {
    if let crate::characters::RuntimeState::Linko(state) = &mut actor.character {
        let generation = state.pulse_generation.wrapping_add(1);
        *state = super::awakenings::State {
            pulse_generation: generation,
            ..Default::default()
        };
    }
}
pub(crate) fn resource_admission_failure(
    request: &crate::simulation::GameScenarioRequest,
    command: usize,
    actor: &crate::simulation::state::ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<AdmissionFailure> {
    if ability == "GA_Radio072_UltraSkill"
        || ability == "GA_Radio072_LTE"
        || ability == "GA_Radio072_SubUltraSkill"
    {
        let command_packet = request.actions.get(command).and_then(|cmd| match cmd {
            crate::simulation::GameActionCommand::InvokeAbility {
                damage_effect_id, ..
            } => Some(damage_effect_id.as_str()),
            _ => None,
        });
        if command_packet == Some("GE_Player_Radio072_LTE_Damage")
            || ability == "GA_Radio072_LTE"
            || ability == "GA_Radio072_SubUltraSkill"
        {
            let state = actor.character.linko().ok()?;
            if oversync_remaining_us(state, now) == 0 {
                return Some(AdmissionFailure {
                    reason: GameAdmissionReason::Other {
                        message: "oversync_required".into(),
                    },
                    message: "Linko LTE requires Oversync mode".into(),
                });
            }
        }
    }
    None
}

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != Some("GE_Player_Radio072_Melee1_Damage")
    {
        return Err("Linko Normal chain requires Melee1 as its admitted primary packet".into());
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.linko_mut()?;
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
            ctx.replace_native_primary_packet("GE_Player_Radio072_Melee2_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Radio072_Melee2_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        2 => {
            ctx.replace_native_primary_packet("GE_Player_Radio072_Melee3_Damage")?;
        }
        3 => {
            ctx.replace_native_primary_packet("GE_Player_Radio072_Melee4_1_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Radio072_Melee4_2_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            for _ in 0..5 {
                ctx.schedule_native_child_packet(
                    "GE_Player_Radio072_Melee4_3_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Radio072_Melee4_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        4 => {
            ctx.replace_native_primary_packet("GE_Player_Radio072_Melee5_Damage")?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1072" {
        return Ok(());
    }

    if ability_id == "GA_Radio072_Melee" {
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.linko_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    let awakening_count = actor_config.awakening.count();

    if ability_id == "GA_Radio072_Skill" {
        let packet_id = ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str());

        let pet_mode = ctx
            .request
            .model_assumptions
            .contains_key(PET_HYPOTHESIS_ID.as_str());
        if pet_mode || packet_id == Some("GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage") {
            ctx.record_hypothesis(PET_HYPOTHESIS_ID)?;
        }

        if packet_id == Some("GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage") {
            ctx.schedule_native_child_packet(
                "GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage2",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        } else if packet_id == Some("GE_Player_Radio072_Skill1_Damage") {
            ctx.record_hypothesis(SKILL_PACKETS_HYPOTHESIS_ID)?;
            for packet in [
                "GE_Player_Radio072_Skill2_Damage",
                "GE_Player_Radio072_Skill3_Damage",
                "GE_Player_Radio072_Skill4_Damage",
            ] {
                ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
            }
        } else {
            return Err(
                "Linko Skill train requires Skill1 or ThrowXiaozhen as its admitted primary packet"
                    .into(),
            );
        }

        if pet_mode || packet_id == Some("GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage") {
            let array_index = ctx
                .native_array_index
                .ok_or("Xiaozhen deployment requires an admitted native Skill coordinate")?;
            let state = ctx.actor.character.linko_mut()?;
            state.pet_active = true;
            state.pulse_generation = state.pulse_generation.wrapping_add(1);
            state.pulse_array_index = array_index;
            state.pulse_command_index = Some(ctx.command_index);
            let generation = state.pulse_generation;
            let interval_us = if awakening_count >= 1 {
                6_000_000
            } else {
                8_000_000
            };
            let first_pulse_us = ctx.hit_time_us + interval_us;
            state.next_pulse_us = Some(first_pulse_us);

            if first_pulse_us <= ctx.request.absolute_limit_us {
                crate::runtime_api::increment_sequence(ctx.sequence)?;
                ctx.queue.push(ScheduledGameEvent {
                    time_us: first_pulse_us,
                    priority: CausalPriority::BuffTickOrExpiry,
                    seq: *ctx.sequence,
                    data: GameEventData::Content(
                        crate::callbacks::ContentEvent::LinkoXiaozhenPulse {
                            actor_index: ctx.actor_index,
                            generation,
                            pulse_index: 0,
                        },
                    ),
                });
            }
        }
    }

    if ability_id == "GA_Radio072_UltraSkill" {
        let packet_id = ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str());

        if packet_id == Some("GE_Player_Radio072_LTE_Damage") {
            ctx.schedule_native_child_packet(
                "GE_Player_Radio072_LTE_Xiaozhen_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            return Ok(());
        }

        let duration_us = if actor_config.awakening.has_resonance_6() {
            PARAMETERS.resonance_field_duration_us
        } else {
            PARAMETERS.field_duration_us
        };
        if actor_config.awakening.has_resonance_6() {
            let value = PARAMETERS.resonance_field_damage_bonus;
            ctx.refresh_buff(
                "linko.resonance_field",
                BuffRecipient::TeamAll,
                duration_us,
                BTreeMap::from([
                    (
                        "damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE".into(),
                        value,
                    ),
                    (
                        "damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_INCANTATION"
                            .into(),
                        value,
                    ),
                ]),
            )?;
        }
        if ctx.request.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            for target_index in 0..ctx.request.targets.len() {
                if target_index == ctx.target_index {
                    continue;
                }
                ctx.schedule_native_child_packet(
                    "GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
            }
        }

        if ctx
            .request
            .model_assumptions
            .contains_key("linko.burst_synced_strike_v1")
        {
            ctx.schedule_native_child_packet(
                "GE_Player_Radio072_LTE_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Radio072_LTE_Xiaozhen_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }

        let state = ctx.actor.character.linko_mut()?;
        state.oversync_until_us = Some(ctx.time_us + 15_000_000);
        for target_idx in 0..ctx.request.targets.len() {
            state
                .hexed_targets
                .insert(target_idx, ctx.time_us + 15_000_000);
        }
    }

    if ability_id == "GA_Radio072_LTE" || ability_id == "GA_Radio072_SubUltraSkill" {
        ctx.schedule_native_child_packet(
            "GE_Player_Radio072_LTE_Xiaozhen_Damage",
            ctx.target_index,
            ctx.hit_time_us,
        )?;
    }

    if ability_id == "GA_Radio072_PerfectEvade" {
        ctx.schedule_native_child_packet(
            "GE_Player_Radio072_PerfectEvadeAttack_1_Damage",
            ctx.target_index,
            ctx.hit_time_us,
        )?;
    }

    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1072" {
        return Ok(());
    }

    if ctx
        .request
        .model_assumptions
        .contains_key("linko.precision_tuning_v1")
    {
        let targets: Vec<usize> = if ctx.config.awakening.count() >= 4 {
            (0..ctx.request.targets.len()).collect()
        } else {
            vec![ctx.target_index]
        };

        let elements = [
            "CHAOS",
            "COSMOS",
            "INCANTATION",
            "LAKSHANA",
            "NATURE",
            "PSYCHE",
        ];
        for target_idx in targets {
            let mut reductions = BTreeMap::new();
            for el in &elements {
                reductions.insert((*el).into(), ExactFraction::new(8, 100)?);
                reductions.insert(
                    format!("ECharacterElementType::CHARACTER_ELEMENT_TYPE_{el}"),
                    ExactFraction::new(8, 100)?,
                );
            }
            apply_target_status(
                ctx.buffs,
                TargetStatusDeclaration {
                    source_actor: ctx.actor_index,
                    source: "linko.precision_tuning".into(),
                    target_index: target_idx,
                    id: format!("linko.precision_tuning:{}", target_idx),
                    now_us: ctx.world_time_us,
                    duration_us: 12_000_000,
                    reductions,
                },
            )?;
        }
    }

    Ok(())
}

pub(crate) fn observe_party_hit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    hit_index: usize,
    hits: &[crate::simulation::GameHitReceipt],
) -> Result<(), String> {
    if owner >= ctx.state.actors.len() || ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }

    let hit = hits
        .get(hit_index)
        .ok_or("Linko Hexed trigger has no settled hit")?;

    // Hexed reaction triggers when a teammate (not Linko herself) lands a hit on the Hexed target
    if hit.actor_index == owner {
        return Ok(());
    }

    let now = ctx.state.current_time_us;
    let target_index = hit.target_index;

    if hit.damage_effect_id == "GE_Player_Radio072_LingZhou_Damage" {
        return Ok(());
    }

    if target_index >= ctx.state.targets.len() || ctx.state.targets[target_index].hp_current == 0 {
        return Ok(());
    }

    let has_hexed = ctx.state.actors[owner]
        .character
        .linko()
        .is_ok_and(|state| {
            state
                .hexed_targets
                .get(&target_index)
                .is_some_and(|&deadline| deadline > now)
        })
        || ctx
            .state
            .buffs
            .active_tagged_stacks_for_target_index(
                target_index,
                HEXED_STATUS_TAG,
                ctx.state.mode_clock.relative_us(),
            )
            .is_ok_and(|s| s > 0)
        || ctx
            .state
            .buffs
            .active_tagged_stacks_for_target_index(
                target_index,
                "Ability.Player.Buff.Reaction.LingZhou",
                ctx.state.mode_clock.relative_us(),
            )
            .is_ok_and(|s| s > 0);

    if !has_hexed {
        return Ok(());
    }

    let command_index = ctx.state.origins.register_from_hit(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: "GA_Radio072_Passive_1".into(),
            time_us: now,
            trigger: crate::simulation::GameCausalTrigger::SettledHit { hit_index },
            recipient_healing: None,
        },
        hits,
    )?;
    schedule_hexed_reaction_damage(
        &ctx.request.content_id,
        &ctx.request.model_assumptions,
        ctx.sequence,
        ctx.queue,
        command_index,
        owner,
        target_index,
        now,
    )
}

pub(crate) fn on_pulse(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    generation: u64,
    pulse_index: u64,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    if actor_index >= ctx.state.actors.len() || ctx.state.actors[actor_index].hp == 0 {
        return Ok(());
    }
    let config = &ctx.request.actors[actor_index];
    let awakening_count = config.awakening.count();
    let state = ctx.state.actors[actor_index].character.linko_mut()?;
    if !state.pet_active || state.pulse_generation != generation {
        return Ok(());
    }
    let array_index = state.pulse_array_index;
    let command_index = state
        .pulse_command_index
        .ok_or("Xiaozhen pulse has no deployment command")?;

    for target_index in 0..ctx.request.targets.len() {
        if ctx.state.targets[target_index].hp_current > 0 {
            schedule_xiaozhen_pulse(
                ctx.request,
                ctx.sequence,
                ctx.queue,
                command_index,
                actor_index,
                target_index,
                now,
                array_index,
            )?;
        }
    }

    let interval_us = if awakening_count >= 1 {
        6_000_000
    } else {
        8_000_000
    };
    let next_pulse_us = now.checked_add(interval_us).ok_or("pulse timer overflow")?;
    state.next_pulse_us = Some(next_pulse_us);

    if next_pulse_us <= ctx.request.absolute_limit_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: next_pulse_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::LinkoXiaozhenPulse {
                actor_index,
                generation,
                pulse_index: pulse_index + 1,
            }),
        });
    }

    Ok(())
}

fn schedule_hexed_reaction_damage(
    content_id: &str,
    model_assumptions: &BTreeMap<String, String>,
    sequence: &mut u64,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    command_index: usize,
    actor_index: usize,
    target_index: usize,
    time_us: u64,
) -> Result<(), String> {
    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        content_id,
        ntesim_data::skill_levels::frozen::tables(),
        "GE_Player_Radio072_LingZhou_Damage",
    )?;
    let mut packet =
        crate::simulation::metadata::resolve_packet_metadata(&native, model_assumptions)?;
    packet.semantic_tags.insert("hexed_reaction".into());
    packet.semantic_provenance = Some(packet.provenance.clone());
    let provenance = packet.provenance.clone();
    let action_meta = crate::simulation::metadata::ActionMetadata {
        categories: BTreeSet::from([crate::simulation::metadata::ActionCategory::Other]),
        gameplay_tags: Vec::new(),
        ability_id: "GA_Radio072_Passive_1".into(),
        native_ability_index: None,
        input_role: Some(crate::simulation::metadata::InputActionRole::Other),
        input_role_provenance: Some(provenance.clone()),
        primary_slot: None,
        source_evidence: packet.source_evidence.clone(),
        identity_evidence: packet.source_evidence.clone(),
        provenance,
    };
    let metadata = Box::new(crate::simulation::metadata::ActionPacketMetadata {
        action: action_meta,
        packet,
        binding: crate::simulation::metadata::ActionPacketBinding {
            energy_group_id: None,
            kind: crate::simulation::metadata::ActionPacketBindingKind::NativeDirect,
            initiating_ability_id: "GA_Radio072_Passive_1".into(),
            packet_ga_name: native.ga_name.clone(),
            provenance: None,
        },
    });
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us,
        priority: CausalPriority::HitDelivery,
        seq: *sequence,
        data: GameEventData::HitDelivery {
            command_index,
            actor_index,
            target_index,
            ability_id: "GA_Radio072_Passive_1".into(),
            damage_effect_id: "GE_Player_Radio072_LingZhou_Damage".into(),
            atk_rate: Ratio {
                numerator: 13,
                denominator: 100,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            metadata,
        },
    });
    Ok(())
}

fn schedule_xiaozhen_pulse(
    request: &crate::simulation::GameScenarioRequest,
    sequence: &mut u64,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    command_index: usize,
    actor_index: usize,
    target_index: usize,
    time_us: u64,
    array_index: usize,
) -> Result<(), String> {
    let (atk_rate, scaling_stat) = crate::simulation::inputs::coefficient(
        request,
        &request.actors[actor_index],
        "GA_Radio072_Skill",
        "GE_Xiaozhen_Skill1_Damage",
        array_index,
    )?;
    let mut metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &request.actors[actor_index].character_id,
        "GA_Radio072_Skill",
        "GE_Xiaozhen_Skill1_Damage",
        None,
        &request.model_assumptions,
    )?;
    metadata
        .packet
        .semantic_tags
        .insert("xiaozhen_pulse".into());
    metadata.packet.semantic_provenance = Some(metadata.packet.provenance.clone());
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us,
        priority: CausalPriority::HitDelivery,
        seq: *sequence,
        data: GameEventData::HitDelivery {
            command_index,
            actor_index,
            target_index,
            ability_id: "GA_Radio072_Skill".into(),
            damage_effect_id: "GE_Xiaozhen_Skill1_Damage".into(),
            atk_rate,
            scaling_stat,
            metadata: Box::new(metadata),
        },
    });
    Ok(())
}
