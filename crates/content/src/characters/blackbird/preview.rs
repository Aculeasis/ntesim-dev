//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "blackbird",
        character_id: "character:1042",
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        weapon_id: "weapon:fork_twinbirds",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1118,
        atk_ratio: Ratio {
            numerator: 25714,
            denominator: 10,
        },
        hp: 15514,
        def: 884,
        ability_id: "GA_BlackBird_Skill",
        damage_effect_id: "GE_Player_BlackBird_Skill_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 9,
            denominator: 5,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
