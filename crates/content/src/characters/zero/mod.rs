pub(crate) mod awakenings;
pub(crate) mod callback;
mod parameters;
pub(crate) mod preview;
pub(crate) mod skill;
pub(crate) mod support;

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Female046_Melee" => Some("GE_Player_Female046_Melee1_Damage"),
        "GA_Female046_Skill" => Some("GE_Player_Female046_Skill1_Damage"),
        "GA_Female046_UltraSkill" => Some("GE_Player_Female046_UltraSkill1_Damage"),
        "GA_Female046_QTE" => Some("GE_Player_Female046_QTE1_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(
    awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    if awakening.has_effect(5) && ability == "GA_Female046_QTE" {
        Some(parameters::PARAMETERS.support_atk_duration_us)
    } else {
        None
    }
}

// Explicit owner completion fallback; no reliable release notify is bound.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::ActionCompletion;
