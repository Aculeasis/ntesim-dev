pub(crate) mod awakenings;
pub(crate) mod pact;
mod parameters;
pub(crate) mod preview;
pub(crate) mod resource;
pub(crate) mod skill;

pub(crate) const SUPPORT: crate::runtime_api::SinglePacketSupport =
    crate::runtime_api::SinglePacketSupport::new(
        "GA_Kuhara_QTE",
        "GE_Player_Kuhara_QTE_Damage",
        683_165,
        1_083_514,
        1_032_863,
        240_000,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Kuhara_Melee" => Some("GE_Player_Kuhara_Melee1_Damage"),
        "GA_Kuhara_Skill" => Some("GE_Player_Kuhara_Skill1_Damage"),
        "GA_Kuhara_UltraSkill" => Some("GE_Player_Kuhara_UltraSkill1_Damage"),
        "GA_Kuhara_QTE" => Some(SUPPORT.entry.damage_effect_id),
        _ => None,
    }
}

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    if ability == "GA_Kuhara_Skill" {
        // Base duration: 25s (25_000_000 us) per DT_KuharaEffectFigure.json Buff_Kuhara_Bud_Duration = 25.0
        Some(parameters::PARAMETERS.pact_duration_us)
    } else {
        None
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 5599441,
        montage: "Kuhara_UltraSkill",
    };
