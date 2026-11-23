//! Finite replay montage state is independent of the real owner's current
//! action. Resource, form, buff and deployed-producer state stays on the owner.
use super::RuntimeState;
#[derive(Clone, Debug)]
pub(crate) enum ReplayClock {
    Mint(Option<Box<super::mint::clock::Clock>>),
    Nanally(super::nanally::clock::Clock),
    Iroi(Option<Box<super::iroi::clock::Clock>>),
    Daffodil(Option<Box<super::daffodil::clock::Clock>>),
    Edgar(Option<Box<super::edgar::clock::Clock>>),
    Blackbird(super::blackbird::clock::State),
    Haniel(Option<Box<super::haniel::clock::Clock>>),
    Fadia(Option<Box<super::fadia::clock::Clock>>),
    Zankou(Option<super::zankou::clock::Clock>),
    Hathor(Option<Box<super::hathor::clock::Clock>>),
    Baicang(Box<super::baicang::clock::State>),
    Chiz {
        clock: Option<Box<super::chiz::clock::Clock>>,
        pressed: Option<super::chiz::pressed::FiniteClock>,
    },
    Chaos(Option<super::chaos::clock::Clock>),
}
impl ReplayClock {
    pub(crate) fn take(state: &mut RuntimeState) -> Option<Self> {
        match state {
            RuntimeState::Mint(state) => Some(Self::Mint(std::mem::take(&mut state.clock))),
            RuntimeState::Nanally(state) => Some(Self::Nanally(std::mem::take(&mut state.clock))),
            RuntimeState::Iroi(state) => Some(Self::Iroi(std::mem::take(&mut state.clock))),
            RuntimeState::Daffodil(state) => Some(Self::Daffodil(std::mem::take(&mut state.clock))),
            RuntimeState::Edgar(state) => Some(Self::Edgar(std::mem::take(&mut state.clock))),
            RuntimeState::Blackbird(state) => {
                Some(Self::Blackbird(std::mem::take(&mut state.clock)))
            }
            RuntimeState::Haniel(state) => Some(Self::Haniel(std::mem::take(&mut state.clock))),
            RuntimeState::Fadia(state) => Some(Self::Fadia(std::mem::take(&mut state.clock))),
            RuntimeState::Zankou(state) => Some(Self::Zankou(std::mem::take(&mut state.clock))),
            RuntimeState::Hathor(state) => Some(Self::Hathor(std::mem::take(&mut state.clock))),
            RuntimeState::Baicang(state) => Some(Self::Baicang(std::mem::take(&mut state.clock))),
            RuntimeState::Chiz(state) => Some(Self::Chiz {
                clock: std::mem::take(&mut state.clock),
                pressed: state.pressed.as_mut().map(|pressed| pressed.take_finite()),
            }),
            RuntimeState::Chaos(state) => Some(Self::Chaos(std::mem::take(&mut state.clock))),
            _ => None,
        }
    }
    pub(crate) fn restore(self, state: &mut RuntimeState) {
        match (self, state) {
            (Self::Mint(clock), RuntimeState::Mint(state)) => state.clock = clock,
            (Self::Nanally(clock), RuntimeState::Nanally(state)) => state.clock = clock,
            (Self::Iroi(clock), RuntimeState::Iroi(state)) => state.clock = clock,
            (Self::Daffodil(clock), RuntimeState::Daffodil(state)) => state.clock = clock,
            (Self::Edgar(clock), RuntimeState::Edgar(state)) => state.clock = clock,
            (Self::Blackbird(clock), RuntimeState::Blackbird(state)) => state.clock = clock,
            (Self::Haniel(clock), RuntimeState::Haniel(state)) => state.clock = clock,
            (Self::Fadia(clock), RuntimeState::Fadia(state)) => state.clock = clock,
            (Self::Zankou(clock), RuntimeState::Zankou(state)) => state.clock = clock,
            (Self::Hathor(clock), RuntimeState::Hathor(state)) => state.clock = clock,
            (Self::Baicang(clock), RuntimeState::Baicang(state)) => state.clock = clock,
            (Self::Chiz { clock, pressed }, RuntimeState::Chiz(state)) => {
                state.clock = clock;
                if let Some(pressed) = pressed {
                    state
                        .pressed
                        .as_mut()
                        .expect("persistent Chiz stock account")
                        .restore_finite(pressed);
                }
            }
            (Self::Chaos(clock), RuntimeState::Chaos(state)) => state.clock = clock,
            _ => unreachable!("replay clock owner changed character"),
        }
    }
    pub(crate) fn freeze(&mut self, owner: usize, start: u64, duration: u64) -> Result<(), String> {
        match self {
            Self::Mint(clock) => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
            }
            Self::Nanally(clock) => clock.global_freeze(owner, start, duration)?,
            Self::Iroi(clock) => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
            }
            Self::Daffodil(clock) => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
            }
            Self::Edgar(clock) => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
            }
            Self::Blackbird(clock) => clock.global_freeze(owner, start, duration)?,
            Self::Haniel(clock) => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
            }
            Self::Fadia(clock) => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
            }
            Self::Zankou(clock) => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
            }
            Self::Hathor(clock) => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
            }
            Self::Baicang(clock) => clock.global_freeze(owner, start, duration)?,
            Self::Chiz { clock, pressed } => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
                if let Some(pressed) = pressed {
                    pressed.freeze(owner, start, duration)?;
                }
            }
            Self::Chaos(clock) => {
                if let Some(clock) = clock {
                    clock.global_freeze(owner, start, duration)?;
                }
            }
        }
        Ok(())
    }
}
