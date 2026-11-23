use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::BaicangPowerWordsV1;

pub(crate) const WORD_A_PACKET: &str = "GE_Player_Cang_WorldA_Damage";
pub(crate) const WORD_B_PACKET: &str = "GE_Player_Cang_WorldB_Damage";
pub(crate) const WORD_C_INITIAL_PACKET: &str = "GE_Player_Cang_WorldC_Damage";
pub(crate) const WORD_C_MISSILE_PACKET: &str = "GE_Player_Cang_WorldC_1_Damage";

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn deduct_hp_cost(hp: u64, hp_max: u64) -> u64 {
    if hp <= 1 {
        return 0;
    }
    let cost = if u128::from(hp) * 2 <= u128::from(hp_max) {
        // 1.5% current HP, nearest integer ties upward
        ((u128::from(hp) * 15 + 500) / 1000) as u64
    } else {
        // 3.0% current HP, nearest integer ties upward
        ((u128::from(hp) * 3 + 50) / 100) as u64
    };
    cost.min(hp.saturating_sub(1))
}

/// One connected input, regardless of its projectile or recipient count.
pub(crate) fn on_connected_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let packet = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();
    let kind = match packet {
        "GE_Player_Cang_Melee1_Damage"
        | "GE_Player_Cang_Melee2_Damage"
        | "GE_Player_Cang_Melee3_Damage"
        | "GE_Player_Cang_Melee4_Damage"
        | "GE_Player_Cang_Melee5_Damage"
        | super::branches::BRANCH1_PACKET
        | super::branches::BRANCH2_PACKET
        | super::branches::BRANCH1_1_PACKET
        | super::branches::BRANCH2_1_PACKET => None,
        "GE_Player_Cang_SkillA_Damage"
        | "GE_Player_Cang_SkillA_Damage2"
        | super::etd::SKIA_PACKET
        | super::fadia_assist::PACKET
        | "GE_Player_Cang_QTE_Damage" => Some(super::awakenings::WordType::Silence),
        "GE_Player_Cang_SkillB_Damage" => Some(super::awakenings::WordType::Objurgate),
        _ => return Ok(()),
    };
    let state = ctx.actor.character.baicang_mut()?;
    if !state.connected_commands.insert(ctx.command_index) {
        return Ok(());
    }
    let kind = match kind {
        Some(kind) => kind,
        None => {
            state.normal_word_counter += 1;
            if state.normal_word_counter < 3 {
                return Ok(());
            }
            state.normal_word_counter = 0;
            super::awakenings::WordType::Bless
        }
    };
    let target = if packet == super::fadia_assist::PACKET {
        0
    } else {
        ctx.target_index
    };
    generate_word(ctx, kind, target)
}

pub(crate) fn generate_word(
    ctx: &mut CastContext<'_>,
    word_type: super::awakenings::WordType,
    target_index: usize,
) -> Result<(), String> {
    ctx.record_hypothesis(HYPOTHESIS_ID)?;

    // Deduct nonlethal HP cost
    let hp_max = crate::simulation::state::effective_hp_max(
        ctx.buffs,
        ctx.actor.hp_max_base,
        &ctx.actor.hp_max_static_bonus,
        ctx.actor_index,
        ctx.actor.hp_max,
    )?;
    let cost = deduct_hp_cost(ctx.actor.hp, hp_max);
    ctx.actor.hp = ctx.actor.hp.saturating_sub(cost);
    ctx.actor.record_hp_loss(cost, hp_max);
    if cost > 0 {
        // Payment is already applied before generation/healing. Notify the
        // cost consumer once, without another HP deduction or incoming damage.
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: ntesim_core::execution::CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::BaicangWordHpConsumed {
                        actor_index: ctx.actor_index,
                        amount: cost,
                    },
                ),
            });
    }

    let is_domain_active = {
        let baicang = ctx.actor.character.baicang_mut()?;
        baicang.word_generation_counter = baicang.word_generation_counter.saturating_add(1);

        // A2: +.06 ATK for 10s, independent expiries, cap 4
        if ctx.config.awakening.has_effect(2) {
            let expiry_us = ctx.time_us.saturating_add(10_000_000);
            baicang.a2_stack_expiries.retain(|&exp| exp > ctx.time_us);
            if baicang.a2_stack_expiries.len() >= 4 {
                baicang.a2_stack_expiries.remove(0);
            }
            baicang.a2_stack_expiries.push(expiry_us);
            baicang.power_word_stacks = baicang.a2_stack_expiries.len() as u32;
            baicang.power_word_until_us = expiry_us;
        }

        baicang.domain_start_us <= ctx.time_us && baicang.domain_until_us > ctx.time_us
    };

    super::scorch::on_word(ctx)?;
    if is_domain_active {
        activate_single_word_in_domain(ctx, word_type, target_index)?;
    } else {
        let cap = if ctx.config.awakening.has_effect(3) {
            4
        } else {
            3
        };
        {
            let baicang = ctx.actor.character.baicang_mut()?;
            baicang.stored_words.retain(|w| w.expiry_us > ctx.time_us);
            if baicang.stored_words.len() >= cap {
                baicang.stored_words.remove(0);
            }
            baicang.stored_words.push(super::awakenings::StoredWord {
                word_type,
                target_index,
                born_us: ctx.time_us,
                expiry_us: ctx.time_us.saturating_add(15_000_000),
                generation: baicang.word_generation_counter,
            });
        }
    }

    Ok(())
}

pub(crate) fn activate_retained_words_on_domain_start(
    ctx: &mut CastContext<'_>,
) -> Result<(), String> {
    let words = {
        let baicang = ctx.actor.character.baicang_mut()?;
        std::mem::take(&mut baicang.stored_words)
    };
    for word in words {
        if word.expiry_us > ctx.time_us {
            activate_single_word_in_domain(ctx, word.word_type, word.target_index)?;
        }
    }
    Ok(())
}

pub(crate) fn field_start(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    command_index: usize,
    generation: u64,
    array_index: u32,
) -> Result<(), String> {
    super::skill::domain_started(ctx, actor_index, generation)?;
    let actor = &mut ctx.state.actors[actor_index];
    if actor.hp == 0
        || !matches!(&actor.character, crate::characters::RuntimeState::Baicang(state) if state.domain_generation == generation)
    {
        return Ok(());
    }
    let config = &ctx.request.actors[actor_index];
    let metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        "GA_Cang_UltraSkill",
        "GE_Player_Cang_UltraSkill2_Damage",
        None,
        &ctx.request.model_assumptions,
    )?;
    let mut cast = CastContext {
        origins: &mut ctx.state.origins,
        command_index,
        native_array_index: Some(array_index as usize),
        execution_assumptions: &mut ctx.state.execution_model_assumptions,
        config,
        request: ctx.request,
        actor_index,
        active_actor_index: ctx.state.active_actor_index,
        target_index: ctx.request.aim_target(command_index),
        ability_id: "GA_Cang_UltraSkill",
        action_metadata: &metadata.action,
        packet_metadata: None,
        hit_time_us: ctx.state.current_time_us,
        world_time_us: ctx.state.mode_clock.relative_us(),
        time_us: ctx.state.current_time_us,
        buffs: &mut ctx.state.buffs,
        actor,
        stamina: ctx.state.combat_stamina.as_mut(),
        team_hp_snapshot: &[],
        targets: &ctx.state.targets,
        queue: ctx.queue,
        sequence: ctx.sequence,
    };
    activate_retained_words_on_domain_start(&mut cast)
}

fn resolve_burst_array_index(ctx: &CastContext<'_>) -> Result<u32, String> {
    if ctx.ability_id == "GA_Cang_UltraSkill"
        && let Some(idx) = ctx.native_array_index
    {
        return idx.try_into().map_err(|_| "index exceeds u32".into());
    }
    let progression = ntesim_data::skill_levels::progression(
        &ctx.config.character_id,
        ctx.config.character_level,
        ctx.config.character_max_level,
    )?;
    let breakthrough: u32 = progression["breakthrough"]
        .as_u64()
        .ok_or("breakthrough missing")?
        .try_into()
        .map_err(|_| "breakthrough exceeds u32")?;
    let level = crate::build::preview_projection::effective_skill_level(
        ntesim_data::skill_levels::frozen::tables(),
        &ctx.config.character_id,
        "GA_Cang_UltraSkill",
        ctx.config
            .talent_levels
            .map_or(ctx.config.purchased_skill_level, |l| l[2]),
        breakthrough,
        &ctx.config.awakening,
    )?;
    level["array_index"]
        .as_u64()
        .ok_or("burst coordinate missing")?
        .try_into()
        .map_err(|_| "coordinate exceeds u32".into())
}

fn schedule_word_hit(
    ctx: &mut CastContext<'_>,
    command_index: usize,
    packet_id: &'static str,
    target_index: usize,
    hit_time_us: u64,
    array_index: u32,
) -> Result<(), String> {
    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &ctx.request.content_id,
            character_id: &ctx.config.character_id,
            ability_id: "GA_Cang_UltraSkill",
            damage_effect_id: packet_id,
            array_index,
            research_metadata: None,
            model_assumptions: &ctx.request.model_assumptions,
        },
    )?;
    let atk_rate: ntesim_core::formula::Ratio =
        serde_json::from_value(projection["compiled_rate"].clone())
            .map_err(|e| format!("rate error: {e}"))?;
    let scaling_stat = serde_json::from_value(projection["compiled_scaling_stat"].clone())
        .map_err(|e| format!("stat error: {e}"))?;
    let mut metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &ctx.config.character_id,
        "GA_Cang_UltraSkill",
        packet_id,
        None,
        &ctx.request.model_assumptions,
    )?;
    // The native ability slot is action identity, never its talent-array coordinate.
    // Keep native tags/evidence while disclosing the owner-triggered child edge.
    metadata.binding.kind = crate::simulation::metadata::ActionPacketBindingKind::OwnerChild;
    metadata.binding.provenance = Some(
        crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
            id: HYPOTHESIS_ID.as_str().into(),
            description: HYPOTHESIS_ID.description().into(),
        },
    );

    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: hit_time_us,
            priority: ntesim_core::execution::CausalPriority::HitDelivery,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::HitDelivery {
                command_index,
                actor_index: ctx.actor_index,
                target_index,
                ability_id: "GA_Cang_UltraSkill".into(),
                damage_effect_id: packet_id.into(),
                atk_rate,
                scaling_stat,
                metadata: Box::new(metadata),
            },
        });
    Ok(())
}

pub(crate) fn activate_single_word_in_domain(
    ctx: &mut CastContext<'_>,
    word_type: super::awakenings::WordType,
    target_index: usize,
) -> Result<(), String> {
    let array_index = resolve_burst_array_index(ctx)?;
    let time_us = ctx.time_us;
    let command_index = if ctx.ability_id == "GA_Cang_UltraSkill"
        || ctx
            .origins
            .identify(ctx.request, ctx.command_index)?
            .1
            .is_some()
    {
        ctx.command_index
    } else {
        ctx.origins.register(
            ctx.request,
            crate::simulation::GameAutonomousEventReceipt {
                actor_index: ctx.actor_index,
                active_actor_index: ctx.active_actor_index,
                ability_id: "GA_Cang_UltraSkill".into(),
                time_us,
                trigger: crate::simulation::GameCausalTrigger::Action {
                    command_index: ctx.command_index,
                },
                recipient_healing: None,
            },
        )?
    };

    match word_type {
        super::awakenings::WordType::Silence => {
            // A1 stack at +1.0s
            let a1_time_us = time_us.saturating_add(1_000_000);
            if ctx.config.awakening.has_effect(1) {
                crate::runtime_api::increment_sequence(ctx.sequence)?;
                ctx.queue
                    .push(crate::simulation::events::ScheduledGameEvent {
                        time_us: a1_time_us,
                        priority: ntesim_core::execution::CausalPriority::ProcCallback,
                        seq: *ctx.sequence,
                        data: crate::simulation::events::GameEventData::Content(
                            crate::callbacks::ContentEvent::BaicangWordSilenceA1 {
                                actor_index: ctx.actor_index,
                            },
                        ),
                    });
            }
            // Damage at +1.4s
            let hit_time_us = time_us.saturating_add(1_400_000);
            schedule_word_hit(
                ctx,
                command_index,
                WORD_A_PACKET,
                target_index,
                hit_time_us,
                array_index,
            )?;

            // A4: 5% MaxHP heal on completed activated Word
            if ctx.config.awakening.has_effect(4) {
                schedule_word_heal(ctx, hit_time_us, 5)?;
            }
        }
        super::awakenings::WordType::Objurgate => {
            // A1 CD reduction at +1.2s
            let a1_time_us = time_us.saturating_add(1_200_000);
            if ctx.config.awakening.has_effect(1) {
                crate::runtime_api::increment_sequence(ctx.sequence)?;
                ctx.queue
                    .push(crate::simulation::events::ScheduledGameEvent {
                        time_us: a1_time_us,
                        priority: ntesim_core::execution::CausalPriority::ProcCallback,
                        seq: *ctx.sequence,
                        data: crate::simulation::events::GameEventData::Content(
                            crate::callbacks::ContentEvent::BaicangWordObjurgateA1 {
                                actor_index: ctx.actor_index,
                            },
                        ),
                    });
            }
            // Damage at +1.2s to ALL living targets (AoE)
            let hit_time_us = time_us.saturating_add(1_200_000);
            for t_idx in 0..ctx.targets.len() {
                if ctx.targets[t_idx].hp_current > 0 {
                    schedule_word_hit(
                        ctx,
                        command_index,
                        WORD_B_PACKET,
                        t_idx,
                        hit_time_us,
                        array_index,
                    )?;
                }
            }

            // A4: 5% MaxHP heal on completed activated Word
            if ctx.config.awakening.has_effect(4) {
                schedule_word_heal(ctx, hit_time_us, 5)?;
            }
        }
        super::awakenings::WordType::Bless => {
            // Initial damage + 5% MaxHP heal (10% with A4) at +1.0s
            let initial_time_us = time_us.saturating_add(1_000_000);
            schedule_word_hit(
                ctx,
                command_index,
                WORD_C_INITIAL_PACKET,
                target_index,
                initial_time_us,
                array_index,
            )?;
            let heal_percent = if ctx.config.awakening.has_effect(4) {
                10
            } else {
                5
            };
            schedule_word_heal(ctx, initial_time_us, heal_percent)?;

            // 2 returning missiles at +1.3s
            let missile_time_us = time_us.saturating_add(1_300_000);
            schedule_word_hit(
                ctx,
                command_index,
                WORD_C_MISSILE_PACKET,
                target_index,
                missile_time_us,
                array_index,
            )?;
            schedule_word_hit(
                ctx,
                command_index,
                WORD_C_MISSILE_PACKET,
                target_index,
                missile_time_us,
                array_index,
            )?;
        }
    }

    Ok(())
}

fn schedule_word_heal(
    ctx: &mut CastContext<'_>,
    heal_time_us: u64,
    heal_percent: u64,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: heal_time_us,
            priority: ntesim_core::execution::CausalPriority::SettlementAndDeath,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::BaicangWordHeal {
                    actor_index: ctx.actor_index,
                    percent: heal_percent,
                },
            ),
        });
    Ok(())
}
