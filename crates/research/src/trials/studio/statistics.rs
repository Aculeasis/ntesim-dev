//! Empirical presentation statistics; exact combat totals remain in raw reports.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Statistics {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
}

pub(super) fn summarize(mut values: Vec<f64>) -> Statistics {
    values.sort_by(f64::total_cmp);
    let count = values.len();
    let mean = if values[0] == values[count - 1] {
        values[0]
    } else {
        values.iter().sum::<f64>() / count as f64
    };
    let percentile = |p: f64| {
        let i = (count - 1) as f64 * p;
        let lower = values[i.floor() as usize];
        lower + (values[i.ceil() as usize] - lower) * i.fract()
    };
    Statistics {
        count,
        min: values[0],
        max: values[count - 1],
        mean,
        std: (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64).sqrt(),
        p25: percentile(0.25),
        p50: percentile(0.5),
        p75: percentile(0.75),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discrete_population_and_interpolated_quantiles() {
        let s = summarize(vec![0., 10., 10., 20., 30.]);
        assert_eq!((s.mean, s.p25, s.p50, s.p75), (14., 10., 10., 20.));
        assert!((s.std - 104_f64.sqrt()).abs() < 1e-12);
        let s = summarize(vec![0.1; 100]);
        assert_eq!((s.mean, s.std, s.min, s.max), (0.1, 0., 0.1, 0.1));
    }
}
