//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "baicang",
        character_id: "character:1023",
        awakening: ntesim_data::AwakeningSelection::prefix(1),
        weapon_id: "weapon:fork_jingmotingyuan",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1094,
        atk_ratio: Ratio {
            numerator: 25162,
            denominator: 10,
        },
        hp: 16604,
        def: 800,
        ability_id: "GA_Cang_Skill",
        damage_effect_id: "GE_Player_Cang_Skill_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 857,
            denominator: 500,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
