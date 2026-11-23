//! Exact integration of the strongest overlapping piecewise-linear slowdown.
//! All inversion remains bounded deadline arithmetic in HitlagTracker.
use super::{HitlagWindow, PPM_SCALE, TimingError};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitlagRatePoint {
    pub offset_us: u64,
    pub rate_ppm: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitlagCurveWindow {
    pub actor_index: usize,
    pub start_real_us: u64,
    pub duration_real_us: u64,
    /// Linear interpolation; endpoint rates remain constant outside their knots.
    pub points: Vec<HitlagRatePoint>,
}
impl HitlagCurveWindow {
    pub(super) fn end(&self) -> Result<u64, TimingError> {
        self.start_real_us
            .checked_add(self.duration_real_us)
            .ok_or(TimingError::DeadlineOverflow)
    }
    pub(super) fn validate(&self) -> Result<(), TimingError> {
        self.end()?;
        if self.points.first().is_none_or(|p| p.offset_us != 0) {
            return Err(TimingError::InvalidCurve("curve must start at offset zero"));
        }
        if self
            .points
            .windows(2)
            .any(|pair| pair[0].offset_us >= pair[1].offset_us)
        {
            return Err(TimingError::InvalidCurve(
                "curve knot offsets must strictly increase",
            ));
        }
        for point in &self.points {
            if u64::from(point.rate_ppm) > PPM_SCALE {
                return Err(TimingError::InvalidRate(point.rate_ppm));
            }
        }
        Ok(())
    }
    fn line_at(&self, time: u64) -> Line {
        let local = time - self.start_real_us;
        let index = self.points.partition_point(|p| p.offset_us <= local) - 1;
        let current = &self.points[index];
        if let Some(next) = self.points.get(index + 1) {
            let slope = (q(next.rate_ppm.into()) - q(current.rate_ppm.into()))
                / q(next.offset_us - current.offset_us);
            let value = q(current.rate_ppm.into()) + &slope * q(local - current.offset_us);
            Line { value, slope }
        } else {
            Line::constant(current.rate_ppm.into())
        }
    }
}

fn q(value: u64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}
struct Line {
    value: BigRational,
    slope: BigRational,
}
impl Line {
    fn constant(value: u64) -> Self {
        Self {
            value: q(value),
            slope: BigRational::zero(),
        }
    }
    fn at(&self, offset: &BigRational) -> BigRational {
        &self.value + &self.slope * offset
    }
}

struct Piece {
    from: BigRational,
    to: BigRational,
    rate: BigRational,
    slope: BigRational,
}
impl Piece {
    fn integral(&self, time: &BigRational) -> BigRational {
        let until = if time < &self.to { time } else { &self.to };
        if until <= &self.from {
            return BigRational::zero();
        }
        let width = until - &self.from;
        (&self.rate * &width + &self.slope * &width * &width / q(2)) / q(PPM_SCALE)
    }
}

fn minimum_pieces(lines: &[Line], length: u64, origin: u64) -> Vec<Piece> {
    let end = q(length);
    let mut cuts = vec![BigRational::zero(), end.clone()];
    // Rates can cross inside a native knot interval. Integrating a minimum of
    // endpoints alone would overestimate progress when two curves cross.
    for (index, left) in lines.iter().enumerate() {
        for right in &lines[..index] {
            let slope = &left.slope - &right.slope;
            if slope.is_zero() {
                continue;
            }
            let crossing = (&right.value - &left.value) / slope;
            if crossing > BigRational::zero() && crossing < end {
                cuts.push(crossing);
            }
        }
    }
    cuts.sort();
    cuts.dedup();
    let mut pieces = Vec::new();
    for pair in cuts.windows(2) {
        let midpoint = (&pair[0] + &pair[1]) / q(2);
        let line = lines
            .iter()
            .min_by_key(|line| line.at(&midpoint))
            .expect("normal clock rate is always present");
        pieces.push(Piece {
            from: q(origin) + &pair[0],
            to: q(origin) + &pair[1],
            rate: line.at(&pair[0]),
            slope: line.slope.clone(),
        });
    }
    pieces
}

fn plan(
    windows: &[HitlagWindow],
    curves: &[HitlagCurveWindow],
    actor: usize,
    start: u64,
    end: u64,
) -> Result<Vec<Piece>, TimingError> {
    if start > end {
        return Err(TimingError::TimeReversal);
    }
    let mut boundaries = vec![start, end];
    let mut constants = Vec::new();
    let mut active_curves = Vec::new();
    for window in windows {
        let window_end = window.end()?;
        if window.actor_index == actor && window.start_real_us < end && window_end > start {
            boundaries.extend([window.start_real_us.max(start), window_end.min(end)]);
            constants.push((window, window_end));
        }
    }
    for curve in curves {
        let curve_end = curve.end()?;
        if curve.actor_index == actor && curve.start_real_us < end && curve_end > start {
            boundaries.extend([curve.start_real_us.max(start), curve_end.min(end)]);
            for point in &curve.points {
                if point.offset_us >= curve.duration_real_us {
                    break;
                }
                let knot = curve.start_real_us + point.offset_us;
                if knot > start && knot < end {
                    boundaries.push(knot);
                }
            }
            active_curves.push((curve, curve_end));
        }
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    let mut pieces = Vec::new();
    for pair in boundaries.windows(2) {
        let mut lines = vec![Line::constant(PPM_SCALE)];
        for &(window, window_end) in &constants {
            if window.start_real_us <= pair[0] && pair[0] < window_end {
                lines.push(Line::constant(window.dilated_rate_ppm.into()));
            }
        }
        for &(curve, curve_end) in &active_curves {
            if curve.start_real_us <= pair[0] && pair[0] < curve_end {
                lines.push(curve.line_at(pair[0]));
            }
        }
        pieces.extend(minimum_pieces(&lines, pair[1] - pair[0], pair[0]));
    }
    Ok(pieces)
}

pub(super) fn elapsed(
    windows: &[HitlagWindow],
    curves: &[HitlagCurveWindow],
    actor: usize,
    start: u64,
    end: u64,
) -> Result<u64, TimingError> {
    plan(windows, curves, actor, start, end)?
        .iter()
        .map(|piece| piece.integral(&piece.to))
        .sum::<BigRational>()
        .to_integer()
        .to_u64()
        .ok_or(TimingError::DeadlineOverflow)
}

pub(super) fn resolve(
    windows: &[HitlagWindow],
    curves: &[HitlagCurveWindow],
    actor: usize,
    start: u64,
    offset: u64,
) -> Result<u64, TimingError> {
    if offset == 0 {
        return Ok(start);
    }
    let mut remaining = q(offset);
    for piece in plan(windows, curves, actor, start, u64::MAX)? {
        let available = piece.integral(&piece.to);
        if remaining > available {
            remaining -= available;
            continue;
        }
        let mut lo = piece
            .from
            .to_integer()
            .to_u64()
            .ok_or(TimingError::DeadlineOverflow)?;
        let mut hi = piece
            .to
            .ceil()
            .to_integer()
            .to_u64()
            .ok_or(TimingError::DeadlineOverflow)?;
        // This is inversion of one analytic integral, never an event/readiness poll.
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if piece.integral(&q(mid)) >= remaining {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        return Ok(lo);
    }
    Err(TimingError::DeadlineOverflow)
}
