//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "chiz",
        character_id: "character:1073",
        awakening: ntesim_data::AwakeningSelection::prefix(6),
        weapon_id: "weapon:fork_mamen",
        set_id: None,
        weapon_tier: 5,
        base_atk: 1110,
        atk_ratio: Ratio {
            numerator: 25530,
            denominator: 10,
        },
        hp: 15514,
        def: 909,
        ability_id: "GA_Chiichan073_Skill",
        damage_effect_id: "GE_Player_Chiichan073_Skill1_1Damage",
        external_atk_rate_claim: Ratio {
            numerator: 163,
            denominator: 500,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
