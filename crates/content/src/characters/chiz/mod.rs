pub(crate) mod awakenings;
pub(crate) mod barren;
pub(crate) mod cancel;
pub(crate) mod clock;
pub(crate) mod combat;
pub(crate) mod events;
pub(crate) mod finance;
pub(crate) mod market;
mod parameters;
pub(crate) mod passives;
pub(crate) mod pressed;
pub(crate) mod preview;
pub(crate) mod resource;
pub(crate) mod secondary;
pub(crate) mod skill;
mod source;
mod values;

pub(crate) fn financial_source_assumptions() -> std::collections::BTreeMap<String, String> {
    [market::MODEL, finance::MODEL, pressed::MODEL, clock::MODEL]
        .into_iter()
        .map(|id| (id.into(), id.description().into()))
        .collect()
}

pub(crate) fn buff_duration(ability: &str) -> Option<u64> {
    if ability == "GA_Chiichan073_UltraSkill" {
        Some(parameters::PARAMETERS.market_duration_us)
    } else {
        None
    }
}

pub(crate) fn primary_packet(ability: &str) -> Option<&'static str> {
    match ability {
        "GA_Chiichan073_Melee" => Some("GE_Player_Chiichan073_Melee1_Damage"),
        "GA_Chiichan073_Skill" => Some("GE_Player_Chiichan073_Skill1_1Damage"),
        "GA_Chiichan073_UltraSkill" => Some("GE_Player_Chiichan073_UltraSkill1_Damage"),
        _ => None,
    }
}

// Common.EndAbilitySkill; retrieved configuration, selected timer model.
pub(crate) const CINEMATIC_RELEASE: super::cinematics::Release =
    super::cinematics::Release::Notify {
        progress_us: 4250937,
        montage: "char_rabbit_ultraskill_1_Montage",
    };
