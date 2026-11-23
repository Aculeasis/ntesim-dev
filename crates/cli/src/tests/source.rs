use crate::commands::{CliCommand, execute, parse_command};

#[test]
fn cli_cooperative_cancel_retains_prefix_and_resumes_from_relocated_checkpoint() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cache/source-control-cli")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).unwrap();
    let checkpoint = directory.join("checkpoint.json");
    let signal = directory.join("cancel");
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/incoming-shield-cycles.nte"
    );
    let args = [
        "run-script".into(),
        source.into(),
        "--stop-after-inputs".into(),
        "3".into(),
    ];
    let (prefix, complete) = execute(&args).unwrap();
    assert!(!complete);
    assert_eq!(prefix["checkpoint"]["completed_inputs"], 3);
    assert_eq!(prefix["execution"]["report"]["execution_completed"], false);
    std::fs::write(
        &checkpoint,
        serde_json::to_vec(&prefix["checkpoint"]).unwrap(),
    )
    .unwrap();
    std::fs::write(&signal, "cancel").unwrap();
    let resume = [
        "run-script".into(),
        source.into(),
        "--resume".into(),
        checkpoint.to_str().unwrap().into(),
        "--cancel-file".into(),
        signal.to_str().unwrap().into(),
    ];
    let (cancelled, complete) = execute(&resume).unwrap();
    assert!(!complete);
    assert_eq!(cancelled, prefix);
    std::fs::remove_file(signal).unwrap();
    let (continued, complete) = execute(&resume).unwrap();
    assert!(complete);
    let (full, complete) = execute(&["run-script".into(), source.into()]).unwrap();
    assert!(complete);
    assert_eq!(continued["execution"], full["execution"]);
    assert!(continued["checkpoint"].is_null());
    std::fs::remove_file(checkpoint).unwrap();
    assert!(
        execute(&[
            "run-script".into(),
            source.into(),
            "--stop-after-inputs".into(),
            "-1".into()
        ])
        .is_err()
    );
}

#[test]
fn cli_exports_lossless_reports_and_replays_relocated_complete_and_failed_prefixes() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cache/source-report-cli")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("input.nte");
    let original = "options duration=10; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0; target hp=100000000 def=0 resist=0; active mint; mint skill;";
    for complete in [true, false] {
        let output = directory.join(if complete { "complete" } else { "failed" });
        std::fs::write(
            &source,
            if complete {
                original.into()
            } else {
                original.replace("mint skill;", "mint burst;")
            },
        )
        .unwrap();
        let args = [
            "export-script".into(),
            source.to_str().unwrap().into(),
            output.to_str().unwrap().into(),
        ];
        let (manifest, success) = execute(&args).unwrap();
        assert_eq!(success, complete);
        assert_eq!(manifest["execution_completed"], complete);
        for (file, hash) in manifest["files"].as_object().unwrap() {
            let bytes = std::fs::read(output.join(file)).unwrap();
            assert_eq!(ntesim_contracts::sha256(&bytes), hash.as_str().unwrap());
        }
        // Existing destinations are refused without overwriting any report.
        assert!(execute(&args).is_err());
        let relocated = directory.join(if complete {
            "relocated-complete.json"
        } else {
            "relocated-failed.json"
        });
        std::fs::copy(output.join("report.json"), &relocated).unwrap();
        std::fs::write(&source, "a later invalid draft").unwrap();
        let (replayed, verified) =
            execute(&["replay-script".into(), relocated.to_str().unwrap().into()]).unwrap();
        assert!(verified);
        assert_eq!(replayed["reconciled"], true);
        assert_eq!(replayed["execution_completed"], complete);
        assert_eq!(replayed["source_sha256"], manifest["source_sha256"]);
        if !complete {
            assert!(replayed["damage"]["total"]["dps"].is_null());
        }
    }
    let output = directory.join("invalid");
    let (invalid, success) = execute(&[
        "export-script".into(),
        source.to_str().unwrap().into(),
        output.to_str().unwrap().into(),
    ])
    .unwrap();
    assert!(!success);
    assert!(!invalid["diagnostics"].as_array().unwrap().is_empty());
    assert!(!output.exists());
}

#[test]
fn cli_source_experiments_read_native_scripts_and_preserve_rejections() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cache/source-experiment-cli")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).unwrap();
    let request_path = directory.join("seeds.json");
    std::fs::write(&request_path, r#"{"schema_version":1,"seeds":[42,43],"max_events_per_run":1000,"max_total_events":10000,"max_receipt_bytes":8388608}"#).unwrap();
    let source_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../presets/native/live-control.nte"
    );
    let request = request_path.to_str().unwrap();
    let (batch, complete) =
        execute(&["batch-script".into(), source_path.into(), request.into()]).unwrap();
    assert!(complete, "{batch}");
    assert_eq!(batch["eligible_runs"], 2);
    assert_eq!(batch["runs"][0]["execution"]["report"]["seed"], 42);
    let (comparison, complete) = execute(&[
        "compare-scripts".into(),
        source_path.into(),
        source_path.into(),
        request.into(),
    ])
    .unwrap();
    assert!(complete);
    assert_eq!(comparison["mean_damage_delta"]["numerator"], "0");
    let failed_path = directory.join("failed.nte");
    std::fs::write(&failed_path, "options duration=10 crit=fixed; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0; target hp=100000000 def=0 resist=0; active mint; mint burst;").unwrap();
    let (failed, complete) = execute(&[
        "batch-script".into(),
        failed_path.to_str().unwrap().into(),
        request.into(),
    ])
    .unwrap();
    assert!(!complete);
    assert_eq!(failed["eligible_runs"], 0);
    assert!(failed["mean_damage"].is_null());
    std::fs::remove_file(request_path).unwrap();
    std::fs::remove_file(failed_path).unwrap();
}

#[test]
fn cli_searches_legal_equipped_sources_and_resumes_a_saved_trial_prefix() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cache/source-search-cli")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut request: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("native-source-search.json")).unwrap())
            .unwrap();
    request["max_new_trials"] = 1.into();
    for candidate in request["candidates"].as_array_mut().unwrap() {
        candidate["source"] = root
            .join(candidate["source"].as_str().unwrap())
            .to_str()
            .unwrap()
            .into();
    }
    let request_path = directory.join("search.json");
    let checkpoint_path = directory.join("checkpoint.json");
    std::fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
    let path = request_path.to_str().unwrap();
    let (first, complete) = execute(&["search-scripts".into(), path.into()]).unwrap();
    assert!(!complete);
    assert_eq!(first["checkpoint"]["runs"].as_array().unwrap().len(), 1);
    assert!(first["ranking"].as_array().unwrap().is_empty());
    std::fs::write(
        &checkpoint_path,
        serde_json::to_vec(&first["checkpoint"]).unwrap(),
    )
    .unwrap();
    request["max_new_trials"] = 64.into();
    std::fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
    let (resumed, complete) = execute(&[
        "search-scripts".into(),
        path.into(),
        checkpoint_path.to_str().unwrap().into(),
    ])
    .unwrap();
    assert!(complete, "{}", resumed["candidates"]);
    let (full, complete) = execute(&["search-scripts".into(), path.into()]).unwrap();
    assert!(complete);
    assert_eq!(resumed, full);
    assert_eq!(full["ranking"].as_array().unwrap().len(), 2);
    assert_eq!(
        full["checkpoint"]["runs"][0]["execution"]["report"]["hits_settled"],
        166
    );
    request["experiment"]["seeds"] = serde_json::json!([43, 42]);
    std::fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
    assert!(
        execute(&[
            "search-scripts".into(),
            path.into(),
            checkpoint_path.to_str().unwrap().into()
        ])
        .unwrap_err()
        .contains("identity mismatch")
    );
    std::fs::remove_file(request_path).unwrap();
    std::fs::remove_file(checkpoint_path).unwrap();
}

#[test]
fn cli_compiles_and_validates_the_same_source_as_the_wasm_frontend() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../presets/studio/s1.nte");
    let args = vec!["compile-script".into(), path.into()];
    assert_eq!(
        parse_command(&args).unwrap(),
        CliCommand::CompileScript {
            source: path,
            validate_only: false
        }
    );
    let (response, valid) = execute(&args).unwrap();
    assert!(valid);
    let source = std::fs::read_to_string(path).unwrap();
    let native = ntesim_content::dsl::source::compile_response(
        &ntesim_content::dsl::compiler::SourceRequest {
            source,
            limits: Default::default(),
        },
    );
    assert!(response == serde_json::to_value(native).unwrap());
    let (inspection, valid) = execute(&["inspect-script".into(), path.into()]).unwrap();
    assert!(valid);
    assert_eq!(inspection["party"], response["party"]);
    assert_eq!(inspection["diagnostics"], serde_json::json!([]));
    let (diagnostics, valid) = execute(&["validate-script".into(), path.into()]).unwrap();
    assert!(valid);
    assert_eq!(diagnostics, serde_json::json!([]));
}

#[test]
fn cli_runs_source_directly_and_exports_the_same_native_trial() {
    for (fixture, hits) in [
        ("live-control", 20),
        ("finance-control", 0),
        ("resource-control", 43),
        ("nanally-reaction-control", 21),
        ("nanally-authority-control", 10),
        ("nanally-normal-control", 28),
        ("nanally-secondary-control", 16),
        ("nanally-burst-control", 212),
        ("nanally-plunge-control", 2),
        ("nanally-heavy-control", 14),
        ("nanally-cycle-control", 80),
        ("nanally-dodge-control", 14),
        ("nanally-full-kit", 275),
    ] {
        let path = format!(
            "{}/../../presets/native/{fixture}.nte",
            env!("CARGO_MANIFEST_DIR")
        );
        let args = vec!["run-script".into(), path.clone()];
        assert_eq!(
            parse_command(&args).unwrap(),
            CliCommand::RunScript {
                source: &path,
                stop_after_inputs: None,
                resume: None,
                cancel_file: None
            }
        );
        let (response, complete) = execute(&args).unwrap();
        assert!(complete);
        let source = std::fs::read_to_string(&path).unwrap();
        let native = ntesim_content::simulation::run_source(
            &ntesim_content::dsl::compiler::SourceRequest {
                source,
                limits: Default::default(),
            },
            &Default::default(),
        )
        .unwrap();
        assert!(response == serde_json::to_value(native).unwrap());
        assert_eq!(response["execution"]["report"]["hits_settled"], hits);
    }
}

#[test]
fn cli_runs_a_saved_compile_response_without_internal_scenario_edits() {
    let source_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../presets/native/live-control.nte"
    );
    let (compiled, valid) = execute(&["compile-script".into(), source_path.into()]).unwrap();
    assert!(valid);
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cache/source-artifact-cli")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("compiled.json");
    std::fs::write(&path, serde_json::to_vec(&compiled).unwrap()).unwrap();
    let args = vec!["run-compiled-script".into(), path.to_str().unwrap().into()];
    let (actual, complete) = execute(&args).unwrap();
    let (expected, source_complete) = execute(&["run-script".into(), source_path.into()]).unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(complete && source_complete);
    assert!(actual == expected);
}

#[test]
fn cli_source_outcomes_survive_saved_artifacts_and_native_export_replay() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cache/source-failure-cli")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).unwrap();
    let source_path = directory.join("failure.nte");
    let artifact_path = directory.join("compiled.json");
    for (rotation, completed) in [
        ("mint attack; let n=1/(.actor.energy-.actor.energy);", false),
        ("wait_for_energy(100); mint burst;", false),
        ("wait(21); mint attack;", true),
        ("mint burst;", false),
    ] {
        let source = format!(
            "options duration=20 crit=fixed; # 😀\n\
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;\
            target hp=100000000 def=0 resist=0; active mint; {rotation}"
        );
        std::fs::write(&source_path, source).unwrap();
        let path = source_path.to_str().unwrap();
        let (expected, complete) = execute(&["run-script".into(), path.into()]).unwrap();
        assert_eq!(complete, completed);
        let (compiled, valid) = execute(&["compile-script".into(), path.into()]).unwrap();
        assert!(valid);
        std::fs::write(&artifact_path, serde_json::to_vec(&compiled).unwrap()).unwrap();
        let (saved, complete) = execute(&[
            "run-compiled-script".into(),
            artifact_path.to_str().unwrap().into(),
        ])
        .unwrap();
        assert_eq!(complete, completed);
        assert_eq!(saved, expected);
        let execution = &saved["execution"];
        let input = serde_json::to_vec(&execution["request"]).unwrap();
        let replay = crate::reports::game_runner_report(&input).unwrap();
        for (key, value) in execution["report"].as_object().unwrap() {
            assert_eq!(&replay[key], value, "saved export replay {key}: {rotation}");
        }
    }
    std::fs::remove_file(source_path).unwrap();
    std::fs::remove_file(artifact_path).unwrap();
}

#[test]
fn cli_runs_continuing_raging_flames_source_through_native_aggregation() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/raging-flames-redirect-cycles.nte"
    );
    let (response, complete) = execute(&["run-script".into(), path.into()]).unwrap();
    assert!(complete, "{}", response["execution"]["diagnostic"]);
    let execution = &response["execution"];
    let input = serde_json::to_vec(&execution["request"]).unwrap();
    let aggregate = crate::reports::game_runner_report(&input).unwrap();
    for (key, value) in execution["report"].as_object().unwrap() {
        assert_eq!(&aggregate[key], value, "native aggregation {key}");
    }
    assert_eq!(aggregate["final_actor_states"][0]["spent_energy"], 200);
    assert_eq!(
        aggregate["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"]
            ["current"],
        serde_json::json!("4/1")
    );
}
