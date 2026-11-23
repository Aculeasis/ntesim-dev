pub(crate) mod awakenings;
pub(crate) mod cadenza;
pub(crate) mod crescendo;
pub(crate) mod harmonics;
mod parameters;
pub(crate) mod plunge;
pub(crate) mod preview;
pub(crate) mod riposte;
pub(crate) mod skill;
pub(crate) mod staccato;

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    match ability {
        "GA_Mitsuki_Skill" => Some(parameters::PARAMETERS.cadenza_duration_us),
        _ => None,
    }
}

pub(crate) const SUPPORT: crate::runtime_api::SinglePacketSupport =
    crate::runtime_api::SinglePacketSupport::new(
        "GA_Mitsuki_QTE",
        "GE_Player_Mitsuki_QTE_Damage",
        805_977,
        1_300_967,
        1_083_416,
        320_000,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Mitsuki_Melee" => Some("GE_Player_Mitsuki_Melee1_Damage"),
        "GA_Mitsuki_AirAttack" => Some("GE_Player_Mitsuki_AirAttack_Damage"),
        "GA_Mitsuki_PerfectEvade" => Some("GE_Player_Mitsuki_PerfectEvadeAtk_Damage"),
        "GA_Mitsuki_PerfectEvadeAtkBullet" => Some("GE_Player_Mitsuki_PerfectAtkBullet_Damage"),
        "GA_Mitsuki_PerfectEvadeAtkBullet_Power" => {
            Some("GE_Player_Mitsuki_PerfectAtkBullet_Power_Damage")
        }
        "GA_Mitsuki_Skill" => Some("GE_Player_Mitsuki_Skill_Damage"),
        "GA_Mitsuki_UltraSkill" => Some("GE_Player_Mitsuki_UltraSkill_Damage"),
        "GA_Mitsuki070_Passive_1" => Some("GE_Reaction_4_new_1070_Damage"),
        "GA_Mitsuki_QTE" => Some(SUPPORT.entry.damage_effect_id),
        _ => None,
    }
}

// Explicit owner completion fallback; no reliable release notify is bound.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::ActionCompletion;
