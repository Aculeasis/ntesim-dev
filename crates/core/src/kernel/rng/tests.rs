use super::*;

fn golden() -> Rng {
    Rng::new(42, BTreeMap::from([("critical".into(), 54)])).unwrap()
}

#[test]
fn matches_published_pcg32_seed_42_sequence_54_words() {
    let mut rng = golden();
    for (index, value) in [
        0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            rng.draw("critical").unwrap(),
            Draw {
                stream: "critical".into(),
                index: index as u64,
                value
            }
        );
    }
}

#[test]
fn streams_do_not_shift_each_other_and_clone_continues_exactly() {
    let mut baseline = golden();
    let mut extended = Rng::new(
        42,
        BTreeMap::from([("critical".into(), 54), ("proc".into(), 0)]),
    )
    .unwrap();
    for _ in 0..100 {
        extended.draw("proc").unwrap();
        extended.draw("proc").unwrap();
        assert_eq!(
            baseline.draw("critical").unwrap(),
            extended.draw("critical").unwrap()
        );
    }
    let mut copy = extended.clone();
    for _ in 0..100 {
        assert_eq!(extended.uniform("proc", 97), copy.uniform("proc", 97));
    }
    assert_eq!(extended, copy);
    let encoded = serde_json::to_value(copy).unwrap();
    assert_eq!(encoded["version"], VERSION);
    assert_eq!(encoded["seed"], 42);
    assert_eq!(encoded["streams"]["critical"]["draws"], 100);
}

#[test]
fn rejects_implicit_or_aliased_streams_and_invalid_bounds() {
    for sequences in [
        BTreeMap::from([("".into(), 0)]),
        BTreeMap::from([("a".into(), 1 << 63)]),
        BTreeMap::from([("a".into(), 0), ("b".into(), 0)]),
    ] {
        assert!(Rng::new(0, sequences).is_err());
    }
    assert!(Rng::new(0, BTreeMap::new()).is_ok());
    assert!(Rng::new(u64::MAX, BTreeMap::from([("a".into(), u64::MAX >> 1)])).is_ok());
    let mut rng = golden();
    let before = rng.clone();
    assert!(!rng.contains("absent"));
    assert!(rng.draw("absent").is_err());
    assert!(rng.uniform("absent", 2).is_err());
    assert!(rng.uniform("critical", 0).is_err());
    assert!(rng.uniform("critical", WORD_RANGE + 1).is_err());
    assert_eq!(rng, before);
}

#[test]
fn bounded_sampling_records_rejections_and_full_word_range() {
    let mut rng = golden();
    let first = rng.uniform("critical", (1 << 31) + 1).unwrap();
    assert_eq!(first.draws.len(), 1);
    assert_eq!(first.value, 0xa15c02b7_u64 % first.bound);
    let second = rng.uniform("critical", (1 << 31) + 1).unwrap();
    assert_eq!(
        second
            .draws
            .iter()
            .map(|d| (d.index, d.value))
            .collect::<Vec<_>>(),
        vec![(1, 0x7b47f409), (2, 0xba1d3330)]
    );
    assert_eq!(second.value, 0xba1d3330_u64 % second.bound);
    let mut rng = golden();
    assert_eq!(
        rng.uniform("critical", WORD_RANGE).unwrap().value,
        0xa15c02b7
    );
    assert_eq!(rng.uniform("critical", 1).unwrap().value, 0);
    assert_eq!(rng.streams["critical"].draws, 2);
}

#[test]
fn counter_overflow_and_attempt_limit_roll_back_all_tentative_draws() {
    let mut rng = golden();
    rng.streams.get_mut("critical").unwrap().draws = u64::MAX;
    let before = rng.clone();
    assert!(
        rng.draw("critical")
            .unwrap_err()
            .contains("counter overflow")
    );
    assert_eq!(rng, before);
    let state = rng.streams.get_mut("critical").unwrap();
    state.state = 0;
    state.draws = u64::MAX - 1;
    let before = rng.clone();
    assert!(
        rng.uniform("critical", (1 << 31) + 1)
            .unwrap_err()
            .contains("counter overflow")
    );
    assert_eq!(rng, before);
    rng.streams.get_mut("critical").unwrap().draws = 0;
    let before = rng.clone();
    assert!(
        rng.uniform_with_limit("critical", (1 << 31) + 1, 1)
            .unwrap_err()
            .contains("attempt limit")
    );
    assert_eq!(rng, before);
}

#[test]
fn exhaustive_small_word_oracle_and_generated_sample_receipts() {
    // Every residue has identical multiplicity after rejecting the short prefix.
    // This is an arithmetic oracle, not a statistical claim about game RNG.
    for bound in 1..=256_u64 {
        let mut counts = vec![0; bound as usize];
        for word in 256 % bound..256 {
            counts[(word % bound) as usize] += 1;
        }
        assert!(counts.iter().all(|count| *count == 256 / bound));
    }
    for seed in 0..64 {
        let mut rng = Rng::new(seed, BTreeMap::from([("a".into(), seed)])).unwrap();
        let mut index = 0;
        for bound in [1, 2, 3, 6, 97, (1 << 31) + 1, WORD_RANGE] {
            for _ in 0..100 {
                let sample = rng.uniform("a", bound).unwrap();
                assert!(sample.value < bound);
                for (i, draw) in sample.draws.iter().enumerate() {
                    assert_eq!(draw.index, index);
                    index += 1;
                    if i + 1 == sample.draws.len() {
                        assert!(u64::from(draw.value) >= WORD_RANGE % bound);
                        assert_eq!(sample.value, u64::from(draw.value) % bound);
                    } else {
                        assert!(u64::from(draw.value) < WORD_RANGE % bound);
                    }
                }
            }
        }
        assert_eq!(rng.streams["a"].draws, index);
    }
}
