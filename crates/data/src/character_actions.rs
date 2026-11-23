//! Consumed baseline action identity, input coordinates and talent prerequisites.
//! Global 2026-10-04: 23 supported owners, four proactive actions each. Native
//! list positions and AbilityIndex are distinct fields, equal to 0..3 and 1..4
//! respectively in this baseline. Currency/item costs and noncombat groups are
//! cache-only; level interpretation remains owned by skill_levels.
use crate::native::Table;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

pub const TABLE_NAME: &str = "typed_character_actions";
pub const SOURCE_ID: &str = "table:typed_character_actions";
const OVERLAY_TABLE: &str = "DT_CharacterAbilityConfig";

#[derive(Clone, Copy, Debug)]
pub struct CharacterActions {
    pub native_id: &'static str,
    pub abilities: [&'static str; 4],
}

pub const REQUIRED_BREAKTHROUGH: [u8; 9] = [2, 3, 3, 4, 4, 5, 5, 6, 6];
pub const CHARACTERS: &[CharacterActions] = &[
    CharacterActions {
        native_id: "1003",
        abilities: [
            "GA_Sagiri_Melee",
            "GA_Sagiri_Skill",
            "GA_Sagiri_UltraSkill",
            "GA_Sagiri_QTE",
        ],
    },
    CharacterActions {
        native_id: "1004",
        abilities: [
            "GA_Lacrimosa_Melee",
            "GA_Lacrimosa_Skill",
            "GA_Lacrimosa_UltraSkill",
            "GA_Lacrimosa_QTE",
        ],
    },
    CharacterActions {
        native_id: "1008",
        abilities: [
            "GA_Skia_Melee",
            "GA_Skia_Skill",
            "GA_Skia_UltraSkill",
            "GA_Skia_QTE",
        ],
    },
    CharacterActions {
        native_id: "1010",
        abilities: [
            "GA_Nanally_Melee",
            "GA_Nanally_Skill",
            "GA_Nanally_UltraSkill",
            "GA_Nanally_QTE",
        ],
    },
    CharacterActions {
        native_id: "1019",
        abilities: [
            "GA_Mint019_Melee",
            "GA_Mint019_Skill",
            "GA_Mint019_UltraSkill",
            "GA_Mint019_QTE",
        ],
    },
    CharacterActions {
        native_id: "1020",
        abilities: [
            "GA_Haniel_Melee",
            "GA_Haniel_Skill",
            "GA_Haniel_UltraSkill",
            "GA_Haniel_QTE",
        ],
    },
    CharacterActions {
        native_id: "1021",
        abilities: [
            "GA_Edgar_Melee",
            "GA_Edgar_Skill",
            "GA_Edgar_UltraSkill",
            "GA_Edgar_QTE",
        ],
    },
    CharacterActions {
        native_id: "1023",
        abilities: [
            "GA_Cang_Melee",
            "GA_Cang_Skill",
            "GA_Cang_UltraSkill",
            "GA_Cang_QTE",
        ],
    },
    CharacterActions {
        native_id: "1025",
        abilities: [
            "GA_Hathor_Melee",
            "GA_Hathor_Skill",
            "GA_Hathor_UltraSkill",
            "GA_Hathor_QTE",
        ],
    },
    CharacterActions {
        native_id: "1033",
        abilities: [
            "GA_Adler_Melee",
            "GA_Adler_Skill",
            "GA_Adler_UltraSkill",
            "GA_Adler_QTE",
        ],
    },
    CharacterActions {
        native_id: "1036",
        abilities: [
            "GA_Zankou_Melee",
            "GA_Zankou_Skill",
            "GA_Zankou_UltraSkill",
            "GA_Zankou_QTE",
        ],
    },
    CharacterActions {
        native_id: "1039",
        abilities: [
            "GA_Fadia_Melee",
            "GA_Fadia_Skill",
            "GA_Fadia_UltraSkill",
            "GA_Fadia_QTE",
        ],
    },
    CharacterActions {
        native_id: "1042",
        abilities: [
            "GA_BlackBird_Melee",
            "GA_BlackBird_Skill",
            "GA_BlackBird_UltraSkill",
            "GA_BlackBird_QTE",
        ],
    },
    CharacterActions {
        native_id: "1046",
        abilities: [
            "GA_Female046_Melee",
            "GA_Female046_Skill",
            "GA_Female046_UltraSkill",
            "GA_Female046_QTE",
        ],
    },
    CharacterActions {
        native_id: "1052",
        abilities: [
            "GA_Jin_Melee",
            "GA_Jin_Skill",
            "GA_Jin_UltraSkill",
            "GA_Jin_QTE",
        ],
    },
    CharacterActions {
        native_id: "1054",
        abilities: [
            "GA_Daffodill_Melee",
            "GA_Daffodill_Skill",
            "GA_Daffodill_UltraSkill",
            "GA_Daffodill_QTE",
        ],
    },
    CharacterActions {
        native_id: "1055",
        abilities: [
            "GA_Kuhara_Melee",
            "GA_Kuhara_Skill",
            "GA_Kuhara_UltraSkill",
            "GA_Kuhara_QTE",
        ],
    },
    CharacterActions {
        native_id: "1070",
        abilities: [
            "GA_Mitsuki_Melee",
            "GA_Mitsuki_Skill",
            "GA_Mitsuki_UltraSkill",
            "GA_Mitsuki_QTE",
        ],
    },
    CharacterActions {
        native_id: "1071",
        abilities: [
            "GA_Chaos071_Melee",
            "GA_Chaos071_Skill",
            "GA_Chaos071_UltraSkill",
            "GA_Chaos071_QTE",
        ],
    },
    CharacterActions {
        native_id: "1072",
        abilities: [
            "GA_Radio072_Melee",
            "GA_Radio072_Skill",
            "GA_Radio072_UltraSkill",
            "GA_Radio072_QTE",
        ],
    },
    CharacterActions {
        native_id: "1073",
        abilities: [
            "GA_Chiichan073_Melee",
            "GA_Chiichan073_Skill",
            "GA_Chiichan073_UltraSkill",
            "GA_Chiichan073_QTE",
        ],
    },
    CharacterActions {
        native_id: "1075",
        abilities: [
            "GA_Oneiroi_Melee",
            "GA_Oneiroi_Skill",
            "GA_Oneiroi_UltraSkill",
            "GA_Oneiroi_QTE",
        ],
    },
    CharacterActions {
        native_id: "1076",
        abilities: [
            "GA_Shinku_Melee",
            "GA_Shinku_Skill",
            "GA_Shinku_UltraSkill",
            "GA_Shinku_QTE",
        ],
    },
];

/// Compact transport for existing metadata, purchase and catalog consumers.
pub fn transport() -> &'static Value {
    &crate::skill_levels::frozen::tables()[TABLE_NAME]
}

// Register once in the existing immutable row/index cache. Injected overlays
// keep their own validation and cannot reuse these pointer-based cache entries.
pub(crate) fn build_transport() -> Value {
    let prerequisites=REQUIRED_BREAKTHROUGH.iter().enumerate().map(|(index,stage)|
            json!({"Level":index+1,"RequireTupoLevel":stage,"RequireAwakenLevel":0})
        ).collect::<Vec<_>>();
    let rows=CHARACTERS.iter().map(|character| {
            let abilities=character.abilities.iter().enumerate().map(|(index,ability)|
                json!({"Key":ability,"Value":{"AbilityIndex":index+1,"AbilityType":"ECharacterAbilityType::Proactive","LevelsCostItems":prerequisites}})
            ).collect::<Vec<_>>();
            (character.native_id,json!({"CharacterAbilityList":abilities}))
        }).collect::<BTreeMap<_,_>>();
    json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
}

/// Explicit test/research transport; missing fields do not merge with defaults.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:DT_CharacterAbilityConfig");
    value
}

pub(crate) fn table(tables: &BTreeMap<String, Value>) -> Result<Table<'_>, String> {
    if let Some(value) = tables.get(OVERLAY_TABLE) {
        Table::new(OVERLAY_TABLE, value)
    } else {
        Table::new(TABLE_NAME, transport())
    }
}

pub fn resolve_evolved_parent(native_character: &str, ability_id: &str) -> Option<&'static str> {
    if native_character == "1042" {
        match ability_id {
            "GA_BlackBird_Evolve_Melee" => Some("GA_BlackBird_Melee"),
            "GA_BlackBird_Evolve_Skill" => Some("GA_BlackBird_Skill"),
            "GA_BlackBird_Evolve_UltraSkill" => Some("GA_BlackBird_UltraSkill"),
            "GA_BlackBird_Evolve_QTE" => Some("GA_BlackBird_QTE"),
            _ => None,
        }
    } else {
        None
    }
}

pub fn rows(
    tables: &BTreeMap<String, Value>,
) -> Result<(&'static str, &Map<String, Value>), String> {
    let name = if tables.contains_key(OVERLAY_TABLE) {
        OVERLAY_TABLE
    } else {
        TABLE_NAME
    };
    Ok((name, table(tables)?.rows()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn actions_cover_only_supported_characters_and_consumed_prerequisites() {
        let expected = crate::character::definitions()
            .iter()
            .map(|owner| owner.native_id)
            .collect::<BTreeSet<_>>();
        let actual = CHARACTERS
            .iter()
            .map(|owner| owner.native_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 23);
        let rows = transport()["rows"].as_object().unwrap();
        for row in rows.values() {
            assert_eq!(row.as_object().unwrap().len(), 1);
            for action in row["CharacterAbilityList"].as_array().unwrap() {
                assert_eq!(action["Value"].as_object().unwrap().len(), 3);
                for step in action["Value"]["LevelsCostItems"].as_array().unwrap() {
                    assert_eq!(step.as_object().unwrap().len(), 3);
                }
            }
        }
        assert!(rows.get("1051").is_none());
        assert!(rows.get("1057").is_none());
    }

    #[test]
    fn independent_purchase_boundaries_hold_for_all_supported_actions() {
        let tables = BTreeMap::new();
        for owner in CHARACTERS {
            for ability in owner.abilities {
                for (purchased, breakthrough, accepted) in [
                    (1, 0, true),
                    (2, 1, false),
                    (2, 2, true),
                    (4, 2, false),
                    (4, 3, true),
                    (10, 5, false),
                    (10, 6, true),
                ] {
                    let result = crate::skill_levels::compile(
                        &tables,
                        &format!("character:{}", owner.native_id),
                        ability,
                        purchased,
                        breakthrough,
                        0,
                    );
                    assert_eq!(
                        result.is_ok(),
                        accepted,
                        "{} {ability} level {purchased}, breakthrough {breakthrough}",
                        owner.native_id
                    );
                }
            }
        }
    }
}
