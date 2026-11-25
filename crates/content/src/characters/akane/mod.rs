pub(crate) mod awakenings;
pub(crate) mod clock;
pub(crate) mod events;
pub(crate) mod parameters;
pub(crate) mod passive;
pub(crate) mod preview;
pub(crate) mod secondary;
pub(crate) mod skill;

pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [
        clock::MODEL,
        skill::CHAIN_HYPOTHESIS_ID,
        crate::HypothesisId::AkaneOrdinaryHitEnergyV1,
        skill::STRING_ENERGY_HYPOTHESIS_ID,
        skill::RHYTHM_WAVE_HYPOTHESIS_ID,
        skill::SONIC_BOOM_HYPOTHESIS_ID,
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
        "GA_Akane_QTE",
        "GE_Player_Akane_QTE_Damage",
        704_643,
        1_424_255,
        875_734,
        0,
    );

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Akane_Melee" => Some("GE_Player_Akane_Melee1_Damage"),
        "GA_Akane_Skill" => Some("GE_Player_Akane_Skill_Damage"),
        "GA_Akane_UltraSkill" => Some("GE_Player_Akane_UltraSkill1_Damage"),
        "GA_Akane_QTE" => Some("GE_Player_Akane_QTE_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    match ability {
        "GA_Akane_Skill" => Some(parameters::PARAMETERS.mode_duration_us),
        "GA_Akane_UltraSkill" => Some(parameters::PARAMETERS.anthem_duration_us),
        _ => None,
    }
}

// Event.Montage.Player.Common.EndAbilitySkill; char_Akane_skill_ultraskill_Montage [4.012833 .. 4.112832].
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 4_012_833,
        montage: "char_Akane_skill_ultraskill_Montage",
    };
