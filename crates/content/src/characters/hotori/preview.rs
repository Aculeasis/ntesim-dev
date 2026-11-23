//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "hotori",
        character_id: "character:1052",
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        weapon_id: "weapon:fork_Time",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1102,
        atk_ratio: Ratio {
            numerator: 25346,
            denominator: 10,
        },
        hp: 16120,
        def: 848,
        ability_id: "GA_Jin_Skill",
        damage_effect_id: "GE_Player_Jin_Skill1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 63,
            denominator: 125,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
