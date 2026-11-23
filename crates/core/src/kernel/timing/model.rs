pub const PPM_SCALE: u64 = 1_000_000;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum TimingError {
    DeadlineOverflow,
    InvalidRate(u32),
    InvalidCurve(&'static str),
    TimeReversal,
}
impl std::fmt::Display for TimingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for TimingError {}
