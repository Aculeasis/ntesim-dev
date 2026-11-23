use serde_json::Value;

pub const USAGE: &str = "usage: ntesim export-script SOURCE.nte OUTPUT_DIR | replay-script REPORT.json | inspect-script SOURCE.nte | validate-script SOURCE.nte | compile-script SOURCE.nte | run-script SOURCE.nte | run-compiled-script ARTIFACT.json | batch-script SOURCE.nte EXPERIMENT.json | compare-scripts BASE.nte CANDIDATE.nte EXPERIMENT.json | search-scripts REQUEST.json [CHECKPOINT.json] | studio-build-catalog | project-studio-actor REQUEST.json | project-studio-actors REQUESTS.json | character-stats REQUEST.json | weapon-stats REQUEST.json | weapon-passive-values REQUEST.json | damage-values REQUEST.json | build-stats REQUEST.json | team-stats REQUEST.json | combat-stats REQUEST.json | lifecycle-stats REQUEST.json | dispatch-stats REQUEST.json | timing-stats REQUEST.json | reaction-stats REQUEST.json | execution-stats REQUEST.json | run-game SCENARIO.json | run-game-suite SCENARIOS.json | run-game-built REQUEST.json | batch-game SCENARIO.json BATCH.json | batch-game-built BUILD_REQUEST.json BATCH.json | published-variants [VARIANTS.csv] | built-variants [VARIANTS.csv] REQUEST.json | profile-audit [VARIANTS.csv] | recommended-gear | search-builds [VARIANTS.csv] | compositions [VARIANTS.csv] | acceptance-spec [OUTPUT_DIR] | pilot [REQUEST.json [CHECKPOINT.json]] | acceptance-run [REQUEST.json [CHECKPOINT.json [VARIANTS.csv CONDITIONS.json]]] | validate-finalist [VARIANTS.csv] REQUEST.json | verify-tabular [FIXTURE.json] | run-synthetic FIXTURE.json | run-rotation-synthetic FIXTURE.json ROTATION.nte [--seed U64] | batch-rotation-synthetic FIXTURE.json ROTATION.nte SEEDS.json | compare-rotation-synthetic BASE.json BASE.nte CANDIDATE.json CANDIDATE.nte SEEDS.json | search-rotation-synthetic FIXTURE.json SEARCH.json | check-cycles-synthetic FIXTURE.json ROTATION.nte CHECKS.json";

pub(crate) fn completion(report: Value, pointers: &[&str]) -> Result<(Value, bool), String> {
    let complete = pointers
        .iter()
        .all(|pointer| report.pointer(pointer) == Some(&Value::Bool(true)));
    Ok((report, complete))
}

pub(crate) fn tabular_completion(report: Value) -> Result<(Value, bool), String> {
    let complete = report["failed_cases"] == 0 && report["passed_cases"] == report["total_cases"];
    Ok((report, complete))
}

pub(crate) fn pilot_completion(report: Value) -> Result<(Value, bool), String> {
    let complete = report["checkpoint"]["is_complete"] == true
        && report["checkpoint"]["failed_units"]
            .as_array()
            .is_some_and(Vec::is_empty);
    Ok((report, complete))
}

#[derive(Debug, PartialEq, Eq)]
pub enum CliCommand<'a> {
    ExportScript {
        source: &'a str,
        output: &'a str,
    },
    ReplayScript {
        input: &'a str,
    },
    SearchScripts {
        request: &'a str,
        checkpoint: Option<&'a str>,
    },
    BatchScript {
        source: &'a str,
        request: &'a str,
    },
    CompareScripts {
        baseline: &'a str,
        candidate: &'a str,
        request: &'a str,
    },
    RunScript {
        source: &'a str,
        stop_after_inputs: Option<&'a str>,
        resume: Option<&'a str>,
        cancel_file: Option<&'a str>,
    },
    RunCompiledScript {
        input: &'a str,
    },
    InspectScript {
        source: &'a str,
    },
    CompileScript {
        source: &'a str,
        validate_only: bool,
    },
    CheckCyclesSynthetic {
        input: &'a str,
        source: &'a str,
        checks: &'a str,
    },
    SearchRotationSynthetic {
        input: &'a str,
        request: &'a str,
    },
    CompareRotationSynthetic {
        left_input: &'a str,
        left_source: &'a str,
        right_input: &'a str,
        right_source: &'a str,
        request: &'a str,
    },
    BatchRotationSynthetic {
        input: &'a str,
        rotation: &'a str,
        request: &'a str,
    },
    RunRotationSynthetic {
        input: &'a str,
        rotation: &'a str,
        seed: Option<u64>,
    },
    RunSynthetic {
        input: &'a str,
    },
    StudioBuildCatalog,
    ProjectStudioActors {
        input: &'a str,
    },
    ProjectStudioActor {
        input: &'a str,
    },
    CharacterStats {
        input: &'a str,
    },
    WeaponStats {
        input: &'a str,
    },
    WeaponPassiveValues {
        input: &'a str,
    },
    DamageValues {
        input: &'a str,
    },
    BuildStats {
        input: &'a str,
    },
    TeamStats {
        input: &'a str,
    },
    CombatStats {
        input: &'a str,
    },
    LifecycleStats {
        input: &'a str,
    },
    DispatchStats {
        input: &'a str,
    },
    TimingStats {
        input: &'a str,
    },
    ReactionStats {
        input: &'a str,
    },
    ExecutionStats {
        input: &'a str,
    },
    RunGameSuite {
        input: &'a str,
    },
    RunGame {
        input: &'a str,
    },
    RunGameProgram {
        input: &'a str,
    },
    RunGameBuilt {
        input: &'a str,
    },
    BatchGame {
        input: &'a str,
        batch: &'a str,
    },
    BatchGameBuilt {
        input: &'a str,
        batch: &'a str,
    },
    PublishedVariants {
        input: Option<&'a str>,
    },
    BuiltVariants {
        csv_path: Option<&'a str>,
        input: &'a str,
    },
    ProfileAudit {
        input: Option<&'a str>,
    },
    RecommendedGear,
    SearchBuilds {
        input: Option<&'a str>,
    },
    Compositions {
        variants_csv: Option<&'a str>,
    },
    AcceptanceSpec {
        out_dir: Option<&'a str>,
    },
    ValidateFinalist {
        variants_csv: Option<&'a str>,
        input: &'a str,
    },
    VerifyTabular {
        fixture_path: Option<&'a str>,
    },
    Pilot {
        input: Option<&'a str>,
        checkpoint: Option<&'a str>,
    },
    AcceptanceRun {
        input: Option<&'a str>,
        checkpoint: Option<&'a str>,
        variants_csv: Option<&'a str>,
        conditions_json: Option<&'a str>,
    },
}
