use crate::HypothesisId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DotKind {
    Heartwrench,
    VileAsh,
}

impl DotKind {
    pub(crate) fn from_packet(packet: &str) -> Option<Self> {
        match packet {
            "GE_Player_Zankou_DotDamage" => Some(Self::Heartwrench),
            "GE_Player_Zankou_DotUltraDamage" => Some(Self::VileAsh),
            _ => None,
        }
    }
    pub(crate) fn packet(self) -> &'static str {
        match self {
            Self::Heartwrench => "GE_Player_Zankou_DotDamage",
            Self::VileAsh => super::vile_ash::DOT_ULTRA_DAMAGE_EFFECT_ID,
        }
    }
    pub(crate) fn native_ability(self) -> &'static str {
        match self {
            Self::Heartwrench => "GA_Zankou_Melee",
            Self::VileAsh => super::vile_ash::ULTRA_ABILITY_ID,
        }
    }
    pub(crate) fn hypothesis(self) -> HypothesisId {
        match self {
            Self::Heartwrench => super::forms::MODEL,
            Self::VileAsh => super::vile_ash::VILE_ASH_SCHEDULING_HYPOTHESIS_ID,
        }
    }
    pub(crate) fn application_stacks(self) -> u32 {
        match self {
            Self::Heartwrench => 1,
            Self::VileAsh => super::vile_ash::VILE_ASH_ULTRA_STACKS,
        }
    }
    pub(crate) fn active_at(self, time_us: u64, deadline_us: u64) -> bool {
        match self {
            Self::Heartwrench => time_us < deadline_us,
            Self::VileAsh => time_us < deadline_us,
        }
    }
    pub(crate) fn trains(
        self,
        state: &mut super::awakenings::State,
    ) -> &mut std::collections::BTreeMap<usize, super::vile_ash::VileAshTrain> {
        match self {
            Self::Heartwrench => &mut state.heartwrench_trains,
            Self::VileAsh => &mut state.vile_ash_trains,
        }
    }
}
