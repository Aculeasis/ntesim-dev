use super::*;
fn ratio(numerator: u64, denominator: u64) -> Ratio {
    Ratio {
        numerator,
        denominator,
    }
}
fn formula() -> Formula {
    Formula {
        unit: "synthetic damage quantum".into(),
        flat: ratio(17, 5),
        terms: vec![Term {
            stat: "attack".into(),
            coefficient: ratio(31, 10),
        }],
        stages: vec![
            Stage {
                id: "bonus".into(),
                multiplier: ratio(11, 10),
            },
            Stage {
                id: "mitigation".into(),
                multiplier: ratio(4, 5),
            },
        ],
        rounding: Rounding::NearestTiesEven,
    }
}
#[test]
fn exact_pipeline_matches_independent_decimal_calculation() {
    // (3.4 + 10.1 * 3.1) * 1.1 * 0.8 = 30.5448 exactly.
    let result = formula()
        .evaluate(&BTreeMap::from([("attack".into(), ratio(101, 10))]))
        .unwrap();
    assert_eq!(
        result.base,
        ExactValue {
            numerator: "3471".into(),
            denominator: "100".into()
        }
    );
    assert_eq!(
        result.stages[0].value,
        ExactValue {
            numerator: "38181".into(),
            denominator: "1000".into()
        }
    );
    assert_eq!(
        result.exact,
        ExactValue {
            numerator: "38181".into(),
            denominator: "1250".into()
        }
    );
    assert_eq!(result.amount, 31);
}
#[test]
fn quantize_only_at_end_and_require_each_referenced_stat() {
    let mut f = formula();
    f.flat = ratio(1, 2);
    f.terms.clear();
    f.stages = vec![Stage {
        id: "double".into(),
        multiplier: ratio(2, 1),
    }];
    f.rounding = Rounding::Floor;
    assert_eq!(f.evaluate(&BTreeMap::new()).unwrap().amount, 1);
    f.terms.push(Term {
        stat: "missing".into(),
        coefficient: ratio(0, 1),
    });
    assert_eq!(
        f.evaluate(&BTreeMap::new()),
        Err(FormulaError::UnknownStat("missing".into()))
    );
    // Unused stats do not enter this formula's dependency closure.
    f.terms.clear();
    assert!(
        f.evaluate(&BTreeMap::from([("unused".into(), ratio(0, 0))]))
            .is_ok()
    );
}
#[test]
fn rounding_modes_match_exhaustive_independent_oracle() {
    for n in 0_u128..121 {
        for d in 1..32 {
            let exact = Exact::new(n, d).unwrap();
            assert_eq!(exact.quantize(Rounding::Floor).unwrap(), (n / d) as u64);
            assert_eq!(
                exact.quantize(Rounding::Ceil).unwrap(),
                n.div_ceil(d) as u64
            );
            let nearest = ((n * 2 + d) / (2 * d)) as u64;
            let nearest_even = if n % d * 2 == d && nearest % 2 == 1 {
                nearest - 1
            } else {
                nearest
            };
            assert_eq!(
                exact.quantize(Rounding::NearestTiesEven).unwrap(),
                nearest_even
            );
            assert_eq!(gcd(exact.n, exact.d), 1);
        }
    }
}
#[test]
fn generated_formulas_match_independent_single_fraction_oracle() {
    for stat in 0..16_u64 {
        for coefficient in 0..9 {
            for divisor in 1..9 {
                for multiplier in 0..5 {
                    for denominator in 1..5 {
                        let f = Formula {
                            unit: "quantum".into(),
                            flat: ratio(1, 3),
                            terms: vec![Term {
                                stat: "x".into(),
                                coefficient: ratio(coefficient, divisor),
                            }],
                            stages: vec![Stage {
                                id: "scale".into(),
                                multiplier: ratio(multiplier, denominator),
                            }],
                            rounding: Rounding::Floor,
                        };
                        let result = f
                            .evaluate(&BTreeMap::from([("x".into(), ratio(stat, 1))]))
                            .unwrap();
                        let n = (divisor + 3 * stat * coefficient) * multiplier;
                        let d = 3 * divisor * denominator;
                        assert_eq!(result.amount, n / d);
                        let exact_n: u128 = result.exact.numerator.parse().unwrap();
                        let exact_d: u128 = result.exact.denominator.parse().unwrap();
                        assert_eq!(exact_n * u128::from(d), u128::from(n) * exact_d);
                    }
                }
            }
        }
    }
}
#[test]
fn zero_denominators_and_stage_identity_are_errors_even_after_zero() {
    let mut f = formula();
    let stats = BTreeMap::from([("attack".into(), ratio(10, 1))]);
    f.flat = ratio(0, 0);
    assert_eq!(f.evaluate(&stats), Err(FormulaError::ZeroDenominator));
    f.flat = ratio(0, 1);
    f.stages[0].multiplier = ratio(0, 1);
    f.stages[1].multiplier = ratio(1, 0);
    assert_eq!(f.evaluate(&stats), Err(FormulaError::ZeroDenominator));
    f.stages[1].multiplier = ratio(1, 1);
    f.stages[1].id = "bonus".into();
    assert_eq!(
        f.evaluate(&stats),
        Err(FormulaError::DuplicateStage("bonus".into()))
    );
    f.stages[1].id = " ".into();
    assert_eq!(f.evaluate(&stats), Err(FormulaError::EmptyStage));
    f.unit.clear();
    assert_eq!(f.evaluate(&stats), Err(FormulaError::EmptyUnit));
}
#[test]
fn cancellation_prevents_false_overflow_and_real_overflow_fails() {
    let large = Exact::new(u128::MAX, 1).unwrap();
    assert_eq!(
        large.multiply(Exact::new(1, u128::MAX).unwrap()).unwrap(),
        Exact { n: 1, d: 1 }
    );
    assert_eq!(
        large.add(Exact::new(1, 1).unwrap()),
        Err(FormulaError::Overflow)
    );
    assert_eq!(
        large.multiply(Exact::new(2, 1).unwrap()),
        Err(FormulaError::Overflow)
    );
    assert_eq!(large.quantize(Rounding::Floor), Err(FormulaError::Overflow));
    assert_eq!(
        Exact::new(u128::MAX - 1, u128::MAX)
            .unwrap()
            .quantize(Rounding::NearestTiesEven)
            .unwrap(),
        1
    );
    let maximum = Exact::new(u128::from(u64::MAX) * 2 + 1, 2).unwrap();
    assert_eq!(maximum.quantize(Rounding::Floor).unwrap(), u64::MAX);
    assert_eq!(
        maximum.quantize(Rounding::Ceil),
        Err(FormulaError::Overflow)
    );
    // Adding equal denominator fractions reduces before their product.
    assert_eq!(
        Exact::new(1, u128::MAX)
            .unwrap()
            .add(Exact::new(u128::MAX - 1, u128::MAX).unwrap())
            .unwrap(),
        Exact { n: 1, d: 1 }
    );
}
#[test]
fn json_requires_rounding_and_rejects_signed_or_unknown_fields() {
    let good = r#"{"unit":"quantum","flat":"1/2","terms":[],"stages":[],"rounding":"ceil"}"#;
    assert_eq!(
        serde_json::from_str::<Formula>(good)
            .unwrap()
            .evaluate(&BTreeMap::new())
            .unwrap()
            .amount,
        1
    );
    for bad in [
        good.replace(",\"rounding\":\"ceil\"", ""),
        good.replace("\"ceil\"", "\"guess\""),
        good.replace("\"1/2\"", "\"-1/2\""),
        good.replace("\"1/2\"", "\"1/0\""),
        good.replace("\"1/2\"", "\"01/2\""),
        good.replace("\"1/2\"", r#"{"numerator":1,"denominator":2}"#),
        good.replace("\"terms\":[]", "\"term\":[]"),
    ] {
        assert!(serde_json::from_str::<Formula>(&bad).is_err());
    }
    assert_eq!(
        serde_json::to_value(Ratio {
            numerator: 18,
            denominator: 100
        })
        .unwrap(),
        serde_json::json!("18/100")
    );
    assert!(
        serde_json::to_value(Ratio {
            numerator: 1,
            denominator: 0
        })
        .is_err()
    );
}
