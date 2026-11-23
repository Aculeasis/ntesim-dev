//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "haniel",
        character_id: "character:1020",
        awakening: ntesim_data::AwakeningSelection::prefix(6),
        weapon_id: "weapon:fork_BitGame",
        set_id: None,
        weapon_tier: 1,
        base_atk: 943,
        atk_ratio: Ratio {
            numerator: 21689,
            denominator: 10,
        },
        hp: 13696,
        def: 848,
        ability_id: "GA_Haniel_Skill",
        damage_effect_id: "GE_Player_Haniel_Skill1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 857,
            denominator: 500,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
