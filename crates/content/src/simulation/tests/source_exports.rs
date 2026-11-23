use super::*;
use crate::simulation::export::SourceReportExport;

fn export(source: &str, limits: ExecutionLimits) -> SourceReportExport {
    let execution = PreparedSourceScenario::new(source, Default::default(), &limits)
        .unwrap()
        .run(42)
        .unwrap();
    SourceReportExport::new(execution, Default::default(), limits).unwrap()
}

#[test]
fn source_export_reconciles_exact_views_and_replays_complete_and_failed_inputs() {
    let source = "options duration=25 crit=random;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        mint add weapon=fork_lingganzhongjiezhe refine=1; mint add cartridge=Suit4 count=4;
        mint add stats atk%=0.5 atk=500;
        target hp=100000000 def=0 resist=0; active mint;
        mint attack:5, skill; wait_until(15); mint attack:5, skill;";
    for (text, limits, complete) in [
        (source.to_string(), ExecutionLimits::default(), true),
        (
            source.replace("mint attack:5, skill;", "mint burst;"),
            ExecutionLimits::default(),
            false,
        ),
        (
            source.to_string(),
            ExecutionLimits {
                max_events: 3,
                ..Default::default()
            },
            false,
        ),
        (
            source.replace("duration=25", "duration=25 stop=victory"),
            ExecutionLimits::default(),
            false,
        ),
        (
            source.replacen("mint attack:5, skill;", "let bad=1/0;", 1),
            ExecutionLimits::default(),
            false,
        ),
    ] {
        let original = export(&text, limits);
        assert_eq!(
            original.execution.report.success && original.execution.report.execution_completed,
            complete,
            "{:?}",
            original.execution.diagnostic
        );
        assert_eq!(original.damage.total.dps.is_some(), complete);
        let bytes = serde_json::to_vec(&original).unwrap();
        let saved: SourceReportExport = serde_json::from_slice(&bytes).unwrap();
        saved.replay().unwrap();
        assert_eq!(saved.csv_tables().unwrap(), original.csv_tables().unwrap());
        assert_eq!(
            saved.party.actors[0].build_decomposition,
            original.party.actors[0].build_decomposition
        );
    }
    let source_limits = crate::dsl::compiler::CompileLimits {
        max_steps: 4,
        ..Default::default()
    };
    let limits = ExecutionLimits::default();
    let execution = PreparedSourceScenario::new(source, source_limits, &limits)
        .unwrap()
        .run(42)
        .unwrap();
    let saved = SourceReportExport::new(execution, source_limits, limits).unwrap();
    assert!(!saved.execution.report.execution_completed);
    assert!(
        saved
            .execution
            .diagnostic
            .as_ref()
            .unwrap()
            .message
            .contains("limit 4 operations")
    );
    assert!(saved.damage.total.dps.is_none());
    saved.replay().unwrap();
}

#[test]
fn source_export_rejects_changed_source_build_report_and_engine_identities() {
    let original = export(
        include_str!("../../../../../presets/native/live-control.nte"),
        Default::default(),
    );
    let value = serde_json::to_value(original).unwrap();
    for pointer in [
        "/schema_version",
        "/engine_source_sha256",
        "/source_sha256",
        "/request_sha256",
        "/input_sha256",
        "/source_limits/maxSteps",
        "/execution_limits/max_events",
        "/execution/source",
        "/damage/total/exact",
        "/party/actors/0/build_decomposition/atk",
        "/execution/report/absolute_elapsed_us",
    ] {
        let mut bad = value.clone();
        let slot = bad
            .pointer_mut(pointer)
            .unwrap_or_else(|| panic!("missing export field {pointer}"));
        *slot = if let Some(text) = slot.as_str() {
            if pointer.ends_with("/atk") || pointer.ends_with("/exact") {
                if text == "0/1" { "1/1" } else { "0/1" }.into()
            } else {
                format!("{text}x").into()
            }
        } else {
            0.into()
        };
        let bad: SourceReportExport = serde_json::from_value(bad).unwrap();
        assert!(bad.replay().is_err(), "{pointer}");
    }
}

#[test]
fn source_csv_uses_exact_decimal_terms_and_quotes_nested_receipts_and_descriptions() {
    let mut saved = export(
        include_str!("../../../../../presets/native/live-control.nte"),
        Default::default(),
    );
    saved
        .execution
        .report
        .model_assumptions
        .insert("test,\"id\"".into(), "first\r\nsecond, \"quoted\"".into());
    saved.damage.contributions[0].totals.exact =
        ExactFraction::new(u128::from(u64::MAX) + 123, 1).unwrap();
    let tables = saved.csv_tables().unwrap();
    assert_eq!(tables.len(), 8);
    assert!(
        tables["assumptions.csv"]
            .contains("\"test,\"\"id\"\"\",\"first\r\nsecond, \"\"quoted\"\"\"\r\n")
    );
    assert!(tables["contributions.csv"].contains("\"18446744073709551738\""));
    assert!(tables["timeline.csv"].contains("\"\"fractional_resources\"\""));
    for table in tables.values() {
        assert!(table.starts_with("\"source_sha256\",\"request_sha256\",\"seed\","));
        assert!(table.ends_with("\r\n"));
    }
}
