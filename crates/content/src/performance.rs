//! Opt-in development measurements; absent from ordinary engine builds.
use std::{cell::RefCell, collections::BTreeMap};

#[derive(Default, serde::Serialize)]
pub struct Measurement {
    pub calls: u64,
    pub nanoseconds: u128,
}

thread_local! {
    static MEASUREMENTS: RefCell<BTreeMap<&'static str, Measurement>> = const { RefCell::new(BTreeMap::new()) };
}

pub struct Span {
    name: &'static str,
    #[cfg(not(target_arch = "wasm32"))]
    start: std::time::Instant,
}

impl Span {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            #[cfg(not(target_arch = "wasm32"))]
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        MEASUREMENTS.with(|measurements| {
            let mut measurements = measurements.borrow_mut();
            let value = measurements.entry(self.name).or_default();
            value.calls += 1;
            #[cfg(not(target_arch = "wasm32"))]
            {
                value.nanoseconds += self.start.elapsed().as_nanos();
            }
        });
    }
}

/// Drain measurements between repetitions. WASM records counts only; host
/// adapters own its monotonic clock rather than `std::time::Instant`.
pub fn take() -> BTreeMap<&'static str, Measurement> {
    MEASUREMENTS.with(|measurements| std::mem::take(&mut *measurements.borrow_mut()))
}
