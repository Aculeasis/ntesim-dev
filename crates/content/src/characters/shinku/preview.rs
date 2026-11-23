//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "shinku",
        character_id: "character:1076",
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        weapon_id: "weapon:fork_LunarPhase",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1086,
        atk_ratio: Ratio {
            numerator: 24978,
            denominator: 10,
        },
        hp: 16120,
        def: 897,
        ability_id: "GA_Shinku_Skill",
        damage_effect_id: "GE_Player_Shinku_Skill1_1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 37,
            denominator: 125,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
