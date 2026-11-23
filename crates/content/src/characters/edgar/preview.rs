//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "edgar",
        character_id: "character:1021",
        awakening: ntesim_data::AwakeningSelection::prefix(6),
        weapon_id: "weapon:fork_Castle",
        set_id: None,
        weapon_tier: 5,
        base_atk: 833,
        atk_ratio: Ratio {
            numerator: 19159,
            denominator: 10,
        },
        hp: 16362,
        def: 824,
        ability_id: "GA_Edgar_Skill",
        damage_effect_id: "GE_Player_Edgar_Skill_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 1371,
            denominator: 1000,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
