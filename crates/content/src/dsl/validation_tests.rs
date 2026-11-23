use super::{validation::*, *};

#[test]
fn duplicate_fields_are_rejected_in_every_configuration_and_action_scope() {
    for source in [
        "options duration=1 duration=2;",
        "mint char lvl=80/80 lvl=1/20;",
        "mint char awakening=0 awakening=[1];",
        "mint char talent=1,1,1 qte=1 talent=2,2,2;",
        "mint char start_energy=1 +params=[start_energy=2];",
        "mint add weapon=example refine=1 refine=2;",
        "mint add weapon=example lvl=80/80 lvl=1/20;",
        "mint add stats atk%=0.1 atk%=0.2;",
        "target hp=100 hp=200;",
        "hurt every interval=1 interval=2;",
        "incoming mint damage=1 damage=2;",
        "mint dash[critical=true,critical=false];",
    ] {
        let diagnostics = validate_frontend(source);
        assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:?}");
        assert_eq!(diagnostics[0].code, DiagnosticCode::Syntax, "{source}");
        assert!(
            diagnostics[0].message.contains("duplicate field"),
            "{source}"
        );
    }
}

#[test]
fn duplicate_ranges_point_to_the_second_key_in_utf16_source() {
    let source = "# 😀\r\noptions duration=1 duration=2;";
    let diagnostics = validate_frontend(source);
    assert_eq!(
        diagnostics[0].loc,
        SourceLocation {
            line: 2,
            column: 20,
            offset: 25,
            length: 8
        }
    );
    let json = serde_json::to_string(&diagnostics).unwrap();
    assert_eq!(
        serde_json::from_str::<Vec<Diagnostic>>(&json).unwrap(),
        diagnostics
    );
}

#[test]
fn declarations_require_preceding_owners_but_stats_rows_still_sum() {
    let diagnostics = validate_frontend(
        "mint add stats atk=1; mint char; active mint; mint add stats atk=2; mint add stats atk=3;",
    );
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("before using"));
    let diagnostics = validate_frontend(
        "mint char; mint char; mint add weapon=x; mint add weapon=y; active mint; active mint;",
    );
    assert_eq!(diagnostics.len(), 3);
}

#[test]
fn action_owner_validation_descends_into_all_control_flow_branches() {
    let source = "mint char; active mint; if 1 { missing attack; } else { repeat 2 { absent skill; } } while 0 { other burst; } for let i=0; i<1; i=i+1 { unknown dash; }";
    let diagnostics = validate_frontend(source);
    assert_eq!(diagnostics.len(), 4, "{diagnostics:?}");
    assert!(
        diagnostics
            .iter()
            .all(|d| d.message.contains("Unknown action character"))
    );
    assert!(
        diagnostics
            .windows(2)
            .all(|d| d[0].loc.offset < d[1].loc.offset)
    );
}

#[test]
fn awakening_selection_checks_identity_and_preserves_nonprefix_sets() {
    for selection in ["7", "[0]", "[7]", "[1,1]"] {
        assert_eq!(
            validate_frontend(&format!("mint char awakening={selection};")).len(),
            1
        );
    }
    for selection in ["0", "6", "[]", "[6,2]"] {
        assert!(validate_frontend(&format!("mint char awakening={selection};")).is_empty());
    }
}

#[test]
fn all_public_presets_pass_frontend_binding_validation() {
    for source in [
        include_str!("../../../../presets/studio/s1.nte"),
        include_str!("../../../../presets/studio/s2.nte"),
        include_str!("../../../../presets/studio/s3.nte"),
        include_str!("../../../../presets/studio/s4.nte"),
        include_str!("../../../../presets/studio/s5.nte"),
        include_str!("../../../../presets/studio/language-reference.nte"),
    ] {
        assert!(
            validate_frontend(source).is_empty(),
            "{:?}",
            validate_frontend(source)
        );
    }
}

#[test]
fn lexical_scopes_reject_forward_references_and_constant_assignment() {
    for (source, message) in [
        ("let x=y; let y=1;", "Unknown variable 'y'"),
        ("let x=x;", "Unknown variable 'x'"),
        ("const n=1; n=2;", "Cannot assign constant 'n'"),
        ("let n=1; let n=2;", "Duplicate or reserved variable 'n'"),
        (
            "if 1 { let local=2; } wait(local);",
            "Unknown variable 'local'",
        ),
        (
            "for let i=0; i<2; i=i+1 {} wait(i);",
            "Unknown variable 'i'",
        ),
        ("for let i=0; i<2; j=i+1 {}", "Unknown variable 'j'"),
        ("let trials=1;", "Duplicate or reserved variable 'trials'"),
        ("break;", "Loop control requires"),
        ("continue;", "Loop control requires"),
        ("custom(1);", "Unknown built-in 'custom'"),
    ] {
        let diagnostics = validate_frontend(source);
        assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:?}");
        assert!(
            diagnostics[0].message.contains(message),
            "{source}: {diagnostics:?}"
        );
    }
    assert!(
        validate_frontend(
            "const n=2; if 1 { let n=3; n=4; } repeat n { continue; } while 1 { break; }"
        )
        .is_empty()
    );
}

#[test]
fn configuration_constants_follow_authored_order_not_ast_storage_order() {
    assert!(
        validate_frontend("const n=2; mint char talent=n,n+1,n+2 qte=n+1; active mint; wait(n);")
            .is_empty()
    );
    let diagnostics = validate_frontend("mint char talent=n,1,1 qte=1; const n=2; active mint;");
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Unknown variable 'n'"));
}
