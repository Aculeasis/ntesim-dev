//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "chaos",
        character_id: "character:1071",
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        weapon_id: "weapon:fork_GoldWool",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1086,
        atk_ratio: Ratio {
            numerator: 24978,
            denominator: 10,
        },
        hp: 15756,
        def: 921,
        ability_id: "GA_Chaos071_Skill",
        damage_effect_id: "GE_Player_Chaos_Skill1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 707,
            denominator: 500,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
