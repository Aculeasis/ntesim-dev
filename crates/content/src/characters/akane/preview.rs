//! Historical manual preview inputs, not a complete or strict game adapter.
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "akane",
        character_id: "character:1057",
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        weapon_id: "weapon:fork_prism",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1086,
        atk_ratio: Ratio {
            numerator: 24978,
            denominator: 10,
        },
        hp: 15998,
        def: 921,
        ability_id: "GA_Akane_Skill",
        damage_effect_id: "GE_Player_Akane_Skill_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 3600,
            denominator: 1000,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
