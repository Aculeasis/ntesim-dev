pub(crate) mod clock;
pub(crate) mod effects;
pub(crate) mod incoming;
pub(crate) mod preview;
pub(crate) mod secondary;
pub(crate) mod skill;
pub(crate) mod stats;

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Fadia_QTE" => Some("GE_Player_Fadia_QTE1_Damage"),
        "GA_Fadia_Melee" => Some("GE_Player_Fadia_Melee1_Damage"),
        "GA_Fadia_Skill" => Some("GE_Player_Fadia_Skill1_Damage"),
        "GA_Fadia_UltraSkill" => Some("GE_Player_Fadia_UltraSkill1_Damage"),
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 4417366,
        montage: "Fadia_UltraSkill",
    };
