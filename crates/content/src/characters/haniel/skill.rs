use crate::HypothesisId;
use crate::runtime_api::CastContext;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) clock: Option<Box<super::clock::Clock>>,
    pub(crate) membership: Option<super::stats::Membership>,
    pub(crate) out_of_combat: bool,
    pub(crate) riposte_ready_us: Option<u64>,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) hootie_until_us: Option<u64>,
    pub(crate) paranormal_ace_until_us: Option<u64>,
    pub(crate) hootie_atk_add: Option<crate::simulation::ExactFraction>,
    pub(crate) hootie_source: Option<Box<super::owned_packet::Source>>,
    pub(crate) ace_source: Option<Box<super::owned_packet::Source>>,
    pub(crate) token: Option<Box<super::hootie::Token>>,
    pub(crate) harmony: u8,
    pub(crate) harmony_ready_us: u64,
    pub(crate) plot: u8,
    pub(crate) plot_cap: u8,
}

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::HanielNormalChainV1;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::HanielSkillBurstPacketsV1;

pub(crate) const TEAM_ATK_CLOCK_HYPOTHESIS_ID: HypothesisId = HypothesisId::HanielHarmonyEnsembleV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::HanielAoeAllInRangeDummyV1;

// The consumed Global Haniel_Skill_AtkAdd and Haniel_UltraSkill_AtkAdd
// rows are identical at all 13 authored talent coordinates: .10, .11, ... .22.
// Keep only the exact ratio and supported domain, without source curve metadata.
fn team_atk_ratio(coordinate: u32) -> Result<crate::simulation::ExactFraction, String> {
    if !(1..=13).contains(&coordinate) {
        return Err("Haniel team ATK talent coordinate must be in 1..=13".into());
    }
    crate::simulation::ExactFraction::new(u128::from(coordinate) + 9, 100)
}

pub(super) fn flat_team_atk(
    ctx: &CastContext<'_>,
) -> Result<crate::simulation::ExactFraction, String> {
    let array_index = ctx
        .native_array_index
        .ok_or("Haniel ATK grant requires an admitted native ability coordinate")?;
    let coordinate = array_index
        .checked_add(1)
        .ok_or("Haniel talent coordinate overflow")?
        .try_into()
        .map_err(|_| "Haniel talent coordinate exceeds u32")?;
    crate::simulation::ExactFraction::new(ctx.config.static_stats.base_atk.into(), 1)?
        .mul(team_atk_ratio(coordinate)?)
}

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err(
            "Haniel normal chain requires the modeled first packet as its admitted input".into(),
        );
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.haniel_mut()?;
    state.normal_stage = crate::characters::normal::after_gap(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let stage = state.normal_stage;
    state.normal_stage = (stage + 1) % 5;
    state.normal_started_us = Some(ctx.time_us);
    let packet = [
        "GE_Player_Haniel_Melee1_Damage",
        "GE_Player_Haniel_Melee2_Damage",
        "GE_Player_Haniel_Melee3_Damage",
        "GE_Player_Haniel_Melee4_Damage",
        "GE_Player_Haniel_Melee5_Damage",
    ][stage];
    if stage != 0 {
        ctx.replace_native_primary_packet(packet)?;
    }
    for _ in 1..[1, 2, 1, 4, 1][stage] {
        ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
    }
    Ok(())
}

fn handle_skill(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err(
            "Haniel packet train requires the modeled first packet as its admitted primary".into(),
        );
    }
    ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
    ctx.schedule_native_child_packet(
        "GE_Player_Haniel_Skill1_Damage",
        ctx.target_index,
        ctx.hit_time_us,
    )?;

    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Haniel_Skill1_Damage",
                target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Haniel_Skill1_Damage",
                target_index,
                ctx.hit_time_us,
            )?;
        }
    }

    let grant = flat_team_atk(ctx)?;
    super::spawn::schedule(ctx, grant)?;
    Ok(())
}

fn handle_ultra_skill(ctx: &mut CastContext<'_>) -> Result<(), String> {
    super::hootie::capture_burst(ctx)?;
    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Haniel_Ultraskill1_Damage",
                target_index,
                ctx.hit_time_us,
            )?;
        }
    }

    activate_ace(ctx)
}

pub(super) fn activate_ace(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(TEAM_ATK_CLOCK_HYPOTHESIS_ID)?;
    let duration_us = 10_000_000;
    let new_ace_until = ctx
        .time_us
        .checked_add(duration_us)
        .ok_or("Haniel Paranormal Ace deadline overflow")?;
    let (hootie_until_current, ace_until_current, hootie_atk_add) = {
        let state = ctx.actor.character.haniel_mut()?;
        (
            state.hootie_until_us,
            state.paranormal_ace_until_us,
            state.hootie_atk_add,
        )
    };
    let hootie_remaining = hootie_until_current
        .filter(|deadline| *deadline > ctx.time_us)
        .map(|deadline| {
            let old_ace_until = ace_until_current.filter(|ace| *ace > ctx.time_us);
            deadline.saturating_sub(old_ace_until.unwrap_or(ctx.time_us))
        });
    if let Some(remaining) = hootie_remaining {
        let hootie_until = new_ace_until
            .checked_add(remaining)
            .ok_or("Haniel Hootie deadline overflow")?;
        ctx.actor.character.haniel_mut()?.hootie_until_us = Some(hootie_until);
        ctx.refresh_buff(
            "haniel.hootie_team_atk",
            ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
            hootie_until
                .checked_sub(ctx.time_us)
                .ok_or("Haniel Hootie deadline precedes cast")?,
            BTreeMap::from([(
                "atk_add".into(),
                hootie_atk_add.ok_or("Haniel Hootie grant missing")?,
            )]),
        )?;
    }
    let burst_grant = flat_team_atk(ctx)?;
    ctx.refresh_buff(
        "haniel.paranormal_ace_team_atk",
        ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
        duration_us,
        BTreeMap::from([("atk_add".into(), burst_grant)]),
    )?;
    ctx.actor.character.haniel_mut()?.paranormal_ace_until_us = Some(new_ace_until);
    let mut self_stats = BTreeMap::new();
    if ctx.config.awakening.has_effect(3) {
        self_stats.insert(
            "crit_rate_bonus".into(),
            crate::simulation::ExactFraction::new(1, 5)?,
        );
    }
    if ctx.config.awakening.has_effect(5) {
        self_stats.insert(
            "damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_PSYCHE".into(),
            crate::simulation::ExactFraction::new(3, 10)?,
        );
    }
    if !self_stats.is_empty() {
        ctx.refresh_buff(
            "haniel.paranormal_ace_self",
            ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
            duration_us,
            self_stats,
        )?;
    }
    if ctx.config.awakening.has_resonance_6() {
        ctx.refresh_buff(
            "haniel.paranormal_ace_team",
            ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
            duration_us,
            BTreeMap::from([(
                "crit_damage_bonus".into(),
                crate::simulation::ExactFraction::new(3, 25)?,
            )]),
        )?;
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1020" {
        return Ok(());
    }
    if super::clock::accepted(ctx.request) {
        return super::clock::cast(ctx);
    }

    if ability_id == "GA_Haniel_Melee" {
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.haniel_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if ability_id == "GA_Haniel_Skill" {
        handle_skill(ctx)?;
    }

    if ability_id == "GA_Haniel_UltraSkill" {
        handle_ultra_skill(ctx)?;
    }
    if ability_id == "GA_Haniel_UltraSkill" {
        super::hootie::schedule_expiry(ctx)?;
    }

    Ok(())
}

#[cfg(test)]
mod parameter_tests {
    use super::team_atk_ratio;
    use crate::simulation::ExactFraction;

    #[test]
    fn team_atk_has_exact_talent_endpoints_and_no_extrapolation() {
        for (level, numerator, denominator) in [(1, 1, 10), (8, 17, 100), (9, 9, 50), (13, 11, 50)]
        {
            assert_eq!(
                team_atk_ratio(level).unwrap(),
                ExactFraction::new(numerator, denominator).unwrap()
            );
        }
        for level in [0, 14, u32::MAX] {
            assert!(team_atk_ratio(level).is_err());
        }
    }
}
