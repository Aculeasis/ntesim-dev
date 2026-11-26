pub(crate) mod awakenings;
mod parameters;
pub(crate) mod preview;
pub(crate) mod skill;

pub(crate) const SUPPORT: crate::runtime_api::SinglePacketSupport =
    crate::runtime_api::SinglePacketSupport::new(
        "GA_Radio072_QTE",
        "GE_Player_Radio072_QTE_Damage",
        100_000,
        600_000,
        600_000,
        0,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Radio072_Melee" => Some("GE_Player_Radio072_Melee1_Damage"),
        "GA_Radio072_Skill" => Some("GE_Player_Radio072_Skill1_Damage"),
        "GA_Radio072_UltraSkill" => Some("GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage"),
        "GA_Radio072_QTE" => Some(SUPPORT.entry.damage_effect_id),
        "GA_Radio072_LTE" | "GA_Radio072_SubUltraSkill" => Some("GE_Player_Radio072_LTE_Damage"),
        "GA_Radio072_AirAttack" => Some("GE_Player_Radio072_AirAttack_Damage"),
        "GA_Radio072_PerfectEvade" => Some("GE_Player_Radio072_PerfectEvadeAttack_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(
    awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    if ability == "GA_Radio072_UltraSkill" {
        if awakening.has_resonance_6() {
            Some(parameters::PARAMETERS.resonance_field_duration_us)
        } else {
            Some(parameters::PARAMETERS.field_duration_us)
        }
    } else {
        None
    }
}

// Explicit owner completion fallback; no reliable release notify is bound.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::ActionCompletion;
