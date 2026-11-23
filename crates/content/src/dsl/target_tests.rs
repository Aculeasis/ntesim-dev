use super::{
    evaluate::{EmptyContext, Evaluator, ExpressionContext, Value},
    number::ExactNumber,
    target::compile_target,
    *,
};

fn compile(source: &str) -> Result<crate::simulation::GameTargetConfig, validation::Diagnostic> {
    let script = parse_nte(source).unwrap();
    compile_target(
        &script.declarations[0],
        0,
        &EmptyContext,
        &mut Evaluator::default(),
    )
}

#[test]
fn target_compilation_preserves_independent_vitals_and_exact_element_resistances() {
    let target =
        compile("target max_hp=100 hp=40+10 def=3*7 resist=0.1+0.2 anima=0 psyche=1/7 boss=true;")
            .unwrap();
    assert_eq!(
        (target.hp_current, target.hp_max, target.defense),
        (50, 100, 21)
    );
    assert!(target.is_boss);
    assert_eq!(
        (target.resistance.numerator, target.resistance.denominator),
        (3, 10)
    );
    assert_eq!(target.elemental_resistances["NATURE"].numerator, 0);
    assert_eq!(
        (
            target.elemental_resistances["PSYCHE"].numerator,
            target.elemental_resistances["PSYCHE"].denominator
        ),
        (1, 7)
    );
    assert!(!target.elemental_resistances.contains_key("CHAOS"));
    assert_eq!(target.level, 80);
    let default = compile("target hp=10 def=0 resist=0;").unwrap();
    assert_eq!(default.hp_max, 10);
    assert!(!default.is_boss);
    assert!(default.break_profile.is_none());
}

#[test]
fn dummy_targets_reject_catalog_and_spatial_inputs() {
    for fields in ["type=\"mob\"", "pack=\"stats\""] {
        let error = compile(&format!("target hp=100 def=0 resist=0 {fields};")).unwrap_err();
        assert!(error.message.contains("Unsupported target field"));
    }
    let target = compile("target hp=100 def=0 resist=0 boss=true;").unwrap();
    for field in ["source_profile_id", "stat_pack_id", "position_mm"] {
        let mut request = serde_json::to_value(&target).unwrap();
        request[field] = serde_json::json!("forbidden-enemy-selector");
        assert!(serde_json::from_value::<crate::simulation::GameTargetConfig>(request).is_err());
    }
    assert!(parse_nte("target hp=100 def=0 resist=0 pos=0,0,0;").is_err());
}

#[test]
fn target_original_atk_is_explicit_nonnegative_and_distinguishes_zero_from_missing() {
    let target = compile("target hp=100 def=0 resist=0 original_atk=80+20;").unwrap();
    assert_eq!(target.original_atk, Some(100));
    assert_eq!(
        compile("target hp=100 def=0 resist=0 original_atk=0;")
            .unwrap()
            .original_atk,
        Some(0)
    );
    assert_eq!(
        compile("target hp=100 def=0 resist=0;")
            .unwrap()
            .original_atk,
        None
    );
    for (value, offset, length) in [("-1", 42, 1), ("1/3", 43, 1), ("true", 42, 4)] {
        let diagnostic = compile(&format!(
            "target hp=100 def=0 resist=0 original_atk={value};"
        ))
        .unwrap_err();
        assert_eq!(diagnostic.code, validation::DiagnosticCode::Configuration);
        assert_eq!(diagnostic.loc.line, 1);
        assert_eq!(diagnostic.loc.column, offset + 1);
        assert_eq!(diagnostic.loc.offset, offset);
        assert_eq!(diagnostic.loc.length, length);
    }
}

#[test]
fn invalid_target_fields_and_types_fail_instead_of_rounding_or_disabling_them() {
    for fields in [
        "hp=0 def=0 resist=0",
        "hp=10 max_hp=9 def=0 resist=0",
        "hp=1.00000000000000001 def=0 resist=0",
        "hp=9007199254740992 def=0 resist=0",
        "hp=10 def=-1 resist=0",
        "hp=10 def=0 resist=-0.1",
        "hp=10 def=0 resist=0 boss=1",
        "hp=10 def=0 resist=0 anima=-1",
        "hp=10 def=0 resist=0 lvl=4294967296",
        "hp=10 def=0 resist=0 unknown=1",
        "hp=10 def=0 resist=0 pos=0",
        "hp=10 def=0 resist=0 break_gauge=100",
        "hp=10 def=0",
        "hp=10 resist=0",
        "def=0 resist=0",
        "hp=10 def=0 resist=true",
    ] {
        assert!(compile(&format!("target {fields};")).is_err(), "{fields}");
    }
}

#[test]
fn targets_resolve_constants_from_the_callers_current_scope_and_keep_indices() {
    struct Constants;
    impl ExpressionContext for Constants {
        fn identifier(&self, name: &str) -> Result<Value, String> {
            if name == "hp" {
                Ok(Value::Number(ExactNumber::from(250)))
            } else {
                EmptyContext.identifier(name)
            }
        }
        fn field(&self, parts: &[String]) -> Result<Value, String> {
            EmptyContext.field(parts)
        }
    }
    let script = parse_nte("target hp=hp*2 def=0 resist=0;").unwrap();
    let target = compile_target(
        &script.declarations[0],
        2,
        &Constants,
        &mut Evaluator::default(),
    )
    .unwrap();
    assert_eq!(
        (target.target_index, target.hp_current, target.hp_max),
        (2, 500, 500)
    );
}

#[test]
fn explicit_break_profiles_preserve_exact_authored_values() {
    for target in ["hp=100 def=0 resist=0", "hp=100 def=0 resist=0 boss=true"] {
        let target = compile(&format!(
            "target {target} break_gauge=1/3 max_break_gauge=2/3 broken_duration=0.1+0.2;"
        ))
        .unwrap();
        let profile = target.break_profile.unwrap();
        assert_eq!(
            (profile.capacity.numerator, profile.capacity.denominator),
            (2, 3)
        );
        assert_eq!(
            (
                profile.current_gauge.numerator,
                profile.current_gauge.denominator
            ),
            (1, 3)
        );
        assert_eq!(profile.broken_duration_us, 300_000);
    }
    let target = compile(
        "target hp=100 def=0 resist=0 break_gauge=0 max_break_gauge=1 broken_duration=0.000001;",
    )
    .unwrap();
    assert_eq!(target.break_profile.unwrap().current_gauge.numerator, 0);

    let target = compile(
        "target hp=100 def=0 resist=0 break_gauge=1 max_break_gauge=1 broken_duration=1 discord_fraction=1;",
    )
    .unwrap();
    assert_eq!(
        target.break_profile.unwrap().discord_fraction.unwrap(),
        crate::formula::Ratio::one()
    );
}

#[test]
fn discord_fraction_requires_a_bar_and_reports_its_own_source_span() {
    for (fraction, expression_offset) in [("0", 0), ("1001/1000", 4)] {
        let source = format!(
            "target hp=100 def=0 resist=0 break_gauge=1 max_break_gauge=1 broken_duration=1 discord_fraction={fraction};"
        );
        let error = compile(&source).unwrap_err();
        assert!(error.message.contains("discord_fraction"));
        assert_eq!(
            error.loc.column,
            source.find("discord_fraction=").unwrap()
                + "discord_fraction=".len()
                + expression_offset
                + 1
        );
    }
    let source = "target hp=100 def=0 resist=0 discord_fraction=1;";
    let error = compile(source).unwrap_err();
    assert!(error.message.contains("explicit Break gauge profile"));
    assert_eq!(
        error.loc.column,
        source.find("discord_fraction=").unwrap() + "discord_fraction=".len() + 1
    );
}

#[test]
fn incomplete_or_invalid_break_profiles_fail_at_source_admission() {
    for fields in [
        "break_gauge=1",
        "max_break_gauge=1",
        "broken_duration=1",
        "break_gauge=1 max_break_gauge=1",
        "break_gauge=1 broken_duration=1",
        "max_break_gauge=1 broken_duration=1",
        "break_gauge=0 max_break_gauge=0 broken_duration=1",
        "break_gauge=2/3 max_break_gauge=1/3 broken_duration=1",
        "break_gauge=-1 max_break_gauge=1 broken_duration=1",
        "break_gauge=true max_break_gauge=1 broken_duration=1",
        "break_gauge=1 max_break_gauge=1 broken_duration=0",
        "break_gauge=1 max_break_gauge=1 broken_duration=-1",
        "break_gauge=1 max_break_gauge=1 broken_duration=0.0000001",
        "break_gauge=1 max_break_gauge=1 broken_duration=true",
        "break_gauge=1 max_break_gauge=1 broken_duration=9007199255",
    ] {
        let error = compile(&format!("target hp=100 def=0 resist=0 {fields};")).unwrap_err();
        assert_eq!(
            error.code,
            validation::DiagnosticCode::Configuration,
            "{fields}"
        );
    }
    assert!(parse_nte("target hp=100 def=0 resist=0 break_gauge=1 break_gauge=1 max_break_gauge=1 broken_duration=1;").is_err());
}

#[test]
fn public_preset_targets_compile_to_the_native_schema() {
    for source in [
        include_str!("../../../../presets/studio/s1.nte"),
        include_str!("../../../../presets/studio/s2.nte"),
        include_str!("../../../../presets/studio/s3.nte"),
        include_str!("../../../../presets/studio/s4.nte"),
        include_str!("../../../../presets/studio/s5.nte"),
        include_str!("../../../../presets/studio/language-reference.nte"),
    ] {
        for declaration in &parse_nte(source).unwrap().declarations {
            if matches!(declaration.kind, DeclarationKind::Target { .. }) {
                assert!(
                    compile_target(declaration, 0, &EmptyContext, &mut Evaluator::default())
                        .is_ok()
                );
            }
        }
    }
}
