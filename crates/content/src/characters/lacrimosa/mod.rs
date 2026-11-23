pub(crate) mod awakenings;
pub(crate) mod bat_exit;
pub(crate) mod dot;
mod parameters;
pub(crate) mod preview;
pub(crate) mod skill;
pub(crate) mod sleep_in;
pub(crate) mod support;

pub(crate) const SUPPORT: crate::runtime_api::SinglePacketSupport =
    crate::runtime_api::SinglePacketSupport::new(
        "GA_Lacrimosa_QTE",
        "GE_Player_Lacrimosa_QTE1_Damage",
        100_000,
        600_000,
        600_000,
        0,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Lacrimosa_Melee" => Some("GE_Player_Lacrimosa_Melee1_Damage"),
        "GA_Lacrimosa_QTE" => Some(SUPPORT.entry.damage_effect_id),
        "GA_Lacrimosa_Skill" => Some("GE_Player_Lacrimosa_Skill1_Damage"),
        "GA_Lacrimosa_UltraSkill" => Some("GE_Player_Lacrimosa_UltraSkill1_Damage"),
        "GA_Lacrimosa_AirAttack" => Some("GE_Player_Lacrimosa_AirAttack_Damage"),
        "GA_Lacrimosa_ExtremEvadeAtk" => Some("GE_Player_Lacrimosa_PerfectEvadeAttack_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(
    awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    if ability == "GA_Lacrimosa_QTE" && awakening.has_effect(2) {
        Some(parameters::PARAMETERS.a2_duration_us)
    } else {
        None
    }
}

// Explicit owner completion fallback; no reliable release notify is bound.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::ActionCompletion;
