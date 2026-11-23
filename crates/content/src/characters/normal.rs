//! Read-only ordinary-chain selection, shared with the owner's cast path.
use super::RuntimeState;
pub(crate) const RESET_GAP_US: u64 = 2_000_000;

/// A queued deadline may be in the future. The exact two-second gap continues;
/// only a strictly longer idle resets the owner's stored ordinary stage.
pub(crate) fn after_gap(stage: usize, started: Option<u64>, now: u64) -> usize {
    if started.is_some_and(|started| now.saturating_sub(started) > RESET_GAP_US) {
        0
    } else {
        stage
    }
}

/// These owners continue only at the exact native queued-input boundary.
pub(crate) fn at_boundary(stage: usize, started: Option<u64>, now: u64) -> usize {
    if continues_at(started, now) { stage } else { 0 }
}

pub(crate) fn continues_at(started: Option<u64>, now: u64) -> bool {
    started == Some(now)
}

impl RuntimeState {
    pub(crate) fn next_normal(&self, now: u64) -> Result<u64, String> {
        macro_rules! ordinary {
            ($state:ident) => {
                after_gap($state.normal_stage, $state.normal_started_us, now)
            };
        }
        let stage = match self {
            Self::Generic => return Err("No native ordinary chain for this actor.".into()),
            Self::Mint(s) => ordinary!(s),
            Self::Sakiri(s) => ordinary!(s),
            Self::Zero(s) => ordinary!(s),
            Self::Iroi(s) if s.clock.is_some() => {
                at_boundary(s.normal_stage, s.normal_started_us, now)
            }
            Self::Iroi(s) => ordinary!(s),
            Self::Daffodil(s) => ordinary!(s),
            Self::Nanally(s) => ordinary!(s),
            Self::Hotori(s) if s.timepiece.mode_active(now) => {
                super::hotori::katana::next_stage(s.timepiece.katana_count).unwrap_or(0)
            }
            Self::Hotori(s) => ordinary!(s),
            Self::Edgar(s) if s.clock.is_some() => {
                at_boundary(s.normal_stage, s.normal_started_us, now)
            }
            Self::Edgar(s) => ordinary!(s),
            Self::Blackbird(s) if s.form.is_some() => {
                after_gap(s.evolve_melee_stage, s.evolve_melee_started_us, now)
            }
            Self::Blackbird(s) => ordinary!(s),
            Self::Skia(s) => ordinary!(s),
            Self::Haniel(s) if s.clock.is_some() => {
                at_boundary(s.normal_stage, s.normal_started_us, now)
            }
            Self::Haniel(s) => ordinary!(s),
            Self::Fadia(s) if s.full_kit && s.lilith_until_us.is_some_and(|end| now < end) => {
                s.lilith_stage
            }
            Self::Fadia(s) => ordinary!(s),
            Self::Lacrimosa(s) => ordinary!(s),
            Self::Zankou(s) => ordinary!(s),
            Self::Hathor(s) if s.clock.is_some() => {
                at_boundary(s.normal_stage, s.normal_started_us, now)
            }
            Self::Hathor(s) => ordinary!(s),
            Self::Shinku(s) => ordinary!(s),
            Self::Baicang(s) => ordinary!(s),
            Self::Aurelia(s) => ordinary!(s),
            Self::Chiz(s) => ordinary!(s),
            Self::Jiuyuan(s) => ordinary!(s),
            Self::Chaos(s) if s.entry_normal_ready => 0,
            Self::Chaos(s) => ordinary!(s),
            Self::Linko(s) => ordinary!(s),
            Self::Adler(s) => ordinary!(s),
        };
        Ok(stage as u64 + 1)
    }
}
