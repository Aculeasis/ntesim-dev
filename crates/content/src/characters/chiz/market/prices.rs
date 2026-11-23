//! Checked finite generator; signed hundredths preserve the source support.
use ntesim_core::kernel::rng::{Rng, WORD_RANGE};

pub(super) const RECORDED: [i16; 14] = [25, -8, 38, 50, 14, -29, 19, 6, -28, 32, 25, -8, -32, 23];
const RISE: [i16; 9] = [25, 38, 50, 14, 19, 6, 32, 25, 23];
const FALL: [i16; 5] = [-8, -29, -28, -8, -32];
const STREAM: &str = "chiz.market";

pub(super) fn new(seed: u64, owner: usize) -> Result<Rng, String> {
    Rng::new(seed, [(STREAM.into(), 1073 + owner as u64)].into())
}

pub(super) fn generate(rng: &mut Rng, a3: bool, recorded: bool) -> Result<[i16; 14], String> {
    let mut result = RECORDED;
    for price in &mut result {
        let sign = u64::from(rng.draw(STREAM)?.value);
        let index = u64::from(rng.draw(STREAM)?.value);
        // Exactly two draws per price: compare/multiply rather than rejection
        // sampling, following the selected floor(u*n) generator.
        if !recorded {
            *price = select(sign, index, a3);
        }
    }
    Ok(result)
}

fn select(sign: u64, index: u64, a3: bool) -> i16 {
    let support: &[i16] = if sign * 10 < WORD_RANGE * if a3 { 6 } else { 5 } {
        &RISE
    } else {
        &FALL
    };
    support[(index * support.len() as u64 / WORD_RANGE) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_normalized_weights_and_floor_selection_keep_duplicate_support_entries() {
        assert!(select(2_147_483_647, 0, false) > 0);
        assert!(select(2_147_483_648, 0, false) < 0);
        assert!(select(2_576_980_377, 0, true) > 0);
        assert!(select(2_576_980_378, 0, true) < 0);
        for (support, sign) in [(&RISE[..], 0), (&FALL[..], WORD_RANGE - 1)] {
            for (index, &price) in support.iter().enumerate() {
                let word = (index as u64 * WORD_RANGE).div_ceil(support.len() as u64);
                assert_eq!(select(sign, word, false), price);
            }
        }
    }
    #[test]
    fn recorded_trace_advances_twenty_eight_draws_and_continues_independently() {
        let mut rng = new(42, 0).unwrap();
        let mut reference = rng.clone();
        assert_eq!(generate(&mut rng, false, true).unwrap(), RECORDED);
        for _ in 0..28 {
            reference.draw(STREAM).unwrap();
        }
        assert_eq!(rng, reference);
        let next = generate(&mut rng, true, false).unwrap();
        assert_eq!(next, generate(&mut reference, true, false).unwrap());
        assert!(next.iter().all(|value| RECORDED.contains(value)));
        assert_ne!(
            next,
            generate(&mut new(42, 0).unwrap(), true, false).unwrap()
        );
    }
}
