//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "fadia",
        character_id: "character:1039",
        awakening: ntesim_data::AwakeningSelection::prefix(1),
        weapon_id: "weapon:fork_Arachne",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1046,
        atk_ratio: Ratio {
            numerator: 24058,
            denominator: 10,
        },
        hp: 17089,
        def: 909,
        ability_id: "GA_Fadia_Skill",
        damage_effect_id: "GE_Player_Fadia_Skill1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 1,
            denominator: 1,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
