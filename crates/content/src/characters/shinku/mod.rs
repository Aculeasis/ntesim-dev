pub(crate) mod awakenings;
mod parameters;
pub(crate) mod preview;
pub(crate) mod skill;

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
        "GA_Shinku_Skill" | "GA_Shinku_UltraSkill" => Some(parameters::PARAMETERS.rage_duration_us),
        _ => None,
    }
}

// Explicit owner completion fallback; no reliable release notify is bound.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::ActionCompletion;
