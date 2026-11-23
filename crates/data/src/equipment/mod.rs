mod compiler;
mod group;

pub(crate) use compiler::derive_stats;
pub(crate) use compiler::set_stats;
pub use compiler::{Request, TABLES, compile};
pub(crate) use group::weapon_group_binding;

#[cfg(test)]
mod tests;
