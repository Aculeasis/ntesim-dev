//! Historical manual preview inputs, not a complete or strict game adapter.
//! Recommendation source: https://www.prydwen.gg/neverness-to-everness/characters/
use crate::registry::PreviewCharacterProfile;
use crate::simulation::GearProcConfig;
use ntesim_core::formula::Ratio;

pub(crate) fn profile() -> PreviewCharacterProfile {
    PreviewCharacterProfile {
        slug: "mint",
        character_id: "character:1019",
        awakening: ntesim_data::AwakeningSelection::prefix(6),
        weapon_id: "weapon:fork_lingganzhongjiezhe",
        set_id: Some("set:Suit2"),
        weapon_tier: 5,
        base_atk: 1008,
        atk_ratio: Ratio {
            numerator: 46383,
            denominator: 20,
        },
        hp: 12120,
        def: 727,
        ability_id: "GA_Mint019_Skill",
        damage_effect_id: "GE_Player_Mint_Skill1_Damage_New",
        external_atk_rate_claim: Ratio {
            numerator: 833,
            denominator: 500,
        },
        energy_cost: 0,
        gear_procs: vec![
            GearProcConfig::MindRoyaleEnergy {
                energy_grant: 18,
                cooldown_us: 10_000_000,
            },
            GearProcConfig::FirefliesCritDamage {
                crit_damage_per_stack: Ratio {
                    numerator: 6,
                    denominator: 100,
                },
                max_stacks: 3,
                duration_us: 15_000_000,
            },
        ],
    }
}
