//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "iroi",
        character_id: "character:1075",
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        weapon_id: "weapon:fork_Door",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1166,
        atk_ratio: Ratio {
            numerator: 13301,
            denominator: 5,
        },
        hp: 17210,
        def: 909,
        ability_id: "GA_Oneiroi_Skill",
        damage_effect_id: "GE_Player_Oneiroi_Skill_Syringe_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 2571,
            denominator: 1000,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
