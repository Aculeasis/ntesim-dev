//! Consumed resonance_3 skill-level additions, not complete awakening kits.
//! The baseline adds one level to each listed skill at three selected awakening
//! effects. Chiz's source resonance uses a buff rather than SkillLevel entries;
//! it declares no level additions. Other awakening mechanics belong to owners.
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub const TABLE_NAME: &str = "typed_resonance_levels";
pub const SOURCE_ID: &str = "table:typed_resonance_levels";

#[derive(Clone, Copy, Debug)]
pub struct ResonanceLevels {
    pub native_id: &'static str,
    pub skills: &'static [&'static str],
}

pub const RESONANCES: &[ResonanceLevels] = &[
    ResonanceLevels {
        native_id: "1003",
        skills: &["GA_Sagiri_Melee", "GA_Sagiri_Skill", "GA_Sagiri_UltraSkill"],
    },
    ResonanceLevels {
        native_id: "1004",
        skills: &[
            "GA_Lacrimosa_Melee",
            "GA_Lacrimosa_Skill",
            "GA_Lacrimosa_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1008",
        skills: &["GA_Skia_Melee", "GA_Skia_Skill", "GA_Skia_UltraSkill"],
    },
    ResonanceLevels {
        native_id: "1010",
        skills: &[
            "GA_Nanally_Melee",
            "GA_Nanally_Skill",
            "GA_Nanally_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1019",
        skills: &[
            "GA_Mint019_Melee",
            "GA_Mint019_Skill",
            "GA_Mint019_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1020",
        skills: &["GA_Haniel_Melee", "GA_Haniel_Skill", "GA_Haniel_UltraSkill"],
    },
    ResonanceLevels {
        native_id: "1021",
        skills: &["GA_Edgar_QTE", "GA_Edgar_Skill", "GA_Edgar_UltraSkill"],
    },
    ResonanceLevels {
        native_id: "1023",
        skills: &["GA_Cang_Skill", "GA_Cang_UltraSkill", "GA_Cang_Melee"],
    },
    ResonanceLevels {
        native_id: "1025",
        skills: &["GA_Hathor_Skill", "GA_Hathor_UltraSkill", "GA_Hathor_Melee"],
    },
    ResonanceLevels {
        native_id: "1033",
        skills: &["GA_Adler_Skill", "GA_Adler_UltraSkill", "GA_Adler_Melee"],
    },
    ResonanceLevels {
        native_id: "1036",
        skills: &["GA_Zankou_Melee", "GA_Zankou_Skill", "GA_Zankou_UltraSkill"],
    },
    ResonanceLevels {
        native_id: "1039",
        skills: &["GA_Fadia_Melee", "GA_Fadia_Skill", "GA_Fadia_UltraSkill"],
    },
    ResonanceLevels {
        native_id: "1042",
        skills: &[
            "GA_BlackBird_Melee",
            "GA_BlackBird_Skill",
            "GA_BlackBird_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1046",
        skills: &[
            "GA_Female046_Melee",
            "GA_Female046_Skill",
            "GA_Female046_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1052",
        skills: &["GA_Jin_Melee", "GA_Jin_Skill", "GA_Jin_UltraSkill"],
    },
    ResonanceLevels {
        native_id: "1054",
        skills: &[
            "GA_Daffodill_Melee",
            "GA_Daffodill_Skill",
            "GA_Daffodill_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1055",
        skills: &["GA_Kuhara_Skill", "GA_Kuhara_UltraSkill", "GA_Kuhara_Melee"],
    },
    ResonanceLevels {
        native_id: "1057",
        skills: &["GA_Akane_Melee", "GA_Akane_Skill", "GA_Akane_UltraSkill"],
    },
    ResonanceLevels {
        native_id: "1070",
        skills: &[
            "GA_Mitsuki_Melee",
            "GA_Mitsuki_Skill",
            "GA_Mitsuki_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1071",
        skills: &[
            "GA_Chaos071_Melee",
            "GA_Chaos071_Skill",
            "GA_Chaos071_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1072",
        skills: &[
            "GA_Radio072_QTE",
            "GA_Radio072_Skill",
            "GA_Radio072_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1073",
        skills: &[],
    },
    ResonanceLevels {
        native_id: "1075",
        skills: &[
            "GA_Oneiroi_Melee",
            "GA_Oneiroi_Skill",
            "GA_Oneiroi_UltraSkill",
        ],
    },
    ResonanceLevels {
        native_id: "1076",
        skills: &["GA_Shinku_Skill", "GA_Shinku_UltraSkill", "GA_Shinku_Melee"],
    },
];

pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let rows = RESONANCES
            .iter()
            .map(|row| {
                let modifications = row
                    .skills
                    .iter()
                    .map(|skill| json!({"SkillName":skill,"SkillLevel":1}))
                    .collect::<Vec<_>>();
                (
                    row.native_id,
                    json!({"AwakenEffectStructList":[{
                        "EffectID":"resonance_3","AwakenType":"EAwakenEffect::Awaken_Resonance",
                        "ModifyDataList":modifications
                    }]}),
                )
            })
            .collect::<BTreeMap<_, _>>();
        json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
    })
}

fn overlay_name(native_id: &str) -> String {
    if native_id == "1019" {
        "1019AwakenEffect1".into()
    } else {
        format!("{native_id}AwakenEffect")
    }
}

/// Explicit owner overlays replace the baseline; no missing-row/field merge.
pub fn table<'a>(tables: &'a BTreeMap<String, Value>, native_id: &str) -> (String, &'a Value) {
    let name = overlay_name(native_id);
    match tables.get_key_value(&name) {
        Some((name, value)) => (name.clone(), value),
        None => (TABLE_NAME.into(), transport()),
    }
}

pub fn overlay(native_id: &str) -> Result<Value, String> {
    let row = transport()["rows"]
        .get(native_id)
        .ok_or("resonance owner missing")?;
    let name = overlay_name(native_id);
    Ok(json!({"schema_version":1,"source_id":format!("table:{name}"),"rows":{native_id:row}}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn resonance_rows_cover_the_supported_roster_and_only_consumed_fields() {
        let expected = crate::character::definitions()
            .iter()
            .map(|owner| owner.native_id)
            .collect::<BTreeSet<_>>();
        let actual = RESONANCES
            .iter()
            .map(|row| row.native_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 24);
        assert_eq!(
            RESONANCES.iter().map(|row| row.skills.len()).sum::<usize>(),
            69
        );
        for row in transport()["rows"].as_object().unwrap().values() {
            assert_eq!(row.as_object().unwrap().len(), 1);
            let effects = row["AwakenEffectStructList"].as_array().unwrap();
            assert_eq!(effects.len(), 1);
            assert_eq!(effects[0].as_object().unwrap().len(), 3);
            for modification in effects[0]["ModifyDataList"].as_array().unwrap() {
                assert_eq!(modification.as_object().unwrap().len(), 2);
                assert_eq!(modification["SkillLevel"], 1);
            }
        }
        assert!(
            transport()["rows"]["1073"]["AwakenEffectStructList"][0]["ModifyDataList"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(overlay("9999").is_err());
        assert_eq!(
            overlay("1019").unwrap()["rows"]["1019"],
            transport()["rows"]["1019"]
        );
    }
}
