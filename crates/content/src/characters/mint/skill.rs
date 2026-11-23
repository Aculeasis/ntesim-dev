use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::CastContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use ntesim_core::execution::CausalPriority;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) riposte_ready: bool,
    pub(crate) airborne: bool,
    pub(crate) entry_normal_pending: bool,
    pub(crate) a1_guard_until_us: Option<u64>,
    pub(crate) clock: Option<Box<super::clock::Clock>>,
}

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::MintNormalChainV1;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::MintSkillBurstPacketsV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::MintAoeAllInRangeDummyV1;

pub(crate) const PASSIVE2_HYPOTHESIS_ID: HypothesisId = HypothesisId::MintPassive2EntryDefV1;

/// Selected `mint.source_schedule_v2` offsets: Skill hit 1 lands at
/// .20017847s and connected hit 2 at .5654236s from the section input. Hit 1's
/// .05s stop at rate .01 supplies the .36524613s gap; a miss leaves the
/// nominal .5159236s coordinate but delivers no damage.
const SKILL_SECOND_HIT_DELAY_US: u64 = 365_246;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1019" {
        return Ok(());
    }
    if ctx.ability_id == "GA_Mint019_Melee" {
        let packet_id = ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str());
        if packet_id.is_some_and(super::plunge::is_plunge_packet) {
            super::plunge::on_cast(ctx)?;
            return Ok(());
        }
        if packet_id.is_some_and(super::hold::is_hold_packet) {
            super::hold::on_cast(ctx)?;
            return Ok(());
        }
        if packet_id != super::primary_packet(ctx.ability_id) {
            return Err(
                "Mint normal chain requires the modeled first packet as its admitted input".into(),
            );
        }
        ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
        let state = ctx.actor.character.mint_mut()?;
        state.normal_stage = crate::characters::normal::after_gap(
            state.normal_stage,
            state.normal_started_us,
            ctx.time_us,
        );
        let stage = state.normal_stage;
        state.normal_stage = (stage + 1) % 5;
        state.normal_started_us = Some(ctx.time_us);
        let packets: &[&str] = match stage {
            0 => &[
                "GE_Player_Mint_Melee1_Damage_New",
                "GE_Player_Mint_Melee2_Damage_New",
            ],
            1 => &["GE_Player_Mint_Melee3_Damage_New"],
            2 => &[
                "GE_Player_Mint_Melee4_Damage_New",
                "GE_Player_Mint_Melee5_Damage_New",
            ],
            3 => &["GE_Player_Mint_Melee6_Damage_New"],
            _ => &[
                "GE_Player_Mint_Melee7_Damage_New",
                "GE_Player_Mint_Melee8_Damage_New",
            ],
        };
        if stage != 0 {
            ctx.replace_native_primary_packet(packets[0])?;
        }
        for child in &packets[1..] {
            ctx.schedule_native_child_packet(child, ctx.target_index, ctx.hit_time_us)?;
        }
    } else {
        let state = ctx.actor.character.mint_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
    let children: &[&str] = match ctx.ability_id {
        "GA_Mint019_Skill" => &["GE_Player_Mint_Skill1_Damage_New"],
        "GA_Mint019_UltraSkill" => &[
            "GE_Player_Mint_UtraSkill2_Damage_New",
            "GE_Player_Mint_UtraSkill3_Damage_New",
            "GE_Player_Mint_UtraSkill4_Damage_New",
            "GE_Player_Mint_UtraSkill5_Damage_New",
            "GE_Player_Mint_UtraSkill6_Damage_New",
            "GE_Player_Mint_UtraSkill7_Damage_New",
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
                "Mint packet train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
        if ctx.request.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        }
        let first_hit_us = ctx.hit_time_us;
        let second_hit_us = (ctx.ability_id == "GA_Mint019_Skill")
            .then(|| {
                first_hit_us
                    .checked_add(SKILL_SECOND_HIT_DELAY_US)
                    .ok_or_else(|| "Mint second-hit delivery deadline overflow".to_string())
            })
            .transpose()?
            .unwrap_or(first_hit_us);
        for child in children {
            ctx.schedule_native_child_packet(child, ctx.target_index, second_hit_us)?;
        }
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            let all_packets: &[&str] = match ctx.ability_id {
                "GA_Mint019_Skill" => &[
                    "GE_Player_Mint_Skill1_Damage_New",
                    "GE_Player_Mint_Skill1_Damage_New",
                ],
                "GA_Mint019_UltraSkill" => &[
                    "GE_Player_Mint_UtraSkill1_Damage_New",
                    "GE_Player_Mint_UtraSkill2_Damage_New",
                    "GE_Player_Mint_UtraSkill3_Damage_New",
                    "GE_Player_Mint_UtraSkill4_Damage_New",
                    "GE_Player_Mint_UtraSkill5_Damage_New",
                    "GE_Player_Mint_UtraSkill6_Damage_New",
                    "GE_Player_Mint_UtraSkill7_Damage_New",
                ],
                _ => &[],
            };
            for (index, packet) in all_packets.iter().enumerate() {
                let hit_us = if index == 1 {
                    second_hit_us
                } else {
                    first_hit_us
                };
                ctx.schedule_native_child_packet(packet, target_index, hit_us)?;
            }
        }
    }
    effects(ctx)
}

pub(crate) fn effects(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let is_mint_skill =
        ctx.config.character_id == "character:1019" && ctx.ability_id == "GA_Mint019_Skill";
    let awakenings = &ctx.config.awakening;
    let actor_idx = ctx.actor_index;
    let cast_time_us = ctx.time_us;
    if is_mint_skill {
        // Mint A5: +25% CRIT DMG for 6.0s (Mint_Adcance_5_1 = 0.25, Mint_Adcance_5_2 = 6.0)
        if awakenings.has_effect(5) {
            ctx.refresh_buff(
                "mint.a5_crit_damage",
                ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
                super::parameters::PARAMETERS.skill_crit_duration_us,
                std::collections::BTreeMap::from([(
                    "crit_damage_bonus".into(),
                    super::parameters::PARAMETERS.skill_crit_damage,
                )]),
            )?;
        }

        // Mint resonance_6 milestone bonus (for 6 total awakenings, distinct from individual node Effect6):
        // grants +2 cycle energy (Mint_Adcance_6_AddEnergy = 2.0)
        if awakenings.has_resonance_6() {
            let queue = &mut *ctx.queue;
            let seq = &mut *ctx.sequence;
            crate::runtime_api::increment_sequence(seq)?;
            queue.push(ScheduledGameEvent {
                time_us: cast_time_us,
                priority: CausalPriority::ProcCallback,
                seq: *seq,
                data: GameEventData::Content(ContentEvent::MintA6EnergyGrant {
                    actor_index: actor_idx,
                    amount: super::parameters::PARAMETERS.resonance_energy,
                }),
            });
        }
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1019" {
        return Ok(());
    }
    if ctx.damage_effect_id == super::riposte::PACKET
        && ctx.applied_hp_loss > 0
        && ctx.config.awakening.has_effect(1)
    {
        let state = ctx.actor.character.mint_mut()?;
        let guarded = state
            .a1_guard_until_us
            .is_some_and(|until| ctx.time_us < until);
        if !guarded
            && let Some(until) = ctx.actor.ability_cooldowns.get("GA_Mint019_Skill").copied()
            && until > ctx.time_us
        {
            ctx.actor.ability_cooldowns.remove("GA_Mint019_Skill");
            let state = ctx.actor.character.mint_mut()?;
            state.a1_guard_until_us =
                Some(ctx.time_us + super::parameters::PARAMETERS.a1_guard_duration_us);
            ctx.trace.push(crate::simulation::GameEventReceipt {
                time_us: ctx.time_us,
                phase: "HitSettlement".into(),
                event_kind: "mint_a1_skill_cooldown_reset".into(),
                active_actor: ctx.active_actor_index,
                hit_index: None,
                from_actor: Some(ctx.actor_index),
                to_actor: Some(ctx.actor_index),
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    ability_id: Some("GA_Mint019_Skill".into()),
                    cooldown_until_us: state.a1_guard_until_us,
                    ..Default::default()
                }),
            });
        }
    }
    Ok(())
}

pub(crate) fn passive2_accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(PASSIVE2_HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(PASSIVE2_HYPOTHESIS_ID.description())
}

pub(crate) fn on_field_entry(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    let actor_config = &ctx.request.actors[actor_index];
    if actor_config.character_id != "character:1019" || !passive2_accepted(ctx.request) {
        return Ok(());
    }
    let breakthrough = ntesim_data::skill_levels::progression(
        &actor_config.character_id,
        actor_config.character_level,
        actor_config.character_max_level,
    )
    .ok()
    .and_then(|p| p["breakthrough"].as_u64())
    .unwrap_or(0);

    if breakthrough >= 4 {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            PASSIVE2_HYPOTHESIS_ID,
        )?;
        let ratio = ntesim_core::formula::Ratio {
            numerator: u64::try_from(super::parameters::PARAMETERS.entry_def_bonus.numerator)
                .map_err(|_| "overflow")?,
            denominator: u64::try_from(super::parameters::PARAMETERS.entry_def_bonus.denominator)
                .map_err(|_| "overflow")?,
        };
        ctx.state
            .buffs
            .apply_buff(
                ntesim_core::buffs::dispatch::BuffDefinition {
                    stacking_scope: ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                    semantic_tags: Default::default(),
                    source_lifetime: ntesim_core::buffs::dispatch::BuffSourceLifetime::WhileAlive,
                    id: "mint.passive2_def".into(),
                    source: "mint.passive2".into(),
                    recipient: ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
                    duration_us: None,
                    refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy::Reset,
                    max_stacks: 1,
                    decay_policy: ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
                    stat_modifiers: std::collections::BTreeMap::from([(
                        "def_percent".into(),
                        ratio,
                    )]),
                },
                actor_index,
                ctx.state.mode_clock.relative_us(),
            )
            .map_err(|e| format!("Mint passive 2 buff failed: {e:?}"))?;
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ActionAdmission".into(),
            event_kind: "mint_passive2_entry".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                buff_id: Some("mint.passive2_def".into()),
                def_percent: Some(super::parameters::PARAMETERS.entry_def_bonus),
                ..Default::default()
            }),
        });
    }
    Ok(())
}

pub(crate) fn on_field_exit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    let actor = &mut ctx.state.actors[actor_index];
    if let crate::characters::RuntimeState::Mint(state) = &mut actor.character {
        state.riposte_ready = false;
    }
    ctx.state
        .buffs
        .remove_owned_buff("mint.passive2_def", "mint.passive2", actor_index);
    Ok(())
}
