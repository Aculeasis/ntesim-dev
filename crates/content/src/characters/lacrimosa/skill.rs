use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) bat_evade_until_us: u64,
    pub(crate) baicang_tag_until_us: u64,
    pub(crate) melee: bool,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) nightmare: std::collections::BTreeMap<usize, super::dot::NightmareTargetState>,
    pub(crate) sleep_in: super::sleep_in::State,
    pub(crate) a3_def_ignore_stacks: u32,
    pub(crate) free_support_until_us: u64,
    pub(crate) free_support_cd_until_us: u64,
}

pub(crate) const FORM_HYPOTHESIS_ID: HypothesisId = HypothesisId::LacrimosaFormsV1;

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::LacrimosaRangedNormalChainV1;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::LacrimosaSkillBurstPacketsV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::LacrimosaAoeAllInRangeDummyV1;

fn handle_form_switch(ctx: &mut CastContext<'_>, packet: Option<&str>) -> Result<bool, String> {
    if ctx.ability_id == "GA_Lacrimosa_Melee"
        && matches!(
            packet,
            Some("GE_Player_Lacrimosa_Pan_Damage" | "GE_Player_Lacrimosa_SwitchModB_Damage")
        )
    {
        ctx.record_hypothesis(FORM_HYPOTHESIS_ID)?;
        let melee = ctx.actor.character.lacrimosa_mut()?.melee;
        let expected = if melee {
            "GE_Player_Lacrimosa_Pan_Damage"
        } else {
            "GE_Player_Lacrimosa_SwitchModB_Damage"
        };
        if packet != Some(expected) {
            return Err("Lacrimosa special packet disagrees with current form".into());
        }
        if melee {
            ctx.schedule_native_child_packet(expected, ctx.target_index, ctx.hit_time_us)?;
        }
        let state = ctx.actor.character.lacrimosa_mut()?;
        state.melee = !melee;
        state.normal_stage = 0;
        state.normal_started_us = None;
        return Ok(true);
    }
    Ok(false)
}

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet("GA_Lacrimosa_Melee")
    {
        return Err(
            "Lacrimosa ranged normal chain requires the modeled first packet as its admitted input"
                .into(),
        );
    }
    super::sleep_in::on_normal(ctx)?;
    let state = ctx.actor.character.lacrimosa_mut()?;
    state.normal_stage = crate::characters::normal::after_gap(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let stage = state.normal_stage;
    state.normal_stage = (stage + 1) % 5;
    state.normal_started_us = Some(ctx.time_us);
    if state.melee {
        ctx.record_hypothesis(FORM_HYPOTHESIS_ID)?;
        let packets: &[&str] = match stage {
            0 => &[
                "GE_Player_Lacrimosa_B_Melee1_Damage",
                "GE_Player_Lacrimosa_B_Melee2_Damage",
            ],
            1 => &[
                "GE_Player_Lacrimosa_B_Melee3_Damage",
                "GE_Player_Lacrimosa_B_Melee3_Damage",
            ],
            2 => &[
                "GE_Player_Lacrimosa_B_Melee4_Damage",
                "GE_Player_Lacrimosa_B_Melee3_1_Damage",
                "GE_Player_Lacrimosa_B_Melee3_1_Damage",
                "GE_Player_Lacrimosa_B_Melee3_1_Damage",
                "GE_Player_Lacrimosa_B_Melee3_Explode_Damage",
            ],
            3 => &[
                "GE_Player_Lacrimosa_B_Melee5_Damage",
                "GE_Player_Lacrimosa_B_Melee6_Damage",
            ],
            _ => &[
                "GE_Player_Lacrimosa_B_Melee7_Damage",
                "GE_Player_Lacrimosa_B_Melee7_Damage",
                "GE_Player_Lacrimosa_B_Melee7_Damage",
                "GE_Player_Lacrimosa_B_Melee8_Damage",
            ],
        };
        ctx.replace_native_primary_packet(packets[0])?;
        for packet in &packets[1..] {
            ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
        }
        return Ok(());
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let packet = [
        "GE_Player_Lacrimosa_Melee1_Damage",
        "GE_Player_Lacrimosa_Melee2_Damage",
        "GE_Player_Lacrimosa_Melee3_Damage",
        "GE_Player_Lacrimosa_Melee4_Damage",
        "GE_Player_Lacrimosa_Melee5_Damage",
    ][stage];
    if stage != 0 {
        ctx.replace_native_primary_packet(packet)?;
    }
    for _ in 1..[1, 3, 1, 3, 1][stage] {
        ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
    }
    if stage == 3 {
        ctx.schedule_native_child_packet(
            "GE_Player_Lacrimosa_Melee6_Damage",
            ctx.target_index,
            ctx.hit_time_us,
        )?;
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1004" {
        return Ok(());
    }

    let packet = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str());
    if handle_form_switch(ctx, packet)? {
        return Ok(());
    }
    if matches!(ability_id, "GA_Lacrimosa_Skill" | "GA_Lacrimosa_UltraSkill") {
        super::dot::apply_nightmare(ctx)?;
    }
    if ability_id == "GA_Lacrimosa_Skill" {
        super::sleep_in::on_skill(ctx)?;
        super::support::arm(ctx)?;
    }
    if ability_id == "GA_Lacrimosa_ExtremEvadeAtk" {
        ctx.schedule_native_child_packet(
            "GE_Player_Lacrimosa_PerfectEvadeAttack_1_Damage",
            ctx.target_index,
            ctx.hit_time_us,
        )?;
    }
    if ability_id == "GA_Lacrimosa_Melee" {
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.lacrimosa_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if ability_id == "GA_Lacrimosa_UltraSkill" && ctx.actor.character.lacrimosa_mut()?.melee {
        ctx.record_hypothesis(FORM_HYPOTHESIS_ID)?;
        ctx.replace_native_primary_packet("GE_Player_Lacrimosa_UltraSkillModB_Damage")?;
        for (packet, count) in [
            ("GE_Player_Lacrimosa_UltraSkillModB_Damage", 1),
            ("GE_Player_Lacrimosa_UltraSkillModB2_Damage", 5),
            ("GE_Player_Lacrimosa_UltraSkillModB3_Damage", 3),
        ] {
            for _ in 0..count {
                ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
            }
        }
        if ctx.request.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            for target_index in 0..ctx.request.targets.len() {
                if target_index == ctx.target_index {
                    continue;
                }
                for (packet, count) in [
                    ("GE_Player_Lacrimosa_UltraSkillModB_Damage", 2),
                    ("GE_Player_Lacrimosa_UltraSkillModB2_Damage", 5),
                    ("GE_Player_Lacrimosa_UltraSkillModB3_Damage", 3),
                ] {
                    for _ in 0..count {
                        ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
                    }
                }
            }
        }
        return Ok(());
    }

    let children: &[&str] = match ctx.ability_id {
        "GA_Lacrimosa_UltraSkill" => &[
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            "GE_Player_Lacrimosa_UltraSkill2_Damage",
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
                "Lacrimosa packet train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
        for child in children {
            ctx.schedule_native_child_packet(child, ctx.target_index, ctx.hit_time_us)?;
        }
    }

    if ctx.request.targets.len() > 1 && ability_id == "GA_Lacrimosa_UltraSkill" {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        let extra_packets: &[&str] = &[
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            "GE_Player_Lacrimosa_UltraSkill1_Damage",
            "GE_Player_Lacrimosa_UltraSkill2_Damage",
        ];
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            for packet in extra_packets {
                ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
            }
        }
    }

    // per 1004AwakenEffect.json / DT_LacrimosaEffectFigure.json Lacrimosa_lv2_Damup = 0.15, Lacrimosa_lv2_DamupDur = 15.0
    if ability_id == "GA_Lacrimosa_QTE" && actor_config.awakening.has_effect(2) {
        ctx.refresh_buff(
            "lacrimosa.a2_damage",
            ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
            PARAMETERS.a2_duration_us,
            std::collections::BTreeMap::from([("damage_bonus".into(), PARAMETERS.a2_damage_bonus)]),
        )?;
    }

    Ok(())
}

pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1004" {
        return Ok(());
    }

    let is_broken = ctx
        .buffs
        .active_tagged_stacks_for_target_index(
            ctx.target_index,
            crate::simulation::break_state::BROKEN_STATUS_TAG,
            ctx.world_time_us,
        )
        .is_ok_and(|s| s > 0);

    let is_stage_4 = ctx.ability_id == "GA_Lacrimosa_Melee"
        && matches!(
            ctx.damage_effect_id,
            "GE_Player_Lacrimosa_Melee4_Damage" | "GE_Player_Lacrimosa_Melee6_Damage"
        );
    let is_skill = ctx.ability_id == "GA_Lacrimosa_Skill"
        && ctx.damage_effect_id == "GE_Player_Lacrimosa_Skill1_Damage";

    if is_broken && (is_stage_4 || is_skill) {
        let hypothesis = HypothesisId::LacrimosaBrokenShatterV1;
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            ctx.execution_assumptions,
            hypothesis,
        )?;
        let native = ntesim_data::native_action_metadata::resolve_damage_effect(
            &ctx.request.content_id,
            ntesim_data::skill_levels::frozen::tables(),
            "GE_Player_Lacrimosa_B_Melee3_Explode_Damage",
        )?;
        let mut packet = crate::simulation::metadata::resolve_packet_metadata(
            &native,
            &ctx.request.model_assumptions,
        )?;
        packet.semantic_tags.insert("shatter".into());
        let provenance = Some(
            crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
                id: hypothesis.as_str().into(),
                description: hypothesis.description().into(),
            },
        );
        packet.semantic_provenance = provenance.clone();
        let metadata = Box::new(crate::simulation::metadata::ActionPacketMetadata {
            action: ctx.action_metadata.clone(),
            packet,
            binding: crate::simulation::metadata::ActionPacketBinding {
                energy_group_id: None,
                kind: crate::simulation::metadata::ActionPacketBindingKind::OwnerChild,
                initiating_ability_id: ctx.ability_id.to_owned(),
                packet_ga_name: native.ga_name.clone(),
                provenance,
            },
        });
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: ntesim_core::execution::CausalPriority::HitDelivery,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::HitDelivery {
                    command_index: ctx.command_index,
                    actor_index: ctx.actor_index,
                    target_index: ctx.target_index,
                    ability_id: ctx.ability_id.to_owned(),
                    damage_effect_id: "GE_Player_Lacrimosa_B_Melee3_Explode_Damage".into(),
                    atk_rate: ntesim_core::formula::Ratio {
                        numerator: 686,
                        denominator: 1000,
                    },
                    scaling_stat: crate::simulation::NativeScalingStat::Atk,
                    metadata,
                },
            });

        super::dot::extend_nightmare(
            ctx.actor.character.lacrimosa_mut()?,
            ctx.target_index,
            ctx.queue,
            ctx.sequence,
            3_000_000,
            ctx.time_us,
        )?;

        if ctx.config.awakening.has_effect(1) {
            super::sleep_in::reduce_cooldown(ctx, 2_000_000)?;
        }

        if ctx.config.awakening.has_resonance_3() {
            let state = ctx.actor.character.lacrimosa_mut()?;
            state.a3_def_ignore_stacks = (state.a3_def_ignore_stacks + 1).min(3);
        }
    }

    Ok(())
}

fn schedule_an_hun_zhou_two(
    content_id: &str,
    model_assumptions: &std::collections::BTreeMap<String, String>,
    sequence: &mut u64,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    command_index: usize,
    actor_index: usize,
    target_index: usize,
    time_us: u64,
    parent_hit_index: Option<usize>,
) -> Result<(), String> {
    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        content_id,
        ntesim_data::skill_levels::frozen::tables(),
        "GE_Player_Lacrimosa_AnHunZhouTwo_Damage",
    )?;
    let packet = crate::simulation::metadata::resolve_packet_metadata(&native, model_assumptions)?;
    let provenance = packet.provenance.clone();
    let action_meta = crate::simulation::metadata::ActionMetadata {
        categories: std::collections::BTreeSet::from([
            crate::simulation::metadata::ActionCategory::Other,
        ]),
        gameplay_tags: Vec::new(),
        ability_id: "GA_lacrimosa_Passive_1".into(),
        native_ability_index: None,
        input_role: None,
        input_role_provenance: None,
        primary_slot: None,
        source_evidence: packet.source_evidence.clone(),
        identity_evidence: None,
        provenance,
    };
    let metadata = Box::new(crate::simulation::metadata::ActionPacketMetadata {
        action: action_meta,
        packet,
        binding: crate::simulation::metadata::ActionPacketBinding {
            energy_group_id: None,
            kind: crate::simulation::metadata::ActionPacketBindingKind::NativeDirect,
            initiating_ability_id: "GA_lacrimosa_Passive_1".into(),
            packet_ga_name: native.ga_name.clone(),
            provenance: None,
        },
    });
    let packet = crate::simulation::events::HitPacket {
        command_index,
        actor_index,
        target_index,
        ability_id: "GA_lacrimosa_Passive_1".into(),
        damage_effect_id: "GE_Player_Lacrimosa_AnHunZhouTwo_Damage".into(),
        atk_rate: ntesim_core::formula::Ratio {
            numerator: 4,
            denominator: 1,
        },
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        metadata,
    };
    let data = match parent_hit_index {
        Some(parent_hit_index) => crate::simulation::events::GameEventData::FollowupHit {
            parent_hit_index,
            packet: Box::new(packet),
        },
        None => crate::simulation::events::GameEventData::HitDelivery {
            command_index: packet.command_index,
            actor_index: packet.actor_index,
            target_index: packet.target_index,
            ability_id: packet.ability_id,
            damage_effect_id: packet.damage_effect_id,
            atk_rate: packet.atk_rate,
            scaling_stat: packet.scaling_stat,
            metadata: packet.metadata,
        },
    };
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(crate::simulation::events::ScheduledGameEvent {
        time_us,
        priority: ntesim_core::execution::CausalPriority::HitDelivery,
        seq: *sequence,
        data,
    });
    Ok(())
}

pub(crate) fn on_discord(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target_index: usize,
    source_command_key: usize,
) -> Result<(), String> {
    let lacrimosa_actor = ctx.request.actors.iter().enumerate().find(|(idx, actor)| {
        actor.character_id == "character:1004" && ctx.state.actors[*idx].hp > 0
    });
    let Some((actor_index, _config)) = lacrimosa_actor else {
        return Ok(());
    };

    let (command_index, autonomous_event_index) = ctx
        .state
        .origins
        .identify(ctx.request, source_command_key)?;
    let trigger = match (command_index, autonomous_event_index) {
        (Some(command_index), None) => {
            crate::simulation::GameCausalTrigger::Action { command_index }
        }
        (None, Some(autonomous_event_index)) => {
            crate::simulation::GameCausalTrigger::OwnerDeadline {
                autonomous_event_index,
            }
        }
        _ => return Err("Discord has no unique causal source".into()),
    };
    let command_key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: "GA_lacrimosa_Passive_1".into(),
            time_us: ctx.state.current_time_us,
            trigger,
            recipient_healing: None,
        },
    )?;
    schedule_an_hun_zhou_two(
        &ctx.request.content_id,
        &ctx.request.model_assumptions,
        ctx.sequence,
        ctx.queue,
        command_key,
        actor_index,
        target_index,
        ctx.state.current_time_us,
        None,
    )
}
