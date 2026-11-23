pub(crate) mod awakenings;
pub(crate) mod branch;
pub(crate) mod bud_copy;
pub(crate) mod callback;
pub(crate) mod clock;
pub(crate) mod dream;
mod parameters;
pub(crate) mod plunge;
pub(crate) mod preview;
pub(crate) mod regression;
pub(crate) mod resource;
pub(crate) mod riposte;
pub(crate) mod skill;

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Oneiroi_Melee" => Some("GE_Player_Oneiroi_Melee1_Damage"),
        "GA_Oneiroi_Skill" => Some("GE_Player_Oneiroi_Skill_Gun_Damage"),
        "GA_Oneiroi_UltraSkill" => Some("GE_Player_Oneiroi_UltraSkill_Damage"),
        "GA_Oneiroi_QTE" => Some("GE_Player_Oneiroi_QTE_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    (ability == "GA_Oneiroi_Skill").then_some(20000000)
}

pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [
        clock::MODEL,
        branch::HYPOTHESIS_ID,
        dream::HYPOTHESIS_ID,
        riposte::HYPOTHESIS_ID,
        plunge::HYPOTHESIS_ID,
        regression::HYPOTHESIS_ID,
    ]
    .into_iter()
    .map(|id| (id.into(), id.description().into()))
    .collect()
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 4004747,
        montage: "Oneiroi_UltraSkill",
    };
