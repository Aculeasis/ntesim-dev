pub(crate) mod awakenings;
pub(crate) mod branches;
pub(crate) mod cancel;
pub(crate) mod clock;
pub(crate) mod entry_jump;
pub(crate) mod etd;
pub(crate) mod events;
pub(crate) mod fadia_assist;
pub(crate) mod hold;
pub(crate) mod interrupt;
pub(crate) mod plunge;
pub(crate) mod preview;
pub(crate) mod riposte;
pub(crate) mod scorch;
pub(crate) mod skill;
pub(crate) mod support;
pub(crate) mod survival;
pub(crate) mod timing;
pub(crate) mod words;

// Global GE_Player_Cang_UltraSkill_Damage_period: one tick every 2 real seconds.
pub(super) const DOMAIN_TICK_INTERVAL_US: u64 = 2_000_000;

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Cang_Melee" => Some("GE_Player_Cang_Melee1_Damage"),
        "GA_Cang_Skill" => Some("GE_Player_Cang_Skill_Damage"),
        "GA_Cang_UltraSkill" => Some("GE_Player_Cang_UltraSkill2_Damage"),
        _ => None,
    }
}

pub(crate) fn buff_duration(
    awakening: &ntesim_data::AwakeningSelection,
    ability: &str,
) -> Option<u64> {
    // Global Cang_FieldDuaration / Cang_FieldDuaration_Lv5: 12s / 16s.
    // The domain scheduler and preview share these exact microsecond values.
    match ability {
        "GA_Cang_UltraSkill" => {
            if awakening.has_effect(5) {
                Some(16_000_000)
            } else {
                Some(12_000_000)
            }
        }
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 2854959,
        montage: "Cang_UltralSkill",
    };
