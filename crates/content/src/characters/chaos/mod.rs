pub(crate) mod awakenings;
pub(crate) mod cancel;
pub(crate) mod clock;
pub(crate) mod events;
pub(crate) mod form;
mod grouped;
mod parameters;
pub(crate) mod preview;
pub(crate) mod remora;
pub(crate) mod resources;
pub(crate) mod secondary;
pub(crate) mod skill;
pub(crate) mod support;
pub(crate) mod verdict;
pub(crate) mod warrant;

pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [
        clock::MODEL,
        resources::MODEL,
        form::MODEL,
        verdict::MODEL,
        secondary::MODEL,
        support::MODEL,
        warrant::TRANSFER,
        remora::MODEL,
    ]
    .into_iter()
    .map(|id| (id.into(), id.description().into()))
    .collect()
}

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Chaos071_Melee" => Some("GE_Player_Chaos_Melee1_Damage"),
        "GA_Chaos071_Skill" => Some("GE_Player_Chaos_Skill1_Damage"),
        "GA_Chaos071_UltraSkill" => Some("GE_Player_Chaos_UltraSkill1_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    match ability {
        "GA_Chaos071_Skill" => Some(parameters::PARAMETERS.warrant_duration_us),
        "GA_Chaos071_UltraSkill" => Some(parameters::PARAMETERS.a3_duration_us),
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 4802493,
        montage: "char_Chaos_skill_ultraskill_Montage",
    };
