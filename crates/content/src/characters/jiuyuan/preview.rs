//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "jiuyuan",
        character_id: "character:1055",
        awakening: ntesim_data::AwakeningSelection::prefix(1),
        weapon_id: "weapon:fork_Butterfly",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1102,
        atk_ratio: Ratio {
            numerator: 25346,
            denominator: 10,
        },
        hp: 15635,
        def: 909,
        ability_id: "GA_Kuhara_Skill",
        damage_effect_id: "GE_Player_Kuhara_Skill1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 453,
            denominator: 1000,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
