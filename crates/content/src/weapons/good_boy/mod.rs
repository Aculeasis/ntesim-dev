mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod cast;
mod control;
pub(crate) mod projection;
pub(crate) use control::on_control;

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::WeaponsGoodBoyV3;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct State {
    command_index: Option<usize>,
    expires_at_us: u64,
    upgraded: bool,
}

impl State {
    pub(crate) fn pause_world(&mut self, from: u64, delta: u64) -> Result<(), String> {
        crate::simulation::timer_state::future(&mut self.expires_at_us, from, delta)
    }
}
