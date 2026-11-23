use super::{ExactFraction, GearProcConfig};
use crate::formula::Ratio;

mod atk_stats;
mod energy;
mod hp_stats;
mod origin;
mod parentage;
pub(crate) use parentage::validate_hit_parent;
mod report;
mod request;
mod resources;
mod source;
mod time;
mod timeline;

pub use atk_stats::*;
pub use energy::*;
pub use hp_stats::*;
pub use origin::*;
pub use report::*;
pub use request::*;
pub use resources::*;
pub use source::*;
pub use time::*;
pub use timeline::*;

#[cfg(test)]
mod tests;
