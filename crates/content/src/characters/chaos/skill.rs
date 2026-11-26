use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) crime: Option<ntesim_core::kernel::state::ExactPool>,
    pub(crate) crime_packets: std::collections::BTreeSet<String>,
    pub(crate) crime_a1_packets: std::collections::BTreeSet<String>,
    pub(crate) clock: Option<super::clock::Clock>,
    pub(crate) warrant_bonus: Option<crate::simulation::ExactFraction>,
    pub(crate) form: Option<super::form::Form>,
    pub(crate) form_generation: u64,
    pub(crate) burst_switch_until_us: u64,
    pub(crate) burst_immunity_until_us: u64,
    pub(crate) verdict: Option<super::verdict::Pending>,
    pub(crate) verdict_immunity_until_us: u64,
    pub(crate) entry_normal_ready: bool,
    pub(crate) riposte_ready: bool,
    pub(crate) remoras: std::collections::BTreeMap<usize, u64>,
    pub(crate) remora_bonuses: std::collections::BTreeMap<usize, crate::simulation::ExactFraction>,
}

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::ChaosNormalChainV1;

pub(crate) const SKILL_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::ChaosSkillBitesV1;

pub(crate) const BURST_PACKETS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ChaosSuppressionPacketsV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::ChaosAoeAllInRangeDummyV1;

fn handle_melee(ctx: &mut CastContext<'_>, ability_id: &str) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        == super::primary_packet(ability_id)
    {
        ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
        let state = ctx.actor.character.chaos_mut()?;
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
                ctx.replace_native_primary_packet("GE_Player_Chaos_Melee2_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Chaos_Melee2_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            2 => {
                ctx.replace_native_primary_packet("GE_Player_Chaos_Melee3_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Chaos_Melee3_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            3 => {
                ctx.replace_native_primary_packet("GE_Player_Chaos_Melee4_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Chaos_Melee4_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Chaos_Melee4_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Chaos_Melee4_2_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            4 => {
                ctx.replace_native_primary_packet("GE_Player_Chaos_Melee5_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Chaos_Melee5_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Chaos_Melee5_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            _ => unreachable!(),
        }
    } else {
        let state = ctx.actor.character.chaos_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1071" {
        return Ok(());
    }
    let ability_id = ctx.ability_id;
    if let Some(kind) = super::secondary::on_cast(ctx)? {
        return super::clock::secondary::cast(ctx, kind);
    }
    ctx.actor.character.chaos_mut()?.riposte_ready = false;
    if super::support::on_cast(ctx)? {
        return super::clock::on_cast(ctx);
    }
    if super::verdict::on_cast(ctx)? {
        return Ok(());
    }
    if super::resources::accepted(ctx.request) && super::grouped::on_cast(ctx)? {
        awakening_buff(ctx)?;
        super::form::on_cast(ctx)?;
        super::warrant::on_cast(ctx)?;
        return super::clock::on_cast(ctx);
    }

    if ability_id == "GA_Chaos071_Melee" {
        handle_melee(ctx, ability_id)?;
    } else {
        let state = ctx.actor.character.chaos_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if ability_id == "GA_Chaos071_Skill" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != Some("GE_Player_Chaos_Skill1_Damage")
        {
            return Err(
                "Chaos Doubtmark requires the modeled first bite as its admitted primary packet"
                    .into(),
            );
        }
        ctx.record_hypothesis(SKILL_PACKETS_HYPOTHESIS_ID)?;
        ctx.schedule_native_child_packet(
            "GE_Player_Chaos_Skill2_Damage",
            ctx.target_index,
            ctx.hit_time_us,
        )?;
    }

    if ability_id == "GA_Chaos071_UltraSkill" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != Some("GE_Player_Chaos_UltraSkill1_Damage")
        {
            return Err(
                "Chaos Retribution requires the modeled first Suppression packet as its admitted primary packet".into(),
            );
        }
        ctx.record_hypothesis(BURST_PACKETS_HYPOTHESIS_ID)?;
        for _ in 0..3 {
            ctx.schedule_native_child_packet(
                "GE_Player_Chaos_UltraSkill1_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        ctx.schedule_native_child_packet(
            "GE_Player_Chaos_UltraSkill2_Damage",
            ctx.target_index,
            ctx.hit_time_us,
        )?;
    }

    if ctx.request.targets.len() > 1
        && (ability_id == "GA_Chaos071_Skill" || ability_id == "GA_Chaos071_UltraSkill")
    {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            let all_packets: &[&str] = match ability_id {
                "GA_Chaos071_Skill" => &[
                    "GE_Player_Chaos_Skill1_Damage",
                    "GE_Player_Chaos_Skill2_Damage",
                ],
                "GA_Chaos071_UltraSkill" => &[
                    "GE_Player_Chaos_UltraSkill1_Damage",
                    "GE_Player_Chaos_UltraSkill1_Damage",
                    "GE_Player_Chaos_UltraSkill1_Damage",
                    "GE_Player_Chaos_UltraSkill1_Damage",
                    "GE_Player_Chaos_UltraSkill2_Damage",
                ],
                _ => &[],
            };
            for packet in all_packets {
                ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
            }
        }
    }

    awakening_buff(ctx)
}

fn awakening_buff(ctx: &mut CastContext<'_>) -> Result<(), String> {
    // Source: DT_ChaosEffectFigure.json Chaos_Star3Duration = 20.0,
    // Chaos_Star3CritDamageUp = 0.30.
    if ctx.ability_id == "GA_Chaos071_UltraSkill" && ctx.config.awakening.has_effect(3) {
        ctx.refresh_buff(
            "chaos.a3_crit_damage",
            ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
            PARAMETERS.a3_duration_us,
            std::collections::BTreeMap::from([(
                "crit_damage_bonus".into(),
                PARAMETERS.a3_crit_damage,
            )]),
        )?;
    }

    Ok(())
}

pub(crate) fn before_hit(ctx: &mut crate::runtime_api::HitContext<'_>) -> Result<(), String> {
    if super::clock::accepted(ctx.request) {
        return Ok(());
    }
    if matches!(
        ctx.ability_id,
        "GA_Chaos071_Skill" | "GA_Chaos071_UltraSkill"
    ) {
        let reductions = if ctx.config.awakening.has_resonance_6() {
            // Source: DT_ChaosEffectFigure.json Chaos_StarExtra2ResDown = 0.10.
            std::collections::BTreeMap::from([(
                "ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA".into(),
                PARAMETERS.resonance_resistance_reduction,
            )])
        } else {
            Default::default()
        };
        ctx.refresh_status("chaos.warrant", PARAMETERS.warrant_duration_us, reductions)?;
    }
    Ok(())
}
