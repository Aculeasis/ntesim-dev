use super::{evaluate::*, number::ExactNumber, *};

fn expression(source: &str) -> Expression {
    let tokens = Lexer::new(source).tokenize_all().unwrap();
    Parser::new(tokens).parse_expression().unwrap()
}

fn run(source: &str) -> Result<Value, EvaluationError> {
    Evaluator::default().evaluate(&expression(source), &EmptyContext)
}

fn number(source: &str) -> ExactNumber {
    match run(source).unwrap() {
        Value::Number(value) => value,
        other => panic!("expected a number, got {other:?}"),
    }
}

#[test]
fn decimal_arithmetic_and_start_hp_settle_exactly() {
    assert_eq!(number("0.1 + 0.2 == 0.3"), ExactNumber::from(1));
    assert_eq!(number("(0.3 - 0.2) * 100"), ExactNumber::from(10));
    assert_eq!(number("0.3 - 0.2").floor_product(1370).unwrap(), 137);
    assert_eq!(number("0.58").floor_product(100).unwrap(), 58);
    assert_eq!(number("1 / 3 + 1 / 6").to_safe_ratio().unwrap(), (1, 2));
    assert_eq!(number("1 + 2 * 3 - 4 / 2"), ExactNumber::from(5));
}

#[test]
fn signed_rounding_remainder_and_numeric_builtins() {
    for (source, expected) in [
        ("floor(-1.2)", -2),
        ("ceil(-1.2)", -1),
        ("floor(1.2)", 1),
        ("ceil(1.2)", 2),
        ("-5.5 % 2", -1),
        ("5.5 % -2", 1),
        ("min(3, -2, 7)", -2),
        ("max(3, -2, 7)", 7),
        ("abs(-3)", 3),
        ("clamp(-2, 0, 8)", 0),
        ("clamp(12, 0, 8)", 8),
        ("clamp(5, 0, 8)", 5),
    ] {
        let expected = if source.contains('%') {
            ExactNumber::decimal("1.5")
                .unwrap()
                .multiply(&ExactNumber::from(expected))
                .unwrap()
        } else {
            ExactNumber::from(expected)
        };
        assert_eq!(number(source), expected, "{source}");
    }
    assert!(run("clamp(1, 3, 2)").is_err());
    for source in [
        "min()",
        "max()",
        "abs(1, 2)",
        "ceil()",
        "clamp(1)",
        "unknown(1)",
    ] {
        assert!(run(source).is_err(), "{source}");
    }
}

#[test]
fn times_share_seconds_and_reject_fractional_microseconds() {
    assert_eq!(
        number("1s == 1000ms && 1000ms == 1000000us"),
        ExactNumber::from(1)
    );
    assert_eq!(
        number("0.1s + 20ms + 3us").to_microseconds().unwrap(),
        120_003
    );
    assert!(number("0.0000001s").to_microseconds().is_err());
    assert!(number("-1s").to_microseconds().is_err());
}

#[test]
fn integer_and_ratio_boundaries_do_not_round_via_floating_point() {
    assert!(number("1.00000000000000001").to_safe_integer().is_err());
    assert_eq!(
        number("9007199254740991").to_safe_integer().unwrap(),
        9_007_199_254_740_991
    );
    assert!(number("9007199254740992").to_safe_integer().is_err());
    assert!(number("0.00000000000000001").to_safe_ratio().is_err());
    assert!(number("-1").to_safe_ratio().is_err());
}

#[test]
fn logical_short_circuit_does_not_sample_unreached_live_fields() {
    assert_eq!(number("0 && .actor.energy"), ExactNumber::from(0));
    assert_eq!(number("1 || (1 / 0)"), ExactNumber::from(1));
    assert_eq!(number("!0 && 2 < 3 && 3 != 4"), ExactNumber::from(1));
    assert!(run("1 && .actor.energy").is_err());
    assert!(run("true + 1").is_err());
    assert!(run("abs(false)").is_err());
    assert_eq!(number("true == 1"), ExactNumber::from(0));
    assert_eq!(run("false").unwrap(), Value::Boolean(false));
}

#[test]
fn evaluation_errors_keep_operator_and_field_source_locations() {
    let error = run("1 / 0").unwrap_err();
    assert_eq!(error.message, "Division by zero.");
    assert_eq!(
        (error.loc.column, error.loc.offset, error.loc.length),
        (3, 2, 1)
    );
    let error = run(".actor.energy").unwrap_err();
    assert_eq!((error.loc.column, error.loc.offset), (1, 0));
}

#[test]
fn live_context_resolves_exact_values_on_each_evaluation() {
    struct Context(i64);
    impl ExpressionContext for Context {
        fn identifier(&self, name: &str) -> Result<Value, String> {
            if name == "threshold" {
                Ok(Value::Number(20.into()))
            } else {
                EmptyContext.identifier(name)
            }
        }
        fn field(&self, parts: &[String]) -> Result<Value, String> {
            if parts == ["actor", "energy"] {
                Ok(Value::Number(self.0.into()))
            } else {
                EmptyContext.field(parts)
            }
        }
    }
    let ast = expression(".actor.energy < threshold");
    let mut evaluator = Evaluator::default();
    assert_eq!(
        evaluator.evaluate(&ast, &Context(19)).unwrap(),
        Value::Number(1.into())
    );
    assert_eq!(
        evaluator.evaluate(&ast, &Context(20)).unwrap(),
        Value::Number(0.into())
    );
}

#[test]
fn literal_arithmetic_and_evaluation_budgets_are_bounded() {
    assert!(ExactNumber::decimal(&"9".repeat(1001)).is_err());
    let value = ExactNumber::decimal(&"9".repeat(1000)).unwrap();
    assert!(value.multiply(&value).is_err());
    let mut evaluator = Evaluator::new(2);
    let error = evaluator
        .evaluate(&expression("1 + 2"), &EmptyContext)
        .unwrap_err();
    assert_eq!(error.code, validation::DiagnosticCode::WorkLimit);
    assert!(error.message.contains("limit 2 operations"));
    assert!(
        error
            .message
            .contains("independent of simulated combat time")
    );
    let mut evaluator = Evaluator::new(1);
    assert!(evaluator.evaluate(&expression("1"), &EmptyContext).is_ok());
    assert!(evaluator.evaluate(&expression("1"), &EmptyContext).is_err());

    let mut nested = expression("1");
    for _ in 0..128 {
        nested = Expression {
            loc: nested.loc.clone(),
            kind: ExpressionKind::Unary {
                op: "+".into(),
                expr: Box::new(nested),
            },
        };
    }
    let error = Evaluator::default()
        .evaluate(&nested, &EmptyContext)
        .unwrap_err();
    assert!(error.message.contains("nesting limit reached: 128 levels"));
}
