//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "aurelia",
        character_id: "character:1070",
        awakening: ntesim_data::AwakeningSelection::prefix(6),
        weapon_id: "weapon:fork_TigerTally",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1054,
        atk_ratio: Ratio {
            numerator: 24242,
            denominator: 10,
        },
        hp: 11878,
        def: 727,
        ability_id: "GA_Mitsuki_Skill",
        damage_effect_id: "GE_Player_Mitsuki_Skill_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 34,
            denominator: 25,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
