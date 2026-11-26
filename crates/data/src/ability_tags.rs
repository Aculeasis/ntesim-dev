//! Consumed Global 2026-10-04 action tags, independent from input roles.
//! Only configured actions of the 25 supported owners are retained. Missing
//! source rows remain missing; tags do not imply a primary button or a trigger.
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub const EVIDENCE_TABLE: &str = "typed:ability_tags";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeTag {
    Melee,
    Skill,
    UltraSkill,
}

impl NativeTag {
    pub fn native_name(self) -> &'static str {
        match self {
            Self::Melee => "Ability.Melee",
            Self::Skill => "Ability.Skill",
            Self::UltraSkill => "Ability.UltraSkill",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AbilityTag {
    pub ability_id: &'static str,
    pub tag: NativeTag,
}

pub const TAGS: &[AbilityTag] = &[
    AbilityTag {
        ability_id: "GA_Adler_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Adler_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Adler_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Akane_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Akane_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Akane_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_BlackBird_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_BlackBird_Evolve_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_BlackBird_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_BlackBird_Evolve_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_BlackBird_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_BlackBird_Evolve_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Cang_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Cang_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Cang_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Chaos071_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Chaos071_Skill",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Chaos071_UltraSkill",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Chiichan073_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Chiichan073_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Chiichan073_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Daffodill_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Daffodill_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Daffodill_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Edgar_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Edgar_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Edgar_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Elyms_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Elyms_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Elyms_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Fadia_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Fadia_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Fadia_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Haniel_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Haniel_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Haniel_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Hathor_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Hathor_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Hathor_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Jin_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Jin_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Jin_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Kuhara_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Kuhara_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Kuhara_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Mint019_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Mint019_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Mint019_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Mitsuki_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Mitsuki_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Mitsuki_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Nanally_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Nanally_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Nanally_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Oneiroi_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Oneiroi_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Oneiroi_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Radio072_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Radio072_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Radio072_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Sagiri_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Sagiri_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Sagiri_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Shinku_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Shinku_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Shinku_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Skia_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Skia_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Skia_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_Zankou_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_Zankou_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_Zankou_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
    AbilityTag {
        ability_id: "GA_lacrimosa_Melee",
        tag: NativeTag::Melee,
    },
    AbilityTag {
        ability_id: "GA_lacrimosa_Skill",
        tag: NativeTag::Skill,
    },
    AbilityTag {
        ability_id: "GA_lacrimosa_UltraSkill",
        tag: NativeTag::UltraSkill,
    },
];

/// JSON transport for the existing metadata boundary and explicit fixture overlays.
/// Hashes identify these consumed fields, not the retired full source rows.
pub fn transport() -> &'static Value {
    static TABLE: OnceLock<Value> = OnceLock::new();
    TABLE.get_or_init(|| {
        let rows = TAGS
            .iter()
            .map(|row| {
                (
                    row.ability_id,
                    json!({"AbilityGameplayTag":{"TagName":row.tag.native_name()}}),
                )
            })
            .collect::<BTreeMap<_, _>>();
        json!({"schema_version":1,"source_id":EVIDENCE_TABLE,"rows":rows})
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn tags_preserve_source_categories_and_absent_rows() {
        let ids = TAGS
            .iter()
            .map(|row| row.ability_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), TAGS.len());
        assert_eq!(ids.len(), 75);
        let rows = &transport()["rows"];
        for ability in ["GA_Chaos071_Skill", "GA_Chaos071_UltraSkill"] {
            assert_eq!(
                rows[ability]["AbilityGameplayTag"]["TagName"],
                "Ability.Melee"
            );
        }
        assert_eq!(
            rows["GA_Mint019_Skill"]["AbilityGameplayTag"]["TagName"],
            "Ability.Skill"
        );
        assert_eq!(
            rows["GA_Mint019_UltraSkill"]["AbilityGameplayTag"]["TagName"],
            "Ability.UltraSkill"
        );
        assert_eq!(
            rows["GA_lacrimosa_UltraSkill"]["AbilityGameplayTag"]["TagName"],
            "Ability.UltraSkill"
        );
        for ability in [
            "GA_Female046_Skill",
            "GA_Female046_UltraSkill",
            "GA_Mint019_QTE",
        ] {
            assert!(
                rows.get(ability).is_none(),
                "absent source row must remain absent: {ability}"
            );
        }
        assert!(rows.as_object().unwrap().values().all(|row| {
            row.as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .eq(["AbilityGameplayTag"])
        }));
    }
}
