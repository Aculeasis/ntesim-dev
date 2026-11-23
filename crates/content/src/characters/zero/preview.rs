//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "zero",
        character_id: "character:1046",
        awakening: ntesim_data::AwakeningSelection::prefix(1),
        weapon_id: "weapon:fork_rishi",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1126,
        atk_ratio: Ratio {
            numerator: 13521,
            denominator: 5,
        },
        hp: 15150,
        def: 909,
        ability_id: "GA_Female046_Skill",
        damage_effect_id: "GE_Player_Female046_Skill1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 343,
            denominator: 1000,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
