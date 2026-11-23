pub(crate) mod authority;
pub(crate) mod awakenings;
pub(crate) mod blossom;
pub(crate) mod callback;
pub(crate) mod clock;
pub(crate) mod combat;
pub(crate) mod fair_duel;
mod parameters;
pub(crate) mod plunge;
pub(crate) mod preview;
pub(crate) mod removal;
pub(crate) mod riposte;
pub(crate) mod skill;
pub(crate) mod support;
pub(crate) mod underboss;

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Nanally_QTE" => Some(support::ENTRY.damage_effect_id),
        "GA_Nanally_Melee" => Some("GE_Player_Nanally_Melee1_Damage"),
        "GA_Nanally_Skill" => Some("GE_Player_Nanally_Skill1_Damage"),
        "GA_Nanally_UltraSkill" => Some("GE_Player_Nanally_UltraSkill1_Damage"),
        "GA_Nanally_ExtremEvadeAtk" => Some("GE_Player_Nanally_PerfectEvadeAttack_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(
    awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    match ability {
        "GA_Nanally_Skill" => {
            if awakening.has_effect(6) {
                Some(parameters::PARAMETERS.a6_authority_duration_us)
            } else {
                Some(parameters::PARAMETERS.authority_duration_us)
            }
        }
        "GA_Nanally_UltraSkill" => {
            if awakening.has_effect(2) {
                Some(parameters::PARAMETERS.a2_underboss_duration_us)
            } else {
                Some(parameters::PARAMETERS.underboss_duration_us)
            }
        }
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 3584608,
        montage: "Nanally_UltralSkill",
    };
