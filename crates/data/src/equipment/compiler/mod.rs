use super::group::weapon_group_binding;
use crate::{
    character,
    native::{Scalar, accumulate_scalar, text},
    weapon,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

mod model;
pub use model::{Request, TABLES};
mod pipeline;
pub use pipeline::compile;
mod sets;
pub(crate) use sets::set_stats;
mod derived;
pub(crate) use derived::derive_stats;
