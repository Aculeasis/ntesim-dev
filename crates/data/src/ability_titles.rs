//! English labels consumed by the Studio action catalog and localization export.
//! Global 2026-10-04 GameplayAbilityTipsData: only supported action IDs and
//! trimmed localized titles are retained. Descriptions, UI assets, passive-only
//! labels and unimplemented owners remain cache-only; titles confer no mechanics.

#[derive(Clone, Copy, Debug)]
pub struct AbilityTitle {
    pub ability_id: &'static str,
    pub title: &'static str,
}

pub const TITLES: &[AbilityTitle] = &[
    AbilityTitle {
        ability_id: "GA_Adler_Melee",
        title: "Deliverance",
    },
    AbilityTitle {
        ability_id: "GA_Adler_QTE",
        title: "Pristine Reflection",
    },
    AbilityTitle {
        ability_id: "GA_Adler_Skill",
        title: "Evil's Bane",
    },
    AbilityTitle {
        ability_id: "GA_Adler_UltraSkill",
        title: "Tranquility",
    },
    AbilityTitle {
        ability_id: "GA_Akane_Melee",
        title: "Stringblade",
    },
    AbilityTitle {
        ability_id: "GA_Akane_QTE",
        title: "Altered Chord",
    },
    AbilityTitle {
        ability_id: "GA_Akane_Skill",
        title: "Rhythm Wave",
    },
    AbilityTitle {
        ability_id: "GA_Akane_UltraSkill",
        title: "Sonic Boom",
    },
    AbilityTitle {
        ability_id: "GA_BlackBird_Evolve_Melee",
        title: "Fleeting Shadow",
    },
    AbilityTitle {
        ability_id: "GA_BlackBird_Evolve_Skill",
        title: "Netherplume Rite",
    },
    AbilityTitle {
        ability_id: "GA_BlackBird_Evolve_UltraSkill",
        title: "Requies Animarum",
    },
    AbilityTitle {
        ability_id: "GA_BlackBird_Melee",
        title: "Fleeting Shadow",
    },
    AbilityTitle {
        ability_id: "GA_BlackBird_QTE",
        title: "Judgment Upon Me",
    },
    AbilityTitle {
        ability_id: "GA_BlackBird_Skill",
        title: "Netherplume Rite",
    },
    AbilityTitle {
        ability_id: "GA_BlackBird_UltraSkill",
        title: "Requies Animarum",
    },
    AbilityTitle {
        ability_id: "GA_Cang_Melee",
        title: "Walk the Talk",
    },
    AbilityTitle {
        ability_id: "GA_Cang_QTE",
        title: "Break Time Over",
    },
    AbilityTitle {
        ability_id: "GA_Cang_Skill",
        title: "Generous Guidance",
    },
    AbilityTitle {
        ability_id: "GA_Cang_UltraSkill",
        title: "Judgment of Autumn",
    },
    AbilityTitle {
        ability_id: "GA_Chaos071_Melee",
        title: "Pursuit",
    },
    AbilityTitle {
        ability_id: "GA_Chaos071_QTE",
        title: "Onslaught",
    },
    AbilityTitle {
        ability_id: "GA_Chaos071_Skill",
        title: "Doubtmark",
    },
    AbilityTitle {
        ability_id: "GA_Chaos071_UltraSkill",
        title: "Retribution",
    },
    AbilityTitle {
        ability_id: "GA_Chiichan073_Melee",
        title: "Exiled Swordplay",
    },
    AbilityTitle {
        ability_id: "GA_Chiichan073_QTE",
        title: "Temporary Entry",
    },
    AbilityTitle {
        ability_id: "GA_Chiichan073_Skill",
        title: "Pink Paws Priority Principle",
    },
    AbilityTitle {
        ability_id: "GA_Chiichan073_UltraSkill",
        title: "Zero-Sum Game",
    },
    AbilityTitle {
        ability_id: "GA_Daffodill_Melee",
        title: "Still Waters",
    },
    AbilityTitle {
        ability_id: "GA_Daffodill_QTE",
        title: "Crossed Blades",
    },
    AbilityTitle {
        ability_id: "GA_Daffodill_Skill",
        title: "Resonance",
    },
    AbilityTitle {
        ability_id: "GA_Daffodill_UltraSkill",
        title: "Finale",
    },
    AbilityTitle {
        ability_id: "GA_Edgar_Melee",
        title: "Combat Practice",
    },
    AbilityTitle {
        ability_id: "GA_Edgar_QTE",
        title: "Weight of Knowledge",
    },
    AbilityTitle {
        ability_id: "GA_Edgar_Skill",
        title: "Wild Current",
    },
    AbilityTitle {
        ability_id: "GA_Edgar_UltraSkill",
        title: "Finnegan's Wake",
    },
    AbilityTitle {
        ability_id: "GA_Elyms_Melee",
        title: "Instant Glitch",
    },
    AbilityTitle {
        ability_id: "GA_Elyms_QTE",
        title: "I'm Here",
    },
    AbilityTitle {
        ability_id: "GA_Elyms_Skill",
        title: "Counting Electric Sheep",
    },
    AbilityTitle {
        ability_id: "GA_Elyms_UltraSkill",
        title: "Do You Get Deja Vu",
    },
    AbilityTitle {
        ability_id: "GA_Fadia_Melee",
        title: "Wordless Rejection",
    },
    AbilityTitle {
        ability_id: "GA_Fadia_QTE",
        title: "Outsider",
    },
    AbilityTitle {
        ability_id: "GA_Fadia_Skill",
        title: "Existence",
    },
    AbilityTitle {
        ability_id: "GA_Fadia_UltraSkill",
        title: "Agony to Euphoria",
    },
    AbilityTitle {
        ability_id: "GA_Female046_Melee",
        title: "Appraisal",
    },
    AbilityTitle {
        ability_id: "GA_Female046_QTE",
        title: "Rift Blossom",
    },
    AbilityTitle {
        ability_id: "GA_Female046_Skill",
        title: "Appraise and Engrave",
    },
    AbilityTitle {
        ability_id: "GA_Female046_UltraSkill",
        title: "Divide by Zero",
    },
    AbilityTitle {
        ability_id: "GA_Haniel_Melee",
        title: "Genesse Technique",
    },
    AbilityTitle {
        ability_id: "GA_Haniel_QTE",
        title: "Easter Egg Time",
    },
    AbilityTitle {
        ability_id: "GA_Haniel_Skill",
        title: "Silent Moonlit Forest Guardian",
    },
    AbilityTitle {
        ability_id: "GA_Haniel_UltraSkill",
        title: "A Melody Named Haniel",
    },
    AbilityTitle {
        ability_id: "GA_Hathor_Melee",
        title: "Rapid Delivery",
    },
    AbilityTitle {
        ability_id: "GA_Hathor_QTE",
        title: "Impact Point Designation",
    },
    AbilityTitle {
        ability_id: "GA_Hathor_Skill",
        title: "Aerial Command",
    },
    AbilityTitle {
        ability_id: "GA_Hathor_UltraSkill",
        title: "Rider Express",
    },
    AbilityTitle {
        ability_id: "GA_Jin_Melee",
        title: "Misty Moon Style",
    },
    AbilityTitle {
        ability_id: "GA_Jin_QTE",
        title: "Shopkeeper's Authority",
    },
    AbilityTitle {
        ability_id: "GA_Jin_Skill",
        title: "Present Replay",
    },
    AbilityTitle {
        ability_id: "GA_Jin_UltraSkill",
        title: "World's Tide",
    },
    AbilityTitle {
        ability_id: "GA_Kuhara_Melee",
        title: "When Secrets Take Shape",
    },
    AbilityTitle {
        ability_id: "GA_Kuhara_QTE",
        title: "Intel Lock-On",
    },
    AbilityTitle {
        ability_id: "GA_Kuhara_Skill",
        title: "Intel Hunter",
    },
    AbilityTitle {
        ability_id: "GA_Kuhara_UltraSkill",
        title: "Final Reckoning",
    },
    AbilityTitle {
        ability_id: "GA_Lacrimosa_Melee",
        title: "Sweet and Sour",
    },
    AbilityTitle {
        ability_id: "GA_Lacrimosa_QTE",
        title: "Microwake",
    },
    AbilityTitle {
        ability_id: "GA_Lacrimosa_Skill",
        title: "Morning Tomato",
    },
    AbilityTitle {
        ability_id: "GA_Lacrimosa_UltraSkill",
        title: "Working Day Judgement",
    },
    AbilityTitle {
        ability_id: "GA_Mint019_Melee",
        title: "Perfect Containment",
    },
    AbilityTitle {
        ability_id: "GA_Mint019_QTE",
        title: "Ultimate: Invincible Tornado Slash",
    },
    AbilityTitle {
        ability_id: "GA_Mint019_Skill",
        title: "Ultimate: Super Claws",
    },
    AbilityTitle {
        ability_id: "GA_Mint019_UltraSkill",
        title: "Ultimate: Thunderous Whirlwind Slash",
    },
    AbilityTitle {
        ability_id: "GA_Mitsuki_Melee",
        title: "Cappella",
    },
    AbilityTitle {
        ability_id: "GA_Mitsuki_QTE",
        title: "Dissonance",
    },
    AbilityTitle {
        ability_id: "GA_Mitsuki_Skill",
        title: "Cadenza Aria",
    },
    AbilityTitle {
        ability_id: "GA_Mitsuki_UltraSkill",
        title: "Canon Chorus",
    },
    AbilityTitle {
        ability_id: "GA_Nanally_Melee",
        title: "Colucci Secret Skill",
    },
    AbilityTitle {
        ability_id: "GA_Nanally_QTE",
        title: "Justice from Above",
    },
    AbilityTitle {
        ability_id: "GA_Nanally_Skill",
        title: "Colucci Howling Technique",
    },
    AbilityTitle {
        ability_id: "GA_Nanally_UltraSkill",
        title: "Colucci Ultimate Technique",
    },
    AbilityTitle {
        ability_id: "GA_Oneiroi_Melee",
        title: "Collaboration",
    },
    AbilityTitle {
        ability_id: "GA_Oneiroi_QTE",
        title: "Bang!",
    },
    AbilityTitle {
        ability_id: "GA_Oneiroi_Skill",
        title: "Future Self Continuity",
    },
    AbilityTitle {
        ability_id: "GA_Oneiroi_UltraSkill",
        title: "3.8-Billion-Year Mirage",
    },
    AbilityTitle {
        ability_id: "GA_Radio072_Melee",
        title: "Poltergeist CQC",
    },
    AbilityTitle {
        ability_id: "GA_Radio072_QTE",
        title: "Spectral Cross",
    },
    AbilityTitle {
        ability_id: "GA_Radio072_Skill",
        title: "Full-Frequency Pulse",
    },
    AbilityTitle {
        ability_id: "GA_Radio072_UltraSkill",
        title: "Oversync",
    },
    AbilityTitle {
        ability_id: "GA_Sagiri_Melee",
        title: "Kiroumaru Headbutt",
    },
    AbilityTitle {
        ability_id: "GA_Sagiri_QTE",
        title: "Squash!",
    },
    AbilityTitle {
        ability_id: "GA_Sagiri_Skill",
        title: "Devour Whole",
    },
    AbilityTitle {
        ability_id: "GA_Sagiri_UltraSkill",
        title: "Feast of Gluttony",
    },
    AbilityTitle {
        ability_id: "GA_Shinku_Melee",
        title: "Basic Attack: Special Combat Arts",
    },
    AbilityTitle {
        ability_id: "GA_Shinku_QTE",
        title: "Breaching Tactics",
    },
    AbilityTitle {
        ability_id: "GA_Shinku_Skill",
        title: "High-Speed Breach",
    },
    AbilityTitle {
        ability_id: "GA_Shinku_UltraSkill",
        title: "Ultimate: Crimson Fury",
    },
    AbilityTitle {
        ability_id: "GA_Skia_Melee",
        title: "Arresting Art",
    },
    AbilityTitle {
        ability_id: "GA_Skia_QTE",
        title: "Arrest Warrant",
    },
    AbilityTitle {
        ability_id: "GA_Skia_Skill",
        title: "Shadow Hound Chase",
    },
    AbilityTitle {
        ability_id: "GA_Skia_UltraSkill",
        title: "The Pack",
    },
    AbilityTitle {
        ability_id: "GA_Zankou_Melee",
        title: "Wildfire",
    },
    AbilityTitle {
        ability_id: "GA_Zankou_QTE",
        title: "Stoked Flame",
    },
    AbilityTitle {
        ability_id: "GA_Zankou_Skill",
        title: "Sanguine Dash",
    },
    AbilityTitle {
        ability_id: "GA_Zankou_UltraSkill",
        title: "Inferno Flamenco",
    },
    AbilityTitle {
        ability_id: "GA_lacrimosa_Passive_1",
        title: "Tomato Banquet",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn supported_action_titles_are_unique_and_keep_native_punctuation() {
        let ids = TITLES
            .iter()
            .map(|row| row.ability_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), TITLES.len());
        assert_eq!(ids.len(), 104);
        assert!(
            TITLES
                .iter()
                .all(|row| !row.title.is_empty() && row.title == row.title.trim())
        );
        assert!(!ids.contains("GA_Mint_Passive_1"));
        assert_eq!(
            TITLES
                .iter()
                .find(|row| row.ability_id == "GA_Mint019_Skill")
                .unwrap()
                .title,
            "Ultimate: Super Claws"
        );
        assert_eq!(
            TITLES
                .iter()
                .find(|row| row.ability_id == "GA_Female046_UltraSkill")
                .unwrap()
                .title,
            "Divide by Zero"
        );
        assert_eq!(
            TITLES
                .iter()
                .find(|row| row.ability_id == "GA_BlackBird_Evolve_Skill")
                .unwrap()
                .title,
            "Netherplume Rite"
        );
    }
}
