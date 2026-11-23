pub(crate) mod awakenings;
pub(crate) mod bird;
pub(crate) mod branches;
mod buffs;
pub(crate) mod cancel;
pub(crate) mod clock;
pub(crate) mod combat;
pub(crate) mod events;
pub(crate) mod form;
pub(crate) mod malice;
pub(crate) mod nova;
mod parameters;
pub(crate) mod preview;
pub(crate) mod secondary;
pub(crate) mod skill;
pub(crate) mod source;
pub(crate) mod stain;
pub(crate) mod support;

pub(crate) fn buff_duration(
    awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    match ability {
        "GA_BlackBird_UltraSkill" | "GA_BlackBird_Evolve_UltraSkill" => {
            if awakening.has_effect(4) {
                Some(parameters::PARAMETERS.a4_team_duration_us)
            } else {
                Some(parameters::PARAMETERS.team_duration_us)
            }
        }
        "GA_BlackBird_Skill" | "GA_BlackBird_Evolve_Skill" => {
            Some(parameters::PARAMETERS.team_duration_us)
        }
        _ => None,
    }
}

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_BlackBird_Melee" => Some("GE_Player_BlackBird_Melee1_Damage"),
        "GA_BlackBird_Skill" => Some("GE_Player_BlackBird_Skill_Damage"),
        "GA_BlackBird_UltraSkill" => Some("GE_Player_BlackBird_UltraSkill_Damage"),
        "GA_BlackBird_Evolve_Melee" => Some("GE_Player_BlackBird_Evolve_Melee1_Damage"),
        "GA_BlackBird_Evolve_Skill" => Some("GE_Player_BlackBird_Evolve_Skill_Damage"),
        "GA_BlackBird_Evolve_UltraSkill" => Some("GE_Player_BlackBird_Evolve_UltraSkill_Damage"),
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::ActionCompletion;
pub(crate) const EVOLVED_CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 5021767,
        montage: "Blackbird_Evolve_UltraSkill",
    };
