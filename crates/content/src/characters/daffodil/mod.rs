pub(crate) mod awakenings;
pub(crate) mod break_damage;
pub(crate) mod broken;
pub(crate) mod clock;
pub(crate) mod discord;
pub(crate) mod held;
pub(crate) mod insight;
mod parameters;
pub(crate) mod phantom;
pub(crate) mod preview;
pub(crate) mod resonance;
pub(crate) mod secondary;
pub(crate) mod skill;

pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [
        clock::MODEL,
        clock::cancel::MODEL,
        secondary::MODEL,
        phantom::MODEL,
        held::MODEL,
    ]
    .into_iter()
    .map(|id| (id.into(), id.description().into()))
    .collect()
}

pub(crate) fn buff_duration(
    _awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    if ability == "GA_Daffodill_UltraSkill" {
        Some(parameters::PARAMETERS.self_insight_duration_us)
    } else {
        None
    }
}

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Daffodill_Melee" => Some("GE_Player_Daffodill_Melee1_Damage"),
        "GA_Daffodill_Skill" => Some("GE_Player_Daffodill_Skill1_Damage"),
        "GA_Daffodill_UltraSkill" => Some("GE_Player_Daffodill_UltraSkill1_Damage"),
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 5099006,
        montage: "Daffodill_UltraSkill",
    };
