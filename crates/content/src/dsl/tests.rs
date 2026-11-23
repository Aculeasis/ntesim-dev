use super::*;

#[test]
fn pathological_source_nesting_and_operator_chains_fail_with_diagnostics() {
    for source in [
        format!("wait({}1{});", "(".repeat(1000), ")".repeat(1000)),
        format!("wait({}1);", "!".repeat(1000)),
        format!("wait({}1);", "1+".repeat(1000)),
        format!("{}{}", "repeat 1 {".repeat(1000), "}".repeat(1000)),
        format!("if 1 {{}} {} {{}}", "else if 1 {} ".repeat(1000)),
    ] {
        assert!(parse_nte(&source).is_err());
    }
}

#[test]
fn decimal_literals_retain_all_authored_digits_and_time_units() {
    let source = "options seed=9007199254740993; wait(0.10000000000000000001ms);";
    let script = parse_nte(source).unwrap();
    assert!(matches!(&script.options[0].value.kind,
        ExpressionKind::Number { decimal, unit: None } if decimal == "9007199254740993"));
    let StatementKind::Expression(Expression {
        kind: ExpressionKind::Call { args, .. },
        ..
    }) = &script.statements[0].kind
    else {
        panic!("expected call")
    };
    assert!(matches!(&args[0].kind,
        ExpressionKind::Number { decimal, unit: Some(TimeUnit::Milliseconds) }
            if decimal == "0.10000000000000000001"));
    let huge = "9".repeat(1_000);
    let tokens = Lexer::new(&huge).tokenize_all().unwrap();
    assert!(matches!(&tokens[0].kind, TokenKind::Number { decimal, .. } if decimal == &huge));
    assert!(Lexer::new(&"9".repeat(1_001)).tokenize_all().is_err());
}

#[test]
fn numeric_boundaries_and_ascii_identifiers_reject_malformed_source() {
    for source in [
        "1.2.3", "1.", "12abc", "1msx", "1us_", "1sfoo", "1e3", "мint",
    ] {
        assert!(Lexer::new(source).tokenize_all().is_err(), "{source}");
    }
    let tokens = Lexer::new("1/2 3%2 4us+5ms").tokenize_all().unwrap();
    assert_eq!(tokens.len(), 10);
}

#[test]
fn integer_syntax_never_truncates_saturates_or_ignores_wrong_types() {
    for declaration in [
        "mint char lvl=80.5/80;",
        "mint char lvl=80/80ms;",
        "mint char awakening=4294967296;",
        "mint char awakening=[1,2.5];",
        "mint add weapon=example refine=wrong;",
        "mint add weapon=example refine=1.00000000000000001;",
        "mint add cartridge=example count=4s;",
        "mint attack:0;",
        "mint attack:2.5;",
        "mint attack:4294967296;",
    ] {
        let source = format!("options duration=10; {declaration}");
        assert!(parse_nte(&source).is_err(), "{source}");
    }
}

#[test]
fn syntax_errors_have_utf16_editor_ranges_after_unicode_comments_and_crlf() {
    let error = parse_nte("# 😀\r\noptions duration=10;\r\nwait(12msx);").unwrap_err();
    let DslError::Lexer(error) = error else {
        panic!("expected lexer error")
    };
    assert_eq!(
        error.loc,
        SourceLocation {
            line: 3,
            column: 6,
            offset: 33,
            length: 4
        }
    );
    let error = parse_nte("# 😀\r\noptions duration=10;\r\nwait(1 2);").unwrap_err();
    let DslError::Parser(error) = error else {
        panic!("expected parser error")
    };
    assert_eq!(
        error.loc,
        SourceLocation {
            line: 3,
            column: 8,
            offset: 35,
            length: 1
        }
    );
    let tokens = Lexer::new("# 😀\n").tokenize_all().unwrap();
    assert_eq!(
        tokens[0].loc,
        SourceLocation {
            line: 2,
            column: 1,
            offset: 5,
            length: 0
        }
    );
}

#[test]
fn declaration_expressions_and_params_preserve_arithmetic_structure() {
    let script = parse_nte("options duration=1+2*3; const n=1; mint char talent=n+1,2*2,3+2 qte=2*2 +params=[start_hp%=0.3-0.2]; active mint; mint skill[hold=1+2,target=t0], attack:5;").unwrap();
    assert!(
        matches!(&script.options[0].value.kind, ExpressionKind::Binary { op, .. } if op == "+")
    );
    let DeclarationKind::Character {
        talents: Some(talents),
        extra_fields,
        ..
    } = &script.declarations[0].kind
    else {
        panic!("expected character")
    };
    assert!(matches!(&talents.burst.kind, ExpressionKind::Binary { op, .. } if op == "+"));
    assert_eq!(extra_fields[0].0, "start_hp%");
    assert!(matches!(&extra_fields[0].1.kind, ExpressionKind::Binary { op, .. } if op == "-"));
}

#[test]
fn comma_delimited_lists_and_declaration_placement_are_checked() {
    for suffix in [
        "wait(1 2);",
        "mint char awakening=[1 2];",
        "mint char +params=[start_energy=0 start_hp=10];",
        "mint skill[hold=1 target=t0];",
        "mint attack; active mint;",
        "repeat 1 { target hp=10; }",
    ] {
        assert!(
            parse_nte(&format!("options duration=10; {suffix}")).is_err(),
            "{suffix}"
        );
    }
}

#[test]
fn public_studio_sources_remain_parseable() {
    for source in [
        include_str!("../../../../presets/studio/s2.nte"),
        include_str!("../../../../presets/studio/s3.nte"),
        include_str!("../../../../presets/studio/s4.nte"),
        include_str!("../../../../presets/studio/s5.nte"),
    ] {
        parse_nte(source).unwrap();
    }
    parse_nte("options duration=10; let params=1; let x=2+params; wait(x);").unwrap();
}

#[test]
fn source_size_bound_and_missing_delimiter_have_editor_ranges() {
    assert!(parse_nte(&format!("#{}", " ".repeat(199_999))).is_ok());
    assert!(parse_nte(&format!("#{}", " ".repeat(200_000))).is_err());
    let error = parse_nte("options duration=1; wait(1").unwrap_err();
    let DslError::Parser(error) = error else {
        panic!("expected parser error")
    };
    assert_eq!(
        error.loc,
        SourceLocation {
            line: 1,
            column: 27,
            offset: 26,
            length: 0
        }
    );
}

#[test]
fn parse_preset_studio_s1() {
    let source = include_str!("../../../../presets/studio/s1.nte");
    let script = parse_nte(source).expect("s1.nte must parse successfully");

    assert_eq!(script.options.len(), 3);
    assert_eq!(script.options[0].key, "iteration");
    assert_eq!(script.options[1].key, "duration");
    assert_eq!(script.options[2].key, "seed");

    // 4 mint declarations + 1 target declaration + 1 active declaration = 6 declarations
    assert_eq!(script.declarations.len(), 6);

    // mint char
    match &script.declarations[0].kind {
        DeclarationKind::Character {
            actor,
            level,
            awakening,
            talents,
            ..
        } => {
            assert_eq!(actor, "mint");
            assert_eq!(
                level,
                &Some(LevelPair {
                    current: 80,
                    cap: 80
                })
            );
            assert_eq!(awakening, &Some(AwakeningSelection::Rank(0)));
            assert!(talents.is_some());
        }
        other => panic!("expected character declaration, got {:?}", other),
    }

    // mint add weapon
    match &script.declarations[1].kind {
        DeclarationKind::Weapon {
            actor,
            weapon_id,
            refinement,
            ..
        } => {
            assert_eq!(actor, "mint");
            assert_eq!(weapon_id, "fork_lingganzhongjiezhe");
            assert_eq!(refinement, &Some(5));
        }
        other => panic!("expected weapon declaration, got {:?}", other),
    }

    // mint add cartridge
    match &script.declarations[2].kind {
        DeclarationKind::Cartridge {
            actor,
            cartridge_id,
            count,
        } => {
            assert_eq!(actor, "mint");
            assert_eq!(cartridge_id, "firefliesandtheforest");
            assert_eq!(*count, 4);
        }
        other => panic!("expected cartridge declaration, got {:?}", other),
    }

    // mint add stats
    match &script.declarations[3].kind {
        DeclarationKind::Stats { actor, pairs } => {
            assert_eq!(actor, "mint");
            assert_eq!(pairs.len(), 6);
            assert_eq!(pairs[5].0, "dmg%");
        }
        other => panic!("expected stats declaration, got {:?}", other),
    }

    // target
    match &script.declarations[4].kind {
        DeclarationKind::Target { pairs } => {
            assert!(pairs.iter().any(|(k, _)| k == "hp"));
            assert!(pairs.iter().any(|(k, _)| k == "lvl"));
        }
        other => panic!("expected target declaration, got {:?}", other),
    }

    // active mint
    match &script.declarations[5].kind {
        DeclarationKind::Active { actor } => {
            assert_eq!(actor, "mint");
        }
        other => panic!("expected active declaration, got {:?}", other),
    }

    // 3 statements: attack, skill, burst
    assert_eq!(script.statements.len(), 3);
}

#[test]
fn parse_preset_studio_language_reference() {
    let source = include_str!("../../../../presets/studio/language-reference.nte");
    let script = parse_nte(source).expect("language-reference.nte must parse");

    assert_eq!(script.options.len(), 3);
    assert_eq!(script.declarations.len(), 6);

    // Character with awakening list [1, 2, 3]
    match &script.declarations[0].kind {
        DeclarationKind::Character {
            awakening: Some(AwakeningSelection::Explicit(list)),
            ..
        } => {
            assert_eq!(list, &[1, 2, 3]);
        }
        other => panic!(
            "expected character with explicit awakening list, got {:?}",
            other
        ),
    }

    // Statements: repeat 2 { mint attack:5; wait(0.5); }, mint skill;, mint burst;
    assert_eq!(script.statements.len(), 3);
    match &script.statements[0].kind {
        StatementKind::Repeat { body, .. } => {
            assert_eq!(body.len(), 2);
            match &body[0].kind {
                StatementKind::ActionList { actor, actions } => {
                    assert_eq!(actor, "mint");
                    assert_eq!(actions.len(), 1);
                    assert_eq!(actions[0].action, "attack");
                    assert!(actions[0].count.is_some());
                }
                other => panic!("expected action list in repeat, got {:?}", other),
            }
            match &body[1].kind {
                StatementKind::Expression(Expression {
                    kind: ExpressionKind::Call { callee, args },
                    ..
                }) => {
                    assert_eq!(callee, "wait");
                    assert_eq!(args.len(), 1);
                }
                other => panic!("expected wait() call in repeat, got {:?}", other),
            }
        }
        other => panic!("expected repeat statement, got {:?}", other),
    }
}

#[test]
fn parse_example_zero_continuing_energy() {
    let source = include_str!("../../../../examples/zero-continuing-energy.nte");
    let script = parse_nte(source).expect("zero-continuing-energy.nte must parse");

    assert_eq!(script.options.len(), 3);
    assert_eq!(script.declarations.len(), 5); // 3 chars + 1 target + 1 active
    assert_eq!(script.statements.len(), 5); // repeat, skill, wait, burst, attack:5
}

#[test]
fn parse_expressions_and_conditionals() {
    let source = r#"
options duration=10;
active mint;
if .target.hp < 50000 {
    wait(1s);
} else {
    mint skill;
}
let x = 10 + 20 * 3;
"#;
    let script = parse_nte(source).expect("script with if and expressions must parse");
    assert_eq!(script.statements.len(), 2);
    match &script.statements[0].kind {
        StatementKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_some());
        }
        other => panic!("expected if statement, got {:?}", other),
    }
}
