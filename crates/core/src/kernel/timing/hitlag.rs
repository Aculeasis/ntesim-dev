//! Checked actor-clock projection. Overlaps use the strongest slowdown, so
//! insertion order never changes time. Fractional PPM is retained across segments.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct HitlagWindow {
    pub actor_index: usize,
    pub start_real_us: u64,
    pub duration_real_us: u64,
    /// 0 = full freeze, 1_000_000 = normal. Speedup is not hitlag.
    pub dilated_rate_ppm: u32,
}
impl HitlagWindow {
    pub(super) fn end(&self) -> Result<u64, TimingError> {
        if u64::from(self.dilated_rate_ppm) > PPM_SCALE {
            return Err(TimingError::InvalidRate(self.dilated_rate_ppm));
        }
        self.start_real_us
            .checked_add(self.duration_real_us)
            .ok_or(TimingError::DeadlineOverflow)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct HitlagTracker {
    windows: Vec<HitlagWindow>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    curve_windows: Vec<HitlagCurveWindow>,
}
impl HitlagTracker {
    pub fn windows(&self) -> &[HitlagWindow] {
        &self.windows
    }

    pub fn curve_windows(&self) -> &[HitlagCurveWindow] {
        &self.curve_windows
    }

    pub fn add_curve_window(&mut self, window: HitlagCurveWindow) -> Result<(), TimingError> {
        window.validate()?;
        self.curve_windows.push(window);
        Ok(())
    }

    pub fn add_window(&mut self, window: HitlagWindow) -> Result<(), TimingError> {
        window.end()?;
        self.windows.push(window);
        Ok(())
    }
    /// Compose a global action freeze with local montage slowdown. Adjacent
    /// segments coalesce so intervening packets do not grow a timer-only series.
    pub fn add_freeze(
        &mut self,
        actor_index: usize,
        start_real_us: u64,
        duration_real_us: u64,
    ) -> Result<(), TimingError> {
        if duration_real_us == 0 {
            return Ok(());
        }
        start_real_us
            .checked_add(duration_real_us)
            .ok_or(TimingError::DeadlineOverflow)?;
        if let Some(previous) = self.windows.last_mut()
            && previous.actor_index == actor_index
            && previous.dilated_rate_ppm == 0
            && previous.end()? == start_real_us
        {
            previous.duration_real_us = previous
                .duration_real_us
                .checked_add(duration_real_us)
                .ok_or(TimingError::DeadlineOverflow)?;
            return Ok(());
        }
        self.add_window(HitlagWindow {
            actor_index,
            start_real_us,
            duration_real_us,
            dilated_rate_ppm: 0,
        })
    }
    pub fn elapsed_actor_time_us(
        &self,
        actor_index: usize,
        start_real_us: u64,
        end_real_us: u64,
    ) -> Result<u64, TimingError> {
        if start_real_us > end_real_us {
            return Err(TimingError::TimeReversal);
        }
        if !self.curve_windows.is_empty() {
            return super::hitlag_curve::elapsed(
                &self.windows,
                &self.curve_windows,
                actor_index,
                start_real_us,
                end_real_us,
            );
        }
        let mut windows = Vec::new();
        let mut boundaries = vec![start_real_us, end_real_us];
        for w in &self.windows {
            let end = w.end()?;
            if w.actor_index == actor_index && w.start_real_us < end_real_us && end > start_real_us
            {
                boundaries.push(w.start_real_us.max(start_real_us));
                boundaries.push(end.min(end_real_us));
                windows.push((w, end));
            }
        }
        boundaries.sort_unstable();
        boundaries.dedup();
        let mut weighted = 0u128;
        for pair in boundaries.windows(2) {
            let rate = windows
                .iter()
                .filter(|(w, end)| w.start_real_us <= pair[0] && pair[0] < *end)
                .map(|(w, _)| u64::from(w.dilated_rate_ppm))
                .min()
                .unwrap_or(PPM_SCALE);
            weighted += u128::from(pair[1] - pair[0]) * u128::from(rate);
        }
        // All rates <= PPM_SCALE, so elapsed cannot exceed the supplied u64 span.
        Ok((weighted / u128::from(PPM_SCALE)) as u64)
    }
    /// Earliest representable real deadline reaching the integer actor offset.
    /// Binary inversion uses the same exact piecewise integration as observation.
    pub fn resolve_real_deadline(
        &self,
        actor_index: usize,
        start_real_us: u64,
        target_offset_us: u64,
    ) -> Result<u64, TimingError> {
        if !self.curve_windows.is_empty() {
            return super::hitlag_curve::resolve(
                &self.windows,
                &self.curve_windows,
                actor_index,
                start_real_us,
                target_offset_us,
            );
        }
        if self.elapsed_actor_time_us(actor_index, start_real_us, u64::MAX)? < target_offset_us {
            return Err(TimingError::DeadlineOverflow);
        }
        let mut lo = start_real_us;
        let mut hi = u64::MAX;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if self.elapsed_actor_time_us(actor_index, start_real_us, mid)? >= target_offset_us {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        Ok(lo)
    }
}
