use super::ExactFraction;
use ntesim_core::combat::DamageKind;
use ntesim_data::native_action_metadata::{
    FieldState, NativeAbilityMetadata, NativeDamageEffectMetadata, NativeEvidence, RowState,
};
use std::collections::{BTreeMap, BTreeSet};

mod binding;
mod native;
mod research;
mod types;

pub(crate) use binding::resolve_action_metadata_with_owner;
use binding::{owner_packet_semantics, resolve_action_packet_binding};
use native::native_fixed_crit_rate;
pub use native::resolve_action_metadata;
pub(crate) use native::{native_input_role, resolve_packet_for_action, resolve_packet_metadata};
pub(crate) use research::resolve_requested_action_packet;
pub use types::*;

#[cfg(test)]
use native::resolve_native_action_packet;
#[cfg(test)]
mod tests;
