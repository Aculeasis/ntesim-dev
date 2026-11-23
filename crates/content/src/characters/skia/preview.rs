//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "skia",
        character_id: "character:1008",
        awakening: ntesim_data::AwakeningSelection::prefix(6),
        weapon_id: "weapon:fork_Kite",
        set_id: None,
        weapon_tier: 5,
        base_atk: 969,
        atk_ratio: Ratio {
            numerator: 22287,
            denominator: 10,
        },
        hp: 13332,
        def: 667,
        ability_id: "GA_Skia_Skill",
        damage_effect_id: "GE_Player_Skia_Skill1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 257,
            denominator: 500,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
