//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "hathor",
        character_id: "character:1025",
        awakening: ntesim_data::AwakeningSelection::prefix(1),
        weapon_id: "weapon:fork_wushoutieyu",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1094,
        atk_ratio: Ratio {
            numerator: 25162,
            denominator: 10,
        },
        hp: 15998,
        def: 884,
        ability_id: "GA_Hathor_Skill",
        damage_effect_id: "GE_Player_Hathor_Skill1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 1513,
            denominator: 250,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
