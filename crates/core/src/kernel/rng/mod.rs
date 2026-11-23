//! Versioned reproducible streams; these are simulator policies, not NTE rules.
// PCG32 transition/seeding adapted from pcg-c-basic, Copyright (c) 2014
// Melissa O'Neill. Licensed under Apache-2.0; see licenses/pcg32-APACHE-2.0.txt
// and licenses/pcg32-NOTICE.txt. Stream validation and receipts are ntesim code.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: &str = "pcg32-xsh-rr-64-32-v1";
pub const WORD_RANGE: u64 = 1 << 32;
pub const MAX_ATTEMPTS: usize = 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Draw {
    pub stream: String,
    /// Zero-based raw draw index; initialization draws are excluded.
    pub index: u64,
    pub value: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    pub bound: u64,
    pub value: u64,
    /// Includes rejected raw words, in consumption order.
    pub draws: Vec<Draw>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct Stream {
    sequence: u64,
    state: u64,
    draws: u64,
}

impl Stream {
    fn word(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(6364136223846793005)
            .wrapping_add((self.sequence << 1) | 1);
        let shifted = (((old >> 18) ^ old) >> 27) as u32;
        shifted.rotate_right((old >> 59) as u32)
    }

    fn new(seed: u64, sequence: u64) -> Self {
        let mut stream = Self {
            sequence,
            state: 0,
            draws: 0,
        };
        stream.word();
        stream.state = stream.state.wrapping_add(seed);
        stream.word();
        stream
    }

    fn draw(&mut self, name: &str) -> Result<Draw, String> {
        let next = self
            .draws
            .checked_add(1)
            .ok_or("RNG draw counter overflow")?;
        let index = self.draws;
        let value = self.word();
        self.draws = next;
        Ok(Draw {
            stream: name.into(),
            index,
            value,
        })
    }
}

/// Explicit sequence selectors avoid implicit stream allocation by call order.
/// Serializable final state is an audit snapshot, not a restoration interface.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Rng {
    version: &'static str,
    seed: u64,
    streams: BTreeMap<String, Stream>,
}

impl Rng {
    pub fn new(seed: u64, sequences: BTreeMap<String, u64>) -> Result<Self, String> {
        let mut seen = BTreeSet::new();
        let mut streams = BTreeMap::new();
        for (name, sequence) in sequences {
            if name.is_empty() {
                return Err("RNG stream name is empty".into());
            }
            if sequence > u64::MAX >> 1 {
                return Err("RNG sequence selector exceeds 63 bits".into());
            }
            if !seen.insert(sequence) {
                return Err("RNG sequence selector is reused".into());
            }
            streams.insert(name, Stream::new(seed, sequence));
        }
        Ok(Self {
            version: VERSION,
            seed,
            streams,
        })
    }

    pub fn contains(&self, name: &str) -> bool {
        self.streams.contains_key(name)
    }

    pub fn draw(&mut self, name: &str) -> Result<Draw, String> {
        self.streams
            .get_mut(name)
            .ok_or("unknown RNG stream")?
            .draw(name)
    }

    /// Uniform integer in [0, bound), with exact rejection rather than modulo bias.
    /// Errors preserve the stream, including draws tentatively rejected.
    pub fn uniform(&mut self, name: &str, bound: u64) -> Result<Sample, String> {
        self.uniform_with_limit(name, bound, MAX_ATTEMPTS)
    }

    fn uniform_with_limit(
        &mut self,
        name: &str,
        bound: u64,
        limit: usize,
    ) -> Result<Sample, String> {
        if !(1..=WORD_RANGE).contains(&bound) {
            return Err("RNG bound must be in 1..=2^32".into());
        }
        let original = self.streams.get_mut(name).ok_or("unknown RNG stream")?;
        let mut candidate = original.clone();
        let threshold = WORD_RANGE % bound;
        let mut draws = Vec::new();
        for _ in 0..limit {
            let draw = candidate.draw(name)?;
            let word = u64::from(draw.value);
            draws.push(draw);
            if word >= threshold {
                *original = candidate;
                return Ok(Sample {
                    bound,
                    value: word % bound,
                    draws,
                });
            }
        }
        Err("RNG rejection attempt limit exceeded".into())
    }
}

#[cfg(test)]
mod tests;
