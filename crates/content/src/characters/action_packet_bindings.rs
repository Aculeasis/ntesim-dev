//! Source-bound relationships between an invoked action and its delivered packet.
//!
//! These declarations cover exact owner-child edges where the native damage
//! row names a different GameplayAbility than the initiating action.

use crate::HypothesisId;

#[derive(Clone, Copy, Debug)]
pub(crate) struct OwnerChildPacketBinding {
    pub(crate) packet_ga_name: &'static str,
    pub(crate) hypothesis_id: HypothesisId,
}

const NANALLY_A3_CHILD: OwnerChildPacketBinding = OwnerChildPacketBinding {
    packet_ga_name: "GA_Nanally_Melee",
    hypothesis_id: HypothesisId::OwnerPacketBindingCharacter1010SkillLv3V1,
};

const NANALLY_UNDERBOSS_CHILD: OwnerChildPacketBinding = OwnerChildPacketBinding {
    packet_ga_name: "GA_Nanally_UltraSkill",
    hypothesis_id: crate::characters::nanally::underboss::HYPOTHESIS_ID,
};

const NANALLY_AUTHORITY_CHILD: OwnerChildPacketBinding = OwnerChildPacketBinding {
    packet_ga_name: "GA_Nanally_Melee",
    hypothesis_id: crate::characters::nanally::authority::HYPOTHESIS_ID,
};

const CHIZ_RIPOSTE_CHILD: OwnerChildPacketBinding = OwnerChildPacketBinding {
    packet_ga_name: "GA_Chiichan073_PerfectEvade",
    hypothesis_id: super::chiz::secondary::MODEL,
};

const HOTORI_KATANA_CHILD: OwnerChildPacketBinding = OwnerChildPacketBinding {
    packet_ga_name: "GA_Jin_UltraSkill_Melee",
    hypothesis_id: HypothesisId::HotoriKatanaV1,
};

pub(crate) fn resolve(
    character_id: &str,
    ability_id: &str,
    damage_effect_id: &str,
) -> Option<OwnerChildPacketBinding> {
    if character_id == "character:1025"
        && ability_id == "GA_Hathor_Melee"
        && matches!(
            damage_effect_id,
            super::hathor::secondary::RIPOSTE | super::hathor::secondary::RIPOSTE2
        )
    {
        return Some(OwnerChildPacketBinding {
            packet_ga_name: "GA_Hathor_PerfectEvade",
            hypothesis_id: super::hathor::secondary::MODEL,
        });
    }
    if character_id == "character:1025"
        && ability_id == "GA_Hathor_Skill"
        && super::hathor::cyclone::packet(damage_effect_id)
    {
        return Some(OwnerChildPacketBinding {
            packet_ga_name: "GA_Hathor_Skill2",
            hypothesis_id: super::hathor::cyclone::MODEL,
        });
    }
    if character_id == "character:1052"
        && ability_id == "GA_Jin_Melee"
        && matches!(
            damage_effect_id,
            "GE_Player_Jin_UltraSkillMelee1_Damage"
                | "GE_Player_Jin_UltraSkillMelee2_Damage"
                | "GE_Player_Jin_UltraSkillMelee2_1_Damage"
                | "GE_Player_Jin_UltraSkillMelee3_Damage"
                | "GE_Player_Jin_UltraSkillMelee3_1_Damage"
                | "GE_Player_Jin_UltraSkillMelee4_Damage"
                | "GE_Player_Jin_UltraSkillMelee4_1_Damage"
                | "GE_Player_Jin_UltraSkillMelee4_2_Damage"
                | "GE_Player_Jin_UltraSkillMelee4_3_Damage"
                | "GE_Player_Jin_UltraSkillMelee5_Damage"
                | "GE_Player_Jin_UltraSkillMelee5_1_Damage"
                | "GE_Player_Jin_UltraSkillMelee6_Damage"
                | "GE_Player_Jin_UltraSkillMelee6Pas2_Damage"
        )
    {
        return Some(HOTORI_KATANA_CHILD);
    }
    if matches!(character_id, "character:1054" | "character:1021")
        || (character_id == "character:1036"
            && damage_effect_id == super::zankou::secondary::RIPOSTE)
    {
        return owner_secondary(character_id, ability_id, damage_effect_id);
    }
    if (character_id, ability_id, damage_effect_id)
        == (
            "character:1039",
            "GA_Fadia_Melee",
            super::fadia::secondary::RIPOSTE,
        )
    {
        return Some(OwnerChildPacketBinding {
            packet_ga_name: "GA_Fadia_ExtremEvadeAtk",
            hypothesis_id: super::fadia::effects::MODEL,
        });
    }
    if character_id == "character:1039"
        && ability_id == "GA_Fadia_Melee"
        && matches!(
            damage_effect_id,
            "GE_Player_Fadia_UltraSkillMelee1_Damage"
                | "GE_Player_Fadia_UltraSkillMelee2_Damage"
                | "GE_Player_Fadia_UltraSkillMelee3_Damage"
                | "GE_Player_Fadia_UltraSkillMelee4_Damage"
                | "GE_Player_Fadia_UltraSkillMelee5_Damage"
                | "GE_Player_Fadia_UltraSkillMelee6_Damage"
                | "GE_Player_Fadia_UltraSkillMelee7_Damage"
                | "GE_Player_Fadia_UltraSkillMelee8_Damage"
        )
    {
        return Some(OwnerChildPacketBinding {
            packet_ga_name: "GA_Fadia_UltraSkill",
            hypothesis_id: super::fadia::effects::MODEL,
        });
    }
    legacy(character_id, ability_id, damage_effect_id)
}

fn legacy(
    character_id: &str,
    ability_id: &str,
    damage_effect_id: &str,
) -> Option<OwnerChildPacketBinding> {
    match (character_id, ability_id, damage_effect_id) {
        (
            "character:1073",
            "GA_Chiichan073_Melee",
            "GE_Player_Chiichan073_PerfectEvadeAttack1_Damage"
            | "GE_Player_Chiichan073_PerfectEvadeAttack2_Damage",
        ) => Some(CHIZ_RIPOSTE_CHILD),
        ("character:1071", "GA_Chaos071_Melee", super::chaos::secondary::RIPOSTE) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Chaos071_PerfectEvade",
                hypothesis_id: super::chaos::secondary::MODEL,
            })
        }
        ("character:1071", "GA_Chaos_Passive_1", super::chaos::remora::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Chaos071_UltraSkill",
                hypothesis_id: super::chaos::remora::MODEL,
            })
        }
        ("character:1057", "GA_Akane_Melee", super::akane::secondary::RIPOSTE) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Akane_PerfectEvade",
                hypothesis_id: super::akane::secondary::MODEL,
            })
        }
        ("character:1090", "GA_Elyms_Melee", super::elyms::secondary::RIPOSTE) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Elyms_PerfectEvade",
                hypothesis_id: super::elyms::secondary::MODEL,
            })
        }
        ("character:1057", "GA_Akane_Passive_1", super::akane::passive::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Akane_Skill",
                hypothesis_id: super::akane::passive::MODEL,
            })
        }
        ("character:1010", "GA_Nanally_Melee", super::nanally::riposte::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Nanally_ExtremEvadeAtk",
                hypothesis_id: super::nanally::riposte::HYPOTHESIS_ID,
            })
        }
        ("character:1010", "GA_Nanally_Melee", super::nanally::riposte::CAT_PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Nanally_UltraSkill",
                hypothesis_id: super::nanally::riposte::HYPOTHESIS_ID,
            })
        }
        ("character:1019", "GA_Mint019_Melee", super::mint::riposte::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Mint_ExtremEvadeAtk",
                hypothesis_id: super::mint::riposte::HYPOTHESIS_ID,
            })
        }
        ("character:1003", "GA_Sagiri_Melee", super::sakiri::riposte::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Sagiri_ExtremEvadeAtk",
                hypothesis_id: super::sakiri::riposte::HYPOTHESIS_ID,
            })
        }
        ("character:1075", "GA_Oneiroi_Melee", super::iroi::riposte::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Oneiroi_PerfectEvade",
                hypothesis_id: super::iroi::riposte::HYPOTHESIS_ID,
            })
        }
        ("character:1033", "GA_Adler_Melee", super::adler::riposte::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Adler_ExtremEvadeAtk",
                hypothesis_id: super::adler::riposte::HYPOTHESIS_ID,
            })
        }
        ("character:1033", "GA_Adler_Melee", super::adler::plunge::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Adler_AirAttack",
                hypothesis_id: super::adler::plunge::HYPOTHESIS_ID,
            })
        }
        ("character:1070", "GA_Mitsuki_Melee", super::aurelia::riposte::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Mitsuki_PerfectEvade",
                hypothesis_id: super::aurelia::riposte::HYPOTHESIS_ID,
            })
        }
        ("character:1070", "GA_Mitsuki_Melee", super::aurelia::staccato::PERFECT_BULLET_PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Mitsuki_PerfectEvadeAtkBullet",
                hypothesis_id: super::aurelia::staccato::PERFECT_DODGE_HYPOTHESIS_ID,
            })
        }
        (
            "character:1070",
            "GA_Mitsuki_Melee",
            super::aurelia::staccato::PERFECT_BULLET_POWER_PACKET,
        ) => Some(OwnerChildPacketBinding {
            packet_ga_name: "GA_Mitsuki_PerfectEvadeAtkBullet_Power",
            hypothesis_id: super::aurelia::staccato::PERFECT_DODGE_HYPOTHESIS_ID,
        }),
        ("character:1070", "GA_Mitsuki_Skill", super::aurelia::harmonics::PACKET)
        | ("character:1070", "GA_Mitsuki_Melee", super::aurelia::harmonics::PACKET) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Mitsuki070_Passive_1",
                hypothesis_id: super::aurelia::harmonics::HYPOTHESIS_ID,
            })
        }
        (
            "character:1055",
            "GA_Kuhara_Skill" | "GA_Kuhara_UltraSkill",
            "GE_Player_Kuhara_Seed_Damage"
            | "GE_Player_Kuhara_BudEnd_Damage"
            | "GE_Player_Kuhara_BudBoom_Damage",
        ) => Some(OwnerChildPacketBinding {
            packet_ga_name: "GA_Kuhara_Melee",
            hypothesis_id: crate::characters::jiuyuan::pact::HYPOTHESIS_ID,
        }),
        ("character:1010", "GA_Nanally_QTE", "GE_Player_Nanally_Cat_QTE_Damage") => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Nanally_UltraSkill",
                hypothesis_id: crate::characters::nanally::support::HYPOTHESIS_ID,
            })
        }
        ("character:1036", "GA_Zankou_Skill", "GE_Player_Zankou_DotDamage") => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Zankou_Melee",
                hypothesis_id: super::zankou::forms::MODEL,
            })
        }
        ("character:1010", "GA_Nanally_Skill", "GE_Nanally010_Lv3_Damage") => {
            Some(NANALLY_A3_CHILD)
        }
        (
            "character:1010",
            "GA_Nanally_UltraSkill" | "GA_Nanally_QTE",
            "GE_Nanally010_Lv3_Damage",
        ) => Some(NANALLY_AUTHORITY_CHILD),
        ("character:1010", "GA_Nanally_Melee", packet)
            if crate::characters::nanally::underboss::is_melee_child(packet) =>
        {
            Some(NANALLY_UNDERBOSS_CHILD)
        }
        ("character:1010", "GA_Nanally_Skill", "GE_Player_Nanally_Cat_Skill_Damage") => {
            Some(NANALLY_UNDERBOSS_CHILD)
        }
        ("character:1023", "GA_Cang_Melee", super::baicang::riposte::PACKET_ID) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Cang_PerfectEvade",
                hypothesis_id: super::baicang::riposte::HYPOTHESIS_ID,
            })
        }
        ("character:1042", "GA_BlackBird_Melee", super::blackbird::secondary::RIPOSTE) => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_BlackBird_PerfectEvade",
                hypothesis_id: super::blackbird::secondary::MODEL,
            })
        }
        (
            "character:1042",
            "GA_BlackBird_Evolve_Melee",
            super::blackbird::secondary::EVOLVE_RIPOSTE
            | super::blackbird::secondary::EVOLVE_RIPOSTE_EXTRA,
        ) => Some(OwnerChildPacketBinding {
            packet_ga_name: "GA_BlackBird_PerfectEvade",
            hypothesis_id: super::blackbird::secondary::EVOLVED_MODEL,
        }),
        (
            "character:1042",
            "GA_BlackBird_Evolve_QTE",
            super::blackbird::support::EVOLVED_PACKET
            | super::blackbird::support::EVOLVED_EXTRA_PACKET,
        ) => Some(OwnerChildPacketBinding {
            packet_ga_name: "GA_BlackBird_QTE",
            hypothesis_id: super::blackbird::support::EVOLVED_MODEL,
        }),
        ("character:1023", "GA_Cang_Skill", "GE_Player_Cang_SkillA_Damage")
        | ("character:1023", "GA_Cang_Skill", "GE_Player_Cang_SkillA_Damage1")
        | ("character:1023", "GA_Cang_Skill", "GE_Player_Cang_SkillA_Damage2") => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Cang_SkillA",
                hypothesis_id: super::baicang::words::HYPOTHESIS_ID,
            })
        }
        ("character:1023", "GA_Cang_Skill", "GE_Player_Cang_SkillB_Damage") => {
            Some(OwnerChildPacketBinding {
                packet_ga_name: "GA_Cang_SkillB",
                hypothesis_id: super::baicang::words::HYPOTHESIS_ID,
            })
        }
        (
            "character:1023",
            super::baicang::fadia_assist::ABILITY,
            super::baicang::fadia_assist::PACKET,
        ) => Some(OwnerChildPacketBinding {
            packet_ga_name: "GA_Cang_Melee",
            hypothesis_id: super::baicang::fadia_assist::HYPOTHESIS_ID,
        }),
        _ => None,
    }
}

fn owner_secondary(
    character: &str,
    ability: &str,
    effect: &str,
) -> Option<OwnerChildPacketBinding> {
    let (packet_ga_name, hypothesis_id) = match (character, ability, effect) {
        ("character:1036", "GA_Zankou_Melee", super::zankou::secondary::RIPOSTE) => {
            ("GA_Zankou_PerfectEvade", super::zankou::forms::MODEL)
        }
        ("character:1054", "GA_Daffodill_QTE", super::daffodil::phantom::APPEAR_SUCCESS) => {
            ("GA_Daffodill_Melee", super::daffodil::phantom::MODEL)
        }
        ("character:1054", "GA_Daffodill_Melee", super::daffodil::secondary::RIPOSTE) => (
            "GA_Daffodill_PerfectEvade",
            super::daffodil::secondary::MODEL,
        ),
        ("character:1021", "GA_Edgar_Melee", super::edgar::secondary::RIPOSTE) => {
            ("GA_Edgar_PerfectEvade", super::edgar::clock::MODEL)
        }
        _ => return None,
    };
    Some(OwnerChildPacketBinding {
        packet_ga_name,
        hypothesis_id,
    })
}

#[allow(dead_code)]
pub(crate) fn known_hypotheses() -> std::collections::BTreeMap<&'static str, &'static str> {
    [
        NANALLY_A3_CHILD,
        NANALLY_UNDERBOSS_CHILD,
        NANALLY_AUTHORITY_CHILD,
    ]
    .into_iter()
    .map(|binding| {
        (
            binding.hypothesis_id.as_str(),
            binding.hypothesis_id.description(),
        )
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_bindings_are_exact_owner_action_packet_edges() {
        assert!(
            resolve(
                "character:1046",
                "GA_Female046_Skill",
                "GE_Player_Female051_Skill_Kill_Damage_lv1"
            )
            .is_none()
        );
        assert_eq!(
            resolve(
                "character:1010",
                "GA_Nanally_Skill",
                "GE_Nanally010_Lv3_Damage",
            )
            .unwrap()
            .packet_ga_name,
            "GA_Nanally_Melee"
        );
        assert!(
            resolve(
                "character:1046",
                "GA_Female046_UltraSkill",
                "GE_Player_Female051_Skill_Kill_Damage_lv1",
            )
            .is_none()
        );
    }
}
