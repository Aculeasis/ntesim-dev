pub(crate) mod awakenings;
pub(crate) mod burst;
pub(crate) mod clock;
pub(crate) mod combat_passives;
mod dot_kind;
pub(crate) mod forms;
pub(crate) mod modifiers;
pub(crate) mod normal;
mod parameters;
pub(crate) mod passives;
pub(crate) mod preview;
mod profile;
pub(crate) mod scorch;
pub(crate) mod secondary;
pub(crate) mod skill;
pub(crate) mod spread;
pub(crate) mod state;
pub(crate) mod vile_ash;
pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [forms::MODEL, modifiers::MODEL, combat_passives::MODEL]
        .into_iter()
        .map(|id| (id.into(), id.description().into()))
        .collect()
}

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    match ability {
        "GA_Zankou_Skill" => Some(parameters::PARAMETERS.metadata_duration_us),
        "GA_Zankou_UltraSkill" => Some(parameters::PARAMETERS.metadata_duration_us),
        _ => None,
    }
}

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Zankou_Melee" => Some("GE_Player_Zankou_Melee1_Damage"),
        "GA_Zankou_Skill" => Some("GE_Player_Zankou_Skill2_Damage"),
        "GA_Zankou_UltraSkill" => Some("GE_Player_Zankou_UltraSkill1_Damage"),
        "GA_Zankou_QTE" => Some("GE_Player_Zankou_QTE_Damage"),
        _ => None,
    }
}

// Explicit owner completion fallback; no reliable release notify is bound.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::ActionCompletion;
