//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "lacrimosa",
        character_id: "character:1004",
        awakening: ntesim_data::AwakeningSelection::prefix(0),
        weapon_id: "weapon:fork_Rose",
        set_id: None,
        weapon_tier: 1,
        base_atk: 1086,
        atk_ratio: Ratio {
            numerator: 24978,
            denominator: 10,
        },
        hp: 15998,
        def: 909,
        ability_id: "GA_Lacrimosa_Skill",
        damage_effect_id: "GE_Boss_017_act01_Dmg_05a_Steal_BP",
        external_atk_rate_claim: Ratio {
            numerator: 542,
            denominator: 125,
        },
        energy_cost: 0,
        gear_procs: Vec::new(),
    }
}
