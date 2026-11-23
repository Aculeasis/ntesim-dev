pub(crate) mod awakenings;
pub(crate) mod callback;
pub(crate) mod plunge;
pub(crate) mod preview;
pub(crate) mod righteous_heart;
pub(crate) mod riposte;
pub(crate) mod skill;
pub(crate) mod temperance;
mod values;

pub(crate) const SUPPORT: crate::runtime_api::SinglePacketSupport =
    crate::runtime_api::SinglePacketSupport::new(
        "GA_Adler_QTE",
        "GE_Player_Adler_QTE_Damage",
        683_333,
        1_040_435,
        1_003_719,
        320_000,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Adler_Melee" => Some("GE_Player_Adler_Melee1_Damage"),
        "GA_Adler_Skill" => Some("GE_Player_Adler_Skill2_Damage"),
        "GA_Adler_UltraSkill" => Some("GE_Player_Adler_UltraSkill_Damage"),
        "GA_Adler_QTE" => Some(SUPPORT.entry.damage_effect_id),
        "GA_Adler_AirAttack" => Some("GE_Player_Adler_AirAttack_Damage"),
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 3025935,
        montage: "Adler_UltralSkill",
    };
