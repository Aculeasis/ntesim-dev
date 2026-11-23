pub(crate) mod air;
pub(crate) mod awakenings;
pub(crate) mod branch;
pub(crate) mod burst;
pub(crate) mod clock;
pub(crate) mod combat;
pub(crate) mod cyclone;
pub(crate) mod gaze;
pub(crate) mod held;
mod parameters;
pub(crate) mod power;
pub(crate) mod preview;
pub(crate) mod remora;
pub(crate) mod secondary;
pub(crate) mod skill;

pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [
        clock::MODEL,
        power::MODEL,
        gaze::MODEL,
        held::MODEL,
        burst::MODEL,
        burst::HYBRID,
        combat::MODEL,
        cyclone::MODEL,
        secondary::MODEL,
        remora::MODEL,
    ]
    .into_iter()
    .map(|id| (id.into(), id.description().into()))
    .collect()
}

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    match ability {
        "GA_Hathor_UltraSkill" => Some(parameters::PARAMETERS.emergency_duration_us),
        _ => None,
    }
}

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Hathor_Melee" => Some("GE_Player_Hathor_Melee1_Damage"),
        "GA_Hathor_Skill" => Some("GE_Player_Hathor_Skill1_Damage"),
        "GA_Hathor_UltraSkill" => Some("GE_Player_Hathor_UltraSkill1_Damage"),
        "GA_Hathor_QTE" => Some(secondary::QTE),
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 4518597,
        montage: "Hathor_UltraSkill",
    };
