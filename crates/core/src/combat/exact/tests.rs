use super::*;

#[test]
fn damage_kind_zones_follow_independent_packet_arithmetic() {
    let q = |n, d| ExactFraction::new(n, d).unwrap();
    let mut request = ExactDirectDamageRequest {
        kind: crate::combat::DamageKind::Elemental,
        attacker_level: 60,
        stat: q(1000, 1),
        ratio: q(2, 1),
        damage_bonus: q(1, 5),
        target_defense: q(960, 1),
        defense_shred: q(0, 1),
        defense_ignore: q(0, 1),
        target_resistance: q(1, 4),
        resistance_reduction: q(0, 1),
        negative_resistance_model: crate::combat::NegativeResistanceModel::StandardRational,
        final_multiplier: q(3, 2),
        special_multiplier: q(7, 5),
        crit_bonus: q(1, 2),
    };
    // 2000 * 6/5 * (960/1920) * 3/4 * 3/2 * 7/5 = 1890.
    for (kind, non_crit, critical, defense, resistance, eligible) in [
        (
            crate::combat::DamageKind::Elemental,
            1890,
            2835,
            q(1, 2),
            q(3, 4),
            true,
        ),
        (
            crate::combat::DamageKind::Mental,
            3780,
            5670,
            q(1, 1),
            q(3, 4),
            true,
        ),
        (
            crate::combat::DamageKind::True,
            2800,
            2800,
            q(1, 1),
            q(1, 1),
            false,
        ),
    ] {
        request.kind = kind;
        assert_eq!(
            evaluate_direct_damage(&request).unwrap(),
            ExactDirectDamageBreakdown {
                non_crit: q(non_crit, 1),
                critical: q(critical, 1),
                defense_factor: defense,
                resistance_factor: resistance,
                crit_eligible: eligible,
            }
        );
    }
    // A different carrier changes only quantities; True still consumes special zone only.
    request.stat = q(123, 1);
    request.ratio = q(4, 1);
    request.damage_bonus = q(99, 1);
    request.final_multiplier = q(99, 1);
    request.crit_bonus = q(99, 1);
    assert_eq!(
        evaluate_direct_damage(&request).unwrap().critical,
        q(3444, 5)
    );
}

#[test]
fn small_damage_with_large_exact_terms_survives_report_json() {
    let q = |n, d| ExactFraction::new(n, d).unwrap();
    let max = u128::from(u64::MAX);
    let request = ExactDirectDamageRequest {
        kind: crate::combat::DamageKind::Mental,
        attacker_level: 80,
        stat: q(max, max - 1),
        ratio: q(3, 2),
        damage_bonus: q(0, 1),
        target_defense: q(0, 1),
        defense_shred: q(0, 1),
        defense_ignore: q(0, 1),
        target_resistance: q(0, 1),
        resistance_reduction: q(0, 1),
        negative_resistance_model: crate::combat::NegativeResistanceModel::StandardRational,
        final_multiplier: q(1, 1),
        special_multiplier: q(1, 1),
        crit_bonus: q(0, 1),
    };
    let damage = evaluate_direct_damage(&request).unwrap().non_crit;
    assert_eq!(
        (damage.numerator, damage.denominator),
        (3 * max, 2 * (max - 1))
    );
    assert_eq!(damage.round_u64().unwrap(), 2);
    let report = serde_json::to_value(damage).unwrap();
    assert_eq!(report, format!("{}/{}", 3 * max, 2 * (max - 1)));
    assert_eq!(
        serde_json::from_value::<ExactFraction>(report).unwrap(),
        damage
    );
    assert_eq!(
        serde_json::from_slice::<ExactFraction>(&serde_json::to_vec(&damage).unwrap()).unwrap(),
        damage
    );
    for invalid in [
        r#"-1/1"#,
        r#""-1/1""#,
        r#""1.5/1""#,
        r#""340282366920938463463374607431768211456/1""#,
        r#""1/0""#,
        r#""0/0""#,
        r#""01/2""#,
        r#""1/02""#,
        r#""1""#,
        r#""1/2/3""#,
        r#"{"numerator":1,"denominator":1}"#,
    ] {
        assert!(
            serde_json::from_str::<ExactFraction>(invalid).is_err(),
            "accepted invalid ExactFraction: {invalid}"
        );
    }
}

#[test]
fn level_and_negative_resistance_use_independent_rational_oracles() {
    let q = |n, d| ExactFraction::new(n, d).unwrap();
    assert_eq!(
        defense_factor(1, q(606, 1), q(0, 1), q(0, 1)).unwrap(),
        q(1, 2)
    );
    assert_eq!(
        defense_factor(80, q(606, 1), q(0, 1), q(0, 1)).unwrap(),
        q(180, 281)
    );
    assert_eq!(
        resistance_factor(
            q(1, 5),
            q(2, 5),
            crate::combat::NegativeResistanceModel::StandardRational
        )
        .unwrap(),
        q(7, 6)
    );
    assert_eq!(
        resistance_factor(
            q(1, 5),
            q(2, 5),
            crate::combat::NegativeResistanceModel::LinearScaled
        )
        .unwrap(),
        q(13, 11)
    );
    assert_eq!(
        resistance_factor(
            q(3, 2),
            q(0, 1),
            crate::combat::NegativeResistanceModel::StandardRational
        )
        .unwrap(),
        q(0, 1)
    );
    assert!(defense_factor(1, q(1, 1), q(0, 1), q(2, 1)).is_err());
}

#[test]
fn healing_modifiers_multiply_once_before_final_rounding() {
    let q = |n, d| ExactFraction::new(n, d).unwrap();
    let base = q(7, 2);
    let gross = exact_healing(base, q(1, 10), q(1, 5)).unwrap();
    assert_eq!(gross, q(231, 50));
    assert_eq!(gross.round_u64().unwrap(), 5);
    assert_eq!(base.round_u64().unwrap(), 4);
    assert!(exact_healing(base, q(u128::MAX, 1), q(1, 1)).is_err());
    for (base, outgoing, incoming) in [
        (
            ExactFraction {
                numerator: 1,
                denominator: 0,
            },
            q(0, 1),
            q(0, 1),
        ),
        (
            q(1, 1),
            ExactFraction {
                numerator: 1,
                denominator: 0,
            },
            q(0, 1),
        ),
        (
            q(1, 1),
            q(0, 1),
            ExactFraction {
                numerator: 1,
                denominator: 0,
            },
        ),
    ] {
        assert!(exact_healing(base, outgoing, incoming).is_err());
    }
}

#[test]
fn typed_arithmetic_and_operators_work_without_heap_strings() {
    let a = ExactFraction::try_new(1, 3).unwrap();
    let b = ExactFraction::try_new(1, 6).unwrap();

    assert_eq!(a.try_add(b), Ok(ExactFraction::try_new(1, 2).unwrap()));
    assert_eq!(a + b, ExactFraction::try_new(1, 2).unwrap());

    assert_eq!(a.try_sub(b), Ok(ExactFraction::try_new(1, 6).unwrap()));
    assert_eq!(a - b, ExactFraction::try_new(1, 6).unwrap());

    assert_eq!(a.try_mul(b), Ok(ExactFraction::try_new(1, 18).unwrap()));
    assert_eq!(a * b, ExactFraction::try_new(1, 18).unwrap());

    assert_eq!(a.try_div(b), Ok(ExactFraction::try_new(2, 1).unwrap()));
    assert_eq!(a / b, ExactFraction::try_new(2, 1).unwrap());

    assert_eq!(
        ExactFraction::try_new(1, 0),
        Err(ExactArithmeticError::ZeroDenominator)
    );
    assert_eq!(
        b.try_sub(a),
        Err(ExactArithmeticError::SubtractionUnderflow)
    );
    assert_eq!(
        a.try_div(ExactFraction::default()),
        Err(ExactArithmeticError::DivisionByZero)
    );

    assert_eq!(
        ExactFraction::try_new(0, 0),
        Err(ExactArithmeticError::ZeroDenominator)
    );
    assert_eq!(
        ExactFraction::try_new(0, u128::MAX).unwrap(),
        ExactFraction::default()
    );
    let one = ExactFraction {
        numerator: 1,
        denominator: 1,
    };
    let largest = ExactFraction {
        numerator: u128::MAX,
        denominator: 1,
    };
    assert_eq!(ExactFraction::try_new(u128::MAX, 1).unwrap(), largest);
    assert_eq!(
        largest.try_add(one),
        Err(ExactArithmeticError::AdditionOverflow)
    );
    assert_eq!(
        largest.try_mul(one + one),
        Err(ExactArithmeticError::MultiplicationOverflow)
    );

    let noncanonical = ExactFraction {
        numerator: 2,
        denominator: 4,
    };
    assert_eq!(noncanonical.try_add(noncanonical).unwrap(), one);
    assert_eq!(
        noncanonical.try_add(b).unwrap(),
        ExactFraction::try_new(2, 3).unwrap()
    );
    assert_eq!(noncanonical.try_mul(one).unwrap(), noncanonical);
    let wide = 1_u128 << 100;
    assert_eq!(
        ExactFraction::try_new(wide * 3, wide * 5),
        ExactFraction::try_new(3, 5)
    );
    assert_eq!(crate::formula::gcd(0, 0), 0);
    assert_eq!(crate::formula::gcd(0, wide), wide);
    assert_eq!(crate::formula::gcd(wide, 0), wide);
    assert_eq!(
        crate::formula::gcd(1_u128 << 63, 1_u128 << 63),
        1_u128 << 63
    );
}
