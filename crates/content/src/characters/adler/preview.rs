//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "adler",
        character_id: "character:1033",
        awakening: ntesim_data::AwakeningSelection::prefix(6),
        weapon_id: "weapon:fork_yuren",
        set_id: None,
        weapon_tier: 5,
        base_atk: 849,
        atk_ratio: Ratio {
            numerator: 19527,
            denominator: 10,
        },
        hp: 14302,
        def: 994,
        ability_id: "GA_Adler_Skill",
        damage_effect_id: "GE_Player_Adler_Skill2_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 1,
            denominator: 1,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
