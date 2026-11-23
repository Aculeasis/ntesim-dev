pub(crate) mod awakenings;
pub(crate) mod katana;
mod parameters;
pub(crate) mod preview;
pub(crate) mod replay;
pub(crate) mod skill;
pub(crate) mod timepiece;
pub mod timing;

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
    actor.character.hotori_mut()?.timepiece.breakthrough =
        u8::try_from(tier).map_err(|_| "Hotori breakthrough tier exceeds the modeled range")?;
    actor.character.hotori_mut()?.timepiece.pool =
        timepiece::initial_pool(config.initial_resources.timepiece)?;
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &crate::runtime_api::DamageContext<'_>,
    mut bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    if let Some(family) = katana::family_crit_damage(ctx)? {
        bonus = bonus.add_crit_damage(family)?;
    }
    awakenings::damage_bonus(ctx, bonus)
}

pub(crate) const SUPPORT: crate::runtime_api::SinglePacketSupport =
    crate::runtime_api::SinglePacketSupport::new(
        "GA_Jin_QTE",
        "GE_Player_Jin_QTE_Damage",
        750_000,
        1_320_046,
        1_283_615,
        320_000,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Jin_Melee" => Some("GE_Player_Jin_Melee1_Damage"),
        "GA_Jin_Skill" => Some("GE_Player_Jin_Skill1_Damage"),
        "GA_Jin_UltraSkill" => Some("GE_Player_Jin_UltraSkill1_Damage"),
        "GA_Jin_QTE" => Some(SUPPORT.entry.damage_effect_id),
        _ => None,
    }
}

// EnterTimeStop hands setup to the actual generation-owned active mode.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::TimeStop;
