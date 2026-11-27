pub(crate) mod awakenings;
pub(crate) mod callback;
mod control;
mod parameters;
pub(crate) mod passive;
pub(crate) mod preview;
pub(crate) mod riposte;
pub(crate) mod skill;
pub(crate) const BURST_ATK_MODEL: crate::HypothesisId = crate::HypothesisId::SakiriBurstAtkV1;
pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [(BURST_ATK_MODEL.into(), BURST_ATK_MODEL.description().into())].into()
}
pub(crate) mod support;

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Sagiri_Melee" => Some("GE_Player_Sagiri_Melee1_Damage"),
        "GA_Sagiri_Skill" => Some("GE_Player_Sagiri_Skill1_Damage"),
        "GA_Sagiri_UltraSkill" => Some("GE_Player_Sagiri_UltraSkill1_Damage"),
        "GA_Sagiri_QTE" => Some("GE_Player_Sagiri_QTE1_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(
    _awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    if ability == "GA_Sagiri_UltraSkill" {
        Some(parameters::PARAMETERS.team_atk_duration_us)
    } else {
        None
    }
}

// Explicit owner completion fallback; no reliable release notify is bound.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::ActionCompletion;
