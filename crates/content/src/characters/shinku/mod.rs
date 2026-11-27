pub(crate) mod awakenings;
pub(crate) mod events;
pub(crate) mod parameters;
pub(crate) mod passive;
pub(crate) mod preview;
pub(crate) mod skill;

use crate::simulation::ExactFraction;
use ntesim_core::kernel::state::{ExactPool, ExactPoolSummary};
use std::collections::BTreeMap;

pub(crate) const SUPPORT: crate::runtime_api::SinglePacketSupport =
    crate::runtime_api::SinglePacketSupport::new(
        "GA_Shinku_QTE",
        "GE_Player_Shinku_QTE_Damage",
        100_000,
        600_000,
        600_000,
        0,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Shinku_Melee" => Some("GE_Player_Shinku_Melee1_1_Damage"),
        "GA_Shinku_Skill" => Some("GE_Player_Shinku_Skill1_1_Damage"),
        "GA_Shinku_UltraSkill" => Some("GE_Player_Shinku_UltraSkill1_Damage"),
        "GA_Shinku_QTE" => Some(SUPPORT.entry.damage_effect_id),
        _ => None,
    }
}

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    match ability {
        "GA_Shinku_UltraSkill" => Some(parameters::PARAMETERS.rage_duration_us),
        _ => None,
    }
}

pub(crate) fn initial_fading_reason() -> Result<ExactPool, String> {
    let capacity = ExactFraction::new(parameters::PARAMETERS.fading_reason_max.into(), 1)?;
    ExactPool::new(capacity, ExactFraction::new(0, 1)?)
}

pub(crate) fn initialize(
    config: &crate::simulation::GameActorConfig,
    actor: &mut crate::simulation::state::ActorRuntime,
) -> Result<(), String> {
    let tier = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?["breakthrough"]
        .as_u64()
        .unwrap_or(0);
    let state = actor.character.shinku_mut()?;
    state.breakthrough =
        u8::try_from(tier).map_err(|_| "Shinku breakthrough tier exceeds the modeled range")?;
    state.fading_reason = Some(initial_fading_reason()?);
    Ok(())
}

pub(crate) fn resource_summaries(
    state: &awakenings::State,
) -> Result<BTreeMap<String, ExactPoolSummary>, String> {
    let mut summaries = BTreeMap::new();
    if let Some(pool) = &state.fading_reason {
        summaries.insert("shinku.fading_reason".into(), pool.summary()?);
    }
    Ok(summaries)
}

pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: parameters::PARAMETERS.burst_default_cinematic_us,
        montage: "Shinku_UltraSkill",
    };

pub(crate) const BOSS_CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: parameters::PARAMETERS.burst_boss_cinematic_us,
        montage: "Shinku_UltraSkill_Boss",
    };

pub(crate) const VERDICT_CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: parameters::PARAMETERS.verdict_cinematic_us,
        montage: "Shinku_UltraSkill_Rage",
    };

pub(crate) fn entry_cinematic_release(is_boss: bool) -> super::cinematics::Release {
    if is_boss {
        BOSS_CINEMATIC_RELEASE
    } else {
        CINEMATIC_RELEASE
    }
}
