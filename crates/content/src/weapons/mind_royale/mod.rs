mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod callback;
pub(crate) mod cast;
pub(crate) mod projection;
#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) ready_at_us: u64,
}
