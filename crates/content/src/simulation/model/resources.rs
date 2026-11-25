use crate::formula::Ratio;
use serde::{Deserialize, Serialize};

/// Authored owner resource balances. Omission retains the owner's combat-entry
/// initialization; explicit zero remains an authored value in saved inputs.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameInitialResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timepiece: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grains: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loans: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub malice: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crime: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moon: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plot: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bullets: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resonance: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub karma: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stocks: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kindling: Option<Ratio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub string_energy: Option<Ratio>,
}

impl GameInitialResources {
    pub fn is_empty(&self) -> bool {
        self.timepiece.is_none()
            && self.grains.is_none()
            && self.loans.is_none()
            && self.malice.is_none()
            && self.crime.is_none()
            && self.power.is_none()
            && self.moon.is_none()
            && self.plot.is_none()
            && self.bullets.is_none()
            && self.resonance.is_none()
            && self.karma.is_none()
            && self.keys.is_none()
            && self.stocks.is_none()
            && self.kindling.is_none()
            && self.string_energy.is_none()
    }
}
