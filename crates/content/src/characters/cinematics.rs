//! Owner bindings for the mode timer, independent of buff/summon lifetimes.
//! Notify progress is from the same retained montage extracts as owner timing.
//! A missing notify uses that owner's consumed action completion, explicitly.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Release {
    Notify {
        progress_us: u64,
        montage: &'static str,
    },
    ActionCompletion,
    TimeStop,
}

/// Existing bounded ordinary authoring recovery for owners without a sourced
/// montage profile. A zero-length caller research recovery cannot erase a pause.
pub(crate) const FALLBACK_ACTION_RECOVERY_US: u64 = 2_000_000;

pub(crate) fn binding(character: &str, ability: &str) -> Result<Release, String> {
    Ok(match character {
        "character:1033" => super::adler::CINEMATIC_RELEASE,
        "character:1070" => super::aurelia::CINEMATIC_RELEASE,
        "character:1023" => super::baicang::CINEMATIC_RELEASE,
        "character:1042" if ability == "GA_BlackBird_Evolve_UltraSkill" => {
            super::blackbird::EVOLVED_CINEMATIC_RELEASE
        }
        "character:1042" => super::blackbird::CINEMATIC_RELEASE,
        "character:1071" => super::chaos::CINEMATIC_RELEASE,
        "character:1073" => super::chiz::CINEMATIC_RELEASE,
        "character:1054" => super::daffodil::CINEMATIC_RELEASE,
        "character:1021" => super::edgar::CINEMATIC_RELEASE,
        "character:1090" if ability == "GA_Elyms_UltraSkillB" => Release::Notify {
            progress_us: 5_499_266,
            montage: "Elyms_UltraSkill2",
        },
        "character:1090" => Release::Notify {
            progress_us: 5_499_266,
            montage: "Elyms_UltraSkill",
        },
        "character:1039" => super::fadia::CINEMATIC_RELEASE,
        "character:1020" => super::haniel::CINEMATIC_RELEASE,
        "character:1025" => super::hathor::CINEMATIC_RELEASE,
        "character:1052" if ability == "GA_Jin_UltraSkill" => super::hotori::CINEMATIC_RELEASE,
        // Explicit research metadata may label a synthetic input Ultimate. It
        // has no native ModeEntry lifecycle, so consume finite completion.
        "character:1052" => Release::ActionCompletion,
        "character:1075" => super::iroi::CINEMATIC_RELEASE,
        "character:1055" => super::jiuyuan::CINEMATIC_RELEASE,
        "character:1057" => super::akane::CINEMATIC_RELEASE,
        "character:1004" => super::lacrimosa::CINEMATIC_RELEASE,
        "character:1072" => super::linko::CINEMATIC_RELEASE,
        "character:1019" => super::mint::CINEMATIC_RELEASE,
        "character:1010" => super::nanally::CINEMATIC_RELEASE,
        "character:1003" => super::sakiri::CINEMATIC_RELEASE,
        "character:1076" => super::shinku::CINEMATIC_RELEASE,
        "character:1008" => super::skia::CINEMATIC_RELEASE,
        "character:1036" => super::zankou::CINEMATIC_RELEASE,
        "character:1046" => super::zero::CINEMATIC_RELEASE,
        _ => {
            return Err(format!(
                "missing Ultimate cinematic binding for {character}"
            ));
        }
    })
}

pub(crate) fn action_binding(
    character: &str,
    primary_slot: Option<crate::simulation::metadata::PrimaryActionSlot>,
    ability: &str,
    damage_effect_id: &str,
) -> Result<Option<Release>, String> {
    if character == "character:1090" {
        return Ok(match damage_effect_id {
            "GE_Player_Elyms_Skill2_Damage" => Some(Release::Notify {
                progress_us: 1_719_655,
                montage: "Elyms_Skill2",
            }),
            "GE_Player_Elyms_Skill3_Damage" => Some(Release::Notify {
                progress_us: 1_719_655,
                montage: "Elyms_Skill3",
            }),
            _ if primary_slot == Some(crate::simulation::metadata::PrimaryActionSlot::Ultimate) => {
                if ability == "GA_Elyms_UltraSkillB"
                    || damage_effect_id.starts_with("GE_Player_Elyms_UltraSkillB")
                {
                    Some(Release::Notify {
                        progress_us: 5_499_266,
                        montage: "Elyms_UltraSkill2",
                    })
                } else {
                    Some(Release::Notify {
                        progress_us: 5_499_266,
                        montage: "Elyms_UltraSkill",
                    })
                }
            }
            _ => None,
        });
    }
    let is_ultimate_cinematic = primary_slot
        == Some(crate::simulation::metadata::PrimaryActionSlot::Ultimate)
        && damage_effect_id != "GE_Player_Shinku_UltraSkillPre_Rage_Damage"
        && damage_effect_id != "GE_Player_Akane_UltraSkill2_Damage";
    if is_ultimate_cinematic {
        binding(character, ability).map(Some)
    } else {
        Ok(None)
    }
}
