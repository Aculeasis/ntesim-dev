//! Explicit content-owner roles for released actions whose frozen effect-config tags are absent.
//!
//! Every entry is keyed by both native character and exact ability identity. These declarations
//! are reviewed model hypotheses, not a suffix convention or a generic missing-tag fallback.
use crate::simulation::metadata::ActionCategory;

#[derive(Clone, Copy, Debug)]
pub(crate) struct OwnerActionRole {
    pub(crate) category: ActionCategory,
    pub(crate) hypothesis_id: &'static str,
    pub(crate) description: &'static str,
}

const QTE_DESCRIPTION: &str = "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.";

const fn qte(hypothesis_id: &'static str) -> OwnerActionRole {
    OwnerActionRole {
        category: ActionCategory::Qte,
        hypothesis_id,
        description: QTE_DESCRIPTION,
    }
}

const fn primary(
    category: ActionCategory,
    hypothesis_id: &'static str,
    description: &'static str,
) -> OwnerActionRole {
    OwnerActionRole {
        category,
        hypothesis_id,
        description,
    }
}

pub(crate) fn resolve(character_id: &str, ability_id: &str) -> Option<OwnerActionRole> {
    let role = match (character_id, ability_id) {
        ("character:1003", "GA_Sagiri_QTE") => qte("owner.action_role.character_1003_qte_v1"),
        ("character:1004", "GA_Lacrimosa_Melee") => primary(
            ActionCategory::Melee,
            "owner.action_role.character_1004_melee_v1",
            "Classify this exact Lacrimosa action as Melee because the frozen character action identity names its primary attack and the effect-config row is absent; owner role is an explicit model hypothesis.",
        ),
        ("character:1004", "GA_Lacrimosa_Skill") => primary(
            ActionCategory::Skill,
            "owner.action_role.character_1004_skill_v1",
            "Classify this exact Lacrimosa action as Skill because the frozen character action identity names its normal skill and the effect-config row is absent; owner role is an explicit model hypothesis.",
        ),
        ("character:1004", "GA_Lacrimosa_UltraSkill") => primary(
            ActionCategory::UltraSkill,
            "owner.action_role.character_1004_ultimate_v1",
            "Classify this exact Lacrimosa action as UltraSkill because the frozen character action identity names its ultimate and the effect-config row is absent; owner role is an explicit model hypothesis.",
        ),
        ("character:1004", "GA_Lacrimosa_QTE") => qte("owner.action_role.character_1004_qte_v1"),
        ("character:1008", "GA_Skia_QTE") => qte("owner.action_role.character_1008_qte_v1"),
        ("character:1010", "GA_Nanally_QTE") => qte("owner.action_role.character_1010_qte_v1"),
        ("character:1019", "GA_Mint019_QTE") => qte("owner.action_role.character_1019_qte_v1"),
        ("character:1020", "GA_Haniel_QTE") => qte("owner.action_role.character_1020_qte_v1"),
        ("character:1021", "GA_Edgar_QTE") => qte("owner.action_role.character_1021_qte_v1"),
        ("character:1023", "GA_Cang_QTE") => qte("owner.action_role.character_1023_qte_v1"),
        ("character:1025", "GA_Hathor_QTE") => qte("owner.action_role.character_1025_qte_v1"),
        ("character:1033", "GA_Adler_QTE") => qte("owner.action_role.character_1033_qte_v1"),
        ("character:1036", "GA_Zankou_QTE") => qte("owner.action_role.character_1036_qte_v1"),
        ("character:1039", "GA_Fadia_QTE") => qte("owner.action_role.character_1039_qte_v1"),
        ("character:1042", "GA_BlackBird_Evolve_QTE") => {
            qte("owner.action_role.character_1042_evolve_qte_v1")
        }
        ("character:1042", "GA_BlackBird_QTE") => qte("owner.action_role.character_1042_qte_v1"),
        ("character:1046", "GA_Female046_Melee") => primary(
            ActionCategory::Melee,
            "owner.action_role.character_1046_melee_v1",
            "Classify this exact Zero action as Melee because the frozen character action identity names its primary attack and the effect-config row is absent; owner role is an explicit model hypothesis.",
        ),
        ("character:1046", "GA_Female046_Skill") => primary(
            ActionCategory::Skill,
            "owner.action_role.character_1046_skill_v1",
            "Classify this exact Zero action as Skill because the frozen character action identity names its normal skill and the effect-config row is absent; owner role is an explicit model hypothesis.",
        ),
        ("character:1046", "GA_Female046_UltraSkill") => primary(
            ActionCategory::UltraSkill,
            "owner.action_role.character_1046_ultimate_v1",
            "Classify this exact Zero action as UltraSkill because the frozen character action identity names its ultimate and the effect-config row is absent; owner role is an explicit model hypothesis.",
        ),
        ("character:1046", "GA_Female046_QTE") => qte("owner.action_role.character_1046_qte_v1"),
        ("character:1051", "GA_Female051_QTE") => qte("owner.action_role.character_1051_qte_v1"),
        ("character:1052", "GA_Jin_QTE") => qte("owner.action_role.character_1052_qte_v1"),
        ("character:1054", "GA_Daffodill_QTE") => qte("owner.action_role.character_1054_qte_v1"),
        ("character:1055", "GA_Kuhara_QTE") => qte("owner.action_role.character_1055_qte_v1"),
        ("character:1057", "GA_Akane_QTE") => qte("owner.action_role.character_1057_qte_v1"),
        ("character:1070", "GA_Mitsuki_QTE") => qte("owner.action_role.character_1070_qte_v1"),
        ("character:1071", "GA_Chaos071_QTE") => qte("owner.action_role.character_1071_qte_v1"),
        ("character:1071", "GA_Chaos071_Skill") => primary(
            ActionCategory::Skill,
            "owner.action_role.character_1071_skill_v1",
            "Classify this exact Chaos action as Skill because the frozen character action identity names its normal skill and the effect-config row is absent; owner role is an explicit model hypothesis.",
        ),
        ("character:1071", "GA_Chaos071_UltraSkill") => primary(
            ActionCategory::UltraSkill,
            "owner.action_role.character_1071_ultimate_v1",
            "Classify this exact Chaos action as UltraSkill because the frozen character action identity names its ultimate and the effect-config row is absent; owner role is an explicit model hypothesis.",
        ),
        ("character:1072", "GA_Radio072_QTE") => qte("owner.action_role.character_1072_qte_v1"),
        ("character:1073", "GA_Chiichan073_QTE") => qte("owner.action_role.character_1073_qte_v1"),
        ("character:1075", "GA_Oneiroi_QTE") => qte("owner.action_role.character_1075_qte_v1"),
        ("character:1076", "GA_Shinku_QTE") => qte("owner.action_role.character_1076_qte_v1"),
        _ => return None,
    };
    Some(role)
}

#[allow(dead_code)]
pub(crate) fn known_hypotheses() -> std::collections::BTreeMap<&'static str, &'static str> {
    let actions = [
        ("character:1003", "GA_Sagiri_QTE"),
        ("character:1004", "GA_Lacrimosa_Melee"),
        ("character:1004", "GA_Lacrimosa_Skill"),
        ("character:1004", "GA_Lacrimosa_UltraSkill"),
        ("character:1004", "GA_Lacrimosa_QTE"),
        ("character:1008", "GA_Skia_QTE"),
        ("character:1010", "GA_Nanally_QTE"),
        ("character:1019", "GA_Mint019_QTE"),
        ("character:1020", "GA_Haniel_QTE"),
        ("character:1021", "GA_Edgar_QTE"),
        ("character:1023", "GA_Cang_QTE"),
        ("character:1025", "GA_Hathor_QTE"),
        ("character:1033", "GA_Adler_QTE"),
        ("character:1036", "GA_Zankou_QTE"),
        ("character:1039", "GA_Fadia_QTE"),
        ("character:1042", "GA_BlackBird_Evolve_QTE"),
        ("character:1042", "GA_BlackBird_QTE"),
        ("character:1046", "GA_Female046_Melee"),
        ("character:1046", "GA_Female046_Skill"),
        ("character:1046", "GA_Female046_UltraSkill"),
        ("character:1046", "GA_Female046_QTE"),
        ("character:1051", "GA_Female051_QTE"),
        ("character:1052", "GA_Jin_QTE"),
        ("character:1054", "GA_Daffodill_QTE"),
        ("character:1055", "GA_Kuhara_QTE"),
        ("character:1057", "GA_Akane_QTE"),
        ("character:1070", "GA_Mitsuki_QTE"),
        ("character:1071", "GA_Chaos071_QTE"),
        ("character:1071", "GA_Chaos071_Skill"),
        ("character:1071", "GA_Chaos071_UltraSkill"),
        ("character:1072", "GA_Radio072_QTE"),
        ("character:1073", "GA_Chiichan073_QTE"),
        ("character:1075", "GA_Oneiroi_QTE"),
        ("character:1076", "GA_Shinku_QTE"),
    ];
    actions
        .into_iter()
        .filter_map(|(character_id, ability_id)| {
            resolve(character_id, ability_id).map(|role| (role.hypothesis_id, role.description))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_are_exact_owner_declarations_and_never_suffix_defaults() {
        assert_eq!(
            resolve("character:1046", "GA_Female046_QTE")
                .unwrap()
                .category,
            ActionCategory::Qte
        );
        assert!(resolve("character:9999", "GA_Any_QTE").is_none());
        assert!(resolve("character:1046", "GA_Female046_Other").is_none());
    }
}
