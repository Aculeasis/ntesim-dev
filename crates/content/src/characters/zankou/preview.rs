//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "zankou",
        character_id: "character:1036",
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        weapon_id: "weapon:fork_DemonBlade",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1110,
        atk_ratio: Ratio {
            numerator: 25530,
            denominator: 10,
        },
        hp: 15514,
        def: 909,
        ability_id: "GA_Zankou_Skill",
        damage_effect_id: "GE_Player_Zankou_Skill2_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 2063,
            denominator: 1000,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
