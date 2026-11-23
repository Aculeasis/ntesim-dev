pub(crate) mod awakenings;
pub(crate) mod callback;
pub(crate) mod hold;
pub(crate) mod levels;
mod parameters;
pub(crate) mod plunge;
pub(crate) mod preview;
pub(crate) mod riposte;
pub(crate) mod skill;

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Mint019_Melee" => Some("GE_Player_Mint_Melee1_Damage_New"),
        "GA_Mint019_Skill" => Some("GE_Player_Mint_Skill1_Damage_New"),
        "GA_Mint019_UltraSkill" => Some("GE_Player_Mint_UtraSkill1_Damage_New"),
        _ => None,
    }
}

pub(crate) fn buff_duration(
    awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    (awakening.has_effect(5) && ability == "GA_Mint019_Skill")
        .then_some(parameters::PARAMETERS.skill_crit_duration_us)
}
pub(crate) mod clock;

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 3903489,
        montage: "Mint_UltraSkill",
    };
