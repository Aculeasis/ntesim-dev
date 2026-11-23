pub(crate) mod awakenings;
pub(crate) mod cannon;
pub(crate) mod clock;
pub(crate) mod hootie;
mod owned_packet;
pub(crate) mod preview;
pub(crate) mod resource;
pub(crate) mod secondary;
pub(crate) mod skill;
mod spawn;
pub(crate) mod stats;

pub(crate) const BURST_ENERGY_COST: u64 = 120;

pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [
        clock::MODEL,
        secondary::MODEL,
        cannon::MODEL,
        hootie::HYPOTHESIS,
        stats::MODEL,
    ]
    .into_iter()
    .map(|id| (id.into(), id.description().into()))
    .collect()
}

pub(crate) fn buff_duration(_awakening: u32, ability: &str) -> Option<u64> {
    if ability == "GA_Haniel_UltraSkill" {
        Some(10_000_000)
    } else {
        None
    }
}

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Haniel_QTE" => Some("GE_Player_Haniel_QTE_Damage"),
        "GA_Haniel_Melee" => Some("GE_Player_Haniel_Melee1_Damage"),
        "GA_Haniel_Skill" => Some("GE_Player_Haniel_Skill1_Damage"),
        "GA_Haniel_UltraSkill" => Some("GE_Player_Haniel_Ultraskill1_Damage"),
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 4237614,
        montage: "char_Haniel_skill_ultraskill_Montage",
    };
