pub(crate) mod awakenings;
pub(crate) mod entry_state;
mod parameters;
pub(crate) mod preview;
pub(crate) mod skill;

pub(crate) const SUPPORT: crate::runtime_api::SinglePacketSupport =
    crate::runtime_api::SinglePacketSupport::new(
        "GA_Skia_QTE",
        "GE_Player_Skia_QTE1_Damage",
        100_000,
        600_000,
        600_000,
        0,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Skia_Melee" => Some("GE_Player_Skia_Melee1_Damage"),
        "GA_Skia_Skill" => Some("GE_Player_Skia_Skill1_Damage"),
        "GA_Skia_UltraSkill" => Some("GE_Player_Skia_UltraSkill1_Damage"),
        "GA_Skia_QTE" => Some(SUPPORT.entry.damage_effect_id),
        _ => None,
    }
}

pub(crate) fn buff_duration(
    awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    if ability == "GA_Skia_Skill" && awakening.has_effect(2) {
        Some(parameters::PARAMETERS.a2_duration_us)
    } else {
        None
    }
}

// Explicit owner completion fallback; no reliable release notify is bound.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::ActionCompletion;
