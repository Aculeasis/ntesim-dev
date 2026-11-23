use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsSnowmanV1;
