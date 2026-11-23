//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "sakiri",
        character_id: "character:1003",
        awakening: ntesim_data::AwakeningSelection::prefix(1),
        weapon_id: "weapon:fork_mofeikesi",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1094,
        atk_ratio: Ratio {
            numerator: 48331,
            denominator: 20,
        },
        hp: 16483,
        def: 909,
        ability_id: "GA_Sagiri_UltraSkill",
        damage_effect_id: "GE_Player_Sagiri_UltraSkill1_Damage",
        external_atk_rate_claim: Ratio {
            numerator: 0,
            denominator: 1,
        },
        energy_cost: 100,
        gear_procs: Vec::new(),
    }
}
