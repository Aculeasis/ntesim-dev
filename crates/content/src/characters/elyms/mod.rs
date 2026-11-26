pub(crate) mod awakenings;
pub(crate) mod clock;
pub(crate) mod events;
pub(crate) mod on_the_line;
pub(crate) mod parameters;
pub(crate) mod passive;
pub(crate) mod preview;
pub(crate) mod secondary;
pub(crate) mod skill;

pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [
        clock::MODEL,
        skill::CHAIN_HYPOTHESIS_ID,
        crate::HypothesisId::ElymsOrdinaryHitEnergyV1,
        skill::SKILL_REDIRECT_HYPOTHESIS_ID,
        on_the_line::MODEL,
        passive::MODEL,
        awakenings::MODEL,
        skill::ALL_IN_RANGE_HYPOTHESIS_ID,
        secondary::MODEL,
    ]
    .into_iter()
    .map(|id| (id.into(), id.description().into()))
    .collect()
}

pub(crate) const SUPPORT: crate::runtime_api::SinglePacketSupport =
    crate::runtime_api::SinglePacketSupport::new(
        "GA_Elyms_QTE",
        "GE_Player_Elyms_QTE_Damage",
        clock::profile::QTE_HIT,
        clock::profile::QTE_RECOVERY,
        clock::profile::QTE_SWITCH_LOCK,
        0,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Elyms_Melee" => Some("GE_Player_Elyms_Melee1_Damage"),
        "GA_Elyms_Skill" => Some("GE_Player_Elyms_Skill1_Damage"),
        "GA_Elyms_UltraSkill" => Some("GE_Player_Elyms_UltraSkillA1_Damage"),
        "GA_Elyms_UltraSkillB" => Some("GE_Player_Elyms_UltraSkillB1_Damage"),
        "GA_Elyms_QTE" => Some("GE_Player_Elyms_QTE_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    match ability {
        "GA_Elyms_UltraSkill" | "GA_Elyms_UltraSkillB" => {
            Some(parameters::PARAMETERS.q_atk_buff_duration_us)
        }
        _ => None,
    }
}
